use alloc::format;
use n64_specs::{
    cpu::registers::Register as CpuRegister,
    rsp::{DMEM_START, instructions::*, registers::Register},
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

// TODO check exceptions

const REG_EXTRA_VALUES: &[u32] = &[0x0000_0123, 0x1234_5678, 0xABCD_0000];

// ADD, ADDU, SUB, SUBU
// SLT, SLTU

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

register_test!(RspInstructionAdd);
reg!(RspInstructionAdd, Add);

register_test!(RspInstructionAddu);
reg!(RspInstructionAddu, Addu);

register_test!(RspInstructionSub);
reg!(RspInstructionSub, Sub);

register_test!(RspInstructionSubu);
reg!(RspInstructionSubu, Subu);

register_test!(RspInstructionSlt);
reg!(RspInstructionSlt, Slt);

register_test!(RspInstructionSltu);
reg!(RspInstructionSltu, Sltu);

// ADDI, ADDIU
// SLTI, SLTIU

macro_rules! imm {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RtRsImm<u32>;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_32(REG_EXTRA_VALUES);

                let imm_values = corner_cases_16(&[0x0002, 0x00C5, 0x04F0, 0xAAAA]);

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

register_test!(RspInstructionAddi);
imm!(RspInstructionAddi, Addi);

register_test!(RspInstructionAddiu);
imm!(RspInstructionAddiu, Addiu);

register_test!(RspInstructionSlti);
imm!(RspInstructionSlti, Slti);

register_test!(RspInstructionSltiu);
imm!(RspInstructionSltiu, Sltiu);
