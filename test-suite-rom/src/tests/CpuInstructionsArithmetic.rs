use alloc::format;
use core::arch::asm;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    data::{
        INIT_64, RdRtRs, RtRs, RtRsImm, corner_cases_16, corner_cases_64, rd_rt_rs_combinations,
        rt_rs_imm_combinations,
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

// ADD, ADDU, SUB, SUBU
// DADD, DADDU, DSUB, DSUBU
// SLT, SLTU

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

// ADDI, ADDIU
// DADDI, DADDIU
// SLTI, SLTIU

macro_rules! imm {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RtRsImm;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                let imm_values = corner_cases_16(&[0x0002, 0x00C5, 0x04F0, 0xAAAA]);

                rt_rs_imm_combinations(reg_values, imm_values)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let ex_handler = install_exception_handler(ExceptionH { occurred: false });

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

// MULT, MULTU, DIV, DIVU
// DMULT, DMULTU, DDIV, DDIVU

macro_rules! mult_div {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RtRs;

            fn cases() -> impl Iterator<Item = Self::Params> {
                // let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                // rt_rs_combinations(reg_values)

                [RtRs {
                    rs: Register::T0,
                    rs_value: 01,
                    rt: Register::T1,
                    rt_value: 0x8000_0000,
                }]
                .into_iter()
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let ex_handler = install_exception_handler(ExceptionH { occurred: false });

                let hi_lo = io::CachedBuffer::<u64>::from_slice(&[INIT_64, INIT_64]);

                Program::new()
                    // Init HI/LO
                    .set_reg64(Register::T7, INIT_64)
                    .push(Mthi::default().with_rs(Register::T7.into()).into())
                    .push(Mtlo::default().with_rs(Register::T7.into()).into())
                    .nop()
                    .nop()
                    // Main instruction
                    .set_reg64(params.rs, params.rs_value)
                    .set_reg64(params.rt, params.rt_value)
                    .push(
                        $instr::default()
                            .with_rs(params.rs.into())
                            .with_rt(params.rt.into())
                            .into(),
                    )
                    .nop()
                    .nop()
                    // Read HI/LO
                    .push(Mfhi::default().with_rd(Register::T6.into()).into())
                    .store_reg64(Register::T6, hi_lo.item_ptr(0) as u32, Register::T7)
                    .push(Mflo::default().with_rd(Register::T6.into()).into())
                    .store_reg64(Register::T6, hi_lo.item_ptr(1) as u32, Register::T7)
                    .run();

                let instr = format!(
                    "{} {}={:08X}, {}={:08X}",
                    stringify!($instr).to_uppercase(),
                    params.rs,
                    params.rs_value,
                    params.rt,
                    params.rt_value
                );

                app.value64(&format!("{}, HI", instr), hi_lo.get(0))?;
                app.value64(&format!("{}, LO", instr), hi_lo.get(1))?;

                app.bool("Exception", ex_handler.occurred)
            }
        }
    };
}

register_test!(CpuInstructionMult);
mult_div!(CpuInstructionMult, Mult);

register_test!(CpuInstructionMultu);
mult_div!(CpuInstructionMultu, Multu);

register_test!(CpuInstructionDiv);
mult_div!(CpuInstructionDiv, Div);

register_test!(CpuInstructionDivu);
mult_div!(CpuInstructionDivu, Divu);

register_test!(CpuInstructionDmult);
mult_div!(CpuInstructionDmult, Dmult);

register_test!(CpuInstructionDmultu);
mult_div!(CpuInstructionDmultu, Dmultu);

register_test!(CpuInstructionDdiv);
mult_div!(CpuInstructionDdiv, Ddiv);

register_test!(CpuInstructionDdivu);
mult_div!(CpuInstructionDdivu, Ddivu);

// MTHI, MTLO
// MFHI, MFLO

macro_rules! move_hi_lo {
    ($test:ident, $mt_instr:ident, $mf_instr:ident) => {
        impl Test for $test {
            type Params = u64;

            fn cases() -> impl Iterator<Item = Self::Params> {
                corner_cases_64(REG_EXTRA_VALUES)
            }

            fn run(rs: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let result = io::CachedBuffer::<u64>::from_slice(&[0]);

                Program::new()
                    .set_reg64(Register::T0, INIT_64)
                    .set_reg64(Register::T1, *rs)
                    .push($mt_instr::default().with_rs(Register::T1.into()).into())
                    .nop() // 2-instruction delay
                    .nop()
                    .push($mf_instr::default().with_rd(Register::T0.into()).into())
                    .store_reg64(Register::T0, result.as_ptr() as u32, Register::T7)
                    .run();

                app.value64(
                    &format!(
                        "{} / {} {:08X}",
                        stringify!($mt_instr).to_uppercase(),
                        stringify!($mf_instr).to_uppercase(),
                        rs,
                    ),
                    result.get(0),
                )
            }
        }
    };
}

register_test!(CpuInstructionMthiMfhi);
move_hi_lo!(CpuInstructionMthiMfhi, Mthi, Mfhi);

register_test!(CpuInstructionMtloMflo);
move_hi_lo!(CpuInstructionMtloMflo, Mtlo, Mflo);
