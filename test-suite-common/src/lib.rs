#![no_std]

pub mod result;
pub mod test;

extern crate alloc;

use serde::{Deserialize, Serialize};

/// Message sent from the ROM to the server
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Message to test the connection
    Hello,
    /// Message to send the test results
    TestResult(result::TestResult),
    /// Message to communicate that the ROM panicked
    Panic,
}
