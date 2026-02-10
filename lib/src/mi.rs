use crate::{data::Data, system::System};

#[derive(Debug)]
#[repr(u32)]
pub enum Interrupt {
    Sp = 1,
    Si = 1 << 1,
    Ai = 1 << 2,
    Vi = 1 << 3,
    Pi = 1 << 4,
    Dp = 1 << 5,
}

pub const START: u32 = 0x0430_0000;
pub const SIZE: u32 = 0x10_0000;
pub const END: u32 = START + SIZE;

pub const MASK: u32 = 0xF;

const MODE_REG: usize = 0;
const VERSION_REG: usize = 1;
const INT_PENDING_REG: usize = 2;
const INT_MASK_REG: usize = 3;

const MODE_DP_CLEAR: u32 = 1 << 11;

const VERSION_DEFAULT: u32 = 0x02020102;

const INT_MASK_SP_CLEAR: u32 = 1;
const INT_MASK_SP_SET: u32 = 1 << 1;
const INT_MASK_SI_CLEAR: u32 = 1 << 2;
const INT_MASK_SI_SET: u32 = 1 << 3;
const INT_MASK_AI_CLEAR: u32 = 1 << 4;
const INT_MASK_AI_SET: u32 = 1 << 5;
const INT_MASK_VI_CLEAR: u32 = 1 << 6;
const INT_MASK_VI_SET: u32 = 1 << 7;
const INT_MASK_PI_CLEAR: u32 = 1 << 8;
const INT_MASK_PI_SET: u32 = 1 << 9;
const INT_MASK_DP_CLEAR: u32 = 1 << 10;
const INT_MASK_DP_SET: u32 = 1 << 11;

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

// TODO set default version register?

impl Mi {
    pub fn read<T: Data>(&self, addr: u32) -> T {
        assert_range(addr);

        let reg = ((addr & MASK) >> 2) as usize;

        match reg {
            // MODE_REG => T::from_u32(self.regs[MODE_REG]),
            // VERSION_REG => T::from_u32(self.regs[VERSION_REG]),
            INT_PENDING_REG => T::from_u32(self.regs[INT_PENDING_REG]),
            INT_MASK_REG => T::from_u32(self.regs[INT_MASK_REG]),
            _ => panic!("Invalid MI register read: {:08X}", reg),
        }
    }

    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        assert_range(addr);

        let reg = ((addr & MASK) >> 2) as usize;

        match reg {
            MODE_REG => {
                log::warn!("write MI_MODE {:X}", data);

                if (data.to_u32() & MODE_DP_CLEAR) == MODE_DP_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Dp as u32);
                }
            }
            VERSION_REG => {}
            INT_PENDING_REG => {}
            INT_MASK_REG => {
                let data = data.to_u32();

                if data & INT_MASK_SP_CLEAR == INT_MASK_SP_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Sp as u32);
                }
                if data & INT_MASK_SP_SET == INT_MASK_SP_SET {
                    s.map.mi.regs[INT_MASK_REG] |= Interrupt::Sp as u32;
                }

                if data & INT_MASK_SI_CLEAR == INT_MASK_SI_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Si as u32);
                }
                if data & INT_MASK_SI_SET == INT_MASK_SI_SET {
                    s.map.mi.regs[INT_MASK_REG] |= Interrupt::Si as u32;
                }

                if data & INT_MASK_AI_CLEAR == INT_MASK_AI_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Ai as u32);
                }
                if data & INT_MASK_AI_SET == INT_MASK_AI_SET {
                    s.map.mi.regs[INT_MASK_REG] |= Interrupt::Ai as u32;
                }

                if data & INT_MASK_VI_CLEAR == INT_MASK_VI_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Vi as u32);
                }
                if data & INT_MASK_VI_SET == INT_MASK_VI_SET {
                    s.map.mi.regs[INT_MASK_REG] |= Interrupt::Vi as u32;
                }

                if data & INT_MASK_PI_CLEAR == INT_MASK_PI_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Pi as u32);
                }
                if data & INT_MASK_PI_SET == INT_MASK_PI_SET {
                    s.map.mi.regs[INT_MASK_REG] |= Interrupt::Pi as u32;
                }

                if data & INT_MASK_DP_CLEAR == INT_MASK_DP_CLEAR {
                    s.map.mi.regs[INT_MASK_REG] &= !(Interrupt::Dp as u32);
                }
                if data & INT_MASK_DP_SET == INT_MASK_DP_SET {
                    s.map.mi.regs[INT_MASK_REG] |= Interrupt::Dp as u32;
                }
            }
            _ => panic!("Invalid MI register write: {:08X} {:X}", addr, data),
        }
    }

    pub fn set_pending_interrupt(&mut self, interrupt: Interrupt) {
        self.regs[INT_PENDING_REG] |= interrupt as u32;
    }

    pub fn clear_pending_interrupt(&mut self, interrupt: Interrupt) {
        self.regs[INT_PENDING_REG] &= !(interrupt as u32);
    }

    pub fn check_interrupt(&self, interrupt: Interrupt) -> bool {
        self.regs[INT_PENDING_REG] & (interrupt as u32) > 0
    }

    // TODO rename
    pub fn has_pending_interrupt(&self) -> bool {
        self.regs[INT_PENDING_REG] & self.regs[INT_MASK_REG] != 0
    }

    pub fn check_interrupts(&self) {
        // if self.check_interrupt(Interrupt::Sp) {
        //     self.regs[INT_PENDING_REG] |= Interrupt::Sp as u32;
        // }
        // if self.check_interrupt(Interrupt::Si) {
        //     self.regs[INT_PENDING_REG] |= Interrupt::Si as u32;
        // }
    }
}

fn assert_range(addr: u32) {
    debug_assert!((START..END).contains(&addr));
}
