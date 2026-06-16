use anyhow::{Result, anyhow, bail};
use postcard::ser_flavors::Flavor;
use test_suite_common::{AUX_SERVER_READY_VALUE, Message};

use crate::io;

const SCR: u32 = 0x1FFF_0000;
const SCR_CMD_BUSY_BIT: u32 = 1 << 31;
const SCR_CMD_ERROR_BIT: u32 = 1 << 30;

const DATA0: u32 = 0x1FFF_0004;
const DATA1: u32 = 0x1FFF_0008;

const KEY: u32 = 0x1FFF_0010;

const AUX: u32 = 0x1FFF_0018;
const AUX_HALT_VALUE: u32 = 0xFF00_0001;
const AUX_REBOOT_VALUE: u32 = 0xFF00_0002;

const ID: u32 = 0x1FFF_000C;

// TODO use rom end/size to find an appropriate staging area? not sure what we're overwriting here
//const CART_STAGING: u32 = 0x1380_0000;
const CART_STAGING_PHYSICAL: u32 = 0x1380_0000;
const CART_STAGING: *mut u32 =
    (n64_specs::map::Segment::KSEG1 as u32 | CART_STAGING_PHYSICAL) as *mut u32;

const MESSAGE_BUFFER_SIZE: usize = 0x1_0000; // ~ 0,065 MB

/// SummerCart64 interface for communicating with the server.
pub struct Sc64 {
    /// Buffered data to be sent over USB.
    buffer: io::CachedBuffer<u8>,
    buffered_bytes: usize,
}

impl Sc64 {
    /// Creates a new SummerCart64 interface.
    ///
    /// The program might not be running on a SummerCart, in which case this will return `None`.
    pub fn try_new() -> Result<Option<Self>> {
        // Unlock the SC registers

        io::write_uncached(KEY, 0x00000000); // reset the sequencer
        io::wait_for_pi();
        io::write_uncached(KEY, 0x5F554E4C); // _UNL
        io::wait_for_pi();
        io::write_uncached(KEY, 0x4F434B5F); // OCK_
        io::wait_for_pi();

        // Verify that the unlock succeeded by reading the IDENTIFIER which should be "SCv2" (0x53437632).
        // If still locked or if we're not on a SummerCart, we'll get 0x000C_000C (open bus).

        let id: u32 = io::read_uncached(ID);

        if id != 0x53437632 {
            return Ok(None);
        }

        // Enable writes to the cart so that we can stage data for USB transfers

        Command::ConfigSet {
            config: Config::RomWriteEnable,
        }
        .run()?;

        Ok(Some(Self {
            buffer: io::CachedBuffer::<u8>::with_alignment(MESSAGE_BUFFER_SIZE, 4),
            buffered_bytes: 0,
        }))
    }

