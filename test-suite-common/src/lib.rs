#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Result of a test
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestResult {
    pub name: String,
    pub cases: Vec<TestCaseResult>,
}

impl TestResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            cases: Vec::new(),
        }
    }
}

/// Result of a test case
#[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCaseResult {
    pub name: Option<String>,
    pub states: Vec<State>,
}

impl TestCaseResult {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            states: Vec::new(),
        }
    }
}

/// Piece of state in a test case
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum State {
    /// An informative comment
    Comment(String),
    /// Program counter
    Pc(u32),
    /// Memory read
    Memory { address: u32, value: u32 },
}

/// Trait for tests
pub trait Test {
    type Params: Default + core::fmt::Debug;

    /// Returns the name of the test.
    fn name() -> &'static str {
        core::any::type_name::<Self>()
            .rfind("::")
            .map(|i| &core::any::type_name::<Self>()[i + 2..])
            .unwrap_or(core::any::type_name::<Self>())
    }

    /// Defines a parameter set for each test case.
    fn cases() -> Vec<Self::Params> {
        Vec::from([Self::Params::default()])
    }

    /// Generates a name for a test case from its parameters.
    fn case_name(_params: &Self::Params) -> Option<String> {
        None
    }

    /// Runs the whole test.
    fn run() -> TestResult {
        let mut result = TestResult::new(Self::name());

        for params in Self::cases() {
            let case_name = Self::case_name(&params);

            let mut case_result = TestCaseResult::new(case_name);

            Self::run_case(&params, &mut case_result);

            result.cases.push(case_result);
        }

        result
    }

    /// Runs a single test case.
    fn run_case(params: &Self::Params, result: &mut TestCaseResult);
}

/// Message sent from the ROM to the server
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Message to test the connection
    Hello,
    /// Message to send the test results
    TestResult(TestResult),
    /// Message to communicate that the ROM panicked
    Panic,
}
