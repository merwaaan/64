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