    /// Sends a message over USB (buffered).
    pub fn send(&mut self, message: Message, flush: bool) -> Result<()> {
        postcard::serialize_with_flavor(&message, BufferedTransfer::new(self))
            .map_err(|e| anyhow!("failed to serialize message: {e}"))?;

        if flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Flushes the buffered data over USB.
    pub fn flush(&mut self) -> Result<()> {
        if self.buffered_bytes > 0 {
            self.send_raw(&self.buffer.as_slice()[..self.buffered_bytes])?;
            self.buffered_bytes = 0;
        }

        Ok(())
    }

    /// Sends raw data over USB.
    fn send_raw(&self, data: &[u8]) -> Result<()> {
        // TODO DMA faster but sometimes doesn't work???
        //let aligned_length = data.len().next_multiple_of(4) as u32;
        // io::pi_dma(
        //     &io::PiDma {
        //         direction: io::PiDmaDirection::RamToPi,
        //         ram_address: u24::from_u32(io::physical(data.as_ptr() as u32)),
        //         pi_address: CART_STAGING,
        //         length: u24::from_u32(aligned_length - 1),
        //     },
        //     true,
        // );

        unsafe {
            // Copy the buffer to the cart's staging area, as u32 since u8 writes seem buggy on hardware

            for (i, chunk_bytes) in data.chunks(4).enumerate() {
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

        Command::UsbWrite {
            address: CART_STAGING_PHYSICAL,
            length: data.len() as u32,
        }
        .run()?;

        // Wait until the transfer completes

        loop {
            let (res0, _res1) = Command::UsbWriteStatus.run()?;

            let completed = res0 & (1 << 31) == 0;

            if completed {
                break;
            }

            // TODO timeout?
        }

        Ok(())
    }

    /// Waits for the server to be ready to receive data.
    pub fn wait_for_server_ready_signal(&self) {
        loop {
            let event: u32 = io::read_uncached(AUX);
            io::wait_for_pi();

            if event == AUX_SERVER_READY_VALUE {
                return;
            }
        }
    }

    /// Waits for the reboot signal.
    pub fn wait_for_reboot_signal(&self) {
        loop {
            // When uploading a ROM with the --reboot flag, the SC64 will send a HALT event via AUX.
            // It expects the same value to be written back to AUX to acknowledge the event.

            let event: u32 = io::read_uncached(AUX);
            io::wait_for_pi();

            if event == AUX_HALT_VALUE {
                io::write_uncached(AUX, AUX_HALT_VALUE);
                io::wait_for_pi();

                // The new ROM is then uploaded and a REBOOT event is sent via AUX.
                // Same logic, write it back.

                loop {
                    let event: u32 = io::read_uncached(AUX);
                    io::wait_for_pi();

                    if event == AUX_REBOOT_VALUE {
                        io::write_uncached(AUX, AUX_REBOOT_VALUE);
                        io::wait_for_pi();

                        return;
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

#[derive(Debug)]
enum Config {
    RomWriteEnable,
}

impl Command {
    fn id(&self) -> u32 {
        match self {
            Command::ConfigSet { .. } => 'C' as u32,
            Command::UsbWrite { .. } => 'M' as u32,
            Command::UsbWriteStatus => 'U' as u32,
        }
    }

    /// Returns the raw DATA0 and DATA1 command arguments.
    fn args(&self) -> (u32, u32) {
        match self {
            Command::ConfigSet { config } => match config {
                Config::RomWriteEnable => (1, 1),
            },
            Command::UsbWrite { address, length } => (*address, (0x02 << 24) | length),
            Command::UsbWriteStatus => (0, 0),
        }
    }

    fn run(&self) -> Result<(u32, u32)> {
        // Send the command

        let (data0, data1) = self.args();

        io::write_uncached(DATA0, data0);
        io::wait_for_pi();

        io::write_uncached(DATA1, data1);
        io::wait_for_pi();

        io::write_uncached(SCR, self.id());
        io::wait_for_pi();

        // Wait until the command completes

        let mut timeout = 0u32;

        io::wait_for_pi();

        while (io::read_uncached::<u32>(SCR) & SCR_CMD_BUSY_BIT) != 0 {
            timeout = timeout.wrapping_add(1);

            if timeout == 0xFFFF_FFFF {
                bail!("SC64 command {:?} timed out", self);
            }
        }

        if (io::read_uncached::<u32>(SCR) & SCR_CMD_ERROR_BIT) != 0 {
            bail!("SC64 command {:?} failed", self);
        }

        Ok((io::read_uncached(DATA0), io::read_uncached(DATA1)))
    }
}

/// Postcard serialization flavor for buffered USB transfers.
///
/// Accumulates the messages raw data in a buffer and flushes it when full.
///
/// This has two significant benefits:
/// - Serializing large messages to a complete buffer might exceed the available memory.
///   So this "streams" such messages into multiple transfers, without allocating the whole deserialized buffer upfront.
/// - Sending each message separately would significantly slow down execution, as we have to wait for each transfer to complete.
///   So this flushes the buffer when full, having typically accumulated multiple messages.
struct BufferedTransfer<'a> {
    sc64: &'a mut Sc64,
}

impl<'a> BufferedTransfer<'a> {
    fn new(sc64: &'a mut Sc64) -> Self {
        Self { sc64 }
    }
}

impl Flavor for BufferedTransfer<'_> {
    type Output = ();

    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        assert!(
            self.sc64.buffered_bytes < self.sc64.buffer.len(),
            "SC64 buffered transfer queue overflow (bytes={}, capacity={})",
            self.sc64.buffered_bytes,
            self.sc64.buffer.len()
        );

        // Queue

        self.sc64.buffer.set(self.sc64.buffered_bytes, byte);
        self.sc64.buffered_bytes += 1;

        // Send the buffered data if the buffer is full

        if self.sc64.buffered_bytes == self.sc64.buffer.len() {
            self.sc64
                .flush()
                .map_err(|_| postcard::Error::SerdeSerCustom)?;
        }

        Ok(())
    }

    fn finalize(self) -> postcard::Result<()> {
        Ok(())
    }
}
