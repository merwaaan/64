//! This test records how the coprocessor 0 registers are masked.
//!
//! Findings:
//! - XContext: unused bits 3-0 are actually writable
//! - TagLo: unused bits 31-28 and 5-0 are actually writable
//! - The unused registers are masked differently
//!   -  7: 0x0000_03A8
//!   - 21: 0x0000_2040
//!   - 22: 0x0000_20B0
//!   - 23: 0x0000_20B0
//!   - 24: 0x0000_03C8
//!   - 25: 0x0000_03C8
//!   - 31: 0x0000_2010
//!  TODO seems to be different when the ROM changes?!
//!
//! No surprises:
//! - All the used registers are masked as documented

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use specs::cop0::Register;
use strum::IntoEnumIterator;

macro_rules! for_cop0_reg {
    ($reg:expr, $row:ident $(, $($extra:tt)* )? ) => {
        match $reg {
            Register::Index => $row!(0 $(, $($extra)*)? ),
            Register::Random => $row!(1 $(, $($extra)*)? ),
            Register::EntryLo0 => $row!(2 $(, $($extra)*)? ),
            Register::EntryLo1 => $row!(3 $(, $($extra)*)? ),
            Register::Context => $row!(4 $(, $($extra)*)? ),
            Register::PageMask => $row!(5 $(, $($extra)*)? ),
            Register::Wired => $row!(6 $(, $($extra)*)? ),
            Register::Unused7 => $row!(7 $(, $($extra)*)? ),
            Register::BadVAddr => $row!(8 $(, $($extra)*)? ),
            Register::Count => $row!(9 $(, $($extra)*)? ),
            Register::EntryHi => $row!(10 $(, $($extra)*)? ),
            Register::Compare => $row!(11 $(, $($extra)*)? ),
            Register::Status => $row!(12 $(, $($extra)*)? ),
            Register::Cause => $row!(13 $(, $($extra)*)? ),
            Register::ExceptionPC => $row!(14 $(, $($extra)*)? ),
            Register::PRId => $row!(15 $(, $($extra)*)? ),
            Register::Config => $row!(16 $(, $($extra)*)? ),
            Register::LLAddr => $row!(17 $(, $($extra)*)? ),
            Register::WatchLo => $row!(18 $(, $($extra)*)? ),
            Register::WatchHi => $row!(19 $(, $($extra)*)? ),
            Register::XContext => $row!(20 $(, $($extra)*)? ),
            Register::Unused21 => $row!(21 $(, $($extra)*)? ),
            Register::Unused22 => $row!(22 $(, $($extra)*)? ),
            Register::Unused23 => $row!(23 $(, $($extra)*)? ),
            Register::Unused24 => $row!(24 $(, $($extra)*)? ),
            Register::Unused25 => $row!(25 $(, $($extra)*)? ),
            Register::PErr => $row!(26 $(, $($extra)*)? ),
            Register::CacheErr => $row!(27 $(, $($extra)*)? ),
            Register::TagLo => $row!(28 $(, $($extra)*)? ),
            Register::TagHi => $row!(29 $(, $($extra)*)? ),
            Register::ErrorPC => $row!(30 $(, $($extra)*)? ),
            Register::Unused31 => $row!(31 $(, $($extra)*)? ),
        }
    };
}

#[inline(always)]
fn mfc0_const<const REG: u32>() -> u32 {
    let value: u32;

    unsafe {
        asm!(
            ".set noat",
            "mfc0 {value}, ${reg}",
            value = out(reg) value,
            reg = const REG
        );
    }

    value
}

#[inline(always)]
fn mtc0_const<const REG: u32>(value: u32) {
    unsafe {
        asm!(
            ".set noat",
            "mtc0 {value}, ${reg}",
            value = in(reg) value,
            reg = const REG
        );
    }
}

macro_rules! mfc0 {
    ($reg_index:literal) => {
        mfc0_const::<$reg_index>()
    };
}

macro_rules! mtc0 {
    ($reg_index:literal, $value:expr) => {
        mtc0_const::<$reg_index>($value)
    };
}

test_suite_rom::run_test!(Cop0RegistersMasking);

impl Test for Cop0RegistersMasking {
    no_params!();

    // TODO 64bits
    // TODO status/config without side effects bits

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        let regs = Register::iter().filter(|reg| {
            !matches!(
                reg,
                // Writing to Status or Cause has side effects so we don't test them
                Register::Status | Register::Config |
                // Ignore Random and Count as their values change over time
                Register::Random | Register::Count
            )
        });

        for reg in regs {
            // Registers have unpredictable initial values so we record which bits are writable instead of the exact values

            for_cop0_reg!(reg, mtc0, 0);
            let zeroed = for_cop0_reg!(reg, mfc0);

            for_cop0_reg!(reg, mtc0, u32::MAX);
            let maxed = for_cop0_reg!(reg, mfc0);

            let writable = maxed & !zeroed;

            app.comment(&format!("{:?}", reg))?;
            app.value(writable)?;
        }

        Ok(())
    }
}
