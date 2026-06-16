use alloc::format;
use anyhow::anyhow;
use core::arch::asm;
use n64_specs::cpu::{instructions::*, registers::Register};

use crate::{
    app::App,
    data::{INIT_64, corner_cases_16},
    exceptions::{ExceptionHandler, install_exception_handler},
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};
// TODO generalize
struct ExceptionH {
    occurred: bool,
    cause: u32,
}

impl ExceptionHandler for ExceptionH {
    fn run(&mut self) {
        // TODO check actual cause!
        // TODO assert

        self.occurred = true;

        let cause: u32;

        unsafe {
            asm!(
                "mfc0 $t0, $14",
                "addiu $t0, $t0, 4",
                "mtc0 $t0, $14",
                "mfc0 {cause}, $13",
                cause = out(reg) cause,
                options(nostack, preserves_flags),
            );
        }

        self.cause = cause;
    }
}

// LB, LH, LW, LD
// LBU, LHU, LWU
// LWL, LWR, LDL, LDR
// LL, LLD

#[derive(Debug)]
pub struct Params {
    rt: Register,
    offset: u16,
}

const RT: [Register; 2] = [Register::T0, Register::R0];

const EXTRA_OFFSETS: [u16; 18] = [
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0x100, 0xCDEF,
];

macro_rules! load_variant {
    ($instr:ident) => {
        type Params = Params;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let offsets = corner_cases_16(&EXTRA_OFFSETS);

            itertools::iproduct!(RT, offsets).map(|(rt, offset)| Params { rt, offset })
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            // Fill RAM with test data
            //
            // We'll target the middle of that buffer since offset are signed so data might get loaded before or after the address

            let data_size = 0xFFFF + 0x100; // max offset + some generous margin

            let mut ram = io::CachedBuffer::<u8>::with_alignment(data_size, 8);
            let ram_mid = (ram.as_ptr() as u32 + (data_size as u32) / 2) & !7;

            for i in 0..data_size {
                ram.set(i, i as u8);
            }

            // Reset LLAddr

            unsafe {
                asm!(
                    "mtc0 $0, $17",
                    options(nostack, preserves_flags),
                );
            }

            // Load

            let result = io::CachedBuffer::<u64>::from_slice(&[0]);

            let ex_handler = install_exception_handler(ExceptionH {
                occurred: false,
                cause: 0,
            });

            Program::new()
                .set_reg64(params.rt, INIT_64)
                .set_reg64(Register::T1, ram_mid as i32 as u64)
                .push(
                    $instr::default()
                        .with_rt(params.rt.into())
                        .with_base(Register::T1.into())
                        .with_offset(params.offset)
                        .into(),
                )
                .store_reg64(params.rt, result.as_ptr() as u32, Register::T7)
                .run();

            app.value64(
                &format!(
                    "{} {}, {:08X}({}={:08X})",
                    stringify!($instr).to_uppercase(),
                    params.rt,
                    params.offset,
                    Register::T1,
                    ram_mid
                ),
                result.get(0),
            )?;

             // LLAddr being an unstable address, we can't record its exact value, but we can check if it changed

            let llAddr: u32;

            unsafe {
                asm!(
                    "mfc0 {llAddr}, $17",
                    llAddr = out(reg) llAddr,
                    options(nostack, preserves_flags),
                );
            }


            app.bool("LLAddr changed", llAddr != 0)?;

            // TODO read LLbit?

            app.bool("Exception", ex_handler.occurred)?;
            app.value("Exception code", ex_handler.cause >> 2)
        }
    };
}

register_test!(CpuInstructionLb);

impl Test for CpuInstructionLb {
    load_variant!(Lb);
}

register_test!(CpuInstructionLh);

impl Test for CpuInstructionLh {
    load_variant!(Lh);
}

register_test!(CpuInstructionLw);

impl Test for CpuInstructionLw {
    load_variant!(Lw);
}

register_test!(CpuInstructionLd);

impl Test for CpuInstructionLd {
    load_variant!(Ld);
}

register_test!(CpuInstructionLbu);

impl Test for CpuInstructionLbu {
    load_variant!(Lbu);
}

register_test!(CpuInstructionLhu);

impl Test for CpuInstructionLhu {
    load_variant!(Lhu);
}

register_test!(CpuInstructionLwu);

impl Test for CpuInstructionLwu {
    load_variant!(Lwu);
}

register_test!(CpuInstructionLwl);

