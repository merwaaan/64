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
    exceptions::{ExceptionHandler, install_exception_handler},
    program::Program,
    register_test,
    test::{Test, TestError},
};

const REGISTERS: [Register; 4] = [Register::R0, Register::T0, Register::T1, Register::T2];

const REGISTER_VALUES: [u64; 15] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x0000_0000_0000_CD15,
    0x0000_0000_2640_044E,
    0x0000_0000_5555_5555,
    0x0000_0000_7FFF_FFFF,
    0x0000_0000_8008_00F0,
    0x0000_0000_AAAA_AAAA,
    0x0000_0000_DBCA_0000,
    0x0000_0000_FFFF_FFFF,
    0x105C_00CE_0000_0000,
    0xC000_FFFF_0000_0007,
    0xFFFF_002F_89AB_F51F,
    0xFFFF_FFFF_0000_0001,
    0xFFFF_FFFF_FFFF_FFFF,
];

#[derive(Debug)]
pub struct RegisterParam {
    reg_value1: u64,
    reg_value2: u64,
    reg_in1: Register,
    reg_in2: Register,
    reg_out: Register,
}

macro_rules! reg_variant {
    ($instr:ident) => {
        type Params = RegisterParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            itertools::iproduct!(
                REGISTERS,
                REGISTER_VALUES,
                REGISTERS,
                REGISTER_VALUES,
                REGISTERS
            )
            .map(
                |(reg_in1, reg_value1, reg_in2, reg_value2, reg_out)| RegisterParam {
                    reg_in1,
                    reg_value1,
                    reg_in2,
                    reg_value2,
                    reg_out,
                },
            )
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            struct ExceptionH {
                occurred: bool,
            }

            impl ExceptionHandler for ExceptionH {
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

            let handler = install_exception_handler(ExceptionH { occurred: false });

            let result: u64 = 0;

            Program::new()
                .set_reg64(params.reg_in1, params.reg_value1)
                .set_reg64(params.reg_in2, params.reg_value2)
                .set_reg64(params.reg_out, 0xAAAA_BBBB_CCCC_DDDD)
                .push(
                    $instr::default()
                        .with_rs(params.reg_in1.into())
                        .with_rt(params.reg_in2.into())
                        .with_rd(params.reg_out.into())
                        .into(),
                )
                .store_reg64(
                    params.reg_out,
                    core::ptr::addr_of!(result) as u32,
                    Register::T3,
                )
                .run();

            app.value64(
                &format!(
                    "{} {}, {}={:08X}, {}={:08X}",
                    stringify!($instr).to_uppercase(),
                    params.reg_out,
                    params.reg_in1,
                    params.reg_value1,
                    params.reg_in2,
                    params.reg_value2,
                ),
                result,
            )?;

            app.bool("Exception", handler.occurred)
        }
    };
}

register_test!(CpuInstructionAdd);

impl Test for CpuInstructionAdd {
    reg_variant!(Add);
}

register_test!(CpuInstructionAddu);

impl Test for CpuInstructionAddu {
    reg_variant!(Addu);
}

register_test!(CpuInstructionSub);

impl Test for CpuInstructionSub {
    reg_variant!(Sub);
}

register_test!(CpuInstructionSubu);

impl Test for CpuInstructionSubu {
    reg_variant!(Subu);
}

register_test!(CpuInstructionSlt);

impl Test for CpuInstructionSlt {
    reg_variant!(Slt);
}

register_test!(CpuInstructionSltu);

impl Test for CpuInstructionSltu {
    reg_variant!(Sltu);
}

register_test!(CpuInstructionDadd);

impl Test for CpuInstructionDadd {
    reg_variant!(Dadd);
}

register_test!(CpuInstructionDaddu);

impl Test for CpuInstructionDaddu {
    reg_variant!(Daddu);
}

register_test!(CpuInstructionDsub);

impl Test for CpuInstructionDsub {
    reg_variant!(Dsub);
}

register_test!(CpuInstructionDsubu);

impl Test for CpuInstructionDsubu {
    reg_variant!(Dsubu);
}

#[derive(Debug)]
pub struct ImmediateParam {
    reg_in: Register,
    reg_in_value: u64,
    imm_value: u16,
    reg_out: Register,
}

macro_rules! imm_variant {
    ($instr:ident) => {
        type Params = ImmediateParam;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let imm_values = [
                0x0000, 0x0001, 0x0002, 0x00C5, 0x04F0, 0x7FFF, 0x8000, 0x8001, 0xAAAA, 0xFFFE,
                0xFFFF,
            ];

            itertools::iproduct!(REGISTERS, REGISTER_VALUES, imm_values, REGISTERS).map(
                |(reg_in, reg_in_value, imm_value, reg_out)| ImmediateParam {
                    reg_in,
                    reg_in_value,
                    imm_value,
                    reg_out,
                },
            )
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            struct ExceptionH {
                occurred: bool,
            }

            impl ExceptionHandler for ExceptionH {
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

            let handler = install_exception_handler(ExceptionH { occurred: false });

            let result: u64 = 0;

            Program::new()
                .set_reg64(params.reg_in, params.reg_in_value)
                .set_reg64(params.reg_out, 0xAAAA_BBBB_CCCC_DDDD)
                .push(
                    $instr::default()
                        .with_rt(params.reg_out.into())
                        .with_rs(params.reg_in.into())
                        .with_imm(params.imm_value)
                        .into(),
                )
                .store_reg64(
                    params.reg_out,
                    core::ptr::addr_of!(result) as u32,
                    Register::T3,
                )
                .run();

            app.value64(
                &format!(
                    "{} {}, {}={:08X}, {:08X}",
                    stringify!($instr).to_uppercase(),
                    params.reg_out,
                    params.reg_in,
                    params.reg_in_value,
                    params.imm_value,
                ),
                result,
            )?;

            app.bool("Exception", handler.occurred)
        }
    };
}

register_test!(CpuInstructionAddi);

impl Test for CpuInstructionAddi {
    imm_variant!(Addi);
}

register_test!(CpuInstructionAddiu);

impl Test for CpuInstructionAddiu {
    imm_variant!(Addiu);
}

register_test!(CpuInstructionDaddi);

impl Test for CpuInstructionDaddi {
    imm_variant!(Daddi);
}

register_test!(CpuInstructionDaddiu);

impl Test for CpuInstructionDaddiu {
    imm_variant!(Daddiu);
}

register_test!(CpuInstructionSlti);

impl Test for CpuInstructionSlti {
    imm_variant!(Slti);
}

register_test!(CpuInstructionSltiu);

impl Test for CpuInstructionSltiu {
    imm_variant!(Sltiu);
}
