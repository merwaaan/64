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
    exceptions::{ExceptionHandler, ExceptionTracker, install_exception_handler},
    no_params,
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
// TODO works whatever the code (x 1024)

// Traps, registers variants

#[derive(Debug)]
pub struct RegParam {
    rs: Register,
    rs_value: u64,
    rt: Register,
    rt_value: u64,
}

const REG_VALUES: [u64; 12] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x0000_0000_1234_5678,
    0x0000_0000_7FFF_FFFE,
    0x0000_0000_7FFF_FFFF,
    0x0000_0000_8000_0000,
    0x0000_0000_8000_0001,
    0x0000_0000_DBCA_BA91,
    0x0000_0000_FFFF_FFFE,
    0x0000_0000_FFFF_FFFF,
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
    ($instr:ident) => {
        type Params = RegParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let basic =
                itertools::iproduct!(REG_VALUES, REG_VALUES).map(|(rs_value, rt_value)| RegParam {
                    rs: Register::T0,
                    rs_value,
                    rt: Register::T1,
                    rt_value,
                });

            let rs_is_rt = REG_VALUES.map(|value| RegParam {
                rs: Register::T0,
                rs_value: value,
                rt: Register::T0,
                rt_value: value,
            });

            let rs_is_r0 = REG_VALUES.map(|rt_value| RegParam {
                rs: Register::R0,
                rs_value: 0,
                rt: Register::T0,
                rt_value: rt_value,
            });

            let rt_is_r0 = REG_VALUES.map(|rs_value| RegParam {
                rs: Register::T0,
                rs_value: rs_value,
                rt: Register::R0,
                rt_value: 0,
            });

            basic.chain(rs_is_rt).chain(rs_is_r0).chain(rt_is_r0)
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

            // TODO more fields
        }
    };
}

register_test!(CpuInstructionTge);

impl Test for CpuInstructionTge {
    trap_reg!(Tge);
}

register_test!(CpuInstructionTgeu);

impl Test for CpuInstructionTgeu {
    trap_reg!(Tgeu);
}

register_test!(CpuInstructionTlt);

impl Test for CpuInstructionTlt {
    trap_reg!(Tlt);
}

register_test!(CpuInstructionTltu);

impl Test for CpuInstructionTltu {
    trap_reg!(Tltu);
}

register_test!(CpuInstructionTeq);

impl Test for CpuInstructionTeq {
    trap_reg!(Teq);
}

register_test!(CpuInstructionTne);

impl Test for CpuInstructionTne {
    trap_reg!(Tne);
}

// Trap instructions have a 10-bit code area,
// check that they are properly decoded when that code is specified

macro_rules! trap_reg_code {
    // We need rs/rt values that trigger the trap for each instruction
    (Tge) => { trap_reg_code!(@impl Tge, 1, 0); };
    (Tgeu) => { trap_reg_code!(@impl Tgeu, 1, 0); };
    (Tlt) => { trap_reg_code!(@impl Tlt, 0, 1); };
    (Tltu) => { trap_reg_code!(@impl Tltu, 0, 1); };
    (Teq) => { trap_reg_code!(@impl Teq, 1, 1); };
    (Tne) => { trap_reg_code!(@impl Tne, 0, 1); };

    (@impl $instr:ident, $rs_value:expr, $rt_value:expr) => {
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
                        .into(), // TODO code
                )
                .run();

            assert!(handler.occurred, "trap should should have caused an exception");

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

            // TODO more fields
        }
    };
}

register_test!(CpuInstructionTgeCode);

impl Test for CpuInstructionTgeCode {
    trap_reg_code!(Tge);
}

register_test!(CpuInstructionTgeuCode);

impl Test for CpuInstructionTgeuCode {
    trap_reg_code!(Tgeu);
}

register_test!(CpuInstructionTltCode);

impl Test for CpuInstructionTltCode {
    trap_reg_code!(Tlt);
}

register_test!(CpuInstructionTltuCode);

impl Test for CpuInstructionTltuCode {
    trap_reg_code!(Tltu);
}

register_test!(CpuInstructionTeqCode);

impl Test for CpuInstructionTeqCode {
    trap_reg_code!(Teq);
}

register_test!(CpuInstructionTneCode);

impl Test for CpuInstructionTneCode {
    trap_reg_code!(Tne);
}

// Traps, immediate variants

#[derive(Debug)]
pub struct ImmParam {
    rs: Register,
    rs_value: u64,
    imm: u16,
}

const IMM_VALUES: [u16; 10] = [
    0x0000, 0x0001, 0x044E, 0x7FFE, 0x7FFF, 0x8000, 0x8001, 0xC123, 0xFFFE, 0xFFFF,
];

macro_rules! trap_imm {
    ($instr:ident) => {
        type Params = ImmParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let basic =
                itertools::iproduct!(REG_VALUES, IMM_VALUES).map(|(rs_value, imm)| ImmParam {
                    rs: Register::T0,
                    rs_value,
                    imm,
                });

            let rs_is_r0 = IMM_VALUES.map(|imm| ImmParam {
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
    };
}

register_test!(CpuInstructionTgei);

impl Test for CpuInstructionTgei {
    trap_imm!(Tgei);
}

register_test!(CpuInstructionTgeiu);

impl Test for CpuInstructionTgeiu {
    trap_imm!(Tgeiu);
}

register_test!(CpuInstructionTlti);

impl Test for CpuInstructionTlti {
    trap_imm!(Tlti);
}

register_test!(CpuInstructionTltiu);

impl Test for CpuInstructionTltiu {
    trap_imm!(Tltiu);
}

register_test!(CpuInstructionTeqi);

impl Test for CpuInstructionTeqi {
    trap_imm!(Teqi);
}

register_test!(CpuInstructionTnei);

impl Test for CpuInstructionTnei {
    trap_imm!(Tnei);
}
