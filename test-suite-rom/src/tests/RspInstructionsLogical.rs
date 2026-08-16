use alloc::format;
use n64_specs::{
    cpu::registers::Register as CpuRegister,
    rsp::{DMEM_START, instructions::*},
};

use crate::{
    app::App,
    data::{
        RdRtRs, RtRsImm, corner_cases_16, corner_cases_32, rd_rt_rs_combinations,
        rt_rs_imm_combinations,
    },
    io, register_test,
    rsp_program::RspProgram,
    test::{Test, TestError},
};

const REG_EXTRA_VALUES: &[u32] = &[0x0000_0A7E, 0x1F00_9BD1, 0xABCD_0000, 0xDDE1_94AA];

// AND, OR, NOR, XOR

macro_rules! reg {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RdRtRs<u32>;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_32(REG_EXTRA_VALUES);

                rd_rt_rs_combinations(reg_values)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                RspProgram::new()
                    .set_reg(params.rd, params.rd_value)
                    .set_reg(params.rs, params.rs_value)
                    .set_reg(params.rt, params.rt_value)
                    .push(
                        $instr::default()
                            .with_rd(params.rd.into())
                            .with_rs(params.rs.into())
                            .with_rt(params.rt.into())
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
                        params.rs,
                        params.rs_value,
                        params.rt,
                        params.rt_value,
                    ),
                    result,
                )
            }
        }
    };
}

register_test!(CpuInstructionAnd);
reg!(CpuInstructionAnd, And);

register_test!(CpuInstructionOr);
reg!(CpuInstructionOr, Or);

register_test!(CpuInstructionNor);
reg!(CpuInstructionNor, Nor);

register_test!(CpuInstructionXor);
reg!(CpuInstructionXor, Xor);

// ANDI, ORI, XORI

macro_rules! imm {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RtRsImm<u32>;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_32(REG_EXTRA_VALUES);

                let imm_values = corner_cases_16(&[0x1002, 0xCD15, 0x044E, 0x5555]);

                rt_rs_imm_combinations(reg_values, imm_values)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                RspProgram::new()
                    .set_reg(params.rt, params.rt_value)
                    .set_reg(params.rs, params.rs_value)
                    .push(
                        $instr::default()
                            .with_rt(params.rt.into())
                            .with_rs(params.rs.into())
                            .with_imm(params.imm)
                            .into(),
                    )
                    .push(
                        Sw::default()
                            .with_rt(params.rt.into())
                            .with_base(CpuRegister::R0.into())
                            .with_offset(0)
                            .into(),
                    )
                    .run();

                let result = io::read_uncached(DMEM_START);

                app.value(
                    &format!(
                        "{} {}, {}={:08X}, {:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rt,
                        params.rs,
                        params.rs_value,
                        params.imm,
                    ),
                    result,
                )
            }
        }
    };
}

register_test!(CpuInstructionAndi);
imm!(CpuInstructionAndi, Andi);

register_test!(CpuInstructionOri);
imm!(CpuInstructionOri, Ori);

register_test!(CpuInstructionXori);
imm!(CpuInstructionXori, Xori);
