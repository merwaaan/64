use core::arch::asm;

use alloc::vec::Vec;
use arbitrary_int::prelude::*;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::io;

pub struct Program {
    instructions: Vec<u32>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.push(instruction.opcode());
        self
    }

    pub fn run(&self) {
        // Copy the program instructions to RAM

        // let mut program_buffer = io::Buffer::<u32>::with_alignment(self.instructions.len() + 2, 4);

        // for (i, opcode) in self.instructions.iter().enumerate() {
        //     program_buffer.set(i, *opcode);
        // }

        // Inject a JR at the end to return from the program

        // program_buffer.set(
        //     self.instructions.len(),
        //     Jr::default().with_rs(u5::from_u8(31)).encode(),
        // );
        // TODO NOP program_buffer.set(i, *opcode);

        // TODO save RA? for the return?

        // Jump to the program location

        //let entry = program_buffer.as_ptr() as u32;

        // TODO to ram!

        for (i, opcode) in self.instructions.iter().enumerate() {
            io::write_uncached(
                n64_specs::rsp::MEMORY_START + 0x1000 + (i as u32 * 4),
                *opcode,
            );
        }

        io::write_uncached(
            n64_specs::rsp::MEMORY_START + 0x1000 + (self.instructions.len() as u32 * 4),
            Jr::default().with_rs(u5::from_u8(31)).raw_value(),
        );

        let entry = io::uncached_ptr(n64_specs::rsp::MEMORY_START + 0x1000) as u32;

        unsafe {
            asm!(
                ".set noat",

                // Save the registers to the stack

                "addiu $sp, $sp, -256", // (32 regs - Zero - SP + HI + LO) * 8 bytes

                "sd  $1,   0($sp)",
                "sd  $2,   8($sp)",
                "sd  $3,  16($sp)",
                "sd  $4,  24($sp)",
                "sd  $5,  32($sp)",
                "sd  $6,  40($sp)",
                "sd  $7,  48($sp)",
                "sd  $8,  56($sp)",
                "sd  $9,  64($sp)",
                "sd $10,  72($sp)",
                "sd $11,  80($sp)",
                "sd $12,  88($sp)",
                "sd $13,  96($sp)",
                "sd $14, 104($sp)",
                "sd $15, 112($sp)",
                "sd $16, 120($sp)",
                "sd $17, 128($sp)",
                "sd $18, 136($sp)",
                "sd $19, 144($sp)",
                "sd $20, 152($sp)",
                "sd $21, 160($sp)",
                "sd $22, 168($sp)",
                "sd $23, 176($sp)",
                "sd $24, 184($sp)",
                "sd $25, 192($sp)",
                "sd $26, 200($sp)",
                "sd $27, 208($sp)",
                "sd $28, 216($sp)",
                "sd $30, 224($sp)",
                "sd $31, 232($sp)",

                "mfhi $31",
                "sd   $31, 240($sp)",
                "mflo $31",
                "sd   $31, 248($sp)",

                // Jump

                "jalr {entry}",
                "nop",

                // Restore the registers from the stack

                "ld  $1,   0($sp)",
                "ld  $2,   8($sp)",
                "ld  $3,  16($sp)",
                "ld  $4,  24($sp)",
                "ld  $5,  32($sp)",
                "ld  $6,  40($sp)",
                "ld  $7,  48($sp)",
                "ld  $8,  56($sp)",
                "ld  $9,  64($sp)",
                "ld $10,  72($sp)",
                "ld $11,  80($sp)",
                "ld $12,  88($sp)",
                "ld $13,  96($sp)",
                "ld $14, 104($sp)",
                "ld $15, 112($sp)",
                "ld $16, 120($sp)",
                "ld $17, 128($sp)",
                "ld $18, 136($sp)",
                "ld $19, 144($sp)",
                "ld $20, 152($sp)",
                "ld $21, 160($sp)",
                "ld $22, 168($sp)",
                "ld $23, 176($sp)",
                "ld $24, 184($sp)",
                "ld $25, 192($sp)",
                "ld $26, 200($sp)",
                "ld $27, 208($sp)",
                "ld $28, 216($sp)",
                "ld $30, 224($sp)",

                "ld   $31, 240($sp)",
                "mthi $31",
                "ld   $31, 248($sp)",
                "mtlo $31",
                "ld   $31, 232($sp)",

                "addiu $sp, $sp, 256",

                ".set at",

                entry = in(reg) entry
            );
        }
    }

    // Loads a value into a register (LUI + ORI).
    pub fn load_reg(&mut self, reg: Register, value: u32) -> &mut Self {
        self.push(
            Lui::default()
                .with_rt(reg.into())
                .with_imm((value >> 16) as u16)
                .into(),
        )
        .push(
            Ori::default()
                .with_rt(reg.into())
                .with_rs(reg.into())
                .with_imm(value as u16)
                .into(),
        )
    }

    pub fn and(&mut self, rd: Register, rs: Register, rt: Register) -> &mut Self {
        self.push(
            And::default()
                .with_rs(rs.into())
                .with_rt(rt.into())
                .with_rd(rd.into())
                .into(),
        )
    }

    pub fn lui(&mut self, rt: Register, immediate: u16) -> &mut Self {
        self.push(Lui::default().with_rt(rt.into()).with_imm(immediate).into())
    }

    pub fn ori(&mut self, rt: Register, rs: Register, immediate: u16) -> &mut Self {
        self.push(
            Ori::default()
                .with_rt(rt.into())
                .with_rs(rs.into())
                .with_imm(immediate)
                .into(),
        )
    }

    pub fn sw(&mut self, rt: Register, base: Register, offset: u16) -> &mut Self {
        self.push(
            Sw::default()
                .with_rt(rt.into())
                .with_base(base.into())
                .with_offset(offset)
                .into(),
        )
    }
}
