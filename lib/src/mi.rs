use strum::{Display, EnumIter};

use crate::{
    cop0::{self, Cop0},
    data::Value,
    interrupt::Interrupt,
    map::Location,
    system::System,
};

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
        let mut regs = [0; 4];

        VERSION_DEFAULT.write_reg(&mut regs, 4);

        Self { regs }
    }
}

impl Mi {
    pub fn read<T: Value>(&self, addr: MiLocation) -> T {
        T::read_reg(&self.regs, addr.relative() & MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: MiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            0 => {
                let mut trigger_bits = [0u32];
                data.write_reg(&mut trigger_bits, addr.relative() & 3);

                let mode_reg = &mut s.map.mi.regs[Register::Mode as usize];

                if trigger_bits[0] & MODE_WRITE_REPEAT_CLEAR_MASK != 0 {
                    *mode_reg &= !MODE_WRITE_REPEAT_CLEAR_MASK;
                } else if trigger_bits[0] & MODE_WRITE_REPEAT_SET_MASK != 0 {
                    *mode_reg |= MODE_WRITE_REPEAT_SET_MASK;
                }

                if trigger_bits[0] & MODE_WRITE_EBUS_CLEAR_MASK != 0 {
                    *mode_reg &= !MODE_WRITE_EBUS_CLEAR_MASK;
                } else if trigger_bits[0] & MODE_WRITE_EBUS_SET_MASK != 0 {
                    *mode_reg |= MODE_WRITE_EBUS_SET_MASK;
                }

                if trigger_bits[0] & MODE_WRITE_DP_CLEAR_MASK != 0 {
                    *mode_reg &= !(Interrupt::Dp as u32);
                }

                if trigger_bits[0] & MODE_WRITE_UPPER_CLEAR_MASK != 0 {
                    *mode_reg &= !MODE_WRITE_UPPER_CLEAR_MASK;
                } else if trigger_bits[0] & MODE_WRITE_UPPER_SET_MASK != 0 {
                    *mode_reg |= MODE_WRITE_UPPER_SET_MASK;
                }

                // TODO repeat count
            }
            1 => {
                // Not writable
            }
            2 => {
                log::warn!("Write MI_INTERRUPT {:X}", data);
            }
            3 => {
                log::warn!("Write MI_MASK {:X}", data);
                let mut trigger_bits = [0u32];
                data.write_reg(&mut trigger_bits, addr.relative() & 3);

                let mask_reg = &mut s.map.mi.regs[Register::Mask as usize];

                // TODO write without conds?

                if trigger_bits[0] & MASK_SP_CLEAR == MASK_SP_CLEAR {
                    *mask_reg &= !(Interrupt::Sp as u32);
                } else if trigger_bits[0] & MASK_SP_SET == MASK_SP_SET {
                    *mask_reg |= Interrupt::Sp as u32;
                }

                if trigger_bits[0] & MASK_SI_CLEAR == MASK_SI_CLEAR {
                    *mask_reg &= !(Interrupt::Si as u32);
                } else if trigger_bits[0] & MASK_SI_SET == MASK_SI_SET {
                    *mask_reg |= Interrupt::Si as u32;
                }

                if trigger_bits[0] & MASK_AI_CLEAR == MASK_AI_CLEAR {
                    *mask_reg &= !(Interrupt::Ai as u32);
                } else if trigger_bits[0] & MASK_AI_SET == MASK_AI_SET {
                    *mask_reg |= Interrupt::Ai as u32;
                }

                if trigger_bits[0] & MASK_VI_CLEAR == MASK_VI_CLEAR {
                    *mask_reg &= !(Interrupt::Vi as u32);
                } else if trigger_bits[0] & MASK_VI_SET == MASK_VI_SET {
                    *mask_reg |= Interrupt::Vi as u32;
                }

                if trigger_bits[0] & MASK_PI_CLEAR == MASK_PI_CLEAR {
                    *mask_reg &= !(Interrupt::Pi as u32);
                } else if trigger_bits[0] & MASK_PI_SET == MASK_PI_SET {
                    *mask_reg |= Interrupt::Pi as u32;
                }

                if trigger_bits[0] & MASK_DP_CLEAR == MASK_DP_CLEAR {
                    *mask_reg &= !(Interrupt::Dp as u32);
                } else if trigger_bits[0] & MASK_DP_SET == MASK_DP_SET {
                    *mask_reg |= Interrupt::Dp as u32;
                }

                Self::update_cause_register(&s.map.mi, &mut s.cop0);
            }
            _ => panic!(
                "Invalid MI register write: {:08X} {:X} {:X}",
                addr.relative(),
                data,
                reg
            ),
        }
    }

    /// Updates the CAUSE register when pending interrupts or masks change
    fn update_cause_register(mi: &Mi, cop0: &mut Cop0) {
        cop0.set_ip2_interrupt(mi.has_pending_unmasked_interrupt());
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

    // pub fn upper_mode(&self) -> bool {
    //     self.regs[Register::Mode as usize] & MODE_READ_UPPER_MASK != 0
    // }

    // pub fn ebus_mode(&self) -> bool {
    //     self.regs[Register::Mode as usize] & MODE_READ_EBUS_MASK != 0
    // }

    // pub fn repeat_mode(&self) -> bool {
    //     self.regs[Register::Mode as usize] & MODE_READ_REPEAT_MASK != 0
    // }

    // pub fn repeat_count(&self) -> u32 {
    //     self.regs[Register::Mode as usize] & MODE_READ_REPEAT_COUNT_MASK
    // }

    // VERSION

    // pub fn version(&self) -> Versions {
    //     let version = self.regs[4 * Register::Version as usize];

    //     Versions {
    //         rsp: (version >> 24) as u8,
    //         rdp: (version >> 16) as u8,
    //         rac: (version >> 8) as u8,
    //         io: version as u8,
    //     }
    // }

    // INT_PENDING

    pub fn set_pending_interrupt(&mut self, interrupt: Interrupt, cop0: &mut Cop0) {
        self.regs[Register::Interrupt as usize] |= interrupt as u32;

        Self::update_cause_register(self, cop0);
    }

    pub fn clear_pending_interrupt(&mut self, interrupt: Interrupt, cop0: &mut Cop0) {
        self.regs[Register::Interrupt as usize] &= !(interrupt as u32);

        Self::update_cause_register(self, cop0);
    }

    pub fn has_pending_interrupt(&self, interrupt: Interrupt) -> bool {
        self.regs[Register::Interrupt as usize] & (interrupt as u32) != 0
    }

    // INT_MASK

    pub fn is_interrupt_enabled(&self, interrupt: Interrupt) -> bool {
        self.regs[Register::Mask as usize] & (interrupt as u32) != 0
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
