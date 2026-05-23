use arbitrary_int::prelude::*;
use bitbybit::bitfield;

pub trait Instruction {
    fn encode(&self) -> u32;
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0x0000_0024, debug)]
pub struct And {
    #[bits(21..=25, rw)]
    rs: u5,

    #[bits(16..=20, rw)]
    rt: u5,

    #[bits(11..=15, rw)]
    rd: u5,
}

impl Instruction for Ori {
    fn encode(&self) -> u32 {
        self.raw_value()
    }
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0x3400_0000, debug)]
pub struct Ori {
    #[bits(21..=25, rw)]
    rs: u5,

    #[bits(16..=20, rw)]
    rt: u5,

    #[bits(0..=15, rw)]
    immediate: u16,
}

impl Instruction for And {
    fn encode(&self) -> u32 {
        self.raw_value()
    }
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0xAC00_0000, debug)]
pub struct Sw {
    #[bits(21..=25, rw)]
    base: u5,

    #[bits(16..=20, rw)]
    rt: u5,

    #[bits(0..=15, rw)]
    offset: u16,
}

impl Instruction for Sw {
    fn encode(&self) -> u32 {
        self.raw_value()
    }
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0x3C00_0000, debug)]
pub struct Lui {
    #[bits(16..=20, rw)]
    rt: u5,

    #[bits(0..=15, rw)]
    immediate: u16,
}

impl Instruction for Lui {
    fn encode(&self) -> u32 {
        self.raw_value()
    }
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0x0000_0008, debug)]
pub struct Jr {
    #[bits(21..=25, rw)]
    rs: u5,
}

impl Instruction for Jr {
    fn encode(&self) -> u32 {
        self.raw_value()
    }
}
