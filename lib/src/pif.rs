use crate::{data::Data, map::Location, system::System};

const RAM_START: u32 = 0x1FC0_07C0;
const RAM_END: u32 = 0x1FC0_0800;

pub type PifRamLocation = Location<RAM_START, RAM_END>;

pub struct Pif {
    // TODO ram
}

impl Default for Pif {
    fn default() -> Self {
        Self {}
    }
}

impl Pif {
    pub fn read<T: Data>(&self, addr: PifRamLocation) -> T {
        log::warn!("read PIF RAM: {:08X}", addr.relative());
        T::default()
    }

    pub fn write<T: Data>(_s: &mut System, addr: PifRamLocation, data: T) {
        log::warn!("write PIF RAM: {:08X} {:X}", addr.relative(), data.to_u32());
    }
}
