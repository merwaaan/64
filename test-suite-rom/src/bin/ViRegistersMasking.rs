//! This test records the masking applied to the VI registers when they are written to (or read from?).
//!
//! Findings:
//! - Control: bits 31-17 are not writable, as specified in the docs, but bit 10 is, even though it's unused
//! - Vertical scale: bits 27-16 and 11-0 are writable, bits 27-26 are writable even if unused
//!
//! No surprises:
//! - Origin: only bits 23-0 are writable
//! - Width: only bits 11-0 are writable
//! - Interrupt line: only bits 9-0 are writable
//! - Burst: only bits 29-0 are writable
//! - Vertical total: only bits 9-0 are writable
//! - Horizontal total: only bits 20-16 and 11-0 are writable
//! - Horizontal total leap: only bits 27-16 and 11-0 are writable
//! - Horizontal video: only bits 25-16 and 9-0 are writable
//! - Vertical video: only bits 25-16 and 9-0 are writable
//! - Vertical burst: only bits 25-16 and 9-0 are writable
//! - Horizontal scale: only bits 27-16 and 11-0 are writable

// TODO test writes to high bits of VI CURRENT clear int?

#![no_std]
#![no_main]

use strum::IntoEnumIterator;

test_suite_rom::run_test! {
    TestWithParams ViRegistersMasking {
        type Params = specs::vi::Register;

        fn cases() -> Vec<Self::Params> {
            specs::vi::Register::iter()
                // Ignore the current line register as it's constantly updated by the video timing circuitry
                .filter(|reg| reg != &specs::vi::Register::CurrentLine)
                .collect()
        }

        fn case_name(params: &Self::Params) -> String {
            format!("{:?}", *params)
        }

        fn run(reg: &specs::vi::Register, result: &mut TestCaseResult) {
            unsafe {
                let reg_ptr = reg_mut_ptr(reg.address());

                // Save/Restore the register value so as not to break display
                let saved = reg_ptr.read_volatile();

                result.push_comment("Clear");
                reg_ptr.write_volatile(0x0000_0000);
                result.push_value(reg_ptr.read_volatile());

                result.push_comment("Set");
                reg_ptr.write_volatile(0xFFFF_FFFF);
                result.push_value(reg_ptr.read_volatile());

                reg_ptr.write_volatile(saved);
            };
        }
    }
}
