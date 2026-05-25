//! Records the VI registers masking when they are written to (or read from?).
//!
//! Findings:
//! - Control: bits 31-17 are not writable, as specified in the docs, but bit 10 is, even though it's unused
//! - Vertical scale: bits 27-16 and 11-0 are writable, as specified in the docs, bits 27-26 are writable even if unused
//!
//! No surprises:
//! - Origin: bits 23-0 writable
//! - Width: bits 11-0 writable
//! - Interrupt line: bits 9-0 writable
//! - Burst: bits 29-0 writable
//! - Vertical total: bits 9-0 writable
//! - Horizontal total: bits 20-16 and 11-0 writable
//! - Horizontal total leap: bits 27-16 and 11-0 writable
//! - Horizontal video: bits 25-16 and 9-0 writable
//! - Vertical video: bits 25-16 and 9-0 writable
//! - Vertical burst: bits 25-16 and 9-0 writable
//! - Horizontal scale: bits 27-16 and 11-0 writable

// TODO test writes to high bits of VI CURRENT clear int?

use alloc::format;
use n64_specs::vi;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

use strum::IntoEnumIterator;

register_test!(ViRegistersMasking);

impl Test for ViRegistersMasking {
    type Params = vi::Register;

    fn cases() -> impl Iterator<Item = Self::Params> {
        vi::Register::iter()
            // Ignore the Current line register as it's constantly updated by the video timing circuitry
            .filter(|reg| reg != &vi::Register::CurrentLine)
    }

    fn run(reg: &vi::Register, app: &mut App) -> Result<(), TestError> {
        app.comment(&format!("Test masking of VI {} register", reg))?;

        // Save/Restore the register value so as not to break display
        let saved = io::read_uncached(reg.address());

        app.comment("Clear all bits")?;
        io::write_uncached(reg.address(), 0x0000_0000);
        app.value(io::read_uncached(reg.address()))?;

        app.comment("Set all bits")?;
        io::write_uncached(reg.address(), 0xFFFF_FFFF);
        app.value(io::read_uncached(reg.address()))?;

        io::write_uncached(reg.address(), saved);

        Ok(())
    }
}
