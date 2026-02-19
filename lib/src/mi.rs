use strum::{Display, EnumIter};

use crate::{data::Data, map::Location, system::System};

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Interrupt {
    Sp = 1,
    Si = 1 << 1,
    Ai = 1 << 2,
    Vi = 1 << 3,
    Pi = 1 << 4,
    Dp = 1 << 5,
}

const START: u32 = 0x0430_0000;
const END: u32 = 0x0440_0000;

pub type MiLocation = Location<START, END>;

const MASK: u32 = 0xF;

#[derive(Display, EnumIter)]
#[repr(u32)]
pub enum Register {
    Mode,
    Version,
    Interrupt,
    Mask,
}

const MODE_READ_REPEAT_COUNT_MASK: u32 = 0x7F;
const MODE_READ_REPEAT_MASK: u32 = 1 << 7;
const MODE_READ_EBUS_MASK: u32 = 1 << 8;
const MODE_READ_UPPER_MASK: u32 = 1 << 9;

const MODE_WRITE_REPEAT_CLEAR_MASK: u32 = 1 << 7;
const MODE_WRITE_REPEAT_SET_MASK: u32 = 1 << 8;
const MODE_WRITE_EBUS_CLEAR_MASK: u32 = 1 << 9;
const MODE_WRITE_EBUS_SET_MASK: u32 = 1 << 10;
const MODE_WRITE_DP_CLEAR_MASK: u32 = 1 << 11;
const MODE_WRITE_UPPER_CLEAR_MASK: u32 = 1 << 12;
const MODE_WRITE_UPPER_SET_MASK: u32 = 1 << 13;

const VERSION_DEFAULT: u32 = 0x02020102;

const MASK_SP_CLEAR: u32 = 1;
const MASK_SP_SET: u32 = 1 << 1;
const MASK_SI_CLEAR: u32 = 1 << 2;
const MASK_SI_SET: u32 = 1 << 3;
const MASK_AI_CLEAR: u32 = 1 << 4;
const MASK_AI_SET: u32 = 1 << 5;
const MASK_VI_CLEAR: u32 = 1 << 6;
const MASK_VI_SET: u32 = 1 << 7;
const MASK_PI_CLEAR: u32 = 1 << 8;
const MASK_PI_SET: u32 = 1 << 9;
const MASK_DP_CLEAR: u32 = 1 << 10;
const MASK_DP_SET: u32 = 1 << 11;

#[derive(Clone, Copy)]
pub struct Mi {
    pub regs: [u32; 4], // TODO not pub
}

impl Default for Mi {
    fn default() -> Self {
        Self {
            regs: [0, VERSION_DEFAULT, 0, 0],
        }
    }
}

impl Mi {
    pub fn read<T: Data>(&self, addr: MiLocation) -> T {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        // TODO mask stuff? or jsut access directly w/o match?
        match reg {
            0 => T::from_u32(self.regs[Register::Mode as usize]),
            1 => T::from_u32(self.regs[Register::Version as usize]),
            2 => T::from_u32(self.regs[Register::Interrupt as usize]),
            3 => T::from_u32(self.regs[Register::Mask as usize]),
            _ => panic!("Invalid MI register read: {:08X}", reg),
        }
    }

