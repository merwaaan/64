use alloc::vec::Vec;
use anyhow::{Result, bail};
use arbitrary_int::prelude::*;
use test_suite_common::Step;

use crate::io;

/// The PI address of the embedded data collected by the corresponding record-mode ROM.
fn embedded_data_pi_address() -> u32 {
    // The build process appends the data to the ROM and patches this symbol with the actual offset.
    #[used(linker)]
    #[unsafe(no_mangle)]
    static EMBEDDED_DATA_ROM_OFFSET: u32 = 0x0BAD_0BAD;

    // Get the value from a runtime memory read to prevent the compiler from const-folding the value and breaking patching
    let offset = unsafe { (&raw const EMBEDDED_DATA_ROM_OFFSET).read_volatile() };

    assert!(
        offset != 0x0BAD_0BAD,
        "EMBEDDED_DATA_ROM_OFFSET has not been patched"
    );

    0x1000_0000 + offset
}

/// The size of the embedded data, patched similarly to the offset.
fn embedded_data_size() -> u32 {
    #[used(linker)]
    #[unsafe(no_mangle)]
    static EMBEDDED_DATA_ROM_SIZE: u32 = 0x0BAD_0BAD;

    let size = unsafe { (&raw const EMBEDDED_DATA_ROM_SIZE).read_volatile() };

    assert!(
        size != 0x0BAD_0BAD,
        "EMBEDDED_DATA_ROM_SIZE has not been patched"
    );

    size
}

/// Compares steps emitted by the program against the embedded recorded steps.
///
/// Progressively copies the embedded steps from ROM to RAM via DMA transfers to avoid filling precious RAM with all the steps at once.
pub struct Comparator {
    /// Raw step data copied from ROM and ready to be deserialized.
    /// Refilled with the next chunk of embedded data from ROM when it's exhausted.
    deserialization_buffer: Vec<u8>,

    /// Current position in the buffer, increases as we deserialize steps.
    /// Deserializing the data starting here should yield a step.
    deserialization_buffer_offset: usize,

    /// Deserialized bytes, to track progress.
    consumed_embedded_data: u32,

    /// Reception buffer for the DMA transfers.
    ///
    /// We use a secondary buffer to deal with the alignment requirements of the DMA transfers that might not match the current state of the deserialization buffer.
    ///
    /// For instance:
    /// - the RAM destination address might not be aligned to 8 bytes as we don't necesarily DMA to its aligned start address
    /// - the PI source address might not be aligned to 2 bytes as each embedded step is not aligned
    dma_buffer: io::Buffer<u8>,

    /// Current address used as the DMA source, increases as we transfer data from ROM to RAM.
    dma_source_address: u32,

    /// Current  test case.
    test_case_index: u32,

    /// Current step in the current test case.
    test_case_step_index: u32,
}

const BUFFER_SIZE: usize = 0x1000;

impl Default for Comparator {
    fn default() -> Self {
        Self {
            deserialization_buffer: alloc::vec![0; BUFFER_SIZE],
            deserialization_buffer_offset: BUFFER_SIZE, // "full" buffer to refill it at the start
            consumed_embedded_data: 0,
            dma_buffer: io::Buffer::<u8>::with_alignment(
                BUFFER_SIZE + 1, // +1 padding to deal with PI misalignment
                n64_specs::pi::DMA_RAM_ALIGNMENT,
            ),
            dma_source_address: embedded_data_pi_address(),
            test_case_index: 0,
            test_case_step_index: 0,
        }
    }
}

pub enum Comparison {
    Same,
    Different {
        expected_step: Step,
        step_index: u32,
    },
    TooManySteps {
        step_index: u32,
    },
}

impl Comparator {
    // Compares a runtime step against the next expected step.
    pub fn compare(&mut self, step: &Step) -> Result<Comparison> {
        // Peek at the next expected step

        let expected_step = self.peek()?;

        // Compare the runtime step against the expected step

        match expected_step {
            Some(expected_step) => {
                // Same runtime and recorded steps, it's all good
                if *step == expected_step {
                    self.test_case_step_index += 1;

                    // Consume the step

                    self.take()?;

                    Ok(Comparison::Same)
                }
                // Mismatch, raise an error
                // (the rest of the test case will still need to be skipped)
                else {
                    Ok(Comparison::Different {
                        expected_step: expected_step.clone(),
                        step_index: self.test_case_step_index,
                    })
                }
            }
            // All the recorded steps have been consumed, the runtime test case emitted too many steps
            None => Ok(Comparison::TooManySteps {
                step_index: self.test_case_step_index,
            }),
        }
    }

    /// Skips the current test's remaining embedded steps.
    ///
    /// In case of mismatch, this advances the steps to the next test.
    /// If the test completed without mismatches, no need to do anything.
    pub fn skip_test(&mut self) -> Result<()> {
        loop {
            match self.peek()? {
                // If there's no more data to consume, we're done
                // (we skipped the last test case OR there's something wrong and the next comparison will fail and report the issue)
                None => return Ok(()),
                // Start of a new test, we're done, we don't consume the step to let the next comparison have it
                Some(Step::StartTest(_)) => {
                    self.test_case_index = 0;
                    self.test_case_step_index = 0;
                    return Ok(());
                }
                // Another step from the test to skip, continue advancing
                Some(_) => {
                    self.take()?;
                }
            }
        }
    }

