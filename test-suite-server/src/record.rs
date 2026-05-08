use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use duct::cmd;
use similar::{ChangeTag, TextDiff};
use test_suite_common::{Message, result::TestResult};
use winnow::{
    Parser as WinnowParser, Partial,
    binary::{be_u8, be_u24, be_u32},
    error::ErrMode,
    prelude::*,
    token::{literal, take},
};

use crate::{list_tests, package_dir};

pub fn run(test_name: &Option<String>) -> Result<()> {
    // Use the provided test or list all the available tests

    let test_paths = if let Some(test_name) = test_name {
        let path = package_dir().join(format!("{test_name}_record.z64"));

        if !path.is_file() {
            bail!("no record-mode ROM at {}", path.display());
        }

        vec![path]
    } else {
        let mut paths = Vec::new();

        for test_path in list_tests()? {
            let test_name = test_path.file_stem().and_then(|s| s.to_str()).unwrap();

            let path = package_dir().join(format!("{}_record.z64", test_name));

            if path.is_file() {
                paths.push(path);
            } else {
                log::warn!("no record-mode ROM for {}", test_name);
            }
        }

        paths
    };

    // Record the test results for each ROM

    for path in test_paths {
        record_test(&path)?;
    }

    Ok(())
}

fn record_test(path: &PathBuf) -> Result<()> {
    log::info!("Recording {}...", path.display());

    // Upload the ROM to the SC64

    upload_rom_to_sc64(path).with_context(|| "failed to upload ROM to SC64")?;

    log::warn!(
        "Reboot the console manually to start the test (automatic reboot not supported yet!)"
    );

    // Wait for the result to be sent back

    let result = listen_for_test_result()?;

    // If requested, repeat the recording to validate determinism

    let repetitions = None; // Some(1); // TODO to arg

    if let Some(repetitions) = repetitions {
        check_determinism(&result, repetitions)?;
    }

    // Save the test result

    save_test_result(&result).with_context(|| "failed to save test result")
}

fn check_determinism(result: &TestResult, repetitions: usize) -> Result<()> {
    log::info!(
        "Checking recording determinism for {} repetitions...",
        repetitions
    );

    let result_text = serde_json::to_string_pretty(&result)?;

    for i in 0..repetitions {
        log::info!("Recording repetition {}/{}...", i + 1, repetitions);

        let repeat = listen_for_test_result()?;

        let repeat_text = serde_json::to_string_pretty(&repeat)?;

        let diff = TextDiff::from_lines(&result_text, &repeat_text);

        if diff.ratio() < 1.0 {
            log::error!("Received different test result");

            for change in diff
                .iter_all_changes()
                .filter(|c| c.tag() != ChangeTag::Equal)
            {
                log::info!("{}{}", change.tag(), change);
            }

            bail!("recording is not deterministic");
        } else {
            log::info!("Received the same test result");
        }
    }

    Ok(())
}
fn sc64deployer_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sc64deployer.exe")
}

pub fn upload_rom_to_sc64(path: &PathBuf) -> Result<()> {
    log::info!("Uploading \"{}\" to SC64...", path.display());

    let result = cmd!(
        sc64deployer_path(),
        "upload",
        path /* TODO , "--reboot" */
    )
    .stderr_to_stdout()
    .stdout_capture()
    .unchecked()
    .run()?;

    if !result.status.success() {
        let stdout = String::from_utf8_lossy(&result.stdout);
        bail!("sc64deployer error, {}", stdout);
    }

    Ok(())
}

/// Listens on the serial port until a `TestResult` is received and saved.
fn listen_for_test_result() -> Result<TestResult> {
    const SERIAL_PORT: &str = "COM3";

    let mut port = serialport::new(SERIAL_PORT, 115_200)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()
        .with_context(|| format!("failed to open serial port {SERIAL_PORT}"))?;

    log::info!("Listening for test result on {SERIAL_PORT}...");

    let mut port_buffer = [0u8; 512];
    let mut acc_buffer = Vec::new();

    loop {
        match port.read(&mut port_buffer) {
            Ok(0) => {
                // timeout or EOF depending on OS
            }
            Ok(n) => {
                //log::debug!("Received {} bytes: {:02X?}", n, &port_buffer[..n]);

                acc_buffer.extend_from_slice(&port_buffer[..n]);

                let messages = parse_messages(&mut acc_buffer);

                for message in messages {
                    //log::debug!("Received message: {:0X?}", message);

                    match message {
                        Message::TestResult(result) => {
                            log::info!("Received test result");

                            return Ok(result);
                        }
                        Message::Panic => {
                            bail!("ROM panicked");
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    // Normal with short timeout if no data
                } else {
                    return Err(e.into());
                }
            }
        }
    }
}

/// Processes possibly-partial data and returns all the fully received messages.
fn parse_messages(buffer: &mut Vec<u8>) -> Vec<Message> {
    let mut messages = Vec::new();

    loop {
        if buffer.is_empty() {
            break;
        }

        let mut cursor = Partial::new(buffer.as_slice());

        match parse_partial_message(&mut cursor) {
            Ok(message) => {
                messages.push(message);

                let consumed = buffer.len() - cursor.len();
                buffer.drain(..consumed);
            }
            Err(ErrMode::Incomplete(_)) => break,
            Err(e) => {
                panic!("Error parsing packet: {:?}", e); // TODO bail!
            }
        }
    }

    messages
}

/// Parses possibly-partial data and returns the first fully received message.
/// https://github.com/Polprzewodnikowy/SummerCart64/blob/main/docs/03_usb_interface.md#sc64---pc-packets
fn parse_partial_message(input: &mut Partial<&[u8]>) -> ModalResult<Message> {
    // Packet identifier
    literal("PKT").parse_next(input)?;

    // Packet type = data
    literal("U").parse_next(input)?;

    let _packet_data_len = be_u32.parse_next(input)?;

    let _data_type = be_u8.parse_next(input)?; // TODO literal?

    let data_len: u32 = be_u24.parse_next(input)?;

    let message_data = take(data_len).parse_next(input)?;

    let message: Message =
        postcard::from_bytes(message_data).expect("failed to deserialize message");

    Ok(message)
}

/// Saves test results.
fn save_test_result(result: &TestResult) -> Result<()> {
    save_json_test_result(result).with_context(|| "failed to save JSON test result")?;
    save_binary_test_result(result).with_context(|| "failed to save binary test result")
}

/// Saves test results as JSON.
fn save_json_test_result(result: &TestResult) -> Result<()> {
    fs::create_dir_all(package_dir())?;

    let path = package_dir().join(format!("{}.json", result.name));

    let json = serde_json::to_string_pretty(result)?;

    let mut f = File::create(&path)?;
    f.write_all(json.as_bytes())?;
    f.sync_all()?;

    log::info!("Saved JSON test result to {}", path.display());

    Ok(())
}

/// Saves test results as binary data.
fn save_binary_test_result(result: &TestResult) -> Result<()> {
    fs::create_dir_all(package_dir())?;

    let path = package_dir().join(format!("{}.bin", result.name));

    let bytes = postcard::to_allocvec(result)?;

    let mut f = File::create(&path)?;
    f.write_all(&bytes)?;
    f.sync_all()?;

    log::info!("Saved binary test result to {}", path.display());

    Ok(())
}
