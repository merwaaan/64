use core::slice;

use arbitrary_int::prelude::*;
use n64_specs as specs;

use crate::{
    events::{EventType, Events},
    location::Location,
    ram::RamLocation,
    system::System,
    value::Value,
};

pub type AiLocation = Location<{ specs::ai::START }, { specs::ai::END }>;

#[derive(Default, Clone, Copy, Debug)]
struct DmaSlot {
    address: u32,
    length: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Ai {
    pub regs: specs::ai::Registers,

    active_dma: Option<DmaSlot>,
    pending_dma: Option<DmaSlot>,
}

impl Ai {
    pub(crate) fn read<T: Value>(s: &System, addr: AiLocation) -> T {
        assert!(T::BYTES == 4, "AI: read with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::ai::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "AI: read from unaligned address {:08X}",
            offset
        );

        match offset {
            // Status
            0xC => {
                let status = [s.ai.regs.status.raw_value()];

                T::read_reg(&status, 0)
            }

            // All the other registers mirror LENGTH
            _ => {
                let length = [s.ai.regs.dma_length.raw_value()];

                T::read_reg(&length, 0)
            }
        }
    }

    pub(crate) fn write<T: Value>(s: &mut System, addr: AiLocation, data: T) {
        assert!(T::BYTES == 4, "AI: write with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::ai::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "AI: write to unaligned address {:08X}",
            offset
        );

        let mut written = 0u32;
        data.write_reg(slice::from_mut(&mut written), 0);

        match offset {
            // DMA RAM Address
            0x0 => {
                s.ai.regs.dma_ram_address.write_masked(written);
            }

            // DMA length: writes start a DMA transfer
            0x4 => {
                s.ai.regs.dma_length.write_masked(written);

                Self::push_dma(
                    s,
                    DmaSlot {
                        address: s.ai.regs.dma_ram_address.raw_value(),
                        length: s.ai.regs.dma_length.raw_value(),
                    },
                );
            }

            // Control
            0x8 => {
                s.ai.regs.control.write_masked(written);

                // Mirror into Status

                s.ai.regs
                    .status
                    .set_dma_enabled(s.ai.regs.control.dma_enabled());
            }

            // Status: read-only, writes clear the AI interrupt
            0xC => {
                s.mi.clear_pending_interrupt(specs::interrupt::Interrupt::Ai, &mut s.cop0);
            }

            // Dac rate
            0x10 => {
                s.ai.regs.dac_rate.write_masked(written);

                // Notify the audio renderer that the sample rate changed

                s.audio_renderer.set_sample_rate(s.ai.sample_rate());
            }

            // Bit rate
            0x14 => {
                s.ai.regs.bit_rate.write_masked(written);
            }

            _ => {
                log::warn!("AI: write to unknown register {:08X}", offset);
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        // TODO move to specs
        // TODO var in sp or something, also correct val?
        (62_500_000.0 / ((self.regs.dac_rate.raw_value() + 1) as f64)) as u32
    }

    fn push_dma(s: &mut System, slot: DmaSlot) {
        if s.ai.pending_dma.is_some() {
            log::warn!("AI: DMA queue full");
        }
        // Active DMA transfer: queue
        else if s.ai.active_dma.is_some() {
            s.ai.pending_dma = Some(slot);
            s.ai.regs.status.set_dma_full(true);
        }
        // No active DMA transfer: execute
        else {
            s.ai.active_dma = Some(slot);

            Self::start_dma(s, slot);
        }
    }

    fn start_dma(s: &mut System, slot: DmaSlot) {
        // Do nothing if DMA is not enabled

        if !s.ai.regs.control.dma_enabled() {
            return;
        }

        log::info!(
            "AI: DMA {:X} bytes from RAM {:08X}",
            slot.length,
            slot.address,
        );

        // Push RAM data to the audio renderer

        s.ram.read_block(
            RamLocation::from_absolute(slot.address),
            slot.length as usize,
            |ram_data| {
                s.audio_renderer.push(ram_data);
            },
        );

        // Schedule completion

        let samples = (slot.length / 4) as f64; // 16-bit stereo = 4 bytes per sample
        let cycles = ((samples / (s.ai.sample_rate() as f64)) * 93_750_000.0) as usize; // TODO correct cycle unit?

        Events::push(s, EventType::AiDmaTransferComplete, cycles);

        s.ai.regs.status.set_dma_busy(true);
    }

    pub(crate) fn dma_completed(s: &mut System) {
        assert!(
            s.ai.active_dma.is_some(),
            "AI DMA: completed, but no active transfer"
        );

        assert!(
            s.ai.regs.status.dma_busy(),
            "AI DMA: completed, but not busy"
        );

        // Raise an AI interrupt

        s.mi.set_pending_interrupt(specs::interrupt::Interrupt::Ai, &mut s.cop0);

        // Switch to the pending DMA, if any

        s.ai.active_dma = s.ai.pending_dma.take();

        if let Some(slot) = s.ai.active_dma {
            s.ai.regs.status.set_dma_full(false);

            Self::start_dma(s, slot);
        } else {
            s.ai.regs.dma_length.set_value(u18::ZERO);

            s.ai.regs.status.set_dma_busy(false);
        }
    }
}
