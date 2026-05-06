use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use duct::cmd;
use test_suite_common::{Message, TestResult};
use winnow::{
    Parser as WinnowParser, Partial,
    binary::{be_u8, be_u24, be_u32},
    error::ErrMode,
    prelude::*,
    token::{literal, take},
};

use crate::package_dir;

pub fn run(test_name: &Option<String>) -> Result<()> {
    // Use the provided test or list all the available tests

    let mut test_paths = Vec::new();

    if let Some(test_name) = test_name {
        let path = package_dir().join(format!("{test_name}_record.z64"));

        if !path.is_file() {
            bail!("no record-mode ROM at {}", path.display());
        }

        test_paths.push(path);
    } else {
        test_paths.extend(list_record_roms()?);
    }

    // Record the test results for each ROM

    for path in test_paths {
        record_test(&path)?;
    }

    Ok(())
}

fn list_record_roms() -> Result<Vec<PathBuf>> {
    log::info!("Listing all record-mode ROMs...");

    let mut paths = Vec::new();

    for entry in fs::read_dir(&package_dir())? {
        let path = entry?.path();

        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.ends_with("_record.z64"))
        {
            paths.push(path);
        }
    }

    paths.sort_by_key(|p| p.to_string_lossy().into_owned()); // TODO needed?

    log::info!("Found {} record-mode ROMs:", paths.len());

    for path in &paths {
        log::info!("  - {}", path.display());
    }

    Ok(paths)
}

fn record_test(path: &PathBuf) -> Result<()> {
    log::info!("Recording test \"{}\"...", path.display());

    // Upload the ROM to the SC64

    upload_rom_to_sc64(path).with_context(|| "failed to upload ROM to SC64")?;

    // Listen for the test result

    let handle = thread::spawn(listen_for_test_result);

    match handle.join() {
        Ok(result) => result,
        Err(_) => bail!("listener thread panicked"),
    }
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
fn listen_for_test_result() -> Result<()> {
    const SERIAL_PORT: &str = "COM3";

    let mut port = serialport::new(SERIAL_PORT, 115_200)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()
        .with_context(|| format!("failed to open serial port {SERIAL_PORT}"))?;

    log::info!("Listening on {SERIAL_PORT}...");

    let mut port_buffer = [0u8; 512];
    let mut acc_buffer = Vec::new();

    loop {
        match port.read(&mut port_buffer) {
            Ok(0) => {
                // timeout or EOF depending on OS
            }
            Ok(n) => {
                log::debug!("Received {} bytes: {:02X?}", n, &port_buffer[..n]);

                acc_buffer.extend_from_slice(&port_buffer[..n]);

                let messages = parse_messages(&mut acc_buffer);

                for message in messages {
                    match message {
                        Message::Hello => {
                            log::info!("Hello!");
                        }
                        Message::TestResult(result) => {
                            log::info!("TestResult: {:0X?}", result);

                            save_test_result(&result)
                                .with_context(|| "failed to save test result")?;

                            return Ok(());
                        }
                        Message::Panic => {
                            bail!("ROM reported panic over serial");
                        }
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
fn parse_partial_message<'s>(input: &mut Partial<&'s [u8]>) -> ModalResult<Message> {
    // Packet identifier
    literal("PKT").parse_next(input)?;

    // Packet type = data
    literal("U").parse_next(input)?;

    let _packet_data_len = be_u32.parse_next(input)?;

    let _data_type = be_u8.parse_next(input)?; // TODO literal?

    let data_len: u32 = be_u24.parse_next(input)?;

    let message_data = take(data_len).parse_next(input)?;

    let message: Message =
        postcard::from_bytes(&message_data).expect("failed to deserialize message");

    Ok(message)
}

/// Saves test results.
fn save_test_result(result: &TestResult) -> Result<()> {
    save_json_test_result(result).with_context(|| "failed to save JSON test result")?;
    save_binary_test_result(result).with_context(|| "failed to save binary test result")?;

    Ok(())
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
