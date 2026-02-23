use crate::cop0::Cop0;
use crate::registers::Registers;
use crate::system::System;

/// Helper to decode opcodes
#[derive(Clone, Copy)]
pub struct Opcode(pub u32);

impl Opcode {
    // Group (special, regimm, cop0, cop1, or just top-level instructions)
    pub(crate) fn group(&self) -> u32 {
        self.0 >> 26
    }

    // x -> register index
    // xv -> register value
    // xn -> register name
    // x0n -> COP0 register name TODO weird?

    // rs

    pub(crate) fn rs(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }

    pub(crate) fn rsv(&self, s: &System) -> u32 {
        s.cpu.regs.gpr[self.rs()].get()
    }

    pub(crate) fn rsv64(&self, s: &System) -> u64 {
        s.cpu.regs.gpr[self.rs()].get64()
    }

    pub(crate) fn rsn(&self) -> &'static str {
        Registers::gpr_name(self.rs())
    }

    // rt

    pub(crate) fn rt(&self) -> usize {
        ((self.0 >> 16) & 0x1F) as usize
    }

    pub(crate) fn rtv(&self, s: &System) -> u32 {
        s.cpu.regs.gpr[self.rt()].get()
    }

    pub(crate) fn rtv64(&self, s: &System) -> u64 {
        s.cpu.regs.gpr[self.rt()].get64()
    }

    pub(crate) fn rtn(&self) -> &'static str {
        Registers::gpr_name(self.rt())
    }

    // rd

    pub(crate) fn rd(&self) -> usize {
        ((self.0 >> 11) & 0x1F) as usize
    }

    pub(crate) fn rdn(&self) -> &'static str {
        Registers::gpr_name(self.rd())
    }

    pub(crate) fn rd0n(&self) -> &'static str {
        Cop0::reg_name(self.rd())
    }

    // fs

    pub(crate) fn fs(&self) -> usize {
        ((self.0 >> 11) & 0x1F) as usize
    }

    pub(crate) fn fsv(&self, s: &System) -> u32 {
        s.cpu.regs.fpr[self.fs()].get()
    }

    pub(crate) fn fsn(&self) -> &'static str {
        Registers::fpr_name(self.fs())
    }

    // base

    pub(crate) fn base(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }

    pub(crate) fn basev(&self, s: &System) -> u32 {
        s.cpu.regs.gpr[self.base()].get()
    }

    pub(crate) fn basen(&self) -> &'static str {
        Registers::gpr_name(self.base())
    }

    // Immediate 16-bits value
    pub(crate) fn imm16(&self) -> u16 {
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
    pub(crate) fn shift(&self) -> u32 {
        (self.0 >> 6) & 0x1F
    }
}
