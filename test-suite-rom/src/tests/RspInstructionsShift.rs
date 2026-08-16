use alloc::format;
use arbitrary_int::u5;
use n64_specs::{
    cpu::registers::Register as CpuRegister,
    rsp::{DMEM_START, instructions::*},
};

use crate::{
    app::App,
    data::{INIT_32, corner_cases_32},
    io, register_test,
    rsp_program::RspProgram,
    test::{Test, TestError},
};

const REG_EXTRA_VALUES: &[u32] = &[0x0000_1F00, 0x2999_45B8, 0xABCD_1234, 0x89AB_F51F];

// sa-operand variants:
// SLL, SRL, SRA

#[derive(Debug)]
pub struct SaParam {
    rd: CpuRegister,
    rd_value: u32,
    rt: CpuRegister,
    rt_value: u32,
    sa: u5,
}

macro_rules! sa {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = SaParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_32(REG_EXTRA_VALUES);

                // TODO use data gen

                let sa_values = (0..=31).map(u5::new);

                let basic = itertools::iproduct!(reg_values.clone(), sa_values.clone()).map(
                    |(rt_value, sa)| SaParam {
                        rd: CpuRegister::T0,
                        rd_value: INIT_32,
                        rt: CpuRegister::T1,
                        rt_value,
                        sa,
                    },
                );

                let rd_is_r0 = itertools::iproduct!(reg_values.clone(), sa_values.clone()).map(
                    |(value, sa)| SaParam {
                        rd: CpuRegister::R0,
                        rd_value: 0,
                        rt: CpuRegister::T0,
                        rt_value: value,
                        sa,
                    },
                );

                let rt_is_r0 = sa_values.clone().map(|sa| SaParam {
                    rd: CpuRegister::T0,
                    rd_value: INIT_32,
                    rt: CpuRegister::R0,
                    rt_value: 0,
                    sa,
                });

                let rd_is_rt = itertools::iproduct!(reg_values.clone(), sa_values.clone()).map(
                    |(rt_value, sa)| SaParam {
                        rd: CpuRegister::T0,
                        rd_value: rt_value,
                        rt: CpuRegister::T0,
                        rt_value,
                        sa,
                    },
                );

                basic.chain(rd_is_r0).chain(rt_is_r0).chain(rd_is_rt)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                RspProgram::new()
                    .set_reg(params.rd, params.rd_value)
                    .set_reg(params.rt, params.rt_value)
                    .push(
                        $instr::default()
                            .with_rd(params.rd.into())
                            .with_rt(params.rt.into())
                            .with_sa(params.sa.into())
                            .into(),
                    )
                    .push(
                        Sw::default()
                            .with_rt(params.rd.into())
                            .with_base(CpuRegister::R0.into())
                            .with_offset(0)
                            .into(),
                    )
                    .run();

                let result = io::read_uncached(DMEM_START);

                app.value(
                    &format!(
                        "{} {}, {}={:08X}, {:0X}",
                        stringify!($instr).to_uppercase(),
                        params.rd,
                        params.rt,
                        params.rt_value,
                        params.sa
                    ),
                    result,
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

// v-operand variants:
// SLLV, SRLV, SRAV

#[derive(Debug)]
pub struct VParam {
    rd: CpuRegister,
    rd_value: u32,
    rt: CpuRegister,
    rt_value: u32,
    rs: CpuRegister,
    rs_value: u32,
}

macro_rules! v {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = VParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_32(REG_EXTRA_VALUES);

                let v_values = (0..=31).chain([0x0000_FFE0, 0x0000_FFE4, 0xFFFF_0004, 0xFFFF_FFFF]);

                // TODO data gen

                let basic = itertools::iproduct!(reg_values.clone(), v_values.clone()).map(
                    |(rt_value, v)| VParam {
                        rd: CpuRegister::T0,
                        rd_value: INIT_32,
                        rt: CpuRegister::T1,
                        rt_value,
                        rs: CpuRegister::T2,
                        rs_value: v,
                    },
                );

                let rd_is_r0 = itertools::iproduct!(reg_values.clone(), v_values.clone()).map(
                    |(rt_value, v)| VParam {
                        rd: CpuRegister::R0,
                        rd_value: 0,
                        rt: CpuRegister::T0,
                        rt_value,
                        rs: CpuRegister::T1,
                        rs_value: v,
                    },
                );

                let rt_is_r0 = v_values.clone().map(|v| VParam {
                    rd: CpuRegister::T0,
                    rd_value: INIT_32,
                    rt: CpuRegister::R0,
                    rt_value: 0,
                    rs: CpuRegister::T1,
                    rs_value: v,
                });

                let rs_is_r0 = reg_values.clone().map(|rt_value| VParam {
                    rd: CpuRegister::T0,
                    rd_value: INIT_32,
                    rt: CpuRegister::T1,
                    rt_value,
                    rs: CpuRegister::R0,
                    rs_value: 0,
                });

                let rd_is_rt = itertools::iproduct!(reg_values.clone(), v_values.clone()).map(
                    |(rt_value, v)| VParam {
                        rd: CpuRegister::T0,
                        rd_value: rt_value,
                        rt: CpuRegister::T0,
                        rt_value,
                        rs: CpuRegister::T1,
                        rs_value: v,
                    },
                );

                let rd_is_rs = itertools::iproduct!(reg_values.clone(), v_values.clone()).map(
                    |(rt_value, v)| VParam {
                        rd: CpuRegister::T0,
                        rd_value: v,
                        rt: CpuRegister::T1,
                        rt_value,
                        rs: CpuRegister::T0,
                        rs_value: v,
                    },
                );

                let rt_is_rs = reg_values.clone().map(|rt_value| VParam {
                    rd: CpuRegister::T0,
                    rd_value: INIT_32,
                    rt: CpuRegister::T1,
                    rt_value,
                    rs: CpuRegister::T1,
                    rs_value: rt_value,
                });

                let rd_is_rt_is_rs = reg_values.clone().map(|value| VParam {
                    rd: CpuRegister::T0,
                    rd_value: value,
                    rt: CpuRegister::T0,
                    rt_value: value,
                    rs: CpuRegister::T0,
                    rs_value: value,
                });

                basic
                    .chain(rd_is_r0)
                    .chain(rt_is_r0)
                    .chain(rs_is_r0)
                    .chain(rd_is_rt)
                    .chain(rd_is_rs)
                    .chain(rt_is_rs)
                    .chain(rd_is_rt_is_rs)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                RspProgram::new()
                    .set_reg(params.rd, params.rd_value)
                    .set_reg(params.rt, params.rt_value)
                    .set_reg(params.rs, params.rs_value)
                    .push(
                        $instr::default()
                            .with_rd(params.rd.into())
                            .with_rt(params.rt.into())
                            .with_rs(params.rs.into())
                            .into(),
                    )
                    .push(
                        Sw::default()
                            .with_rt(params.rd.into())
                            .with_base(CpuRegister::R0.into())
                            .with_offset(0)
                            .into(),
                    )
                    .run();

                let result = io::read_uncached(DMEM_START);

                app.value(
                    &format!(
                        "{} {}, {}={:08X}, {}={:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rd,
                        params.rt,
                        params.rt_value,
                        params.rs,
                        params.rs_value,
                    ),
                    result,
                )
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
