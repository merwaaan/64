use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use similar::{ChangeTag, TextDiff};
use test_suite_common::{Message, Step};
use winnow::{
    Parser as WinnowParser, Partial,
    binary::{be_u8, be_u24, be_u32},
    error::ErrMode,
    prelude::*,
    token::{literal, take},
};

use crate::{Mode, find_test_rom, list_tests, release_dir};

/// Records the results of either a specific test ROM of all the built record-mode ROMs by executing them on hardware.
pub fn run(test: &Option<String>) -> Result<()> {
    let tests = if let Some(test_name) = test {
        let path = find_test_rom(&test_name, Mode::Record);

        if let Some(path) = path {
            vec![(test_name.clone(), path)]
        } else {
            bail!("no record-mode ROM for {test_name}");
        }
    } else {
        list_tests()?
            .into_iter()
            .filter_map(|test| find_test_rom(&test, Mode::Record).map(|path| (test, path)))
            .collect()
    };

    if tests.is_empty() {
        bail!("no tests to record in {}", release_dir().display());
    }

    for (test, test_rom_path) in tests {
        record_test(&test, &test_rom_path)?;
    }

    Ok(())
}

fn record_test(test: &str, test_rom_path: &PathBuf) -> Result<()> {
    log::info!("Recording \"{}\"...", test);

    // Upload the ROM to the SC64

    upload_rom_to_sc64(test_rom_path).with_context(|| "failed to upload ROM to SC64")?;

    log::warn!(
        "  Reboot the console manually to start the test (automatic reboot not supported yet!)"
    );

    // Wait for the result to be sent back

    let result = listen_for_test_result()?;

    // If requested, repeat the recording to validate determinism

    let repetitions = None; //Some(1); // TODO to arg

    if let Some(repetitions) = repetitions {
        check_determinism(&result, repetitions)?;
    }

    // Save the test result

    save_test_result(test, &result).with_context(|| "failed to save test result")
}

fn check_determinism(steps: &Vec<Step>, repetitions: usize) -> Result<()> {
    log::info!(
        "  Checking recording determinism for {} repetitions...",
        repetitions
    );

    let steps_text = serde_json::to_string_pretty(&steps)?;

    for i in 0..repetitions {
        log::info!("    Recording repetition {}/{}...", i + 1, repetitions);

        let repeat = listen_for_test_result()?;

        let repeat_text = serde_json::to_string_pretty(&repeat)?;

        let diff = TextDiff::from_lines(&steps_text, &repeat_text);

        if diff.ratio() < 1.0 {
            log::error!("      Received different test result");

            for change in diff
                .iter_all_changes()
                .filter(|c| c.tag() != ChangeTag::Equal)
            {
                log::info!("{}{}", change.tag(), change);
            }

            bail!("recording is not deterministic");
        } else {
            log::info!("      Received the same test result");
        }
    }

    Ok(())
}

fn sc64deployer_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sc64deployer.exe")
}

