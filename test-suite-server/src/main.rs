use std::{
    fs::{self, File},
    io::{Read, Write},
    time::Duration,
};

use clap::Parser;
use test_suite_common::{Message, State, TestResult};
use winnow::{
    Parser as WinnowParser, Partial,
    binary::{be_u8, be_u24, be_u32},
    error::ErrMode,
    prelude::*,
    token::{literal, take},
};

#[derive(Parser, Debug)]
#[command(
    name = "test_suite_server",
    about = "SC64 serial test message listener"
)]
struct Args {
    /// Serial port
    #[arg(value_name = "PORT", default_value = "COM3")]
    port: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    listen(args.port)
}

/// Listens to the selected port and processes the incoming messages.
fn listen(port_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut port = serialport::new(&port_name, 115_200)
        .timeout(Duration::from_millis(500))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;

    println!("Opened {}, listening...", port_name);

    let mut port_buffer = [0u8; 512];
    let mut acc_buffer = Vec::new();

    loop {
        match port.read(&mut port_buffer) {
            Ok(0) => {
                // timeout or EOF depending on OS
            }
            Ok(n) => {
                //println!("read: {:0X?}", &port_buffer[..n]);

                acc_buffer.extend_from_slice(&port_buffer[..n]);

                println!("acc_buffer: {:0X?}", acc_buffer.len());

                let messages = parse_messages(&mut acc_buffer);

                for message in messages {
                    match message {
                        Message::Hello => {
                            println!("Hello!");
                        }
                        Message::TestResult(result) => {
                            println!("TestResult: {:0X?}", result);

                            if let Err(e) = save_test_result(&result) {
                                eprintln!("failed to save test result: {e}");
                            }
                        }
                        Message::Panic => {
                            eprintln!("Panic!");
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

        match parse_message(&mut cursor) {
            Ok(message) => {
                messages.push(message);

                let consumed = buffer.len() - cursor.len();
                buffer.drain(..consumed);
            }
            Err(ErrMode::Incomplete(_)) => break,
            Err(e) => {
                panic!("Error parsing packet: {:?}", e);
            }
        }
    }

    messages
}

/// Parses possibly-partial data and returns the first fully received message.
/// https://github.com/Polprzewodnikowy/SummerCart64/blob/main/docs/03_usb_interface.md#sc64---pc-packets
fn parse_message<'s>(input: &mut Partial<&'s [u8]>) -> ModalResult<Message> {
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
fn save_test_result(result: &TestResult) -> Result<(), Box<dyn std::error::Error>> {
    save_test_result_to_json(result)?;
    save_test_result_to_bin(result)?;

    Ok(())
}

const OUTPUT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../_test_suite_output");

/// Saves test results as JSON.
fn save_test_result_to_json(result: &TestResult) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)?;

    let path = format!("{}/{}.json", OUTPUT_DIR, result.name);

    let json = serde_json::to_string_pretty(result)?;

    let mut f = File::create(&path)?;
    f.write_all(json.as_bytes())?;
    f.sync_all()?;

    println!("Saved JSON test result to {}", path);

    Ok(())
}

/// Saves test results as binary data.
fn save_test_result_to_bin(result: &TestResult) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)?;

    let path = format!("{}/{}.bin", OUTPUT_DIR, result.name);

    let bytes = postcard::to_allocvec(result)?;

    let mut f = File::create(&path)?;
    f.write_all(&bytes)?;
    f.sync_all()?;

    println!("Saved binary test result to {}", path);

    Ok(())
}
