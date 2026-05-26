//! SLL, SRL, SRA
//! DSLL, DSLL32, DSLLV
//! DSLL32, DSRL32, DSRA32
//! SLLV, SRLV, SRAV
//! DSLLV, DSRLV, DSRAV

use alloc::{format, vec::Vec};
use arbitrary_int::u5;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

const REGISTERS: [Register; 3] = [Register::R0, Register::T0, Register::T1];

const REGISTER_IN_VALUES: [u64; 11] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x0000_0000_0000_1F00,
    0x0000_0000_2999_45B8,
    0x0000_0000_7FFF_FFFF,
    0x0000_0000_8008_00F0,
    0x0000_0000_FFFF_FFFF,
    0x0000_0001_FFFF_FFFF,
    0x105C_00CE_0000_0000,
    0xFFFF_002F_89AB_F51F,
    0xFFFF_FFFF_FFFF_FFFF,
];

const REGISTER_OUT_VALUES: [u64; 3] = [
    0x0000_0000_0000_0000,
    0x105C_00CE_0012_C0FE,
    0xFFFF_FFFF_FFFF_FFFF,
];

#[derive(Debug)]
pub struct SaParam {
    reg_in: Register,
    reg_in_value: u64,
    shift: u5,
    reg_out: Register,
    reg_out_value: u64,
}

macro_rules! sa_variant {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = SaParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let shifts: Vec<_> = (0..=31).map(|i| u5::from_u8(i as u8)).collect();

                itertools::iproduct!(
                    REGISTERS,
                    REGISTER_IN_VALUES,
                    shifts,
                    REGISTERS,
                    REGISTER_OUT_VALUES
                )
                .map(
                    |(reg_in, reg_in_value, shift, reg_out, reg_out_value)| SaParam {
                        reg_in,
                        reg_in_value,
                        shift,
                        reg_out,
                        reg_out_value,
                    },
                )
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                app.comment(&format!(
                    "{} {}={:08X}, {}={:08X}, {:0X}",
                    stringify!($instr).to_uppercase(),
                    params.reg_out,
                    params.reg_out_value,
                    params.reg_in,
                    params.reg_in_value,
                    params.shift
                ))?;

                let result = io::Buffer::<u64>::new(1);

                Program::new()
                    .set_reg64(params.reg_in, params.reg_in_value)
                    .push(
                        $instr::default()
                            .with_rd(params.reg_out.into())
                            .with_rt(params.reg_in.into())
                            .with_sa(params.shift.into())
                            .into(),
                    )
                    .store_reg64(params.reg_out, result.as_ptr() as u32, Register::T3)
                    .run();

                app.value64(result.get(0))
            }
        }
    };
}

register_test!(CpuInstructionSll);
sa_variant!(CpuInstructionSll, Sll);

register_test!(CpuInstructionSrl);
sa_variant!(CpuInstructionSrl, Srl);

register_test!(CpuInstructionSra);
sa_variant!(CpuInstructionSra, Sra);

register_test!(CpuInstructionDsll);
sa_variant!(CpuInstructionDsll, Dsll);

register_test!(CpuInstructionDsrl);
sa_variant!(CpuInstructionDsrl, Dsrl);
register_test!(CpuInstructionDsra);
sa_variant!(CpuInstructionDsra, Dsra);

register_test!(CpuInstructionDsll32);
sa_variant!(CpuInstructionDsll32, Dsll32);

register_test!(CpuInstructionDsrl32);
sa_variant!(CpuInstructionDsrl32, Dsrl32);

register_test!(CpuInstructionDsra32);
sa_variant!(CpuInstructionDsra32, Dsra32);

#[derive(Debug)]
pub struct VParam {
    reg_in1: Register,
    reg_in1_value: u64,
    reg_in2: Register,
    reg_in2_value: u64,
    reg_out: Register,
    reg_out_value: u64,
}

macro_rules! v_variant {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = VParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let shifts: Vec<_> = (0..=31)
                    .chain([
                        0x0000_0000_0000_FFE0,
                        0x0000_0000_0000_FFE4,
                        0xABCD_0000_FFFF_0004,
                        0xFFFF_FFFF_FFFF_FFFF,
                    ])
                    .collect();

                itertools::iproduct!(
                    REGISTERS,
                    REGISTER_IN_VALUES,
                    REGISTERS,
                    shifts,
                    REGISTERS,
                    REGISTER_OUT_VALUES
                )
                .map(
                    |(reg_in1, reg_in1_value, reg_in2, reg_in2_value, reg_out, reg_out_value)| {
                        VParam {
                            reg_in1,
                            reg_in1_value,
                            reg_in2,
                            reg_in2_value,
                            reg_out,
                            reg_out_value,
                        }
                    },
                )
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                app.comment(&format!(
                    "{} {}={:08X}, {}={:08X}, {}={:08X}",
                    stringify!($instr).to_uppercase(),
                    params.reg_out,
                    params.reg_out_value,
                    params.reg_in1,
                    params.reg_in1_value,
                    params.reg_in2,
                    params.reg_in2_value
                ))?;

                let result = io::Buffer::<u64>::new(3);

                Program::new()
                    .set_reg64(params.reg_in1, params.reg_in1_value)
                    .set_reg64(params.reg_in2, params.reg_in2_value)
                    .push(
                        $instr::default()
                            .with_rd(params.reg_out.into())
                            .with_rs(params.reg_in1.into())
                            .with_rt(params.reg_in2.into())
                            .into(),
                    )
                    .store_reg64(params.reg_out, result.as_ptr() as u32, Register::T3)
                    .run();

                app.value64(result.get(0))
                // TODO others?
            }
        }
    };
}

register_test!(CpuInstructionSllv);
v_variant!(CpuInstructionSllv, Sllv);

register_test!(CpuInstructionSrlv);
v_variant!(CpuInstructionSrlv, Srlv);

register_test!(CpuInstructionSrav);
v_variant!(CpuInstructionSrav, Srav);

register_test!(CpuInstructionDsllv);
v_variant!(CpuInstructionDsllv, Dsllv);

register_test!(CpuInstructionDsrlv);
v_variant!(CpuInstructionDsrlv, Dsrlv);

register_test!(CpuInstructionDsrav);
v_variant!(CpuInstructionDsrav, Dsrav);
