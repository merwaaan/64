#![no_std]

extern crate alloc;
use alloc::{format, string::String, vec::Vec};
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

    /// Returns a description of the first difference with another test result.
    pub fn first_diff(&self, expected: &TestResult) -> Option<String> {
        if self.name != expected.name {
            return Some(format!(
                "different test name:\ncurrent = {}\nexpected = {}",
                self.name, expected.name
            ));
        }

        if self.cases.len() != expected.cases.len() {
            return Some(format!(
                "different case count:\ncurrent = {}\nexpected = {}",
                self.cases.len(),
                expected.cases.len()
            ));
        }

        for (i, (own_case, expected_case)) in
            self.cases.iter().zip(expected.cases.iter()).enumerate()
        {
            if own_case.name != expected_case.name {
                return Some(format!(
                    "different name for case #{i}:\ncurrent = {:?}\nexpected = {:?}",
                    own_case.name, expected_case.name
                ));
            }

            if own_case.states.len() != expected_case.states.len() {
                return Some(format!(
                    "different state count for case #{i}:\ncurrent = {}\nexpected = {}",
                    own_case.states.len(),
                    expected_case.states.len()
                ));
            }

            for (j, (own_state, expected_state)) in own_case
                .states
                .iter()
                .zip(expected_case.states.iter())
                .enumerate()
            {
                if own_state != expected_state {
                    return Some(format!(
                        "different state for case #{i} state #{j}:\ncurrent = {:?}\nexpected = {:?}",
                        own_state, expected_state
                    ));
                }
            }
        }

        None
    }
}

/// Result of a test case
#[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCaseResult {
    name: Option<String>,
    states: Vec<State>,
}

impl TestCaseResult {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            states: Vec::new(),
        }
    }

    pub fn push_comment(&mut self, comment: &str) {
        self.states.push(State::Comment(String::from(comment)));
    }

    pub fn push_value(&mut self, value: u32) {
        self.states.push(State::Value(value));
    }

    pub fn push_pc(&mut self) {
        self.states.push(State::Pc(0)); // TODO get actual PC
    }

    pub fn push_memory(&mut self, address: u32) {
        let value = unsafe {
            ((n64_specs::map::Segment::KSEG1 as u32 | address) as *mut u32).read_volatile()
        };

        self.states.push(State::Memory { address, value });
    }
}

/// Piece of state in a test case
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum State {
    /// A descriptive comment
    Comment(String),
    /// Some arbitrary value relevant to the test
    Value(u32),
    /// Program counter TODO use it?
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
