//! Audio interface
//!
//! Manages DMA transfers of audio samples between RAM and the audio renderer.
//! The transferred audio data is 16-bit stereo, so 4 bytes per sample.
//!
//! https://n64brew.dev/wiki/Audio_Interface

use core::slice;

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::{
    events::{EventType, Events},
    location::Location,
    mi::Interrupt,
    ram::RamLocation,
    system::System,
    value::Value,
};

pub type AiLocation = Location<0x0450_0000, 0x0460_0000>;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0x0110_0000, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Status {
    #[bit(31, rw)]
    dma_full: bool,

    #[bit(30, rw)]
    dma_busy: bool,

    #[bit(25, rw)]
    dma_enabled: bool,

    #[bit(19, rw)]
    word_clock: bool,

    #[bit(16, rw)]
    bit_clock: bool,

    #[bits(1..=14, rw)]
    count: u14,

    #[bit(0, rw)]
    dma_full_mirror: bool,
}

const REGISTERS_MASK: u32 = 0x1F;

#[derive(Default, Clone, Copy, Debug)]
struct DmaSlot {
    address: u32,
    length: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Ai {
    pub dma_ram_address: u32,
    pub dma_length: u32,
    pub dma_enabled: bool,
    pub status: Status,
    pub dac_rate: u32,

    active_dma: Option<DmaSlot>,
    pending_dma: Option<DmaSlot>,
}

impl Ai {
    pub fn read<T: Value>(s: &System, addr: AiLocation) -> T {
        assert!(T::BYTES == 4, "AI: read with invalid size {}", T::BYTES);

        let offset = addr.relative() & REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "AI: read from unaligned address {:08X}",
            offset
        );

        match offset {
            // Status
            0xC => {
                let status_data = bytemuck::cast_slice(bytemuck::bytes_of(&s.ai.status));
                T::read_reg(status_data, offset & 3)
            }

            // All the other registers mirror LENGTH
            _ => {
                let length_data = slice::from_ref(&s.ai.dma_length);
                T::read_reg(length_data, offset & 3)
            }
        }
    }

    pub fn write<T: Value>(s: &mut System, addr: AiLocation, data: T) {
        assert!(T::BYTES == 4, "AI: write with invalid size {}", T::BYTES);

        let offset = addr.relative() & REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "AI: write to unaligned address {:08X}",
            offset
        );

        match offset {
            // DMA RAM Address
            0x0 => {
                data.write_reg(slice::from_mut(&mut s.ai.dma_ram_address), offset & 3);
                s.ai.dma_ram_address &= 0x00FF_FFF8;
            }

            // DMA length: writes start a DMA transfer
            0x4 => {
                data.write_reg(slice::from_mut(&mut s.ai.dma_length), offset & 3);
                s.ai.dma_length &= 0x0003_FFF8;

                Self::push_dma(
                    s,
                    DmaSlot {
                        address: s.ai.dma_ram_address,
                        length: s.ai.dma_length,
                    },
                );
            }

            // Control
            0x8 => {
                let mut reg = 0;
                data.write_reg(slice::from_mut(&mut reg), offset & 3);
                s.ai.dma_enabled = reg & 1 == 1;

                // Mirror into Status

                s.ai.status.set_dma_enabled(s.ai.dma_enabled);
            }

            // Status: read-only,writes clear the AI interrupt
            0xC => {
                s.mi.clear_pending_interrupt(Interrupt::Ai, &mut s.cop0);
            }

            // Dac rate
            0x10 => {
                data.write_reg(slice::from_mut(&mut s.ai.dac_rate), offset & 3);
                s.ai.dac_rate &= 0x0003_FFFF;

                // Notify the audio renderer when the sample rate changes

                s.audio_renderer.set_sample_rate(s.ai.sample_rate());
            }

            // Bit rate
            0x14 => {
                // TODO?
            }

            _ => {
                log::warn!("AI: write to unknown register {:08X}", offset);
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        // TODO var in sp or something, also correct val?
        (62_500_000.0 / ((self.dac_rate.value() + 1) as f64)) as u32
    }

    fn push_dma(s: &mut System, slot: DmaSlot) {
        if s.ai.pending_dma.is_some() {
            log::warn!("AI: DMA queue full");
        }
        // Active DMA transfer: queue
        else if s.ai.active_dma.is_some() {
            s.ai.pending_dma = Some(slot);
            s.ai.status.set_dma_full(true);
        }
        // No active DMA transfer: execute
        else {
            s.ai.active_dma = Some(slot);
            s.ai.status.set_dma_busy(true);

            Self::start_dma(s, slot);
        }
    }

    fn start_dma(s: &mut System, slot: DmaSlot) {
        // Do nothing if DMA is not enabled

        if !s.ai.dma_enabled {
            return;
        }

        // log::info!(
        //     "AI: DMA {:X} bytes from RAM {:08X}",
        //     slot.length.raw_value,
        //     slot.address.raw_value,
        // );

        // Push RAM data to the audio renderer

        s.ram.read_block(
            RamLocation::from_absolute(slot.address.value()),
            slot.length.value() as usize,
            |ram_data| {
                s.audio_renderer.push(ram_data);
            },
        );

        // Schedule completion

        let samples = (slot.length.value() / 4) as f64; // 16-bit stereo = 4 bytes per sample
        let cycles = ((samples / (s.ai.sample_rate() as f64)) * 93_750_000.0) as usize; // TODO correct cycle unit?

        Events::push(s, EventType::AiDmaTransferComplete, cycles);
    }

    pub fn dma_completed(s: &mut System) {
        assert!(
            s.ai.active_dma.is_some(),
            "AI DMA: completed, but no active transfer"
        );

        assert!(s.ai.status.dma_busy(), "AI DMA: completed, but not busy");

        // Raise an AI interrupt

        s.mi.set_pending_interrupt(Interrupt::Ai, &mut s.cop0);

        // Switch to the pending DMA, if any

        s.ai.active_dma = s.ai.pending_dma.take();

        if let Some(slot) = s.ai.active_dma {
            s.ai.status.set_dma_full(false);

            Self::start_dma(s, slot);
        } else {
            s.ai.status.set_dma_busy(false);
        }
    }
}
