use crate::{data::Data, map::Location, system::System};

pub const START: u32 = 0x0450_0000;
pub const SIZE: u32 = 0x10_0000;
pub const END: u32 = START + SIZE;

pub type AiLocation = Location<START, END>;

pub const MASK: u32 = 0x1F;

const DRAM_ADDR_REG: usize = 0;
const LENGTH_REG: usize = 1;
const CONTROL_REG: usize = 2;
const STATUS_REG: usize = 3;
const DACRATE_REG: usize = 4;
const BITRATE_REG: usize = 5;

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

impl Ai {
    pub fn read<T: Data>(&self, addr: AiLocation) -> T {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            _ => panic!("Invalid AI register read: {:08X}", reg),
        }
    }

    pub fn write<T: Data>(_s: &mut System, addr: AiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            CONTROL_REG => {
                // Mirror of LENGTH register TODO in read
                log::warn!("Write AI_CONTROL {:X}", data.to_u32());
            }
            STATUS_REG => {
                // TODO ack interrupt
                log::warn!("Write AI_STATUS {:X}", data.to_u32());
            }
            DACRATE_REG => {
                // Mirror of LENGTH register TODO in read
                log::warn!("Write AI_DACRATE {:X}", data.to_u32());
            }
            BITRATE_REG => {
                // Mirror of LENGTH register TODO in read
                log::warn!("Write AI_BITRATE {:X}", data.to_u32());
            }
            _ => panic!(
                "Invalid AI register write: {:08X} {:X} {:X}",
                addr.relative(),
                data,
                reg
            ),
        }
    }

    pub fn reg_info(addr: AiLocation) -> Option<&'static str> {
        // TODO mask?
        match addr.relative() >> 2 {
            0 => Some("AI_DRAM_ADDR"),
            1 => Some("AI_LENGTH"),
            2 => Some("AI_CONTROL"),
            3 => Some("AI_STATUS"),
            4 => Some("AI_DACRATE"),
            5 => Some("AI_BITRATE"),
            _ => None,
        }
    }
}
