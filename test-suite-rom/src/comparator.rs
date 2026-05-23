use alloc::vec::Vec;
use anyhow::{Result, anyhow, bail};
use arbitrary_int::prelude::*;
use test_suite_common::Step;

use crate::{
    io,
    test::{Mismatch, TestError},
};

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
    unsafe { (&raw const EMBEDDED_DATA_ROM_OFFSET).read_volatile() }
}

fn embedded_data_rom_size() -> u32 {
    unsafe { (&raw const EMBEDDED_DATA_ROM_SIZE).read_volatile() }
}

/// Compares steps emitted by the program against the embedded recorded steps.
///
/// This copies the embedded steps from ROM to RAM via a basic streaming mechanism to avoid filling the memory with all the steps at once,
/// which has been shown to cause out-of-memory situations for tests that record a lot of data, like large memory regions.
pub struct Comparator {
    /// Raw step data copied from ROM and ready to be deserialized.
    /// Refilled with the next chunk of raw data from ROM when it's exhausted.
    deserialization_buffer: Vec<u8>,

    /// Current position in the buffer, increases as we deserialize steps.
    deserialization_buffer_offset: usize,

    /// Reception buffer for the DMA transfers.
    /// We use a secondary buffer to deal with the alignment requirements of the DMA transfers that might not match the current state of the deserialization buffer
    /// (eg. the destination address might not be aligned to 8 bytes, the source address might not be aligned to 2 bytes).
    dma_buffer: io::Buffer<u8>,

    /// Current address used as the DMA source, increases as we transfer data from ROM to RAM.
    dma_source_address: usize,

    /// Whether the comparator has been kickstarted by validating the first recorded step.
    started: bool,

    /// Current step in the current test case.
    test_case_step_index: u32,
}

const BUFFER_SIZE: usize = 100; //0x1000; TODO

impl Default for Comparator {
    fn default() -> Self {
        Self {
            deserialization_buffer: alloc::vec![0; BUFFER_SIZE],
            deserialization_buffer_offset: BUFFER_SIZE,
            dma_buffer: io::Buffer::<u8>::with_alignment(
                BUFFER_SIZE,
                n64_specs::pi::DMA_RAM_ALIGNMENT,
            ),
            dma_source_address: 0x1000_0000 + embedded_data_rom_offset() as usize,
            test_case_step_index: 0,
            started: false,
        }
    }
}

impl Comparator {
    // Compares a runtime step against the next expected step.
    pub fn compare(&mut self, step: &Step) -> Result<(), TestError> {
        // Kickstart the comparator by checking that the first recorded step is indeed the start of a test case

        if !self.started {
            let step = self.take()?;

            if step != Step::StartTestCase {
                return Err(anyhow!("recorded steps start with unexpected {:?}", step).into());
            }

            self.started = true;
        }

        // Ignore comments as they have been stripped from the embedded steps

        if matches!(step, Step::Comment(_)) {
            self.test_case_step_index += 1;

            return Ok(());
        }

        // Retrieve the next expected step

        let expected_step = self.take()?;

        // Compare the runtime step against the expected step

        if *step == expected_step {
            self.test_case_step_index += 1;

            Ok(())
        } else {
            // The compare-mode test does not emit StartTestCase, so if we get one, it emitted more steps than expected

            if expected_step == Step::StartTestCase {
                Err(TestError::Mismatch(Mismatch::ExcessSteps {
                    step_index: self.test_case_step_index,
                }))
            } else {
                Err(TestError::Mismatch(Mismatch::DifferentStep {
                    runtime_step: step.clone(),
                    expected_step,
                    step_index: self.test_case_step_index,
                }))
            }
        }
    }

    /// Finalizes a test case comparison by checking that all the recorded steps have been compared.
    pub fn finalize_case(&mut self) -> Result<(), TestError> {
        // TODO max size

        // If everything went well, the next step should be the start of the next test case

        let step = self.take()?;

        if step != Step::StartTestCase {
            Err(TestError::Mismatch(Mismatch::MissingSteps {
                step_index: self.test_case_step_index,
            }))
        } else {
            self.test_case_step_index = 0;

            Ok(())
        }
    }

    /// Skips the current test case remaining steps.
    pub fn skip_case(&mut self) -> Result<()> {
        // TODO max size

        loop {
            let step = self.take()?;

            if step == Step::StartTestCase {
                self.test_case_step_index = 0;

                return Ok(());
            }
        }
    }

    /// Returns the next embedded step and advances the buffer.
    fn take(&mut self) -> Result<Step> {
        self.next(true)
    }

    /// Returns the next embedded step without advancing the buffer.
    fn peek(&mut self) -> Result<Step> {
        self.next(false)
    }

    fn next(&mut self, advance: bool) -> Result<Step> {
        // The comparison relies on the next expected step getting deserialized.
        //
        // Because embedded steps are "streamed" in chunks from ROM via DMA transfers, all the data of that next step might not have been buffered yet.
        // So we try to deserialize the step once, which might work, but if the buffer is exhausted, we refill it and try again.
        //
        // Our steps are tiny (basically a discriminant and a number value) so a single retry will be enough.
        // If we wanted to support streaming steps with varying lengths, we would need a more sophisticated solution,
        // but since we stripped the comments from the embedded steps, this is not required :)
        //
        // So the second time, we're guaranteed to have the whole step buffered.

        for attempt in 0..2 {
            match postcard::take_from_bytes::<Step>(
                &self.deserialization_buffer[self.deserialization_buffer_offset..],
            ) {
                Ok((expected_step, rest)) => {
                    if advance {
                        self.deserialization_buffer_offset =
                            self.deserialization_buffer.len() - rest.len();
                    }

                    return Ok(expected_step);
                }

                Err(postcard::Error::DeserializeUnexpectedEnd) => {
                    if attempt == 0 {
                        // Refill and try again
                        self.refill()?;
                    }
                }

                Err(e) => {
                    bail!("failed to deserialize step: {:?}", e);
                }
            }
        }

        bail!("the comparator buffer is exhausted");
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

        // panic!(
        //     "dma_source_address: {:0X?} {:0X?} {:0X?}",
        //     self.dma_source_address, dma_source_address_misalignment, bytes_to_transfer
        // );

        io::pi_dma(
            &io::PiDma {
                direction: io::PiDmaDirection::PiToRam,
                ram_address: u24::from_u32(io::physical(self.dma_buffer.as_ptr() as u32)),
                pi_address: (self.dma_source_address - dma_source_address_misalignment) as u32,
                length: u24::from_u32(
                    bytes_to_transfer as u32 + dma_source_address_misalignment as u32 - 1,
                ),
            },
            true,
        );

        // Copy the new data from the DMA buffer to the deserialization buffer,
        // discarding the possibly redundant byte transferred for alignment reasons

        let copy_start = self.deserialization_buffer.len() - self.deserialization_buffer_offset;

        for i in 0..bytes_to_transfer {
            self.deserialization_buffer[copy_start + i] =
                self.dma_buffer.get(i + dma_source_address_misalignment);
        }

        self.deserialization_buffer_offset = 0;
        self.dma_source_address += bytes_to_transfer;

        Ok(())
    }
}
