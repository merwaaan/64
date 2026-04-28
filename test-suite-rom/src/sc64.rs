use test_suite_common::Message;

const SCR: *mut u32 = 0xBFFF_0000 as *mut u32;
const DATA0: *mut u32 = 0xBFFF_0004 as *mut u32;
const DATA1: *mut u32 = 0xBFFF_0008 as *mut u32;
const KEY: *mut u32 = 0xBFFF_0010 as *mut u32;

const CMD_CONFIG_SET: u32 = 'C' as u32;
const CMD_USB_WRITE: u32 = 'M' as u32;

pub struct Sc64;

// TODO instance?
impl Sc64 {
    /// Configures the SC64 before using it.
    pub fn configure() {
        unsafe {
            // Unlock the SC registers with the magic code

            KEY.write_volatile(0x00000000);
            KEY.write_volatile(0x5F554E4C);
            KEY.write_volatile(0x4F434B5F);

            // Enable writes to the cart

            DATA0.write_volatile(1); // ROM_WRITE_ENABLE
            DATA1.write_volatile(1); // enabled
            SCR.write_volatile(CMD_CONFIG_SET);
            Self::wait();
        }
    }

    /// Sends a message over USB.
    pub fn send(message: Message) {
        const CART_STAGING: *mut u32 = 0xB300_0000 as *mut u32;
        // TODO use __rom_end to find an apporpriate staging area?

        // Serialize the message

        let mut buffer = postcard::to_allocvec(&message).unwrap();

        let data_length = buffer.len() as u32;

        // Pad the buffer to be aligned to 4-bytes

        buffer.resize(buffer.len().next_multiple_of(4), 0);

        unsafe {
            // Copy the buffer to the cart staging area, as u32 since u8 writes seem buggy on hardware

            for (i, chunk_bytes) in buffer.chunks(4).enumerate() {
                let x = u32::from_be_bytes([
                    chunk_bytes[0],
                    chunk_bytes[1],
                    chunk_bytes[2],
                    chunk_bytes[3],
                ]);

                CART_STAGING.add(i).write_volatile(x);
            }

            // Send the data

            DATA0.write_volatile(0x1300_0000);
            DATA1.write_volatile((0x02 << 24) | data_length);
            SCR.write_volatile(CMD_USB_WRITE);

            Self::wait();
        }
    }

    /// Waits until a command is complete.
    fn wait() {
        const CMD_BUSY: u32 = 1 << 31;

        unsafe {
            let mut t = 0u32;

            while SCR.read_volatile() & CMD_BUSY != 0 {
                t = t.wrapping_add(1);

                if t == 0xFFFF_FFFF {
                    break;
                }
            }
        }
    }
}
