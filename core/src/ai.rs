use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::{
    events::{EventType, Events},
    location::Location,
    mi::Interrupt,
    ram::RamLocation,
    register_overlaps,
    system::System,
    value::Value,
};

/// Audio interface
///
/// https://n64brew.dev/wiki/Audio_Interface
/// TODO doc

pub type AiLocation = Location<0x0450_0000, 0x0460_0000>;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaRamAddress {
    #[bits(0..=23, rw)]
    value: u24,
}

const DMA_RAM_ADDRESS_MASK: u32 = 0x00FF_FFF8; // Low bits can be written but are ignored?

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaLength {
    #[bits(0..=17, rw)]
    value: u18,
}

const DMA_LENGTH_MASK: u32 = 0x0003_FFF8;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Control {
    #[bit(0, rw)]
    dma_enabled: bool,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Status {
    #[bit(0, rw)]
    full_mirror: bool,

    #[bits(1..=14, rw)]
    count: u14,

    #[bit(16, rw)]
    bit_clock: bool,

    #[bit(19, rw)]
    word_clock: bool,

    #[bit(25, rw)]
    dma_enabled: bool,

    #[bit(30, rw)]
    dma_busy: bool,

    #[bit(31, rw)]
    dma_full: bool,
}

// Bits 24 and 20 stay set, bits 17 and 18 stay cleared
const STATUS_DEFAULT_MASK: u32 = 0x0116_0000;
const STATUS_DEFAULT_BITS: u32 = 0x0110_0000;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DacRate {
    #[bits(0..=13, rw)]
    value: u14,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct BitRate {
    #[bits(0..=3, rw)]
    rate: u4,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Registers {
    pub ram_address: DmaRamAddress,
    pub length: DmaLength,
    pub control: Control,
    pub status: Status,
    pub dac_rate: DacRate,
    pub bit_rate: BitRate,
}

const REGISTERS_MASK: u32 = 0x1F; // TODO not exactly correct

impl Registers {
    pub fn read<T: Value>(&self, offset: u32) -> T {
        let words = bytemuck::cast_slice(bytemuck::bytes_of(self));

        T::read_reg(words, offset)

        // TODO All the other registers mirror LENGTH
    }

    pub fn write<T: Value>(&mut self, offset: u32, data: T) {
        let mut words = bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(self));

        data.write_reg(&mut words, offset);

        // Mask out read-only bits

        self.ram_address.raw_value &= DMA_RAM_ADDRESS_MASK;

        self.length.raw_value &= DMA_LENGTH_MASK;

        self.status.raw_value &= !STATUS_DEFAULT_MASK;
        self.status.raw_value |= STATUS_DEFAULT_BITS;

        // The DMA_ENABLE flag is mirrored in bit 25 of STATUS

        self.status.set_dma_enabled(self.control.dma_enabled());
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct DmaSlot {
    address: DmaRamAddress,
    length: DmaLength,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Ai {
    regs: Registers,

    active_dma: Option<DmaSlot>,
    pending_dma: Option<DmaSlot>,
}

impl Ai {
    pub fn regs(&self) -> &Registers {
        &self.regs
    }

    pub fn read<T: Value>(s: &System, addr: AiLocation) -> T {
        s.ai.regs.read(addr.relative() & REGISTERS_MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: AiLocation, data: T) {
        let offset = addr.relative() & REGISTERS_MASK; // TODO how does mirroring work? not aligned

        s.ai.regs.write(offset, data);

        // Writing to the length register starts a DMA transfer

        if register_overlaps!(offset, offset + T::BYTES as u32, Registers::length) {
            Self::push_dma(s);
        }

        // Writing to the status register clears the AI interrupt

        if register_overlaps!(offset, offset + T::BYTES as u32, Registers::status) {
            s.mi.clear_pending_interrupt(Interrupt::Ai, &mut s.cop0);
        }

        // Notify the audio renderer when the sample rate changes

        if register_overlaps!(offset, offset + T::BYTES as u32, Registers::dac_rate) {
            s.audio_renderer.set_sample_rate(s.ai.sample_rate());
        }
    }

    // TODO use it!
    pub fn dma_enabled(&self) -> bool {
        self.regs.control.dma_enabled()
    }

    pub fn sample_rate(&self) -> u32 {
        // TODO var in sp or something, also correct val?
        (62_500_000.0 / ((self.regs.dac_rate.raw_value + 1) as f64)) as u32
    }

    fn push_dma(s: &mut System) {
        if s.ai.pending_dma.is_some() {
            log::warn!("AI: DMA transfer already pending");
        }
        // Active DMA transfer: queue
        else if s.ai.active_dma.is_some() {
            s.ai.pending_dma = Some(DmaSlot {
                address: s.ai.regs.ram_address,
                length: s.ai.regs.length,
            });

            s.ai.regs.status.set_dma_full(true);
        }
        // No active DMA transfer: execute
        else {
            let slot = DmaSlot {
                address: s.ai.regs.ram_address,
                length: s.ai.regs.length,
            };

            s.ai.active_dma = Some(slot);

            // TODO ENABLED?

            Self::start_dma(s, slot);
        }
    }

    fn start_dma(s: &mut System, slot: DmaSlot) {
        // log::info!(
        //     "AI: DMA {:X} bytes from RAM {:08X}",
        //     slot.length.raw_value,
        //     slot.address.raw_value,
        // );

        // Push RAM data to the audio renderer

        s.ram.read_block(
            RamLocation::from_absolute(slot.address.raw_value),
            slot.length.raw_value as usize,
            |ram_data| {
                s.audio_renderer.push(ram_data);
            },
        );

        // Update the status register

        s.ai.regs.status.set_dma_busy(true);

        // Schedule completion

        let samples = (slot.length.raw_value / 4) as f64; // 16-bit stereo = 4 bytes per sample
        let cycles = ((samples / (s.ai.sample_rate() as f64)) * 93_750_000.0) as usize; // TODO correct cycle unit?

        Events::push(s, EventType::AiDmaTransferComplete, cycles);
    }

    pub fn dma_completed(s: &mut System) {
        debug_assert!(s.ai.active_dma.is_some(), "AI DMA transfer not in progress");

        s.ai.regs.status.set_dma_busy(false);
        s.ai.regs.status.set_dma_full(false);

        // Start the pending DMA, if any

        s.ai.active_dma = s.ai.pending_dma.take();

        if let Some(slot) = s.ai.active_dma {
            Self::start_dma(s, slot);
        }

        // Raise an AI interrupt

        s.mi.set_pending_interrupt(Interrupt::Ai, &mut s.cop0);
    }
}
