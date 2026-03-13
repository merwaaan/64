use crate::{value::Value, location::Location, system::System};

const RAM_START: u32 = 0x0500_0000;
const RAM_END: u32 = 0x0600_0800;

pub type DdLocation = Location<RAM_START, RAM_END>;

#[derive(Default)]
pub struct Dd {
    // TODO ram
}

impl Dd {
    pub fn read<T: Value>(&self, addr: DdLocation) -> T {
        log::warn!("read DD RAM: {:08X}", addr.relative());
        T::default()
    }

    pub fn write<T: Value>(_s: &mut System, addr: DdLocation, data: T) {
        log::warn!("write DD RAM: {:08X} {:X}", addr.relative(), data);
    }
}
