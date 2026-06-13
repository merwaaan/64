//! ADD, ADDU, SUB, SUBU
//! DADD, DADDU, DSUB, DSUBU
//! SLT, SLTU
//!
//! ADDI, ADDIU
//! DADDI, DADDIU
//! SLTI, SLTIU

use alloc::format;
use core::arch::asm;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    data::{
        RdRtRs, RtRsImm, corner_cases_16, corner_cases_64, rd_rt_rs_combinations,
        rt_rs_combinations,
    },
    exceptions::{ExceptionHandler, install_exception_handler},
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

// TODO generalize
struct ExceptionH {
    occurred: bool,
}

impl ExceptionHandler for ExceptionH {
    fn run(&mut self) {
        // TODO check actual cause!
        // TODO assert

        self.occurred = true;

        unsafe {
            asm!(
                "mfc0 $t0, $14",
                "addiu $t0, $t0, 4",
                "mtc0 $t0, $14",
                options(nostack, preserves_flags),
            );
        }
    }
}
const REG_EXTRA_VALUES: &[u64] = &[
    0x0000_0000_0000_CD15,
    0x0000_0000_2640_044E,
    0x0000_0000_5555_5555,
    0x0000_0000_DBCA_0000,
    0x105C_00CE_0000_0000,
    0xC000_FFFF_0000_0007,
    0xFFFF_002F_89AB_F51F,
    0xFFFF_FFFF_0000_1251,
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
                let ex_handler = install_exception_handler(ExceptionH { occurred: false });

                let mut result = io::Buffer::<u64>::new(1);
                result.push(0);

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
                )?;

                app.bool("Exception", ex_handler.occurred)
            }
        }
    };
}

register_test!(CpuInstructionAdd);
reg!(CpuInstructionAdd, Add);

register_test!(CpuInstructionAddu);
reg!(CpuInstructionAddu, Addu);

register_test!(CpuInstructionSub);
reg!(CpuInstructionSub, Sub);

register_test!(CpuInstructionSubu);
reg!(CpuInstructionSubu, Subu);

register_test!(CpuInstructionSlt);
reg!(CpuInstructionSlt, Slt);

register_test!(CpuInstructionSltu);
reg!(CpuInstructionSltu, Sltu);

register_test!(CpuInstructionDadd);
reg!(CpuInstructionDadd, Dadd);

register_test!(CpuInstructionDaddu);
reg!(CpuInstructionDaddu, Daddu);

register_test!(CpuInstructionDsub);
reg!(CpuInstructionDsub, Dsub);

register_test!(CpuInstructionDsubu);
reg!(CpuInstructionDsubu, Dsubu);

macro_rules! imm {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RtRsImm;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                let imm_values = corner_cases_16(&[0x0002, 0x00C5, 0x04F0, 0xAAAA]);

                rt_rs_combinations(reg_values, imm_values)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let ex_handler = install_exception_handler(ExceptionH { occurred: false });

                let mut result = io::Buffer::<u64>::new(1);
                result.push(0);

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
                )?;

                app.bool("Exception", ex_handler.occurred)
            }
        }
    };
}

register_test!(CpuInstructionAddi);
imm!(CpuInstructionAddi, Addi);

register_test!(CpuInstructionAddiu);
imm!(CpuInstructionAddiu, Addiu);

register_test!(CpuInstructionDaddi);
imm!(CpuInstructionDaddi, Daddi);

register_test!(CpuInstructionDaddiu);
imm!(CpuInstructionDaddiu, Daddiu);

register_test!(CpuInstructionSlti);
imm!(CpuInstructionSlti, Slti);

register_test!(CpuInstructionSltiu);
imm!(CpuInstructionSltiu, Sltiu);
