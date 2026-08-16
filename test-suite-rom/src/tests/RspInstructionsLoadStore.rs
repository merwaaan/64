use alloc::format;
use arbitrary_int::prelude::*;
use n64_specs::{
    cpu::registers::Register as CpuRegister,
    rsp::{DMEM_START, instructions::*},
};

use crate::{
    app::App,
    data::INIT_32,
    io, register_test,
    rsp_program::RspProgram,
    test::{Test, TestError},
};

#[derive(Debug)]
pub struct Params {
    rt_value: u32,
    rd: u5,
    e: u4,
}

register_test!(RspInstructionMtc2);

impl Test for RspInstructionMtc2 {
    type Params = Params;

    fn cases() -> impl Iterator<Item = Self::Params> {
        let rd = (0..8).map(u5::new);

        let e = (0..=15).map(u4::new);

        itertools::iproduct!(rd, e).map(|(rd, e)| Params {
            rt_value: 0xD07F_1234,
            rd,
            e,
        })
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        RspProgram::new()
            .clear_vec_regs()
            .set_reg(CpuRegister::T0, params.rt_value)
            .push(
                Mtc2::default()
                    .with_rt(CpuRegister::T0.into())
                    .with_rd(params.rd)
                    .with_e(params.e)
                    .into(),
            )
            .push(
                Sqv::default()
                    .with_base(CpuRegister::R0.into()) // = DMEM_START
                    .with_vt(params.rd)
                    .with_e(u4::ZERO)
                    .with_voffset(u7::ZERO)
                    .into(),
            )
            .run();

        // TODO check other vregs?

        app.memory_region(
            &format!(
                "MTC2 T0={:08X}, v{}[{}]",
                params.rt_value, params.rd, params.e
            ),
            io::uncached_addr(DMEM_START),
            16,
        )
    }
}

register_test!(RspInstructionMfc2);

impl Test for RspInstructionMfc2 {
    type Params = Params;

    fn cases() -> impl Iterator<Item = Self::Params> {
        let rd = (0..8).map(u5::new);

        let e = (0..=15).map(u4::new);

        itertools::iproduct!(rd, e).map(|(rd, e)| Params {
            rt_value: 0xD07F_1234, // TODO remove field
            rd,
            e,
        })
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Fill DMEM

        for offset in (0..(16 * 8)).step_by(4) {
            // Only 32-bit writes work reliably
            let word = (offset << 24) | ((offset + 1) << 16) | ((offset + 2) << 8) | (offset + 3);

            io::write_uncached(DMEM_START + offset, word);
        }

        // Copy to vector registers

        let mut program = RspProgram::new();

        for vreg in 0..8 {
            program.push(
                Lqv::default()
                    .with_base(CpuRegister::R0.into()) // = DMEM_START
                    .with_voffset(u7::new(vreg * 16))
                    .with_vt(u5::new(vreg))
                    .into(),
            );
        }

        // MFC2

        program
            .set_reg(CpuRegister::T0, INIT_32)
            .push(
                Mfc2::default()
                    .with_rt(CpuRegister::T0.into())
                    .with_rd(params.rd)
                    .with_e(params.e)
                    .into(),
            )
            .push(
                Sw::default()
                    .with_rt(CpuRegister::T0.into())
                    .with_offset(0x500)
                    .with_base(CpuRegister::R0.into())
                    .into(),
            )
            .run();

        // TODO not working, only records 0
        app.memory(
            &format!("MFC2 T0, v{}[{}]", params.rd, params.e),
            io::uncached_addr(DMEM_START + 500),
        )
    }
}
