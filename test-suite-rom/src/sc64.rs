use test_suite_common::Message;

const SCR: *mut u32 = 0xBFFF_0000 as *mut u32;
const SCR_CMD_BUSY_BIT: u32 = 1 << 31;
const SCR_CMD_ERROR_BIT: u32 = 1 << 30;

const DATA0: *mut u32 = 0xBFFF_0004 as *mut u32;
const DATA1: *mut u32 = 0xBFFF_0008 as *mut u32;

const CMD_CONFIG_SET: u32 = 'C' as u32;
const CMD_CONFIG_SET_ROM_WRITE_ENABLE: u32 = 1;
const CMD_USB_WRITE: u32 = 'M' as u32;

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
    pub fn configure() {
        unsafe {
            // Unlock the SC registers

            KEY.write_volatile(0x00000000);
            Self::pi_wait();
            KEY.write_volatile(0x5F554E4C); // _UNL
            Self::pi_wait();
            KEY.write_volatile(0x4F434B5F); // OCK_
            Self::pi_wait();

            // Verify that the unlock succeeded by reading the IDENTIFIER whichshould be "SCv2" (0x53437632)

            let id = ID.read_volatile();

            if id != 0x53437632 {
                panic!("SC64 unlock failed, ID={:08X}", id);
            }

            // Enable writes to the cart

            DATA0.write_volatile(CMD_CONFIG_SET_ROM_WRITE_ENABLE);
            Self::pi_wait();
            DATA1.write_volatile(1); // enabled
            Self::pi_wait();
            SCR.write_volatile(CMD_CONFIG_SET);
            Self::pi_wait();
            Self::wait_for_command();

            // Enable AUX IRQ to receive halt/reboot events

            //AUX_IRQ.write_volatile(AUX_IRQ_ENABLE_BIT);
        }
    }

    /// Sends a message over USB.
    pub fn send(message: Message) {
        // Serialize the message

        // TODO to_slice to save mem?
        let mut buffer = postcard::to_allocvec(&message).unwrap();

        let data_length = buffer.len() as u32;

        // Pad the buffer to be aligned to 4-bytes

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
            }

            // Send the data

            DATA0.write_volatile(CART_STAGING_PHYSICAL as u32);
            Self::pi_wait();
            DATA1.write_volatile((0x02 << 24) | data_length);
            Self::pi_wait();
            SCR.write_volatile(CMD_USB_WRITE);
            Self::pi_wait();

            Self::wait_for_command();
        }
    }

    /// Waits until a command is complete.
    fn wait_for_command() {
        unsafe {
            let mut t = 0u32;

            while (SCR.read_volatile() & SCR_CMD_BUSY_BIT) != 0 {
                t = t.wrapping_add(1);

                if t == 0xFFFF_FFFF {
                    panic!("SC64 command timed out");
                }
            }

            if (SCR.read_volatile() & SCR_CMD_ERROR_BIT) != 0 {
                panic!("SC64 command failed");
            }
        }
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
