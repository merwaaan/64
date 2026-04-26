use crate::{cop0::Cop0, cpu::opcode::Opcode, registers::Registers, system::System};

/// Instruction operands extracted from an opcode
#[derive(Default, Clone, Copy)]
pub struct Operands {
    pub rd: u8,
    pub rt: u8,
    pub rs: u8,
    pub sa: u8,
    // TODO optim: rs/sa mutually exclusive so use the same slot? (would break from_opcode)
    // TODO union with other operands?
}

impl Operands {
    pub(crate) const fn default() -> Self {
        Self {
            rd: 0,
            rt: 0,
            rs: 0,
            sa: 0,
        }
    }

    pub(crate) const fn from_opcode(opcode: Opcode) -> Self {
        Self {
            rd: opcode.rd() as u8,
            rt: opcode.rt() as u8,
            rs: opcode.rs() as u8,
            sa: opcode.shift() as u8,
        }
    }

    // rd

    pub(crate) fn rd(&self) -> usize {
        self.rd as usize
    }

    pub(crate) fn rdn(&self) -> &'static str {
        Registers::gpr_name(self.rd())
    }

    pub(crate) fn rd0n(&self) -> &'static str {
        Cop0::reg_name(self.rd())
    }

    // rs

    pub(crate) fn rs(&self) -> usize {
        self.rs as usize
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
        self.rt as usize
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

    // fs (= rd)

    pub(crate) fn fs(&self) -> usize {
        self.rd()
    }

    pub(crate) fn fsv(&self, s: &System) -> u32 {
        s.cop1.get32(self.fs(), s.cop0.f64())
    }

    pub(crate) fn fsv64(&self, s: &System) -> u64 {
        s.cop1.get64(self.fs(), s.cop0.f64())
    }

    pub(crate) fn fsn(&self) -> &'static str {
        Registers::fpr_name(self.fs())
    }

    // ft (= rt)

    pub(crate) fn ft(&self) -> usize {
        self.rt()
    }

    pub(crate) fn ftn(&self) -> &'static str {
        Registers::fpr_name(self.ft())
    }

    // fd (= sa)

    pub(crate) fn fd(&self) -> usize {
        self.sa as usize
    }

    pub(crate) fn fdn(&self) -> &'static str {
        Registers::fpr_name(self.fd())
    }

    // base (= rs)

    // TODO rm?

    // pub(crate) fn ft(&self) -> usize {
    //     self.rt as usize
    // }

    // pub(crate) fn ftv(&self, s: &System) -> u32 {
    //     s.cop1.get32(self.ft(), s.cop0.f64())
    // }

    // pub(crate) fn ftv64(&self, s: &System) -> u64 {
    //     s.cop1.get64(self.ft(), s.cop0.f64())
    // }

    pub(crate) fn basen(&self) -> &'static str {
        Registers::gpr_name(self.rs())
    }

    // Shift

    pub(crate) fn shift(&self) -> u32 {
        self.sa as u32
    }
}
