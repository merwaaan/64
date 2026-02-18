use crate::data::Data;

// Open bus: https://n64brew.dev/wiki/Parallel_Interface#Open_bus_behavior

pub fn read<T: Data>(addr: u32) -> T {
    log::warn!("read open bus {:08X}", addr);

    let lo = addr & 0xFFFF;
    T::from_u32((lo << 16) | lo) // TODO weirddd
}

pub fn write<T: Data>(addr: u32, data: T) {
    log::warn!("write open bus {:08X} {:X}", addr, data.to_u32());
}
