use alloc::vec::Vec;
use anyhow::{Result, bail};
use arbitrary_int::prelude::*;
use test_suite_common::Step;

use crate::io;

/// The offset in ROM of the embedded data produced by the corresponding record-mode test.
///
/// The build process appends the data to the ROM and patches this symbol with the actual offset.
#[used(linker)]
#[unsafe(no_mangle)]
static EMBEDDED_DATA_ROM_OFFSET: u32 = 0x0BAD_0BAD;

/// The size of the embedded data, patched similarly to the offset.
#[used(linker)]
#[unsafe(no_mangle)]
static EMBEDDED_DATA_ROM_SIZE: u32 = 0x0BAD_0BAD;

fn embedded_data_rom_offset() -> u32 {
    // Get the value from a runtime memory read to prevent the compiler from const-folding the value and breaking patching
    unsafe { (&raw const EMBEDDED_DATA_ROM_OFFSET as *const u32).read_volatile() }
}

fn embedded_data_rom_size() -> u32 {
    unsafe { (&raw const EMBEDDED_DATA_ROM_SIZE as *const u32).read_volatile() }
}

/// Compares steps emitted by the program against the embedded recorded steps.
///
/// This copies the embedded steps from ROM to RAM via a basic streaming mechanism to avoid filling the memory with all the steps at once,
/// which has been shown to cause out-of-memory situations for tests that record a lot of data, like large memory regions.
pub struct Comparator {
    /// Raw step data copied from ROM and ready to be deserialized into steps.
    /// Refilled with the next chunk of step data from ROM when it's exhausted.
    deserialization_buffer: Vec<u8>,

    /// Current position in the buffer, increases as we deserialize steps.
    deserialization_buffer_offset: usize,

    /// Reception buffer for the DMA transfers.
    /// We use a secondary buffer to deal with the alignment requirements of the DMA transfers that might not match the current state of the deserialization buffer
    /// (eg. the destination address might not be aligned to 8 bytes, the source address might not be aligned to 2 bytes).
    dma_buffer: Vec<u8>, // TODO align for the DMA???

    /// Current address used as the DMA source, increases as we transfer data from ROM to RAM.
    dma_source_address: usize,
}

const BUFFER_SIZE: usize = 10; //0x1000;

impl Default for Comparator {
    fn default() -> Self {
        let mut comparator = Self {
            deserialization_buffer: alloc::vec![0; BUFFER_SIZE],
            deserialization_buffer_offset: 0,
            dma_buffer: alloc::vec![0; BUFFER_SIZE],
            dma_source_address: 0x1000_0000 + embedded_data_rom_offset() as usize,
        };

        // Initial transfer
        //TODO
        // comparator
        //     .refill()
        //     .expect("failed to refill the comparator buffer");

        comparator
    }
}

impl Comparator {
    // TODO doc
    // TODO return bool
    pub fn compare(&mut self, step: &Step) -> Result<Step> {
        // Ignore comments as they have been stripped from the embedded steps

        if matches!(step, Step::Comment(_)) {
            return Ok(step.clone());
        }

        // The comparison relies on the next expected step getting deserialized.
        //
        // Because embedded steps are "streamed" in chunks from ROM via DMA transfers, all the data of that next step might not have been buffered yet.
        // So we try to deserialize the step once, which might work, but if the buffer is exhausted, we refill it and try again.
        // The second time, we're guaranteed to have the whole step buffered.
        //
        // Our steps are tiny (basically a discriminant and a number value) so a single retry will be enough.
        // If we wanted to support streaming steps with varying lengths, we would need a more sophisticated solution,
        // but since we stripped the string descriptions from the embedded steps, this is not required :)

        let mut expected_step = self.next()?;

        if expected_step.is_none() {
            self.refill()?;

            expected_step = self.next()?;
        }

        match expected_step {
            None => {
                bail!("the comparator buffer is exhausted");
            }

            // Compare the runtime step against the expected step
            Some(expected_step) => {
                if *step == expected_step {}

                return Ok(expected_step);
                // panic!("Comparison result: {:?} {:?}", step, expected_step);
            }
        }
    }

    /// Tries to return the next embedded step.
    /// Returns the step if successful.
    /// Returns None if the buffer is exhausted and needs to be refilled.
    fn next(&mut self) -> Result<Option<Step>> {
        match postcard::take_from_bytes::<Step>(
            &self.deserialization_buffer[self.deserialization_buffer_offset..],
        ) {
            Ok((expected_step, rest)) => {
                self.deserialization_buffer_offset = self.deserialization_buffer.len() - rest.len();

                return Ok(Some(expected_step.clone()));
            }

            Err(postcard::Error::DeserializeUnexpectedEnd) => Ok(None),

            Err(e) => {
                bail!("failed to deserialize step: {:?}", e);
            }
        }
    }

    /// Refills the buffer so that the remaining data moves to the front, followed by new data from ROM.
    fn refill(&mut self) -> Result<()> {
        // TODO error if end of rom reached, clamp

        // Slide the remaining data to the front

        self.deserialization_buffer
            .copy_within(self.deserialization_buffer_offset.., 0);

        // Transfer new data from ROM to the DMA buffer
        //
        // The destination DMA buffer is aligned to 8 bytes, so there are no issues on that side.
        // However, the source address might not be aligned to 2 bytes, in which case we need to copy from the previous byte and discard it later.

        let bytes_to_transfer = self.deserialization_buffer_offset;

        let dma_source_address_misalignment = self.dma_source_address & 1;

        io::pi_dma(&io::PiDma {
            direction: io::PiDmaDirection::PiToRam,
            ram_address: u24::from_u32(io::physical(self.dma_buffer.as_ptr() as u32)),
            pi_address: (self.dma_source_address - dma_source_address_misalignment) as u32,
            length: u24::from_u32(
                bytes_to_transfer as u32 + dma_source_address_misalignment as u32 - 1,
            ),
        });

        io::wait_until(|| io::read_uncached(n64_specs::pi::Status::ADDRESS) & 0x1 == 0);

        // Copy the new data from the DMA buffer to the deserialization buffer,
        // discarding the possibly redundant byte transferred for alignment reasons

        let dma_buffer_uncached = io::uncached_ptr(self.dma_buffer.as_ptr() as u32) as *mut u8;

        let copy_start = self.deserialization_buffer.len() - self.deserialization_buffer_offset;

        for i in 0..bytes_to_transfer {
            self.deserialization_buffer[copy_start + i] = unsafe {
                dma_buffer_uncached
                    .add(i + dma_source_address_misalignment)
                    .read_volatile()
            };
        }

        self.deserialization_buffer_offset = 0;
        self.dma_source_address += bytes_to_transfer;

        unsafe {
            panic!(
                "dma res: {:0X?} {:0X?} {:0X?} {:0X?} {:0X?} {:0X?} {:0X?} {:0X?} {:0X?} {:0X?}",
                dma_buffer_uncached.add(0).read_volatile(),
                dma_buffer_uncached.add(1).read_volatile(),
                dma_buffer_uncached.add(2).read_volatile(),
                dma_buffer_uncached.add(3).read_volatile(),
                dma_buffer_uncached.add(4).read_volatile(),
                dma_buffer_uncached.add(5).read_volatile(),
                dma_buffer_uncached.add(6).read_volatile(),
                dma_buffer_uncached.add(7).read_volatile(),
                dma_buffer_uncached.add(8).read_volatile(),
                dma_buffer_uncached.add(9).read_volatile(),
            );
        }

        panic!("buff: {:0X?}", self.deserialization_buffer);

        Ok(())
    }
}
