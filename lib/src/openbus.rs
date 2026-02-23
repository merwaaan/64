use crate::data::Value;

// Open bus: https://n64brew.dev/wiki/Parallel_Interface#Open_bus_behavior

pub fn read<T: Value>(addr: u32) -> T {
    //log::warn!("read open bus {:08X}", addr);

    let lo = addr as u16 as u32;

    T::read_reg(&[(lo << 16) | lo], addr & 3) // TODO what if > 16????
}

pub fn write<T: Value>(addr: u32, data: T) {
    log::warn!("write open bus {:08X} {:X}", addr, data);
}
