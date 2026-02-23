use crate::{data::Value, map::Location, system::System};

const DATA_START: u32 = 0x0000_0000;
const DATA_END: u32 = 0x03F0_0000;

const DATA_MAPPED_SIZE: u32 = 0x080_0000; // TODO more with exp pack?

pub type RdramLocation = Location<DATA_START, DATA_END>;

const REG_START: u32 = DATA_END;
const REG_END: u32 = 0x0400_0000;
// TODO diff first/second halves? "broadcast" registers?

pub type RdramRegsLocation = Location<REG_START, REG_END>;

const INTERFACE_START: u32 = 0x0470_0000;
const INTERFACE_END: u32 = 0x0480_0000;

pub type RdramInterfaceLocation = Location<INTERFACE_START, INTERFACE_END>;

pub struct Rdram {
    data: Vec<u8>,
    //regs: [u32; 13],
}

impl Default for Rdram {
    fn default() -> Self {
        Self {
            data: vec![0; DATA_MAPPED_SIZE as usize],
            //regs: [0; 13],
        }
    }
}

impl Rdram {
    pub fn read<T: Value>(&self, addr: RdramLocation) -> T {
        match addr.relative() {
            0..DATA_MAPPED_SIZE => T::read_mem(&self.data, addr.relative()),
            _ => {
                //log::warn!("Invalid RDRAM data read: {:08X}", addr.relative());

                T::default()
            }
        }
    }

    pub fn write<T: Value>(s: &mut System, addr: RdramLocation, data: T) {
        match addr.relative() {
            0..DATA_MAPPED_SIZE => data.write_mem(&mut s.map.rdram.data, addr.relative()),
            _ => {} // _ => log::warn!(
                    //     "Invalid RDRAM data write: {:08X} {:X}",
                    //     addr.relative(),
                    //     data
                    // ),
        }
    }

    pub fn read_reg<T: Value>(&self, addr: RdramRegsLocation) -> T {
        log::warn!("Read RDRAM reg UNIMPLEMENTED: {:08X}", addr.relative());

        T::default()
    }

    pub fn write_reg<T: Value>(_s: &mut System, addr: RdramRegsLocation, data: T) {
        log::warn!(
            "Write RDRAM reg UNIMPLEMENTED: {:08X} {:X}",
            addr.relative(),
            data
        );
    }

    pub fn read_interface<T: Value>(&self, addr: RdramInterfaceLocation) -> T {
        if addr.relative() == 0x0C {
            log::warn!(
                "Reading from RDRAM interface 0x14 RI_SELECT: {:08X}",
                addr.relative()
            );

            // TODO temp
            T::read_reg(&[0x14u32], addr.relative() & 3)
        } else {
            log::warn!(
                "Read RDRAM interface UNIMPLEMENTED: {:08X}",
                addr.relative()
            );

            T::default()
        }
    }

    pub fn write_interface<T: Value>(_s: &mut System, addr: RdramInterfaceLocation, data: T) {
        log::warn!(
            "Write RDRAM interface UNIMPLEMENTED: {:08X} {:X}",
            addr.relative(),
            data
        );
    }

    pub fn reg_info(addr: RdramRegsLocation) -> Option<&'static str> {
        match addr.relative() & 0x27 {
            0x00 => Some("RDRAM device type"),
            0x04 => Some("RDRAM device ID"),
            0x08 => Some("RDRAM delay"),
            0x0C => Some("RDRAM mode"),
            0x10 => Some("RDRAM RefInterval"),
            0x14 => Some("RDRAM RefRow"),
            0x18 => Some("RDRAM RasInterval"),
            0x1C => Some("RDRAM MinInterval "),
            0x20 => Some("RDRAM AddressSelect  "),
            0x24 => Some("RDRAM DeviceManufacturer  "),
            _ => None,
        }
    }

    pub fn interface_info(addr: RdramInterfaceLocation) -> Option<&'static str> {
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
