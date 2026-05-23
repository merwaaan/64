#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// One step of a test.
///
/// Each test emits a sequence of steps.
/// In record mode, the steps are sent to the server to build a compare-mode ROM.
/// In replay mode, the steps are compared against the embedded recorded steps.
#[derive(
    Clone, PartialEq, Debug, Serialize, Deserialize, strum::Display, strum::EnumDiscriminants,
)]
pub enum Step {
    // Start of a test case.
    StartTestCase,
    /// A descriptive comment.
    Comment(String),
    /// Some value relevant to the test
    Value(u32),
}

/// Strips the comments from a list of steps.
/// This reduces the size of the embedded data and we still have comments in the human-readable JSON files produced by the server.
pub fn strip_comments(steps: &[Step]) -> Vec<Step> {
    steps
        .iter()
        .cloned()
        .filter_map(|step| match step {
            Step::Comment(_) => None,
            _ => Some(step),
        })
        .collect()
}

/// Message sent from the N64 program to the server.
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
