use n64_specs as specs;

use crate::{location::Location, system::System, value::Value};

pub type DdLocation = Location<{ specs::dd::START }, { specs::dd::END }>;

#[derive(Default)]
pub struct Dd;

impl Dd {
    pub fn read<T: Value>(&self, addr: DdLocation) -> T {
        log::warn!("DD: read {:08X}", addr.relative());

        T::default()
    }

    pub fn write<T: Value>(_s: &mut System, addr: DdLocation, data: T) {
        log::warn!("DD: write {:08X} ={:X}", addr.relative(), data);
    }
}
