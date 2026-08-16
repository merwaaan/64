use alloc::vec::Vec;
use arbitrary_int::prelude::*;

use n64_specs::{
    cpu::registers::Register,
    rsp::{IMEM_START, instructions::*, registers::*},
};

use crate::io;

/// TODO doc
pub struct RspProgram {
    /// Program instructions.
    instructions: Vec<u32>,
}

impl RspProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.push(instruction.opcode());
        self
    }

    pub fn nop(&mut self) -> &mut Self {
        // NOP = SLL r0, r0, r0
        self.push(Sll::default().into())
    }

    pub fn clear_vec_regs(&mut self) -> &mut Self {
        for i in 0..8 {
            let vreg = u5::from_u8(i);

            self.push(
                Vxor::default()
                    .with_vt(vreg)
                    .with_vs(vreg)
                    .with_vd(vreg)
                    .into(),
            );
        }
        self
    }

    pub fn run(&self) {
        // Wait for the RSP to halt, in case it was running

        loop {
            let status = Status::new_with_raw_value(io::read_uncached::<u32>(Status::ADDRESS));

            if status.halted() {
                break;
            }
        }

        // Copy the program to IMEM + add a final break

        for (i, opcode) in self.instructions.iter().enumerate() {
            io::write_uncached(IMEM_START + (i as u32 * 4), *opcode);
        }

        io::write_uncached(
            IMEM_START + (self.instructions.len() as u32 * 4),
            Break::default().raw_value(),
        );

        // Start the program

        io::write_uncached(
            Status::ADDRESS,
            StatusWrite::default()
                .with_clear_interrupt_on_break(true)
                .with_clear_halt(true)
                .with_clear_broke(true)
                .with_clear_sig0(true)
                .with_clear_sig1(true)
                .with_clear_sig2(true)
                .with_clear_sig3(true)
                .with_clear_sig4(true)
                .with_clear_sig5(true)
                .with_clear_sig6(true)
                .with_clear_sig7(true),
        );

        // Wait for the program to finish

        loop {
            let status = Status::new_with_raw_value(io::read_uncached::<u32>(Status::ADDRESS));

            if status.halted() {
                break;
            }
        }
    }

    /// Sets a 32-bit value into a register.
    pub fn set_reg(&mut self, reg: Register, value: u32) -> &mut Self {
        let lo = value as u16;
        let hi = (value >> 16) as u16;

        self.push(Lui::default().with_rt(reg.into()).with_imm(hi).into())
            .push(
                Ori::default()
                    .with_rt(reg.into())
                    .with_rs(reg.into())
                    .with_imm(lo)
                    .into(),
            )
    }
}
