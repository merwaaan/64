//! AND, OR, NOR, XOR
//! ANDI, ORI, XORI

use alloc::format;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    data::{
        RdRtRs, RtRsImm, corner_cases_16, corner_cases_64, rd_rt_rs_combinations,
        rt_rs_imm_combinations,
    },
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

const REG_EXTRA_VALUES: &[u64] = &[
    0x0000_0000_0000_CD15,
    0x0000_0000_2640_044E,
    0x0000_0000_5555_5555,
    0x0000_0000_AAAA_AAAA,
    0x0000_0000_DBCA_0000,
    0x105C_00CE_0000_0000,
    0xC000_FFFF_0000_0007,
    0xFFFF_002F_89AB_F51F,
];

macro_rules! reg {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RdRtRs;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                rd_rt_rs_combinations(reg_values)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let result = io::CachedBuffer::<u64>::from_slice(&[0]);

                Program::new()
                    .set_reg64(params.rd, params.rd_value)
                    .set_reg64(params.rs, params.rs_value)
                    .set_reg64(params.rt, params.rt_value)
                    .push(
                        $instr::default()
                            .with_rd(params.rd.into())
                            .with_rs(params.rs.into())
                            .with_rt(params.rt.into())
                            .into(),
                    )
                    .store_reg64(params.rd, result.as_ptr() as u32, Register::T7)
                    .run();

                app.value64(
                    &format!(
                        "{} {}, {}={:08X}, {}={:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rd,
                        params.rs,
                        params.rs_value,
                        params.rt,
                        params.rt_value,
                    ),
                    result.get(0),
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

#[derive(Debug)]
pub struct ImmediateParam {
    rt: Register,
    rs: Register,
    rs_value: u64,
    imm: u16,
}

macro_rules! imm {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RtRsImm;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                let imm_values = corner_cases_16(&[0x1002, 0xCD15, 0x044E, 0x5555]);

                rt_rs_imm_combinations(reg_values, imm_values)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let result = io::CachedBuffer::<u64>::from_slice(&[0]);

                Program::new()
                    .set_reg64(params.rt, params.rt_value)
                    .set_reg64(params.rs, params.rs_value)
                    .push(
                        $instr::default()
                            .with_rt(params.rt.into())
                            .with_rs(params.rs.into())
                            .with_imm(params.imm)
                            .into(),
                    )
                    .store_reg64(params.rt, result.as_ptr() as u32, Register::T7)
                    .run();

                app.value64(
                    &format!(
                        "{} {}, {}={:08X}, {:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rt,
                        params.rs,
                        params.rs_value,
                        params.imm,
                    ),
                    result.get(0),
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
