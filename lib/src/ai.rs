use crate::{data::Data, system::System};

pub const START: u32 = 0x0450_0000;
pub const SIZE: u32 = 0x10_0000;
pub const END: u32 = START + SIZE;

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
    pub fn read<T: Data>(&self, addr: u32) -> T {
        assert_range(addr);

        let reg = ((addr & MASK) >> 2) as usize;

        match reg {
            _ => panic!("Invalid AI register read: {:08X}", reg),
        }
    }

    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        assert_range(addr);

        let reg = ((addr & MASK) >> 2) as usize;

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
                addr, data, reg
            ),
        }
    }
}

fn assert_range(addr: u32) {
    debug_assert!((START..END).contains(&addr));
}
