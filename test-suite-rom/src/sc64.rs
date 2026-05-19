use alloc::vec::Vec;
use anyhow::{Result, anyhow, bail};
use postcard::ser_flavors::Flavor;
use test_suite_common::Message;

use crate::io;

const SCR: *mut u32 = 0xBFFF_0000 as *mut u32;
const SCR_CMD_BUSY_BIT: u32 = 1 << 31;
const SCR_CMD_ERROR_BIT: u32 = 1 << 30;

const DATA0: *mut u32 = 0xBFFF_0004 as *mut u32;
const DATA1: *mut u32 = 0xBFFF_0008 as *mut u32;

const KEY: *mut u32 = 0xBFFF_0010 as *mut u32;

const AUX: *mut u32 = 0xBFFF_0018 as *mut u32;
const AUX_HALT_VALUE: u32 = 0xFF00_0001;
const AUX_REBOOT_VALUE: u32 = 0xFF00_0002;

const ID: *const u32 = 0xBFFF_000C as *const u32;

// TODO use rom end/size to find an appropriate staging area?
const CART_STAGING_PHYSICAL: u32 = 0x1380_0000;
const CART_STAGING: *mut u32 =
    (n64_specs::map::Segment::KSEG1 as u32 | CART_STAGING_PHYSICAL) as *mut u32;

pub struct Sc64;

impl Default for Sc64 {
    fn default() -> Self {
        unsafe {
            // Unlock the SC registers

            KEY.write_volatile(0x00000000); // reset the sequencer
            io::wait_for_pi();
            KEY.write_volatile(0x5F554E4C); // _UNL
            io::wait_for_pi();
            KEY.write_volatile(0x4F434B5F); // OCK_
            io::wait_for_pi();

            // Verify that the unlock succeeded by reading the IDENTIFIER which should be "SCv2" (0x53437632).
            // If still locked, we'll get 0x000C_000C (open bus).

            let id = ID.read_volatile();

            if id != 0x53437632 {
                panic!("failed to unlock SC64 (ID={:08X})", id);
            }
        }

        // Enable writes to the cart so that we can stage data for USB transfers

        Self::run_command(Command::ConfigSet {
            config: Config::RomWriteEnable,
        })
        .expect("failed to enable writes to the SC64");

        Self
    }
}

impl Sc64 {
    // TODO move to command and return res?
    fn run_command(command: Command) -> Result<()> {
        unsafe {
            // Send the command

            let (data0, data1) = command.args();

            DATA0.write_volatile(data0);
            io::wait_for_pi();

            DATA1.write_volatile(data1);
            io::wait_for_pi();

            SCR.write_volatile(command.id());
            io::wait_for_pi();

            // Wait until the command completes

            let mut timeout = 0u32;

            io::wait_for_pi();

            while (SCR.read_volatile() & SCR_CMD_BUSY_BIT) != 0 {
                timeout = timeout.wrapping_add(1);

                if timeout == 0xFFFF_FFFF {
                    bail!("SC64 command {:?} timed out", command);
                }
            }

            if (SCR.read_volatile() & SCR_CMD_ERROR_BIT) != 0 {
                bail!("SC64 command {:?} failed", command);
            }
        }

        Ok(())
    }

    /// Sends a message over USB.
    /// TODO reword, acc in buffer, send less often, rename queue, add flush
    pub fn send(&self, message: Message) -> Result<()> {
        postcard::serialize_with_flavor(&message, ChunkedTransfer::new(self))
            .map_err(|e| anyhow!("failed to serialize message: {e}"))?;

        Ok(())
    }

    fn send_raw(&self, data: &[u8]) -> Result<()> {
        // The buffer must be aligned to 4 bytes or we'll get errors

        let aligned_length = data.len().next_multiple_of(4) as u32;

        unsafe {
            // Copy the buffer to the cart's staging area, as u32 since u8 writes seem buggy on hardware

            for (i, chunk_bytes) in data.chunks(4).enumerate() {
                // Pad with 0 if not aligned on 4 bytes, the deserialization will ignore the extra bytes

                let word = u32::from_be_bytes([
                    *chunk_bytes.get(0).unwrap_or(&0),
                    *chunk_bytes.get(1).unwrap_or(&0),
                    *chunk_bytes.get(2).unwrap_or(&0),
                    *chunk_bytes.get(3).unwrap_or(&0),
                ]);

                CART_STAGING.add(i).write_volatile(word);
                io::wait_for_pi();
            }
        }

        // Send the data

        Self::run_command(Command::UsbWrite {
            address: CART_STAGING_PHYSICAL,
            length: aligned_length,
        })?;

        // Wait until the transfer completes

        loop {
            Self::run_command(Command::UsbWriteStatus)?;

            let completed = unsafe { DATA0.read_volatile() } & (1 << 31) == 0;

            if completed {
                break;
            }

            // TODO timeout?
        }

        Ok(())
    }