fn upload_rom_to_sc64(path: &PathBuf) -> Result<()> {
    log::info!("  Uploading \"{}\" to SC64...", path.display());

    // TODO download helper

    let result = duct::cmd!(
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
fn listen_for_test_result() -> Result<Vec<Step>> {
    const SERIAL_PORT: &str = "COM3";

    let mut port = serialport::new(SERIAL_PORT, 115_200)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()
        .with_context(|| format!("failed to open serial port {SERIAL_PORT}"))?;

    log::info!("  Listening for test result on {SERIAL_PORT}...");

    // Reception buffer
    let mut reception_buffer = [0u8; 512];

    // Raw packet data accumulated from the reception buffer but not decoded yet
    // (large packets might be split across multiple reads)
    let mut raw_packets_buffer = Vec::new();

    // Raw message data extracted from packets but not decoded yet
    // (the test ROMs stream data in chunks so a single message can be split across multiple packets)
    let mut raw_messages_buffer = Vec::new();

    // All the decoded steps received so far
    let mut steps = Vec::new();

    loop {
        match port.read(&mut reception_buffer) {
            Ok(0) => {
                // timeout or EOF depending on OS
            }
            Ok(n) => {
                //log::debug!("Received {} bytes: {:02X?}", n, &reception_buffer[..n]);

                raw_packets_buffer.extend_from_slice(&reception_buffer[..n]);

                // log::debug!(
                //     "{} pending bytes: {:02X?}",
                //     raw_packets_buffer.len(),
                //     &raw_packets_buffer
                // );

                let messages = parse_messages(&mut raw_packets_buffer, &mut raw_messages_buffer);

                for message in messages {
                    //log::debug!("Received message: {:0X?}", message);

                    match message {
                        Message::TestStarted => {
                            //log::debug!("    Test started");
                        }
                        Message::TestStep(step) => {
                            steps.push(step);
                        }
                        Message::TestCompleted => {
                            //log::debug!("    Test completed");

                            return Ok(steps);
                        }
                        Message::Panic => {
                            bail!("ROM panicked");
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

/// Processes pending data and returns the fully received messages.
fn parse_messages(
    raw_packets_buffer: &mut Vec<u8>,
    raw_messages_buffer: &mut Vec<u8>,
) -> Vec<Message> {
    let mut messages = Vec::new();

    // As long as there are packets to parse...

    loop {
        if raw_packets_buffer.is_empty() {
            break;
        }

        let mut cursor = Partial::new(raw_packets_buffer.as_slice());

        // Try to parse the first packet, which might be fully received or not

        match parse_packet(&mut cursor) {
            Ok(data) => {
                // We got a full packet, buffer its data

                raw_messages_buffer.extend_from_slice(&data);

                let consumed = raw_packets_buffer.len() - cursor.len();
                raw_packets_buffer.drain(..consumed);

                //println!("raw_messages_buffer: {:02X?}", raw_messages_buffer);

                // Try to deserialize the packet messages.
                // The last one might be incomplete if split across multiple packets.

                loop {
                    match postcard::take_from_bytes(raw_messages_buffer) {
                        Ok((message, rest)) => {
                            messages.push(message);

                            let consumed = raw_messages_buffer.len() - rest.len();
                            raw_messages_buffer.drain(..consumed);
                        }
                        Err(postcard::Error::DeserializeUnexpectedEnd) => {
                            // Incomplete message, wait for more data

                            //log::debug!("incomplete message, waiting for more packets");

                            break;
                        }
                        Err(e) => {
                            panic!("failed to deserialize message, {:?}", e);
                        }
                    }
                }
            }
            Err(ErrMode::Incomplete(_)) => {
                // Incomplete packet, wait for more data

                break;
            }
            Err(e) => {
                panic!("failed to parse packet, {:?}", e); // TODO bail!
            }
        }
    }

    messages
}

/// Parses a possibly-partial packet and returns the raw message data that it contains.
// https://github.com/Polprzewodnikowy/SummerCart64/blob/main/docs/03_usb_interface.md#sc64---pc-packets
fn parse_packet(input: &mut Partial<&[u8]>) -> ModalResult<Vec<u8>> {
    literal("PKT").parse_next(input)?;
    literal("U").parse_next(input)?; // type = data
    let _packet_data_len = be_u32.parse_next(input)?;
    let _data_type = be_u8.parse_next(input)?; // TODO literal?

    let data_len: u32 = be_u24.parse_next(input)?;
    let data = take(data_len).parse_next(input)?.to_vec();

    Ok(data)
}

/// Saves test results as JSON.
fn save_test_result(test: &str, result: &Vec<Step>) -> Result<()> {
    fs::create_dir_all(release_dir())?;

    let path = release_dir().join(format!("{}.json", test));

    let json = serde_json::to_string_pretty(result)?;

    let mut f = File::create(&path)?;
    f.write_all(json.as_bytes())?;
    f.sync_all()?;

    log::info!("  Saved JSON test result to {}", path.display());

    Ok(())
}
