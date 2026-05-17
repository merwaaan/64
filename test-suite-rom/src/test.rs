extern crate alloc;

use alloc::{string::String, vec::Vec};
use anyhow::Result;
use test_suite_common::Message;

use crate::app::App;

/// Tests must implement this trait.
pub trait Test {
    /// The parameters passed to each test case.
    type Params: core::fmt::Debug;

    /// The name of the test.
    fn name() -> &'static str {
        core::any::type_name::<Self>()
            .rfind("::")
            .map(|i| &core::any::type_name::<Self>()[i + 2..])
            .unwrap_or(core::any::type_name::<Self>())
    }

    /// Defines a parameter set for each test case.
    fn cases() -> Vec<Self::Params>;

    /// Generates a name for a test case from its parameters.
    fn case_name(params: &Self::Params) -> String;

    /// Runs all the test cases.
    fn run_all(app: &mut App) -> Result<()> {
        app.send(Message::TestStarted)?;

        for params in Self::cases() {
            app.test_case(Self::case_name(&params))?;

            Self::run(&params, app)?;
        }

        app.send(Message::TestCompleted)
    }

    /// Runs a single test case.
    fn run(params: &Self::Params, app: &mut App) -> Result<()>;
}

/// Helper to avoid having to specify empty boilerplate for tests without parameters.
#[macro_export]
macro_rules! no_params {
    () => {
        type Params = ();

        fn cases() -> Vec<Self::Params> {
            Vec::from([()])
        }

        fn case_name(params: &Self::Params) -> String {
            "".to_string()
        }
    };
}