    /// Waits for the SC64 to receive a reboot request and jumps to the bootloader to start fresh.
    pub fn wait_for_reboot(&self) -> ! {
        unsafe {
            loop {
                // When uploading a ROM with the --reboot flag, the SC64 will send a HALT event via AUX.
                // It expects the same value to be written back to AUX to acknowledge the event.

                let event = AUX.read_volatile();
                io::wait_for_pi();

                if event == AUX_HALT_VALUE {
                    AUX.write_volatile(AUX_HALT_VALUE);
                    io::wait_for_pi();

                    // The new ROM is then uploaded and a REBOOT event is sent via AUX.
                    // Same logic, write it back.

                    loop {
                        let event = AUX.read_volatile();
                        io::wait_for_pi();

                        if event == AUX_REBOOT_VALUE {
                            AUX.write_volatile(AUX_REBOOT_VALUE);
                            io::wait_for_pi();

                            // TODO how to actually reboot? (various addresses don't work, bootloader must be enabled via regs?)

                            // unsafe extern "C" {
                            //     static __boot_start: u32;
                            // }

                            // let boot_start = (&raw const __boot_start).addr();

                            //panic!("REBOOT?");

                            // let reboot: extern "C" fn() -> ! =
                            //     core::mem::transmute(0xBFC0_0000usize);
                            // reboot()

                            // TODO BOOTLOADER_SWITCH????

                            // let reboot: extern "C" fn() -> ! =
                            //     core::mem::transmute(0xA4001000usize);
                            // reboot();

                            // #define IPL3_ENTRY          0xA4000040
                            // #define REBOOT_ADDRESS      0xA4001000
                            //self.cpu.regs.pc = 0xA4000040;

                            // let boot_vector = 0xA400_0040 as *const extern "C" fn();
                            // (*boot_vector)();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum Command {
    ConfigSet { config: Config },
    UsbWrite { address: u32, length: u32 },
    UsbWriteStatus,
}

impl Command {
    fn id(&self) -> u32 {
        match self {
            Command::ConfigSet { .. } => 'C' as u32,
            Command::UsbWrite { .. } => 'M' as u32,
            Command::UsbWriteStatus => 'U' as u32,
        }
    }

    fn args(&self) -> (u32, u32) {
        match self {
            Command::ConfigSet { config } => match config {
                Config::RomWriteEnable => (1, 1),
            },
            Command::UsbWrite { address, length } => (*address, (0x02 << 24) | length),
            Command::UsbWriteStatus => (0, 0),
        }
    }
}

#[derive(Debug)]
enum Config {
    RomWriteEnable,
}

/// Postcard serialization flavor for chunked USB transfers.
/// Serializing large events (eg. MemoryRegion) to a complete buffer might exceed the available memory.
/// This flavor serializes the data in chunks and streams them.
struct ChunkedTransfer<'a> {
    sc64: &'a Sc64,
    buffer: Vec<u8>,
}

impl<'a> ChunkedTransfer<'a> {
    fn new(sc64: &'a Sc64) -> Self {
        Self {
            sc64,
            buffer: Vec::with_capacity(1024),
        }
    }

    fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.sc64.send_raw(&self.buffer)?;
            self.buffer.clear();
        }

        Ok(())
    }
}

impl Flavor for ChunkedTransfer<'_> {
    type Output = ();

    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        // Send the data if the buffer is full

        if self.buffer.len() == self.buffer.capacity() {
            self.flush().map_err(|_| postcard::Error::SerdeSerCustom)?;
        }

        // Queue

        self.buffer.push(byte);

        Ok(())
    }

    fn finalize(mut self) -> postcard::Result<()> {
        // Send the remaining data

        self.flush().map_err(|_| postcard::Error::SerdeSerCustom)?;

        Ok(())
    }
}
