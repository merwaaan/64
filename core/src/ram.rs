use crate::{data::Value, location::Location, system::System};

const DATA_START: u32 = 0x0000_0000;
const DATA_END: u32 = 0x03F0_0000;

const DATA_MAPPED_SIZE: u32 = 0x080_0000; // TODO more with exp pack?

pub type RamLocation = Location<DATA_START, DATA_END>;

const REG_START: u32 = DATA_END;
const REG_END: u32 = 0x0400_0000;
// TODO diff first/second halves? "broadcast" registers?

pub type RamRegsLocation = Location<REG_START, REG_END>;

const INTERFACE_START: u32 = 0x0470_0000;
const INTERFACE_END: u32 = 0x0480_0000;

pub type RamInterfaceLocation = Location<INTERFACE_START, INTERFACE_END>;

pub struct Ram {
    data: Vec<u8>,
    //regs: [u32; 13],
}

impl Default for Ram {
    fn default() -> Self {
        Self {
            data: vec![0; DATA_MAPPED_SIZE as usize],
            //regs: [0; 13],
        }
    }
}

impl Ram {
    pub fn read<T: Value>(s: &System, addr: RamLocation) -> T {
        match addr.relative() {
            0..DATA_MAPPED_SIZE => T::read_mem(&s.ram.data, addr.relative()),
            _ => {
                //log::warn!("Invalid RAM data read: {:08X}", addr.relative());

                T::default()
            }
        }
    }

    pub fn write<T: Value>(s: &mut System, addr: RamLocation, data: T) {
        match addr.relative() {
            0..DATA_MAPPED_SIZE => data.write_mem(&mut s.ram.data, addr.relative()),
            _ => {} // _ => log::warn!(
                    //     "Invalid RAM data write: {:08X} {:X}",
                    //     addr.relative(),
                    //     data
                    // ),
        }
    }

    pub fn read_reg<T: Value>(&self, addr: RamRegsLocation) -> T {
        log::warn!("Read RAM reg UNIMPLEMENTED: {:08X}", addr.relative());

        T::default()
    }

    pub fn write_reg<T: Value>(_s: &mut System, addr: RamRegsLocation, data: T) {
        log::warn!(
            "Write RAM reg UNIMPLEMENTED: {:08X} {:X}",
            addr.relative(),
            data
        );
    }

    pub fn read_interface<T: Value>(&self, addr: RamInterfaceLocation) -> T {
        if addr.relative() == 0x0C {
            log::warn!(
                "Reading from RAM interface 0x14 RI_SELECT: {:08X}",
                addr.relative()
            );

            // TODO temp
            T::read_reg(&[0x14u32], addr.relative() & 3)
        } else {
            log::warn!("Read RAM interface UNIMPLEMENTED: {:08X}", addr.relative());

            T::default()
        }
    }

    pub fn write_interface<T: Value>(_s: &mut System, addr: RamInterfaceLocation, data: T) {
        log::warn!(
            "Write RAM interface UNIMPLEMENTED: {:08X} {:X}",
            addr.relative(),
            data
        );
    }

    pub fn reg_info(addr: RamRegsLocation) -> Option<&'static str> {
        match addr.relative() & 0x27 {
            0x00 => Some("RAM device type"),
            0x04 => Some("RAM device ID"),
            0x08 => Some("RAM delay"),
            0x0C => Some("RAM mode"),
            0x10 => Some("RAM RefInterval"),
            0x14 => Some("RAM RefRow"),
            0x18 => Some("RAM RasInterval"),
            0x1C => Some("RAM MinInterval "),
            0x20 => Some("RAM AddressSelect  "),
            0x24 => Some("RAM DeviceManufacturer  "),
            _ => None,
        }
    }

    pub fn interface_info(addr: RamInterfaceLocation) -> Option<&'static str> {
        match addr.relative() & 0x3F {
            0x00 => Some("RI_MODE"),
            0x04 => Some("RI_CONFIG"),
            0x08 => Some("RI_CURRENT_LOAD"),
            0x0C => Some("RI_SELECT"),
            0x10 => Some("RI_REFRESH"),
            0x14 => Some("RI_LATENCY"),
            0x18 => Some("RI_ERROR"),
            0x1C => Some("RI_BANK_STATUS"),
            _ => None,
        }
    }
}
