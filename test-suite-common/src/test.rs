extern crate alloc;
use alloc::{string::String, vec::Vec};

use crate::result::{TestCaseResult, TestResult};

macro_rules! test_name {
    () => {
        fn name() -> &'static str {
            core::any::type_name::<Self>()
                .rfind("::")
                .map(|i| &core::any::type_name::<Self>()[i + 2..])
                .unwrap_or(core::any::type_name::<Self>())
        }
    };
}

/// Simple test without parameters, will be executed as a single test case
pub trait TestNoParams {
    test_name!();

    fn run_all() -> TestResult {
        let mut result = TestResult::new(Self::name());

        let mut case_result = TestCaseResult::new(None);

        Self::run(&mut case_result);

        result.cases.push(case_result);

        result
    }

    fn run(result: &mut TestCaseResult);
}

/// Test with parameters, will be split into separate test cases
pub trait TestWithParams {
    test_name!();

    type Params: core::fmt::Debug;

    /// Defines a parameter set for each test case.
    fn cases() -> Vec<Self::Params>;

    /// Generates a name for a test case from its parameters.
    fn case_name(params: &Self::Params) -> String;

    fn run_all() -> TestResult {
        let mut result = TestResult::new(Self::name());

        for params in Self::cases() {
            let case_name = Self::case_name(&params);

            let mut case_result = TestCaseResult::new(Some(case_name));

            Self::run(&params, &mut case_result);

            result.cases.push(case_result);
        }

        result
    }

    /// Runs a single test case.
    fn run(params: &Self::Params, result: &mut TestCaseResult);
}