    pub fn write<T: Data>(s: &mut System, addr: MiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            0 => {
                let reg = &mut s.map.mi.regs[Register::Mode as usize];

                let data = data.to_u32();

                *reg = *reg & !MODE_READ_REPEAT_COUNT_MASK | data & MODE_READ_REPEAT_COUNT_MASK;

                if data & MODE_WRITE_REPEAT_CLEAR_MASK != 0 {
                    *reg &= !MODE_WRITE_REPEAT_CLEAR_MASK;
                }
                if data & MODE_WRITE_REPEAT_SET_MASK != 0 {
                    *reg |= MODE_WRITE_REPEAT_SET_MASK;
                }

                if data & MODE_WRITE_EBUS_CLEAR_MASK != 0 {
                    *reg &= !MODE_WRITE_EBUS_CLEAR_MASK;
                }
                if data & MODE_WRITE_EBUS_SET_MASK != 0 {
                    *reg |= MODE_WRITE_EBUS_SET_MASK;
                }

                if data & MODE_WRITE_DP_CLEAR_MASK != 0 {
                    *reg &= !(Interrupt::Dp as u32);
                }

                if data & MODE_WRITE_UPPER_CLEAR_MASK != 0 {
                    *reg &= !MODE_WRITE_UPPER_CLEAR_MASK;
                }
                if data & MODE_WRITE_UPPER_SET_MASK != 0 {
                    *reg |= MODE_WRITE_UPPER_SET_MASK;
                }
            }
            1 => {}
            2 => {
                log::warn!("Write MI_INTERRUPT {:X}", data.to_u32());
            }
            3 => {
                let data = data.to_u32();

                if data & MASK_SP_CLEAR == MASK_SP_CLEAR {
                    s.map.mi.regs[Register::Mask as usize] &= !(Interrupt::Sp as u32);
                }
                if data & MASK_SP_SET == MASK_SP_SET {
                    s.map.mi.regs[Register::Mask as usize] |= Interrupt::Sp as u32;
                }

                if data & MASK_SI_CLEAR == MASK_SI_CLEAR {
                    s.map.mi.regs[Register::Mask as usize] &= !(Interrupt::Si as u32);
                }
                if data & MASK_SI_SET == MASK_SI_SET {
                    s.map.mi.regs[Register::Mask as usize] |= Interrupt::Si as u32;
                }

                if data & MASK_AI_CLEAR == MASK_AI_CLEAR {
                    s.map.mi.regs[Register::Mask as usize] &= !(Interrupt::Ai as u32);
                }
                if data & MASK_AI_SET == MASK_AI_SET {
                    s.map.mi.regs[Register::Mask as usize] |= Interrupt::Ai as u32;
                }

                if data & MASK_VI_CLEAR == MASK_VI_CLEAR {
                    s.map.mi.regs[Register::Mask as usize] &= !(Interrupt::Vi as u32);
                }
                if data & MASK_VI_SET == MASK_VI_SET {
                    s.map.mi.regs[Register::Mask as usize] |= Interrupt::Vi as u32;
                }
                //s.map.mi.regs[MASK_REG] |= Interrupt::Vi as u32; /////////////s

                if data & MASK_PI_CLEAR == MASK_PI_CLEAR {
                    s.map.mi.regs[Register::Mask as usize] &= !(Interrupt::Pi as u32);
                }
                if data & MASK_PI_SET == MASK_PI_SET {
                    s.map.mi.regs[Register::Mask as usize] |= Interrupt::Pi as u32;
                }

                if data & MASK_DP_CLEAR == MASK_DP_CLEAR {
                    s.map.mi.regs[Register::Mask as usize] &= !(Interrupt::Dp as u32);
                }
                if data & MASK_DP_SET == MASK_DP_SET {
                    s.map.mi.regs[Register::Mask as usize] |= Interrupt::Dp as u32;
                }
            }
            _ => panic!(
                "Invalid MI register write: {:08X} {:X} {:X}",
                addr.relative(),
                data,
                reg
            ),
        }
    }

    pub fn reg_info(addr: MiLocation) -> Option<&'static str> {
        // TODO mask?
        match addr.relative() >> 2 {
            0 => Some("MI_MODE"),
            1 => Some("MI_VERSION"),
            2 => Some("MI_INTERRUPT"),
            3 => Some("MI_MASK"),
            _ => None,
        }
    }

    // MODE

    pub fn upper_mode(&self) -> bool {
        self.regs[Register::Mode as usize] & MODE_READ_UPPER_MASK != 0
    }

    pub fn ebus_mode(&self) -> bool {
        self.regs[Register::Mode as usize] & MODE_READ_EBUS_MASK != 0
    }

    pub fn repeat_mode(&self) -> bool {
        self.regs[Register::Mode as usize] & MODE_READ_REPEAT_MASK != 0
    }

    pub fn repeat_count(&self) -> u32 {
        self.regs[Register::Mode as usize] & MODE_READ_REPEAT_COUNT_MASK
    }

    // VERSION

    pub fn version(&self) -> Versions {
        let version = self.regs[Register::Version as usize];

        Versions {
            rsp: (version >> 24) as u8,
            rdp: (version >> 16) as u8,
            rac: (version >> 8) as u8,
            io: version as u8,
        }
    }

    // INT_PENDING

    pub fn set_pending_interrupt(&mut self, interrupt: Interrupt) {
        self.regs[Register::Interrupt as usize] |= interrupt as u32;
    }

    pub fn clear_pending_interrupt(&mut self, interrupt: Interrupt) {
        self.regs[Register::Interrupt as usize] &= !(interrupt as u32);
    }

    pub fn has_pending_interrupt(&self, interrupt: Interrupt) -> bool {
        self.regs[Register::Interrupt as usize] & (interrupt as u32) != 0
    }

    // INT_MASK

    pub fn is_interrupt_masked(&self, interrupt: Interrupt) -> bool {
        self.regs[Register::Mask as usize] & (interrupt as u32) == 0
    }

    // TODO rename
    pub fn has_pending_unmasked_interrupt(&self) -> bool {
        self.regs[Register::Interrupt as usize] & self.regs[Register::Mask as usize] != 0
    }

    // TODO useless with enum?
    pub fn reg_name(index: usize) -> &'static str {
        const NAMES: [&str; 4] = ["MODE", "VERSION", "INT_PENDING", "INT_MASK"];

        NAMES.get(index).copied().unwrap_or("?") // TODO copied?
    }
}

pub struct Versions {
    pub rsp: u8,
    pub rdp: u8,
    pub rac: u8,
    pub io: u8,
}
