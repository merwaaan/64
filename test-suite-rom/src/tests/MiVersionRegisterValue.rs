//! This test record the value of the MI Version register.
//! It might be different on different hardware revisions though.

use n64_specs::mi;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

register_test!(MiVersionRegisterValue);

impl Test for MiVersionRegisterValue {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        app.value(
            "MI Version register",
            io::read_uncached(mi::Version::ADDRESS),
        )
    }
}
