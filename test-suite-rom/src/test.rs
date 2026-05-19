extern crate alloc;

use alloc::{string::String, vec::Vec};
use anyhow::Result;
use test_suite_common::Step;

use crate::app::App;

/// Tests must implement this trait.
pub trait Test {
    /// The parameters passed to each test case.
    type Params: core::fmt::Debug;

    /// The name of the test (its type name).
    fn name() -> &'static str {
        core::any::type_name::<Self>()
            .rfind("::")
            .map(|i| &core::any::type_name::<Self>()[i + 2..])
            .unwrap_or(core::any::type_name::<Self>())
    }

    /// Defines a parameter set for each test case.
    fn cases() -> Vec<Self::Params>;

    /// Runs a single test case.
    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError>;
}

/// Helper to avoid having to specify empty boilerplate for tests without parameters.
#[macro_export]
macro_rules! no_params {
    () => {
        type Params = ();

        fn cases() -> Vec<Self::Params> {
            Vec::from([()])
        }
    };
}

/// Errors that can occur when running a test.
///
/// Either a true error with `Other`, or a comparison mismatch with `ComparisonMismatch`.
///
/// It's convenient to model a mismatch as an error as it can then interrupt failed tests immediately.
pub enum TestError {
    Mismatch(Mismatch),
    Other(anyhow::Error),
}

impl From<anyhow::Error> for TestError {
    fn from(e: anyhow::Error) -> Self {
        TestError::Other(e)
    }
}

impl From<TestError> for anyhow::Error {
    fn from(err: TestError) -> Self {
        match err {
            TestError::Mismatch(mismatch) => {
                anyhow::anyhow!("comparison mismatch {:?}", mismatch)
            }
            TestError::Other(err) => err,
        }
    }
}

#[derive(Debug, strum::Display)]
pub enum Mismatch {
    /// The recorded values are different from the runtime values.
    DifferentStep {
        expected_step: Step,
        step_index: u32,
    },
    /// The runtime test emitted extra steps after all the recorded steps have been compared.
    ExcessSteps { step_index: u32 },
    /// The runtime test has completed without comparing all the recorded steps.
    MissingSteps { step_index: u32 },
}
