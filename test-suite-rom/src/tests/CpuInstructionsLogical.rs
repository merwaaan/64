//! AND, OR, NOR, XOR
//! ANDI, ORI, XORI

use alloc::format;
use n64_specs::cpu::{
    instructions::{And, Andi, Nor, Or, Ori, Xor, Xori},
    registers::Register,
};

use crate::{
    app::App,
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

#[derive(Debug)]
pub struct RegisterParam {
    reg_value1: u32,
    reg_value2: u32,
    reg_in1: Register,
    reg_in2: Register,
    reg_out: Register,
}

// TODO just make the instruction a param instead of using macro?

macro_rules! reg_variant {
    ($instr:ident) => {
        type Params = RegisterParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let values = [
                0,
                1,
                0x0000_CD15,
                0x2640_044E,
                0x5555_5555,
                0x7FFF_FFFF,
                /*0x8008_00F0,
                0xAAAA_AAAA,
                0xDBCA_0000,
                0xFFFF_FFFF,*/
            ];

            let regs = [
                Register::R0,
                Register::AT,
                Register::V0,
                Register::V1,
                Register::A1,
                // TODO how many?
            ];

            itertools::iproduct!(values, values, regs, regs, regs).map(
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
            app.comment(&format!(
                "{} {}, {}={:08X}, {}={:08X}",
                "TODO name",
                params.reg_out,
                params.reg_in1,
                params.reg_value1,
                params.reg_in2,
                params.reg_value2,
            ))?;

            let result: io::Buffer<u32> = io::Buffer::<u32>::new(1);

            Program::new()
                .load_reg(params.reg_in1, params.reg_value1)
                .load_reg(params.reg_in2, params.reg_value2)
                .push(
                    $instr::default()
                        .with_rs(params.reg_in1.into())
                        .with_rt(params.reg_in2.into())
                        .with_rd(params.reg_out.into())
                        .into(),
                )
                .load_reg(Register::T3, result.as_ptr() as u32) // TODO other reg func
                .sw(params.reg_out, Register::T3, 0)
                .run();

            app.value(result.get(0))
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
    reg_value: u32,
    imm_value: u16,
    reg_in: Register,
    reg_out: Register,
}

macro_rules! imm_variant {
    ($instr:ident) => {
        type Params = ImmediateParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let reg_values = [
                0,
                1,
                0x0000_CD15,
                0x2640_044E,
                0x5555_5555,
                0x7FFF_FFFF,
                /*0x8008_00F0,
                0xAAAA_AAAA,
                0xDBCA_0000,
                0xFFFF_FFFF,*/
            ];

            // TODO pick vals
            let imm_values = [
                0, 1, 0xCD15, 0x044E, 0x5555,
                0xFFFF,
                /*0x8008_00F0,
                0xAAAA_AAAA,
                0xDBCA_0000,
                0xFFFF_FFFF,*/
            ];

            let regs = [
                Register::R0,
                Register::AT,
                Register::V0,
                Register::V1,
                Register::A1, // TODO?
            ];

            itertools::iproduct!(reg_values, imm_values, regs, regs).map(
                |(reg_value, imm_value, reg_in, reg_out)| ImmediateParam {
                    reg_value,
                    imm_value,
                    reg_in,
                    reg_out,
                },
            )
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            app.comment(&format!(
                "{} {}, {}={:08X}, {:08X}",
                "TODO name", params.reg_out, params.reg_in, params.reg_value, params.imm_value,
            ))?;

            let result: io::Buffer<u32> = io::Buffer::<u32>::new(1);

            Program::new()
                .load_reg(params.reg_in, params.reg_value)
                .push(
                    $instr::default()
                        .with_rs(params.reg_in.into())
                        .with_rt(params.reg_out.into())
                        .with_imm(params.imm_value)
                        .into(),
                )
                .load_reg(Register::T3, result.as_ptr() as u32) // TODO other reg func
                .sw(params.reg_out, Register::T3, 0)
                .run();

            app.value(result.get(0))
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
