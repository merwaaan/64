//! TODO

use alloc::format;
use arbitrary_int::prelude::*;
use n64_specs::cpu::{instructions::Lb, registers::Register};

use crate::{
    app::App,
    io,
    program::Program,
    register_test,
    test::{Test, TestError},
};

#[derive(Debug)]
pub struct Params {
    rt: Register,
    base: Register,
    offset: u16,
}

const RT: [Register; 1] = [Register::R0 /* , Register::T0*/];
const BASE: [Register; 1] = [Register::T0 /*, Register::T1*/];
const OFFSETS: [u16; 1] = [
    0, /*, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0x100, 0x7FFF, 0x8000, 0xFFFF, 0xFFFE*/
];

macro_rules! load_variant {
    ($instr:ident) => {
        type Params = Params;

        fn cases() -> impl Iterator<Item = Self::Params> {
            itertools::iproduct!(RT, BASE, OFFSETS).map(|(rt, base, offset)| Params {
                rt,
                base,
                offset,
            })
        }

        fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
            let data_size = OFFSETS[OFFSETS.len() - 1] as usize + 0x10;

            let mut ram = io::Buffer::<u8>::with_alignment(data_size, 8);

            for i in 0..data_size {
                ram.push(i as u8);
            }

            let result: u64 = 0;
            // panic!(
            //     "{} {}, {}({:08X}) --- {:0X?} {:0X?} {:0X}",
            //     stringify!($instr).to_uppercase(),
            //     params.rt,
            //     params.base,
            //     params.offset,
            //     ram.as_ptr(),
            //     result.as_ptr(),
            //     data_size
            // );

            Program::new()
                .set_reg64(Register::T0, 0xA000_0000)
                .push(
                    $instr::default()
                        .with_rt(u5::ZERO)
                        .with_base(Register::T0.into())
                        .with_offset(0)
                        .into(),
                )
                //.store_reg64(params.rt, result.as_ptr() as u32, Register::T7)
                .run();

            // Program::new()
            //     .set_reg64(params.base, ram.as_ptr() as u64)
            //     .push(
            //         // $instr::default()
            //         //     .with_rt(params.rt.into())
            //         //     .with_base(params.base.into())
            //         //     .with_offset(params.offset)
            //         //     .into(),
            //     )
            //     //.store_reg64(params.rt, core::ptr::addr_of!(result) as u32, Register::T7)
            //     .run();

            app.value64(
                &format!(
                    "{} {}, {:08X}({})",
                    stringify!($instr).to_uppercase(),
                    params.rt,
                    params.offset,
                    params.base,
                ),
                result,
            )
        }
    };
}

register_test!(CpuInstructionLb);

impl Test for CpuInstructionLb {
    load_variant!(Lb);
}

// register_test!(CpuInstructionOr);

// impl Test for CpuInstructionOr {
//     reg_variant!(Or);
// }

// register_test!(CpuInstructionNor);

// impl Test for CpuInstructionNor {
//     reg_variant!(Nor);
// }

// register_test!(CpuInstructionXor);

// impl Test for CpuInstructionXor {
//     reg_variant!(Xor);
// }

// #[derive(Debug)]
// pub struct ImmediateParam {
//     reg_value: u32,
//     imm_value: u16,
//     reg_in: Register,
//     reg_out: Register,
// }

// macro_rules! imm_variant {
//     ($instr:ident) => {
//         type Params = ImmediateParam;

//         fn cases() -> impl Iterator<Item = Self::Params> {
//             let reg_values = [
//                 0,
//                 1,
//                 0x0000_CD15,
//                 0x2640_044E,
//                 0x5555_5555,
//                 0x7FFF_FFFF,
//                 /*0x8008_00F0,
//                 0xAAAA_AAAA,
//                 0xDBCA_0000,
//                 0xFFFF_FFFF,*/
//             ];

//             // TODO pick vals
//             let imm_values = [
//                 0, 1, 0xCD15, 0x044E, 0x5555,
//                 0xFFFF,
//                 /*0x8008_00F0,
//                 0xAAAA_AAAA,
//                 0xDBCA_0000,
//                 0xFFFF_FFFF,*/
//             ];

//             let regs = [
//                 Register::R0,
//                 Register::AT,
//                 Register::V0,
//                 Register::V1,
//                 Register::A1, // TODO?
//             ];

//             itertools::iproduct!(reg_values, imm_values, regs, regs).map(
//                 |(reg_value, imm_value, reg_in, reg_out)| ImmediateParam {
//                     reg_value,
//                     imm_value,
//                     reg_in,
//                     reg_out,
//                 },
//             )
//         }

//         fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
//             app.comment(&format!(
//                 "{} {}, {}={:08X}, {:08X}",
//                 "TODO name", params.reg_out, params.reg_in, params.reg_value, params.imm_value,
//             ))?;

//             let result: u64 = 0;

//             Program::new()
//                 .load_reg(params.reg_in, params.reg_value)
//                 .push(
//                     $instr::default()
//                         .with_rs(params.reg_in.into())
//                         .with_rt(params.reg_out.into())
//                         .with_imm(params.imm_value)
//                         .into(),
//                 )
//                 .load_reg(Register::T3, core::ptr::addr_of!(result) as u32) // TODO other reg func
//                 .sw(params.reg_out, Register::T3, 0)
//                 .run();

//             app.value(result)
//         }
//     };
// }

// register_test!(CpuInstructionAndi);

// impl Test for CpuInstructionAndi {
//     imm_variant!(Andi);
// }

// register_test!(CpuInstructionOri);

// impl Test for CpuInstructionOri {
//     imm_variant!(Ori);
// }

// register_test!(CpuInstructionXori);

// impl Test for CpuInstructionXori {
//     imm_variant!(Xori);
// }
