use alloc::format;
use n64_specs::cpu::registers::Register;

use crate::{
    app::App,
    io,
    program::Program,
    test::{Test, TestError},
};

pub struct CpuInstructionAnd;

#[derive(Debug)]
pub struct Param {
    value1: u32,
    value2: u32,
    reg_in1: Register,
    reg_in2: Register,
    reg_out: Register,
}

impl Test for CpuInstructionAnd {
    type Params = Param;

    fn cases() -> impl Iterator<Item = Self::Params> {
        let values = [
            0,
            1,
            0x0000_CD15,
            0x2640_044E,
            0x5555_5555,
            0x7FFF_FFFF,
            0x8008_00F0,
            0xAAAA_AAAA,
            0xDBCA_0000,
            0xFFFF_FFFF,
        ];

        let regs = [
            Register::R0,
            Register::AT,
            Register::V0,
            Register::V1,
            Register::A0,
            Register::A1,
            /*Register::A2,
            Register::A3,
            Register::T0,
            Register::T1,
            Register::T2,
            Register::T3,
            Register::T4,
            Register::T5,
            Register::T6,
            Register::T7,
            Register::S0,
            Register::S1,
            Register::S2,
            Register::S3,
            Register::S4,
            Register::S5,
            Register::S6,
            Register::S7,
            Register::T8,
            Register::T9,*/
            // TODO how many?
        ];

        itertools::iproduct!(values, values, regs, regs, regs).map(
            |(value1, value2, reg_in1, reg_in2, reg_out)| Param {
                value1,
                value2,
                reg_in1,
                reg_in2,
                reg_out,
            },
        )
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        app.comment(&format!(
            "AND {}, {}={:08X}, {}={:08X}",
            params.reg_out, params.reg_in1, params.value1, params.reg_in2, params.value2,
        ))?;

        let result: io::Buffer<u32> = io::Buffer::<u32>::new(1);

        Program::new()
            .load_reg(params.reg_in1, params.value1)
            .load_reg(params.reg_in2, params.value2)
            .and(params.reg_out, params.reg_in1, params.reg_in2)
            .load_reg(Register::T3, result.as_ptr() as u32) // TODO other reg func
            .sw(params.reg_out, Register::T3, 0)
            .run();

        app.value(result.get(0))?;

        Ok(())
    }
}
