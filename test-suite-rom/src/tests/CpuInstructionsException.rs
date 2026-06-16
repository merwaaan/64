//! SYSCALL
//!
//! TGE, TGEU, TLT, TLTU, TEQ, TNE
//! TGEI, TGEIU, TLTI, TLTIU, TEQI, TNEI

use alloc::format;
use arbitrary_int::u10;
use core::arch::asm;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    data::{corner_cases_16, corner_cases_64},
    exceptions::{ExceptionHandler, install_exception_handler},
    program::Program,
    register_test,
    test::{Test, TestError},
};

//register_test!(CpuInstructionSyscall);

// impl Test for CpuInstructionSyscall {
//     no_params!();

//     fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
//         let exception_tracker = install_exception_handler(ExceptionTracker::new());

//         Program::new().push(Syscall::default().into()).run();

//         app.bool("Exception occurred", exception_tracker.occurred)?;
//         app.bool("Syscall exception occurred", exception_tracker.syscall)?;

//         // TODO delay slot case?

//         Ok(())
//     }
// }

// TODO branch delay? special test?

// Traps, registers variants

#[derive(Debug)]
pub struct RegParam {
    rs: Register,
    rs_value: u64,
    rt: Register,
    rt_value: u64,
}

const REG_EXTRA_VALUES: &[u64] = &[
    0x0000_0000_1234_5678,
    0x0000_0000_DBCA_BA91,
    0x105C_00CE_0000_0000,
    0xC000_FFFF_0000_0001,
];

// TODO generalize
struct TrapCatcher {
    occurred: bool,
}

impl ExceptionHandler for TrapCatcher {
    fn run(&mut self) {
        // TODO check actual cause!

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

macro_rules! trap_reg {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = RegParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                let basic = itertools::iproduct!(reg_values.clone(), reg_values.clone()).map(
                    |(rs_value, rt_value)| RegParam {
                        rs: Register::T0,
                        rs_value,
                        rt: Register::T1,
                        rt_value,
                    },
                );

                let rs_is_r0 = reg_values.clone().map(|rt_value| RegParam {
                    rs: Register::R0,
                    rs_value: 0,
                    rt: Register::T0,
                    rt_value: rt_value,
                });

                let rt_is_r0 = reg_values.clone().map(|rs_value| RegParam {
                    rs: Register::T0,
                    rs_value: rs_value,
                    rt: Register::R0,
                    rt_value: 0,
                });

                let rs_is_rt = reg_values.clone().map(|value| RegParam {
                    rs: Register::T0,
                    rs_value: value,
                    rt: Register::T0,
                    rt_value: value,
                });

                basic.chain(rs_is_r0).chain(rt_is_r0).chain(rs_is_rt)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let handler = install_exception_handler(TrapCatcher { occurred: false });

                Program::new()
                    .set_reg64(params.rs, params.rs_value)
                    .set_reg64(params.rt, params.rt_value)
                    .push(
                        $instr::default()
                            .with_rs(params.rs.into())
                            .with_rt(params.rt.into())
                            .into(),
                    )
                    .run();

                app.bool(
                    &format!(
                        "{} {}={:08X}, {}={:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rs,
                        params.rs_value,
                        params.rt,
                        params.rt_value,
                    ),
                    handler.occurred,
                )

                // TODO more fields?
            }
        }
    };
}

register_test!(CpuInstructionTge);
trap_reg!(CpuInstructionTge, Tge);

register_test!(CpuInstructionTgeu);
trap_reg!(CpuInstructionTgeu, Tgeu);

register_test!(CpuInstructionTlt);
trap_reg!(CpuInstructionTlt, Tlt);

register_test!(CpuInstructionTltu);
trap_reg!(CpuInstructionTltu, Tltu);

register_test!(CpuInstructionTeq);
trap_reg!(CpuInstructionTeq, Teq);

register_test!(CpuInstructionTne);
trap_reg!(CpuInstructionTne, Tne);

// Trap instructions have a 10-bit code area,
// check that instructions are properly decoded when that code is specified

