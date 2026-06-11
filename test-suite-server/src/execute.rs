use std::{fs::File, io::Write, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use clap::Args;
use serialport::SerialPort;
use similar::{ChangeTag, TextDiff};
use test_suite_common::{AUX_SERVER_READY_VALUE, Message, Step};
use tracing::{debug, error, info, info_span, instrument, warn};
use winnow::{
    Parser as WinnowParser, Partial,
    binary::{be_u8, be_u24, be_u32},
    error::ErrMode,
    prelude::*,
    token::take,
};

use crate::{Mode, RecordRom, RecordRomOutput, ReplayRom, SourceArgs, list::List, upload::Upload};

#[derive(Args, Debug)]
pub struct Execute {
    #[command(flatten)]
    pub source: SourceArgs,

    #[arg(long, short, value_enum)]
    pub mode: Mode,

    /// Records multiple times to ensure that the test is deterministic.
    #[arg(long, short)]
    pub repeat: Option<usize>,
}

impl Execute {
    #[instrument(name = "Execute tests", skip_all, fields(source = %self.source, mode = %self.mode, repeat = ?self.repeat))]
    pub fn run(&self) -> Result<()> {
        let source = self.source.clone().into();

        let roms = List::find_roms(&source, self.mode)?;

        if roms.is_empty() {
            if source.is_filtering() {
                bail!("no matching {} ROMs for {}", self.mode, source)
            } else {
                bail!("no matching {} ROMs", self.mode)
            }
        }

        match self.mode {
            Mode::Record => {
                for rom in roms {
                    record_test_rom(&rom, self.repeat)?;
                }
            }
            Mode::Replay => {
                for rom in roms {
                    replay_test_rom(&rom, self.repeat)?;
                }
            }
        }

        Ok(())
    }

    #[instrument(name = "Record test", skip_all, fields(rom = ?rom))]
    pub fn record(rom: &RecordRom, repeat: Option<usize>) -> Result<RecordRomOutput> {
        let steps_path = record_test_rom(&rom.rom_path, repeat)?;

        Ok(RecordRomOutput {
            record_rom: rom.clone(),
            steps_path,
        })
    }

    #[instrument(name = "Replay test", skip_all, fields(rom = ?rom))]
    pub fn replay(rom: &ReplayRom, repeat: Option<usize>) -> Result<()> {
        replay_test_rom(&rom.rom_path, repeat)
    }
}

fn record_test_rom(rom_path: &PathBuf, repeat: Option<usize>) -> Result<PathBuf> {
    // Upload the ROM

    Upload::new(rom_path.clone())
        .run()
        .context("failed to upload ROM")?;

    // Collect test steps

    let mut steps = Vec::new();

    receive(&mut |message| on_record_message(message, &mut steps))?;

    // If requested, repeat the recording to validate determinism

    if let Some(repeat) = repeat {
        check_recording_determinism(rom_path, &steps, repeat)?;
    }

    // Save the test result

    let steps_path =
        save_test_steps(rom_path, &steps).with_context(|| "failed to save test steps")?;

    Ok(steps_path)
}

fn replay_test_rom(rom_path: &PathBuf, repeat: Option<usize>) -> Result<()> {
    let mut test_success = false;

    let repetitions = repeat.unwrap_or(0);

    for repetition in 0..repetitions + 1 {
        info_span!(
            "Repetition",
            repetition = repetition,
            repetitions = repetitions
        )
        .in_scope(|| -> Result<()> {
            if repetition > 0 {
                info!("Repeating replay ({}/{})", repetition, repetitions);
            }

            // Upload the ROM

            Upload::new(rom_path.clone())
                .run()
                .context("failed to upload ROM")?;

            // Check that the test completes successfully

            receive(&mut |message| on_replay_message(message, &mut test_success))?;

            if !test_success {
                bail!(
                    "test program failed (repetition {}/{})",
                    repetition,
                    repetitions
                );
            }

            Ok(())
        })?;
    }

    info!("test completed successfully ✅");

    Ok(())
}

#[instrument(name = "Check recording determinism", skip_all, fields(rom_path = ?rom_path, repetitions = %repetitions))]
fn check_recording_determinism(
    rom_path: &PathBuf,
    reference_steps: &Vec<Step>,
    repetitions: usize,
) -> Result<()> {
    let steps_text = serde_json::to_string_pretty(&reference_steps)?;

    for repetition in 0..repetitions {
        info_span!(
            "Repetition",
            repetition = repetition + 1,
            repetitions = repetitions
        )
        .in_scope(|| -> Result<()> {
            Upload::new(rom_path.clone())
                .run()
                .context("failed to upload ROM")?;

            let mut repeat_steps = Vec::new();

            receive(&mut |message| on_record_message(message, &mut repeat_steps))?;

            let repeat_text = serde_json::to_string_pretty(&repeat_steps)?;

            let diff = TextDiff::from_lines(&steps_text, &repeat_text);

            if diff.ratio() < 1.0 {
                error!("Received different test result");

                for change in diff
                    .iter_all_changes()
                    .filter(|c| c.tag() != ChangeTag::Equal)
                {
                    info!("{}{}", change.tag(), change);
                }

                bail!("recording is not deterministic");
            } else {
                info!("Received the same test result ✅");
            }

            Ok(())
        })?;
    }

    Ok(())
}

/// Message handler for record mode: collects test steps.
fn on_record_message(message: Message, steps: &mut Vec<Step>) -> Result<bool> {
    match message {
        Message::ProgramStarted => Ok(false),
        Message::TestStep(step) => {
            steps.push(step);
            Ok(false)
        }
        Message::ProgramCompleted { success } => {
            assert!(success, "test program reported failure");
            Ok(true)
        }
        Message::ProgramPanicked => bail!("ROM panicked"),
    }
}

/// Message handler for replay mode: returns the program outcome.
fn on_replay_message(message: Message, test_success: &mut bool) -> Result<bool> {
    match message {
        Message::ProgramCompleted { success } => {
            *test_success = success;
            Ok(true)
        }
        Message::ProgramStarted | Message::TestStep(_) => Ok(false),
        Message::ProgramPanicked => bail!("ROM panicked"),
    }
}

/// Listens on the serial port until a `TestResult` is received and saved.
#[instrument(name = "Listen for test results", skip_all)]
fn receive<F>(mut on_message: F) -> Result<()>
where
    F: FnMut(Message) -> Result<bool>,
{
    const SERIAL_PORT: &str = "COM3";

    let mut port = serialport::new(SERIAL_PORT, 115_200)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()
        .with_context(|| format!("failed to open serial port {SERIAL_PORT}"))?;

    debug!("port: {SERIAL_PORT}");

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

    loop {
        match port.read(&mut reception_buffer) {
            Ok(0) => {
                // timeout or EOF depending on OS
            }
            Ok(n) => {
                //debug!("Received {} bytes: {:02X?}", n, &reception_buffer[..n]);

                raw_packets_buffer.extend_from_slice(&reception_buffer[..n]);

                // debug!(
                //     "{} pending bytes: {:02X?}",
                //     raw_packets_buffer.len(),
                //     &raw_packets_buffer
                // );

                let messages = parse_messages(&mut raw_packets_buffer, &mut raw_messages_buffer)?;

                for message in messages {
                    //debug!("Received message: {:0X?}", message);

                    if on_message(message)? {
                        return Ok(());
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

                //debug!("raw_messages_buffer: {:02X?}", raw_messages_buffer);

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

                            //debug!("incomplete message, waiting for more packets");

                            break;
                        }
                        Err(e) => {
                            panic!("failed to deserialize message, {:?}", e);
                        }
                    }
                }
            }
            Ok(Packet::Cmp) => {
                //debug!("Response: {:0X?}", data);

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
    Cmp,
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
            //debug!("Received PKT {} {:0X?}", id, data);

            if id == b'U' {
                Ok(Packet::Pkt { data })
            } else {
                error!("Received PKT with unexpected id {:0X?}", id);

                Err(ErrMode::Cut(winnow::error::ContextError::new()))
            }
        }
        b"CMP" => {
            //debug!("Received CMP {} {:0X?}", id, data);

            Ok(Packet::Cmp)
        }
        b"ERR" => {
            //debug!("Received ERR {} {:0X?}", id, data);

            Ok(Packet::Err { data })
        }
        _ => Err(ErrMode::Backtrack(winnow::error::ContextError::new())),
    }
}

/// Saves test results as JSON.
#[instrument(name = "Save test steps", skip_all)]
fn save_test_steps(rom_path: &PathBuf, result: &Vec<Step>) -> Result<PathBuf> {
    let json = serde_json::to_string_pretty(result)?;

    let json_path = rom_path.with_extension(".json");

    let mut json_file = File::create(&json_path)?;
    json_file.write_all(json.as_bytes())?;

    info!("📃 {}", json_path.display());

    Ok(json_path)
}
