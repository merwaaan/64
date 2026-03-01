use crate::{data::Value, map::Location, system::System};

const RAM_START: u32 = 0x1FC0_07C0;
const RAM_END: u32 = 0x1FC0_0800;

pub type PifRamLocation = Location<RAM_START, RAM_END>;

pub struct Pif {
    data: [u8; 0x40],
}

impl Default for Pif {
    fn default() -> Self {
        Self { data: [0; 0x40] }
    }
}

impl Pif {
    pub fn read<T: Value>(&self, addr: PifRamLocation) -> T {
        log::error!(
            "read PIF RAM: {:08X} {:X}",
            addr.relative(),
            self.data[addr.relative() as usize]
        );

        T::read_mem(&self.data, addr.relative())
    }

    pub fn write<T: Value>(s: &mut System, addr: PifRamLocation, data: T) {
        log::error!("write PIF RAM: {:08X} {:X}", addr.relative(), data);

        data.write_mem(&mut s.map.pif.data, addr.relative());

        // TODO could be offset dep on width?
        if addr.relative() == 0x3C {
            log::error!("PIF COMMAND??? {:X}", data);

            s.map.pif.data[0x3C] = 0;
            s.map.pif.data[0x3D] = 0; // TODO single byte or word?
            s.map.pif.data[0x3E] = 0;
            s.map.pif.data[0x3F] = 0;
        }
    }
}