    /// Skips the current test case's remaining steps.
    pub fn skip_case(&mut self) -> Result<()> {
        loop {
            let next_step = self.peek()?;

            match next_step {
                None => {
                    return Ok(());
                }
                Some(Step::StartTestCase(_)) => {
                    self.test_case_index += 1;
                    self.test_case_step_index = 0;

                    return Ok(());
                }
                Some(_) => {
                    self.take()?;
                }
            }
        }
    }

    /// Returns the next embedded step and advances the buffer.
    fn take(&mut self) -> Result<Option<Step>> {
        self.next(true)
    }

    /// Returns the next embedded step without advancing the buffer.
    fn peek(&mut self) -> Result<Option<Step>> {
        self.next(false)
    }

    fn next(&mut self, advance: bool) -> Result<Option<Step>> {
        if self.consumed_embedded_data >= embedded_data_size() {
            return Ok(None);
        }

        // The comparison relies on the next expected step getting deserialized.
        //
        // Because embedded steps are "streamed" in chunks from ROM via DMA transfers, all the data of that next step might not have been buffered yet.
        // So we try to deserialize the step once, which might work, but if the buffer is exhausted, we refill it and try again.
        //
        // Our steps are tiny (basically a discriminant and a number value) so a single retry will be enough.
        // If we wanted to support streaming steps with varying lengths, we would need a more sophisticated solution, but this is not currently an issue :)
        //
        // So the second time, we're guaranteed to have the whole step buffered.

        for attempt in 0..2 {
            let deserialization_slice =
                &self.deserialization_buffer[self.deserialization_buffer_offset..];

            match postcard::take_from_bytes::<Step>(deserialization_slice) {
                Ok((expected_step, rest)) => {
                    if advance {
                        self.deserialization_buffer_offset =
                            self.deserialization_buffer.len() - rest.len();

                        self.consumed_embedded_data +=
                            (deserialization_slice.len() - rest.len()) as u32;
                    }

                    return Ok(Some(expected_step));
                }

                Err(postcard::Error::DeserializeUnexpectedEnd) => {
                    // First attempt: refill and try a second time
                    if attempt == 0 {
                        self.refill()?;
                    }
                    // Second attempt: something is wrong
                    else {
                        bail!("failed to deserialize step after refill");
                    }
                }

                Err(e) => {
                    bail!("failed to deserialize step, {:?}", e);
                }
            }
        }

        Ok(None)
    }

    /// Remaining non-transferred bytes in the embedded data.
    fn remaining_embedded_bytes(&self) -> u32 {
        let transferred_bytes =
            (self.dma_source_address as u32).saturating_sub(embedded_data_pi_address());

        embedded_data_size().saturating_sub(transferred_bytes)
    }

    /// Refills the buffer so that the remaining data moves to the front, followed by new data from ROM.
    fn refill(&mut self) -> Result<()> {
        // If we already consumed all the embedded data, there's nothing to do

        if self.consumed_embedded_data >= embedded_data_size() {
            return Ok(());
        }

        // Slide the remaining buffered data to the front if any, it's the start of the next step

        self.deserialization_buffer
            .copy_within(self.deserialization_buffer_offset.., 0);

        // Transfer new data from ROM to the DMA buffer
        //
        // The destination DMA buffer is aligned to 8 bytes, so there are no issues on that side.
        // However, the source PI address might not be aligned to 2 bytes, in which case we need to copy from the previous byte and discard it later.

        let bytes_to_transfer =
            (self.deserialization_buffer_offset as u32).min(self.remaining_embedded_bytes());

        let dma_source_address_misalignment = self.dma_source_address & 1;

        io::pi_dma(
            &io::PiDma {
                direction: io::PiDmaDirection::PiToRam,
                ram_address: u24::from_u32(io::physical_addr(self.dma_buffer.as_ptr() as u32)),
                pi_address: self.dma_source_address - dma_source_address_misalignment,
                length: u24::from_u32(bytes_to_transfer + dma_source_address_misalignment - 1),
            },
            true,
        );

        // Copy the new data from the DMA buffer to the deserialization buffer,
        // discarding the possibly redundant byte transferred for alignment reasons

        let serialization_buffer_copy_offset =
            (self.deserialization_buffer.len() - self.deserialization_buffer_offset) as usize;

        for i in 0..bytes_to_transfer as usize {
            self.deserialization_buffer[serialization_buffer_copy_offset + i] = self
                .dma_buffer
                .get(i + dma_source_address_misalignment as usize);
        }

        self.deserialization_buffer_offset = 0;
        self.dma_source_address += bytes_to_transfer;

        Ok(())
    }
}
