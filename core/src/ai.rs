use strum::{Display, EnumIter};

use crate::{
    cpu,
    data::Value,
    events::{EventType, Events},
    location::Location,
    mi::Interrupt,
    system::System,
};

/// Audio interface
///
/// TODO doc

const START: u32 = 0x0450_0000;
const END: u32 = 0x0460_0000;

pub type AiLocation = Location<START, END>;

// TODO generally, what's faster? match get optimized? compute index from >> 2?
const REG_MASK: u32 = 0x1F;

const DRAM_ADDR_REG: u32 = 0;

const LENGTH_REG: u32 = 1;

const CONTROL_REG: u32 = 2;

const STATUS_REG: u32 = 3;
const STATUS_DMA_BUSY_MASK: u32 = 1 << 30;

const DACRATE_REG: u32 = 4;

const BITRATE_REG: u32 = 5;

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Register {
    DramAddress = DRAM_ADDR_REG,
    Length = LENGTH_REG,
    Control = CONTROL_REG,
    Status = STATUS_REG,
    DacRate = DACRATE_REG,
    BitRate = BITRATE_REG,
}

#[derive(Default, Clone, Copy)]
pub struct Ai {
    pub regs: [u32; 6], // TODO not pub
}

// TODO ENABLE FLAG???

impl Ai {
    pub fn read<T: Value>(s: &System, addr: AiLocation) -> T {
        match (addr.relative() >> 2) & REG_MASK {
            STATUS_REG => T::read_reg(&s.ai.regs, addr.relative() & REG_MASK),

            // All the other registers mirror LENGTH
            _ => T::read_reg(&s.ai.regs, LENGTH_REG + (addr.relative() & 3)),
        }
    }

    pub fn write<T: Value>(s: &mut System, addr: AiLocation, data: T) {
        // TODO possible to write mult regs???
        debug_assert!(T::BYTES <= 4, "Writing to multiple AI registers");

        match (addr.relative() >> 2) & REG_MASK {
            DRAM_ADDR_REG => {
                data.write_reg(&mut s.ai.regs, addr.relative() & REG_MASK);

                s.ai.regs[DRAM_ADDR_REG as usize] &= 0x00FF_FFF8;
            }
            LENGTH_REG => {
                data.write_reg(&mut s.ai.regs, addr.relative() & REG_MASK);

                s.ai.regs[LENGTH_REG as usize] &= 0x0003_FFFF;

                // TODO depends on DMA_ENABLE???
                Self::start_dma(s);
            }
            CONTROL_REG => {
                data.write_reg(&mut s.ai.regs, addr.relative() & REG_MASK);
            }
            STATUS_REG => {
                // Writing any value acknowledges the interrupt

                s.mi.clear_pending_interrupt(Interrupt::Ai, &mut s.cop0);
            }
            DACRATE_REG => {
                data.write_reg(&mut s.ai.regs, addr.relative() & REG_MASK);

                s.ai.regs[DACRATE_REG as usize] &= 0x0000_3FFF;
            }
            BITRATE_REG => {
                data.write_reg(&mut s.ai.regs, addr.relative() & REG_MASK);

                s.ai.regs[BITRATE_REG as usize] &= 0x0000_000F;
            }
            _ => panic!(
                "Invalid AI register write: {:08X} {:X}",
                addr.relative(),
                data
            ),
        }
    }

    pub fn sample_rate(&self) -> usize {
        (cpu::FREQUENCY / ((self.regs[DACRATE_REG as usize] + 1) as f64)) as usize
    }

    fn start_dma(s: &mut System) {
        // TODO actually do something

        // log::info!(
        //     "AI DMA transfer: {} bytes from {:08X}",
        //     s.ai.regs[LENGTH_REG as usize],
        //     s.ai.regs[DRAM_ADDR_REG as usize]
        // );

        // Update the status register

        s.ai.regs[STATUS_REG as usize] |= STATUS_DMA_BUSY_MASK;
        // TODO enabled mirror?
        // TODO others

        // Raise the interrupt when starting the transfer
        // TODO when played instead? depnding on rate?

        s.mi.set_pending_interrupt(Interrupt::Ai, &mut s.cop0);

        Events::push(
            s,
            EventType::AiDmaTransferComplete,
            1000000, // TODO random lol
        );
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.ai.regs[STATUS_REG as usize] &= !STATUS_DMA_BUSY_MASK;
        // TODO IO busy?
    }

    pub fn reg_info(addr: AiLocation) -> Option<&'static str> {
        // TODO mask?
        match (addr.relative() >> 2) & REG_MASK {
            DRAM_ADDR_REG => Some("AI_DRAM_ADDR"),
            LENGTH_REG => Some("AI_LENGTH"),
            CONTROL_REG => Some("AI_CONTROL"),
            STATUS_REG => Some("AI_STATUS"),
            DACRATE_REG => Some("AI_DACRATE"),
            BITRATE_REG => Some("AI_BITRATE"),
            _ => None,
        }
    }
}
