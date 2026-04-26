use arbitrary_int::prelude::*;

use crate::{cop1, system::System};

// TODO rewrite with bitfiedstruct??

/// Instruction opcode
#[derive(Clone, Copy)]
pub struct Opcode(pub u32);

impl Opcode {
    // Group (special, regimm, cop0, cop1, or just top-level instructions)
    pub(crate) const fn group(&self) -> u32 {
        self.0 >> 26
    }

    pub(crate) const fn rs(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }

    pub(crate) const fn rt(&self) -> usize {
        ((self.0 >> 16) & 0x1F) as usize
    }

    pub(crate) const fn rd(&self) -> usize {
        ((self.0 >> 11) & 0x1F) as usize
    }

    pub(crate) const fn base(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }

    pub(crate) fn basev(&self, s: &System) -> u32 {
        s.cpu.regs.gpr[self.base()].get()
    }

    // Immediate 16-bits value
    pub(crate) const fn imm16(&self) -> u16 {
        self.0 as u16
    }

    // Address generated from base + immediate
    pub(crate) fn offset_addr(&self, s: &System) -> u32 {
        self.basev(s)
            .wrapping_add(self.imm16() as i16 as i32 as u32)
    }

    // Branch offset
    pub(crate) fn branch_offset(&self) -> u32 {
        (self.imm16() as i16 as i32 as u32) << 2
    }

    pub(crate) fn branch_target(&self, s: &System) -> u32 {
        s.cpu
            .regs
            .pc
            .wrapping_add(4)
            .wrapping_add(self.branch_offset())
    }

    // Shift amount
    pub(crate) const fn shift(&self) -> u32 {
        (self.0 >> 6) & 0x1F
    }

    // COP1

    pub(crate) fn cop1_format(&self) -> Option<cop1::Format> {
        cop1::Format::new_with_raw_value(u5::new(((self.0 >> 21) & 0x1F) as u8)).ok()
    }

    pub(crate) fn cop1_condition(&self) -> cop1::Condition {
        // TODO move to cop1
        match (self.0 >> 16) & 0x1F {
            0x00 => cop1::Condition::False,
            0x01 => cop1::Condition::True,
            0x02 => cop1::Condition::FalseLikely,
            0x03 => cop1::Condition::TrueLikely,
            _ => unreachable!(),
        }
    }

    pub(crate) fn cop1_comparison(&self) -> cop1::Comparison {
        cop1::Comparison::new_with_raw_value(u4::new((self.0 & 0xF) as u8))
    }
}
