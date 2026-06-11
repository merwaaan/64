#![no_std]

extern crate alloc;

use serde::{Deserialize, Serialize};

/// Message sent from the server to the test program via the SummerCart AUX register to notify that the server is ready to receive data.
///
/// The server uploads the ROM to the SC64 and then listens to incoming messages.
/// However, the ROM starts running as soon as the upload completes and it might send messages before the server even starts listening, making us miss them.
/// We cannot just start listening and THEN upload the ROM, because the server and sc64deployer both use the same serial port.
///
/// So this basic handshake is required to ensure that the test program waits for the server to be ready to listen before sending messages.
pub const AUX_SERVER_READY_VALUE: u32 = 0xFF00_ABCD;

/// One step of a test.
///
/// Each test emits a sequence of steps.
/// In record mode, the steps are sent to the server to build a compare-mode ROM.
/// In replay mode, the steps are compared against the embedded recorded steps.
#[derive(
    Clone, PartialEq, Debug, Serialize, Deserialize, strum::Display, strum::EnumDiscriminants,
)]

pub enum Step {
    /// Start of a test.
    StartTest,
    /// End of a test.
    EndTest,
    // Start of a test case.
    StartTestCase,
    // End of a test case.
    EndTestCase,
    /// A boolean value relevant to the test
    Bool(bool),
    /// A 32-bit value relevant to the test
    Value(u32),
    /// A 64-bit value relevant to the test
    Value64(u64),
}

/// Message sent from the N64 program to the server.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// The program started.
    ProgramStarted,
    /// The program completed.
    ProgramCompleted { success: bool },
    /// The program panicked.
    ProgramPanicked,
    /// Test step.
    TestStep(Step),
}
