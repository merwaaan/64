//! This test record the value of the MI Version register.
//! It might be different on different hardware revisions though.

#![no_std]
#![no_main]

test_suite_rom::run_test!(MiVersionRegisterValue);

impl Test for MiVersionRegisterValue {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<()> {
        app.value(io::read_uncached(specs::mi::Version::ADDRESS))
    }
}
