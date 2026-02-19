use strum::{Display, EnumIter};

use crate::{
    data::Data,
    events::{Event, EventType},
    map::Location,
    mi::Interrupt,
    system::System,
};

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

#[derive(Clone, Copy)]
pub struct Ai {
    pub regs: [u32; 6], // TODO not pub
}

impl Default for Ai {
    fn default() -> Self {
        Self {
            regs: [0, 0, 0, 0, 0, 0],
        }
    }
}

// TODO ENABLE FLAG???

impl Ai {
    pub fn read<T: Data>(&self, addr: AiLocation) -> T {
        match (addr.relative() >> 2) & REG_MASK {
            STATUS_REG => T::from_u32(self.regs[STATUS_REG as usize]),

            // All the other registers mirror LENGTH
            _ => T::from_u32(self.regs[Register::Length as usize]),
        }
    }

    pub fn write<T: Data>(s: &mut System, addr: AiLocation, data: T) {
        match (addr.relative() >> 2) & REG_MASK {
            DRAM_ADDR_REG => {
                s.map.ai.regs[DRAM_ADDR_REG as usize] = data.to_u32() & 0x00FF_FFF8;
            }
            LENGTH_REG => {
                s.map.ai.regs[LENGTH_REG as usize] = data.to_u32() & 0x0003_FFFF;

                // TODO depends on DMA_ENABLE???
                Self::start_dma(s);
            }
            CONTROL_REG => {
                s.map.ai.regs[CONTROL_REG as usize] = data.to_u32();
            }
            STATUS_REG => {
                // Writing any value acknowledges the interrupt
                s.map.mi.clear_pending_interrupt(Interrupt::Ai);
            }
            DACRATE_REG => {
                s.map.ai.regs[DACRATE_REG as usize] = data.to_u32() & 0x0000_3FFF;
            }
            BITRATE_REG => {
                s.map.ai.regs[BITRATE_REG as usize] = data.to_u32() & 0x0000_000F;
            }
            _ => panic!(
                "Invalid AI register write: {:08X} {:X}",
                addr.relative(),
                data
            ),
        }
    }

    fn start_dma(s: &mut System) {
        // TODO actually do something

        log::info!(
            "AI DMA transfer: {} bytes from {:08X}",
            s.map.ai.regs[LENGTH_REG as usize],
            s.map.ai.regs[DRAM_ADDR_REG as usize]
        );

        // Update the status register

        s.map.ai.regs[STATUS_REG as usize] |= STATUS_DMA_BUSY_MASK;
        // TODO enabled mirror?
        // TODO others

        // Raise the interrupt when starting the transfer

        s.map.mi.set_pending_interrupt(Interrupt::Ai);

        s.events.push(Event {
            id: EventType::AiDmaTransferComplete,
            cycle: s.cycles + 10000, // TODO random lol
        });
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.map.ai.regs[STATUS_REG as usize] &= !STATUS_DMA_BUSY_MASK;
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
