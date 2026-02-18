use crate::{data::Data, map::Location, system::System};

const RAM_START: u32 = 0x0500_0000;
const RAM_END: u32 = 0x0600_0800;

pub type DdLocation = Location<RAM_START, RAM_END>;

pub struct Dd {
    // TODO ram
}

impl Default for Dd {
    fn default() -> Self {
        Self {}
    }
}

impl Dd {
    pub fn read<T: Data>(&self, addr: DdLocation) -> T {
        log::warn!("read PIF RAM: {:08X}", addr.relative());
        T::default()
    }

    pub fn write<T: Data>(_s: &mut System, addr: DdLocation, data: T) {
        log::warn!("write PIF RAM: {:08X} {:X}", addr.relative(), data.to_u32());
    }
}
