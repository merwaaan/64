use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use serialport::SerialPort;
use similar::{ChangeTag, TextDiff};
use test_suite_common::{AUX_SERVER_READY_VALUE, Message, Step};
use winnow::{
    Parser as WinnowParser, Partial,
    binary::{be_u8, be_u24, be_u32},
    error::ErrMode,
    prelude::*,
    token::take,
};

// TODO logs messy, how to deal with indentation?

use crate::{Mode, find_test_rom, list_tests, release_dir};

/// Records the results of either a specific test ROM of all the built record-mode ROMs by executing them on hardware.
pub fn run(test_name: &Option<String>, repeat: Option<usize>) -> Result<()> {
    let tests = if let Some(test_name) = test_name {
        let path = find_test_rom(&test_name, Mode::Record);

        if let Some(path) = path {
            vec![(test_name.clone(), path)]
        } else {
            bail!("no record-mode ROM for {test_name}");
        }
    } else {
        let tests: Vec<_> = list_tests()?
            .into_iter()
            .filter_map(|test| {
                find_test_rom(&test.name, Mode::Record).map(|path| (test.name.clone(), path))
            })
            .collect();

        if tests.is_empty() {
            bail!("no tests to record in {}", release_dir().display());
        }

        tests
    };

    for (test, test_rom_path) in tests {
        record_test(&test, &test_rom_path, repeat)?;
    }

    Ok(())
}

fn record_test(test_name: &str, test_rom_path: &PathBuf, repeat: Option<usize>) -> Result<()> {
    log::info!("Recording \"{}\"...", test_name);

    // Upload the ROM to the SC64

    upload_rom_to_sc64(test_rom_path).with_context(|| "failed to upload ROM to SC64")?;

    // Wait for the result to be sent back

    let steps = listen_for_test_steps()?;

    // If requested, repeat the recording to validate determinism

    if let Some(repeat) = repeat {
        check_determinism(&test_rom_path, &steps, repeat)?;
    }

    // Save the test result

    save_test_steps(&test_name, &steps).with_context(|| "failed to save test steps")
}

fn check_determinism(
    test_rom_path: &PathBuf,
    reference_steps: &Vec<Step>,
    repeat: usize,
) -> Result<()> {
    log::info!(
        "  Checking recording determinism for {} repetitions...",
        repeat
    );

    let steps_text = serde_json::to_string_pretty(&reference_steps)?;

    for i in 0..repeat {
        log::info!("    Recording repetition {}/{}...", i + 1, repeat);

        upload_rom_to_sc64(test_rom_path).with_context(|| "failed to upload ROM to SC64")?;

        let repeat = listen_for_test_steps()?;

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

    let result = duct::cmd!(sc64deployer_path(), "upload", path, "--reboot")
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
fn listen_for_test_steps() -> Result<Vec<Step>> {
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

    // Notify the test program that we're ready to receive data

    send_ready_to_n64(&mut *port)?;

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

                let messages = parse_messages(&mut raw_packets_buffer, &mut raw_messages_buffer)?;

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

fn send_ready_to_n64(port: &mut dyn SerialPort) -> Result<()> {
    port.write_all(b"CMD")?;
    port.write_all(b"X")?; // AUX_WRITE
    port.write_all(&AUX_SERVER_READY_VALUE.to_be_bytes())?; // Data 0
    port.write_all(&0u32.to_be_bytes())?; // Data 1
    port.flush()?;

    Ok(())
}

/// Processes pending data and returns the fully received messages.
fn parse_messages(
    raw_packets_buffer: &mut Vec<u8>,
    raw_messages_buffer: &mut Vec<u8>,
) -> Result<Vec<Message>> {
    let mut messages = Vec::new();

    // As long as there are packets to parse...

    loop {
        if raw_packets_buffer.is_empty() {
            break;
        }

        let mut cursor = Partial::new(raw_packets_buffer.as_slice());

        // Try to parse the first packet, which might be fully received or not

        match parse_packet(&mut cursor) {
            Ok(Packet::Pkt { data }) => {
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
            Ok(Packet::Cmp { data: _, .. }) => {
                //log::debug!("    Response: {:0X?}", data);

                let consumed = raw_packets_buffer.len() - cursor.len();
                raw_packets_buffer.drain(..consumed);
            }
            Ok(Packet::Err { data, .. }) => {
                bail!("Received error from SC64, {:0X?}", data);
            }
            Err(ErrMode::Incomplete(_)) => {
                // Incomplete packet, wait for more data

                break;
            }
            Err(e) => {
                bail!("failed to parse packet, {:?}, {:0X?}", e, cursor);
            }
        }
    }

    Ok(messages)
}

enum Packet {
    Pkt { data: Vec<u8> },
    Cmp { data: Vec<u8> },
    Err { data: Vec<u8> },
}

/// Parses a possibly-partial packet and returns the raw message data that it contains.
// https://github.com/Polprzewodnikowy/SummerCart64/blob/main/docs/03_usb_interface.md#sc64---pc-packets
fn parse_packet(input: &mut Partial<&[u8]>) -> ModalResult<Packet> {
    let kind = take(3u8).parse_next(input)?;
    let id = be_u8.parse_next(input)?;

    let packet_data_len = be_u32.parse_next(input)?;

    let data = if packet_data_len > 0 {
        let _data_type = be_u8.parse_next(input)?;
        let data_len = be_u24.parse_next(input)?;
        take(data_len).parse_next(input)?.to_vec()
    } else {
        Vec::new()
    };

    match kind {
        b"PKT" => {
            //log::debug!("    Received PKT {} {:0X?}", id, data);

            if id == b'U' {
                Ok(Packet::Pkt { data })
            } else {
                log::error!("Received PKT with unexpected id {:0X?}", id);

                Err(ErrMode::Cut(winnow::error::ContextError::new()))
            }
        }
        b"CMP" => {
            //log::debug!("    Received CMP {} {:0X?}", id, data);

            Ok(Packet::Cmp { data })
        }
        b"ERR" => {
            //log::debug!("    Received ERR {} {:0X?}", id, data);

            Ok(Packet::Err { data })
        }
        _ => Err(ErrMode::Backtrack(winnow::error::ContextError::new())),
    }
}

/// Saves test results as JSON.
fn save_test_steps(test_name: &str, result: &Vec<Step>) -> Result<()> {
    fs::create_dir_all(release_dir())?;

    let path = release_dir().join(format!("{}.json", test_name));

    let json = serde_json::to_string_pretty(result)?;

    let mut f = File::create(&path)?;
    f.write_all(json.as_bytes())?;
    f.sync_all()?;

    log::info!("  Saved JSON test result to {}", path.display());

    Ok(())
}
