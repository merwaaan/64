//! SLL, SRL, SRA
//! DSLL, DSRL, DSRA
//! DSLL32, DSRL32, DSRA32
//!
//! SLLV, SRLV, SRAV
//! DSLLV, DSRLV, DSRAV

use alloc::format;
use arbitrary_int::u5;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

const REG_VALUES: [u64; 13] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x0000_0000_0000_1F00,
    0x0000_0000_2999_45B8,
    0x0000_0000_7FFF_FFFF,
    0x0000_0000_8000_0000,
    0x0000_0000_8000_0001,
    0x0000_0000_ABCD_1234,
    0x0000_0000_FFFF_FFFF,
    0x0000_0001_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x105C_00CE_0000_0000,
    0xFFFF_002F_89AB_F51F,
];

// TODO rm?
const REGISTER_OUT_VALUES: [u64; 3] = [
    0x0000_0000_0000_0000,
    0x105C_00CE_0012_C0FE,
    0xFFFF_FFFF_FFFF_FFFF,
];

#[derive(Debug)]
pub struct SaParam {
    rd: Register,
    rt: Register,
    rt_value: u64,
    sa: u5,
}

macro_rules! sa {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = SaParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let sa = (0..=31).map(u5::new);

                let basic =
                    itertools::iproduct!(REG_VALUES, sa.clone()).map(|(rt_value, sa)| SaParam {
                        rd: Register::T0,
                        rt: Register::T1,
                        rt_value,
                        sa,
                    });

                let rd_is_rt =
                    itertools::iproduct!(REG_VALUES, sa.clone()).map(|(rt_value, sa)| SaParam {
                        rd: Register::T0,
                        rt: Register::T0,
                        rt_value,
                        sa,
                    });

                let rd_is_r0 =
                    itertools::iproduct!(REG_VALUES, sa.clone()).map(|(value, sa)| SaParam {
                        rd: Register::R0,
                        rt: Register::T0,
                        rt_value: value,
                        sa,
                    });

                let rt_is_r0 = sa.clone().map(|sa| SaParam {
                    rd: Register::T0,
                    rt: Register::R0,
                    rt_value: 0,
                    sa,
                });

                basic.chain(rd_is_rt).chain(rd_is_r0).chain(rt_is_r0)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let result = io::Buffer::<u64>::new(1);

                Program::new()
                    .set_reg64(params.rd, 0x1234_5678_ABCD_ABCD)
                    .set_reg64(params.rt, params.rt_value)
                    .push(
                        $instr::default()
                            .with_rd(params.rd.into())
                            .with_rt(params.rt.into())
                            .with_sa(params.sa.into())
                            .into(),
                    )
                    .store_reg64(params.rd, result.as_ptr() as u32, Register::T3)
                    .run();

                app.value64(
                    &format!(
                        "{} {}, {}={:08X}, {:0X}",
                        stringify!($instr).to_uppercase(),
                        params.rd,
                        params.rt,
                        params.rt_value,
                        params.sa
                    ),
                    result.get(0),
                )
            }
        }
    };
}

register_test!(CpuInstructionSll);
sa!(CpuInstructionSll, Sll);

register_test!(CpuInstructionSrl);
sa!(CpuInstructionSrl, Srl);

register_test!(CpuInstructionSra);
sa!(CpuInstructionSra, Sra);

register_test!(CpuInstructionDsll);
sa!(CpuInstructionDsll, Dsll);

register_test!(CpuInstructionDsrl);
sa!(CpuInstructionDsrl, Dsrl);

register_test!(CpuInstructionDsra);
sa!(CpuInstructionDsra, Dsra);

register_test!(CpuInstructionDsll32);
sa!(CpuInstructionDsll32, Dsll32);

register_test!(CpuInstructionDsrl32);
sa!(CpuInstructionDsrl32, Dsrl32);

register_test!(CpuInstructionDsra32);
sa!(CpuInstructionDsra32, Dsra32);

#[derive(Debug)]
pub struct VParam {
    rd: Register,
    rt: Register,
    rt_value: u64,
    rs: Register,
    rs_value: u64,
}

macro_rules! v {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = VParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let v = (0..=31).chain([
                    0x0000_0000_0000_FFE0,
                    0x0000_0000_0000_FFE4,
                    0xFFFF_498C_FFFF_FFA9,
                    0xABCD_0000_FFFF_0004,
                    0xFFFF_FFFF_FFFF_FFFF,
                ]);

                let basic =
                    itertools::iproduct!(REG_VALUES, v.clone()).map(|(rt_value, rs_value)| {
                        VParam {
                            rd: Register::T0,
                            rt: Register::T1,
                            rt_value,
                            rs: Register::T2,
                            rs_value,
                        }
                    });

                let rd_is_rt =
                    itertools::iproduct!(REG_VALUES, v.clone()).map(|(rt_value, rs_value)| {
                        VParam {
                            rd: Register::T0,
                            rt: Register::T0,
                            rt_value,
                            rs: Register::T1,
                            rs_value,
                        }
                    });

                let rd_is_r0 =
                    itertools::iproduct!(REG_VALUES, v.clone()).map(|(rt_value, rs_value)| {
                        VParam {
                            rd: Register::R0,
                            rt: Register::T0,
                            rt_value,
                            rs: Register::T1,
                            rs_value,
                        }
                    });

                let rt_is_r0 = v.clone().map(|rs_value| VParam {
                    rd: Register::T0,
                    rt: Register::R0,
                    rt_value: 0,
                    rs: Register::T1,
                    rs_value,
                });

                let rs_is_r0 = REG_VALUES.map(|rt_value| VParam {
                    rd: Register::T0,
                    rt: Register::T1,
                    rt_value,
                    rs: Register::R0,
                    rs_value: 0,
                });

                basic
                    .chain(rd_is_rt)
                    .chain(rd_is_r0)
                    .chain(rt_is_r0)
                    .chain(rs_is_r0)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let result = io::Buffer::<u64>::new(3);

                Program::new()
                    .set_reg64(params.rd, 0x1234_5678_ABCD_ABCD)
                    .set_reg64(params.rt, params.rt_value)
                    .set_reg64(params.rs, params.rs_value)
                    .push(
                        $instr::default()
                            .with_rd(params.rd.into())
                            .with_rt(params.rt.into())
                            .with_rs(params.rs.into())
                            .into(),
                    )
                    .store_reg64(params.rd, result.as_ptr() as u32, Register::T3)
                    .run();

                app.value64(
                    &format!(
                        "{} {}, {}={:08X}, {}={:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rd,
                        params.rt,
                        params.rt_value,
                        params.rs,
                        params.rs_value,
                    ),
                    result.get(0),
                )

                // TODO others?
            }
        }
    };
}

register_test!(CpuInstructionSllv);
v!(CpuInstructionSllv, Sllv);

register_test!(CpuInstructionSrlv);
v!(CpuInstructionSrlv, Srlv);

register_test!(CpuInstructionSrav);
v!(CpuInstructionSrav, Srav);

register_test!(CpuInstructionDsllv);
v!(CpuInstructionDsllv, Dsllv);

register_test!(CpuInstructionDsrlv);
v!(CpuInstructionDsrlv, Dsrlv);

register_test!(CpuInstructionDsrav);
v!(CpuInstructionDsrav, Dsrav);
