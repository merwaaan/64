use anyhow::{Result, anyhow, bail};
use test_suite_common::Message;

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

// TODO use __rom_end to find an apporpriate staging area?
const CART_STAGING_PHYSICAL: u32 = 0x1380_0000;
const CART_STAGING: *mut u32 =
    (n64_specs::map::Segment::KSEG1 as u32 | CART_STAGING_PHYSICAL) as *mut u32;

pub struct Sc64;

impl Sc64 {
    /// Configures the SC64 before using it.
    pub fn configure() -> Result<()> {
        unsafe {
            // Unlock the SC registers

            KEY.write_volatile(0x00000000); // reset the sequencer
            Self::pi_wait();
            KEY.write_volatile(0x5F554E4C); // _UNL
            Self::pi_wait();
            KEY.write_volatile(0x4F434B5F); // OCK_
            Self::pi_wait();

            // Verify that the unlock succeeded by reading the IDENTIFIER which should be "SCv2" (0x53437632).
            // If still locked, we'll get 0x000C_000C (open bus).

            let id = ID.read_volatile();

            if id != 0x53437632 {
                bail!("SC64 unlock failed, ID={:08X}", id);
            }

            // Enable writes to the cart

            Self::run_command(Command::ConfigSet {
                config: Config::RomWriteEnable,
            })
        }
    }

    fn run_command(command: Command) -> Result<()> {
        unsafe {
            // Send the command

            let (data0, data1) = command.args();

            DATA0.write_volatile(data0);
            Self::pi_wait();

            DATA1.write_volatile(data1);
            Self::pi_wait();

            SCR.write_volatile(command.id());
            Self::pi_wait();

            // Wait until the command is complete

            let mut timeout = 0u32;

            Self::pi_wait();

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
    pub fn send(message: Message) -> Result<()> {
        // Serialize the message

        // TODO to_slice to save mem?
        let mut buffer = postcard::to_allocvec(&message)
            .map_err(|e| anyhow!("failed to serialize message: {e}"))?;

        // Pad the buffer to be aligned to 4-bytes or we'll get errors

        buffer.resize(buffer.len().next_multiple_of(4), 0);

        unsafe {
            // Copy the buffer to the cart staging area, as u32 since u8 writes seem buggy on hardware

            for (i, chunk_bytes) in buffer.chunks(4).enumerate() {
                let word = u32::from_be_bytes([
                    chunk_bytes[0],
                    chunk_bytes[1],
                    chunk_bytes[2],
                    chunk_bytes[3],
                ]);

                CART_STAGING.add(i).write_volatile(word);
                Self::pi_wait();
            }
        }

        // Send the data

        Self::run_command(Command::UsbWrite {
            address: CART_STAGING_PHYSICAL,
            length: buffer.len() as u32,
        })
    }

    /// Waits for the SC64 to receive a reboot request and jumps to the bootloader to start fresh.
    pub fn wait_for_reboot() -> ! {
        unsafe {
            loop {
                // When uploading a ROM with the --reboot flag, the SC64 will send a HALT event via AUX.
                // It expects the same value to be written back to AUX to acknowledge the event.

                let event = AUX.read_volatile();
                Self::pi_wait();

                if event == AUX_HALT_VALUE {
                    AUX.write_volatile(AUX_HALT_VALUE);
                    Self::pi_wait();

                    // The new ROM is then uploaded and a REBOOT event is sent via AUX.
                    // Same logic, write it back.

                    loop {
                        let event = AUX.read_volatile();
                        Self::pi_wait();

                        if event == AUX_REBOOT_VALUE {
                            AUX.write_volatile(AUX_REBOOT_VALUE);
                            Self::pi_wait();

                            // TODO how to actually reboot? (various addresses don't work, bootloader must be enabled via regs?)

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

    // TODO write io_read/write that always do that
    fn pi_wait() {
        const PI_STATUS: *mut u32 = 0xA460_0010 as *mut u32;
        unsafe { while PI_STATUS.read_volatile() & 0x3 != 0 {} }
    }
}

#[derive(Debug)]
enum Command {
    ConfigSet { config: Config },
    UsbWrite { address: u32, length: u32 },
}

impl Command {
    fn id(&self) -> u32 {
        match self {
            Command::ConfigSet { .. } => 'C' as u32,
            Command::UsbWrite { .. } => 'M' as u32,
        }
    }

    fn args(&self) -> (u32, u32) {
        match self {
            Command::ConfigSet { config } => match config {
                Config::RomWriteEnable => (1, 1),
            },
            Command::UsbWrite { address, length } => (*address, (0x02 << 24) | length),
        }
    }
}

#[derive(Debug)]
enum Config {
    RomWriteEnable,
}