impl Test for CpuInstructionLwl {
    load_variant!(Lwl);
}

register_test!(CpuInstructionLwr);

impl Test for CpuInstructionLwr {
    load_variant!(Lwr);
}

register_test!(CpuInstructionLdl);

impl Test for CpuInstructionLdl {
    load_variant!(Ldl);
}

register_test!(CpuInstructionLdr);

impl Test for CpuInstructionLdr {
    load_variant!(Ldr);
}

register_test!(CpuInstructionLl);

impl Test for CpuInstructionLl {
    load_variant!(Ll);
}

register_test!(CpuInstructionLld);

impl Test for CpuInstructionLld {
    load_variant!(Lld);
}

// SB, SH, SW, SD
// SWL, SWR, SDL, SDR
// SC, SCD

macro_rules! store_variant {
    ($instr:ident) => {
        type Params = Params;

        fn cases() -> impl Iterator<Item = Self::Params> {
            let offsets = corner_cases_16(&EXTRA_OFFSETS);

            itertools::iproduct!(RT, offsets).map(|(rt, offset)| Params { rt, offset })
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            // Fill RAM with test data

            let data_size = 0xFFFF + 0x100; // max offset + some generous margin

            let mut ram = io::CachedBuffer::<u8>::with_alignment(data_size, 8);
            let ram_mid = (ram.as_ptr() as u32 + (data_size as u32) / 2) & !7;

            for i in 0..data_size {
                ram.set(i, i as u8);
            }

            // Reset LLAddr

            // unsafe {
            //     asm!(
            //         "mtc0 $0, $17",
            //         options(nostack, preserves_flags),
            //     );
            // }

            // Store

            let ex_handler = install_exception_handler(ExceptionH {
                occurred: false,
                cause: 0,
            });

            Program::new()
                .set_reg64(params.rt, 0x1234_5678_ABCD_EF01)
                .set_reg64(Register::T1, ram_mid as i32 as u64)
                .push(
                    $instr::default()
                        .with_rt(params.rt.into())
                        .with_base(Register::T1.into())
                        .with_offset(params.offset)
                        .into(),
                )
                .run();

            // We can't realistically record all the RAM but let's at least record the region around the source address

            let start = (ram_mid + (params.offset as i16 as i32 - 0x10) as u32) & !3;

            app.memory_region(
                &format!(
                    "{} {}, {:08X}({}={:08X}) (reading 0x20 bytes from {:08X})",
                    stringify!($instr).to_uppercase(),
                    params.rt,
                    params.offset,
                    Register::T1,
                    ram_mid,
                    start
                ),
                start,
                0x20,
            )?;

            // LLAddr being an unstable address, we can't record its exact value, but we can check if it changed

            // let llAddr: u32;

            // unsafe {
            //     asm!(
            //         "mfc0 {llAddr}, $17",
            //         llAddr = out(reg) llAddr,
            //         options(nostack, preserves_flags),
            //     );
            // }

            //app.bool("LLAddr changed", llAddr != 0)?;

            // TODO read LLbit?

            app.bool("Exception", ex_handler.occurred)?;
            app.value("Exception code", ex_handler.cause >> 2)
        }
    };
}

register_test!(CpuInstructonSb);

impl Test for CpuInstructonSb {
    store_variant!(Sb);
}

register_test!(CpuInstructonSh);

impl Test for CpuInstructonSh {
    store_variant!(Sh);
}

register_test!(CpuInstructonSw);

impl Test for CpuInstructonSw {
    store_variant!(Sw);
}

register_test!(CpuInstructonSd);

impl Test for CpuInstructonSd {
    store_variant!(Sd);
}

register_test!(CpuInstructonSwl);

impl Test for CpuInstructonSwl {
    store_variant!(Swl);
}

register_test!(CpuInstructonSwr);

impl Test for CpuInstructonSwr {
    store_variant!(Swr);
}

register_test!(CpuInstructonSdl);

impl Test for CpuInstructonSdl {
    store_variant!(Sdl);
}

register_test!(CpuInstructonSdr);

impl Test for CpuInstructonSdr {
    store_variant!(Sdr);
}

// TODO sep test?
// register_test!(CpuInstructonSc);

// impl Test for CpuInstructonSc {
//     store_variant!(Sc);
// }

// register_test!(CpuInstructonScd);

// impl Test for CpuInstructonScd {
//     store_variant!(Scd);
// }
