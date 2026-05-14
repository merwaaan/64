#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// /// Result of a test
// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// pub struct TestResult {
//     pub name: String,
//     pub cases: Vec<TestCaseResult>,
// }

// impl TestResult {
//     pub fn new(name: &str) -> Self {
//         Self {
//             name: String::from(name),
//             cases: Vec::new(),
//         }
//     }

//     /// Returns a description of the first difference with another test result.
//     pub fn first_diff(&self, expected: &TestResult) -> Option<String> {
//         if self.name != expected.name {
//             return Some(format!(
//                 "different test name:\ncurrent = {}\nexpected = {}",
//                 self.name, expected.name
//             ));
//         }

//         if self.cases.len() != expected.cases.len() {
//             return Some(format!(
//                 "different case count:\ncurrent = {}\nexpected = {}",
//                 self.cases.len(),
//                 expected.cases.len()
//             ));
//         }

//         for (i, (own_case, expected_case)) in
//             self.cases.iter().zip(expected.cases.iter()).enumerate()
//         {
//             if own_case.name != expected_case.name {
//                 return Some(format!(
//                     "different name for case #{i}:\ncurrent = {:?}\nexpected = {:?}",
//                     own_case.name, expected_case.name
//                 ));
//             }

//             if own_case.states.len() != expected_case.states.len() {
//                 return Some(format!(
//                     "different state count for case #{i}:\ncurrent = {}\nexpected = {}",
//                     own_case.states.len(),
//                     expected_case.states.len()
//                 ));
//             }

//             for (j, (own_state, expected_state)) in own_case
//                 .states
//                 .iter()
//                 .zip(expected_case.states.iter())
//                 .enumerate()
//             {
//                 if own_state != expected_state {
//                     return Some(format!(
//                         "different state for case #{i} state #{j}:\ncurrent = {:?}\nexpected = {:?}",
//                         own_state, expected_state
//                     ));
//                 }
//             }
//         }

//         None
//     }
// }

/// Piece of state in a test case
// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// pub enum State {
//     /// A descriptive comment
//     Comment(String),
//     /// Some arbitrary value relevant to the test
//     Value(u32),
//     /// Program counter TODO use it?
//     Pc(u32),
//     /// Memory read
//     Memory { address: u32, value: u32 },
//     /// Memory region read
//     MemoryRegion { address: u32, values: Vec<u8> },
// }

/// Step of a test.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Step {
    //
    TestCase {
        name: String,
    },
    /// A descriptive comment.
    Comment(String),
    /// Some arbitrary value relevant to the test
    Value(u32),
    /// Program counter. TODO use it?
    Pc(u32),
    /// Memory read.
    Memory {
        address: u32,
        value: u32,
    },
    /// Memory region read.
    MemoryRegion {
        address: u32,
        values: Vec<u32>,
    },
}

/// Message sent from the ROM to the server.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// The test started.
    TestStarted,
    /// Test step.
    TestStep(Step),
    /// The test completed.
    TestCompleted,
    /// The test panicked.
    Panic,
}