macro_rules! trap_reg_code {
    // We need rs/rt values that trigger the trap for each instruction
    ($test:ident, Tge) => { trap_reg_code!(@impl $test, Tge, 1, 0); };
    ($test:ident, Tgeu) => { trap_reg_code!(@impl $test, Tgeu, 1, 0); };
    ($test:ident, Tlt) => { trap_reg_code!(@impl $test, Tlt, 0, 1); };
    ($test:ident, Tltu) => { trap_reg_code!(@impl $test, Tltu, 0, 1); };
    ($test:ident, Teq) => { trap_reg_code!(@impl $test, Teq, 1, 1); };
    ($test:ident, Tne) => { trap_reg_code!(@impl $test, Tne, 0, 1); };

    (@impl $test:ident, $instr:ident, $rs_value:expr, $rt_value:expr) => {
        impl Test for $test {
            type Params = u10; // the code

            fn cases() -> impl Iterator<Item = Self::Params> {
                (0..1024).map(u10::new)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let handler = install_exception_handler(TrapCatcher { occurred: false });

                Program::new()
                    .set_reg64(Register::T0, $rs_value)
                    .set_reg64(Register::T1, $rt_value)
                    .push(
                        $instr::default()
                            .with_rs(Register::T0.into())
                            .with_rt(Register::T1.into())
                            .with_code(*params)
                            .into(),
                    )
                    .run();

                assert!(handler.occurred, "{} should should have caused an exception", stringify!($instr));

                app.bool(
                    &format!(
                        "{} T0={:08X}, T1={:08X} with code {:04X}",
                        stringify!($instr).to_uppercase(),
                        $rs_value,
                        $rt_value,
                        params,
                    ),
                    handler.occurred,
                )

                // TODO more fields?
            }
        }
    };
}

register_test!(CpuInstructionTgeCode);
trap_reg_code!(CpuInstructionTgeCode, Tge);

register_test!(CpuInstructionTgeuCode);
trap_reg_code!(CpuInstructionTgeuCode, Tgeu);

register_test!(CpuInstructionTltCode);
trap_reg_code!(CpuInstructionTltCode, Tlt);

register_test!(CpuInstructionTltuCode);
trap_reg_code!(CpuInstructionTltuCode, Tltu);

register_test!(CpuInstructionTeqCode);
trap_reg_code!(CpuInstructionTeqCode, Teq);

register_test!(CpuInstructionTneCode);
trap_reg_code!(CpuInstructionTneCode, Tne);

// Traps, immediate variants

#[derive(Debug)]
pub struct ImmParam {
    rs: Register,
    rs_value: u64,
    imm: u16,
}

macro_rules! trap_imm {
    ($test:ident, $instr:ident) => {
        impl Test for $test {
            type Params = ImmParam;

            fn cases() -> impl Iterator<Item = Self::Params> {
                let reg_values = corner_cases_64(REG_EXTRA_VALUES);

                let imm_values = corner_cases_16(&[0x044E, 0xC123]);

                let basic = itertools::iproduct!(reg_values.clone(), imm_values.clone()).map(
                    |(rs_value, imm)| ImmParam {
                        rs: Register::T0,
                        rs_value,
                        imm,
                    },
                );

                let rs_is_r0 = imm_values.map(|imm| ImmParam {
                    rs: Register::R0,
                    rs_value: 0,
                    imm,
                });

                basic.chain(rs_is_r0)
            }

            fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
                let handler = install_exception_handler(TrapCatcher { occurred: false });

                Program::new()
                    .set_reg64(params.rs, params.rs_value)
                    .push(
                        $instr::default()
                            .with_rs(params.rs.into())
                            .with_imm(params.imm)
                            .into(),
                    )
                    .run();

                app.bool(
                    &format!(
                        "{} {}={:08X}, {:08X}",
                        stringify!($instr).to_uppercase(),
                        params.rs,
                        params.rs_value,
                        params.imm,
                    ),
                    handler.occurred,
                )

                // TODO more fields
            }
        }
    };
}

register_test!(CpuInstructionTgei);
trap_imm!(CpuInstructionTgei, Tgei);

register_test!(CpuInstructionTgeiu);
trap_imm!(CpuInstructionTgeiu, Tgeiu);

register_test!(CpuInstructionTlti);
trap_imm!(CpuInstructionTlti, Tlti);

register_test!(CpuInstructionTltiu);
trap_imm!(CpuInstructionTltiu, Tltiu);

register_test!(CpuInstructionTeqi);
trap_imm!(CpuInstructionTeqi, Teqi);

register_test!(CpuInstructionTnei);
trap_imm!(CpuInstructionTnei, Tnei);
