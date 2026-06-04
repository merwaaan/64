//! AND, OR, NOR, XOR
//! ANDI, ORI, XORI

use alloc::format;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

const REGISTERS: [Register; 3] = [Register::R0, Register::T0, Register::T1];

const REGISTER_VALUES: [u64; 14] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x0000_0000_0000_CD15,
    0x0000_0000_2640_044E,
    0x0000_0000_5555_5555,
    0x0000_0000_7FFF_FFFF,
    0x0000_0000_8008_00F0,
    0x0000_0000_AAAA_AAAA,
    0x0000_0000_DBCA_0000,
    0x0000_0000_FFFF_FFFF,
    0x105C_00CE_0000_0000,
    0xC000_FFFF_0000_0007,
    0xFFFF_002F_89AB_F51F,
    0xFFFF_FFFF_FFFF_FFFF,
];

#[derive(Debug)]
pub struct RegisterParam {
    reg_value1: u64,
    reg_value2: u64,
    reg_in1: Register,
    reg_in2: Register,
    reg_out: Register,
}

macro_rules! reg_variant {
    ($instr:ident) => {
        type Params = RegisterParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            itertools::iproduct!(
                REGISTER_VALUES,
                REGISTER_VALUES,
                REGISTERS,
                REGISTERS,
                REGISTERS
            )
            .map(
                |(reg_value1, reg_value2, reg_in1, reg_in2, reg_out)| RegisterParam {
                    reg_value1,
                    reg_value2,
                    reg_in1,
                    reg_in2,
                    reg_out,
                },
            )
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            let result: u64 = 0;

            Program::new()
                .set_reg64(params.reg_in1, params.reg_value1)
                .set_reg64(params.reg_in2, params.reg_value2)
                .push(
                    $instr::default()
                        .with_rs(params.reg_in1.into())
                        .with_rt(params.reg_in2.into())
                        .with_rd(params.reg_out.into())
                        .into(),
                )
                .store_reg64(
                    params.reg_out,
                    core::ptr::addr_of!(result) as u32,
                    Register::T3,
                )
                .run();

            app.value64(
                &format!(
                    "{} {}, {}={:08X}, {}={:08X}",
                    stringify!($instr).to_uppercase(),
                    params.reg_out,
                    params.reg_in1,
                    params.reg_value1,
                    params.reg_in2,
                    params.reg_value2,
                ),
                result,
            )
        }
    };
}

register_test!(CpuInstructionAnd);

impl Test for CpuInstructionAnd {
    reg_variant!(And);
}

register_test!(CpuInstructionOr);

impl Test for CpuInstructionOr {
    reg_variant!(Or);
}

register_test!(CpuInstructionNor);

impl Test for CpuInstructionNor {
    reg_variant!(Nor);
}

register_test!(CpuInstructionXor);

impl Test for CpuInstructionXor {
    reg_variant!(Xor);
}

#[derive(Debug)]
pub struct ImmediateParam {
    reg_value: u64,
    imm_value: u16,
    reg_in: Register,
    reg_out: Register,
}

macro_rules! imm_variant {
    ($instr:ident) => {
        type Params = ImmediateParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let imm_values = [0, 1, 0x1002, 0xCD15, 0x044E, 0x5555, 0xFFFF];

            itertools::iproduct!(REGISTER_VALUES, imm_values, REGISTERS, REGISTERS).map(
                |(reg_value, imm_value, reg_in, reg_out)| ImmediateParam {
                    reg_value,
                    imm_value,
                    reg_in,
                    reg_out,
                },
            )
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            let result = io::Buffer::<u64>::new(1);

            Program::new()
                .set_reg64(params.reg_in, params.reg_value)
                .push(
                    $instr::default()
                        .with_rs(params.reg_in.into())
                        .with_rt(params.reg_out.into())
                        .with_imm(params.imm_value)
                        .into(),
                )
                .store_reg64(params.reg_out, result.as_ptr() as u32, Register::T3)
                .run();

            app.value64(
                &format!(
                    "{} {}, {}={:08X}, {:08X}",
                    stringify!($instr).to_uppercase(),
                    params.reg_out,
                    params.reg_in,
                    params.reg_value,
                    params.imm_value,
                ),
                result.get(0),
            )
        }
    };
}

register_test!(CpuInstructionAndi);

impl Test for CpuInstructionAndi {
    imm_variant!(Andi);
}

register_test!(CpuInstructionOri);

impl Test for CpuInstructionOri {
    imm_variant!(Ori);
}

register_test!(CpuInstructionXori);

impl Test for CpuInstructionXori {
    imm_variant!(Xori);
}
