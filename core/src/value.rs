use std::{
    fmt::{LowerHex, UpperHex},
    mem,
};

// TODO optim: simpler logic if aligned?

/// The Nintendo 64 is big-endian.
/// The computer running the emulator is little-endian.
///
/// We model individual registers as native-endian u32/u64 values since it's clearer and more efficient.
///
/// We model blocks of memory like RAM as u8 arrays so that big-endian addressing just works.
/// However, we have to convert data from/to native endianness when writing/reading memory.
///
/// This trait adds methods to read from and write to big-endian memory and little-endian registers using the N64 big-endian addressing.
/// It handles the mixed endianness at the boundary between memory and registers and spares us from having to converting between BE and LE everywhere.
///
/// ```
/// use n64_core::value::Value;
///
/// // N64 emulated memory, big-endian
/// let mut mem = [0xAA, 0xBB, 0xCC, 0xDD, 0x11, 0x22, 0x33, 0x44];
///
/// // Read data from memory, the value is correctly interpreted as big-endian
/// let data = u32::read_mem(&mem, 2);
/// assert_eq!(data, 0xCCDD1122);
/// assert_eq!(data.to_be_bytes(), [0xCC, 0xDD, 0x11, 0x22]);
/// assert_eq!(data.to_le_bytes(), [0x22, 0x11, 0xDD, 0xCC]);
///
/// // Write data to memory, it's stored in big-endian
/// let data: u16 = 0x7788;
/// assert_eq!(data.to_be_bytes(), [0x77, 0x88]);
/// assert_eq!(data.to_le_bytes(), [0x88, 0x77]);
/// data.write_mem(&mut mem, 4);
/// assert_eq!(mem, [0xAA, 0xBB, 0xCC, 0xDD, 0x77, 0x88, 0x33, 0x44]);
///
/// // N64 emulated registers, little-endian
/// let mut regs = [0x33445566];
///
/// // N64 instructions use big-endian addressing, we can use read_reg/write_reg in their implementation
/// let data = u8::read_reg(&regs, 1);
/// assert_eq!(regs[0].to_be_bytes(), [0x33, 0x44, 0x55, 0x66]);
/// assert_eq!(regs[0].to_le_bytes(), [0x66, 0x55, 0x44, 0x33]);
/// assert_eq!(data, 0x44);
///
/// data.write_reg(&mut regs, 3);
/// assert_eq!(regs, [0x33445544]);
/// ```
pub trait Value: Sized + Copy + Default + LowerHex + UpperHex + std::fmt::Debug {
    const BYTES: usize = mem::size_of::<Self>();

    /// Reads a value from a memory slice at a given big-endian address.
    fn read_mem(mem: &[u8], address: u32) -> Self;

    // Writes a value to a memory slice at a given big-endian address.
    fn write_mem(self, mem: &mut [u8], address: u32);

    /// Reads a value from a register slice at a given big-endian address.
    fn read_reg(regs: &[u32], address: u32) -> Self;

    /// Writes a value to a register slice at a given big-endian address.
    fn write_reg(self, regs: &mut [u32], address: u32);
}

/// Converts a little-endian address to a big-endian address in a u32 slice.
#[inline(always)]
fn le_to_be_address_32(address: u32) -> usize {
    let word_start = address & !3;
    let byte_offset = address & 3;

    (word_start + (3 - byte_offset)) as usize
}

impl Value for u8 {
    fn read_mem(mem: &[u8], addr: u32) -> u8 {
        mem[addr as usize]
    }

    fn write_mem(self, mem: &mut [u8], addr: u32) {
        mem[addr as usize] = self;
    }

    fn read_reg(regs: &[u32], address: u32) -> u8 {
        let bytes: &[u8] = bytemuck::cast_slice(regs);

        bytes[le_to_be_address_32(address)]
    }

    fn write_reg(self, regs: &mut [u32], address: u32) {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(regs);

        bytes[le_to_be_address_32(address)] = self as u8;
    }
}

impl Value for u16 {
    fn read_mem(mem: &[u8], address: u32) -> u16 {
        u16::from_be_bytes([mem[address as usize], mem[address as usize + 1]])
    }

    fn write_mem(self, mem: &mut [u8], address: u32) {
        mem[address as usize] = (self >> 8) as u8;
        mem[address as usize + 1] = self as u8;
    }

    fn read_reg(regs: &[u32], address: u32) -> u16 {
        let bytes: &[u8] = bytemuck::cast_slice(regs);

        u16::from_be_bytes([
            bytes[le_to_be_address_32(address)],
            bytes[le_to_be_address_32(address + 1)],
        ])
    }

    fn write_reg(self, regs: &mut [u32], address: u32) {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(regs);

        bytes[le_to_be_address_32(address)] = (self >> 8) as u8;
        bytes[le_to_be_address_32(address + 1)] = self as u8;
    }
}

impl Value for u32 {
    fn read_mem(mem: &[u8], addr: u32) -> u32 {
        u32::from_be_bytes([
            mem[addr as usize],
            mem[addr as usize + 1],
            mem[addr as usize + 2],
            mem[addr as usize + 3],
        ])
    }

    fn write_mem(self, mem: &mut [u8], addr: u32) {
        mem[addr as usize] = (self >> 24) as u8;
        mem[addr as usize + 1] = (self >> 16) as u8;
        mem[addr as usize + 2] = (self >> 8) as u8;
        mem[addr as usize + 3] = self as u8;
    }

    fn read_reg(regs: &[u32], address: u32) -> u32 {
        let bytes: &[u8] = bytemuck::cast_slice(regs);

        u32::from_be_bytes([
            bytes[le_to_be_address_32(address)],
            bytes[le_to_be_address_32(address + 1)],
            bytes[le_to_be_address_32(address + 2)],
            bytes[le_to_be_address_32(address + 3)],
        ])
    }

    fn write_reg(self, regs: &mut [u32], address: u32) {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(regs);

        bytes[le_to_be_address_32(address)] = (self >> 24) as u8;
        bytes[le_to_be_address_32(address + 1)] = (self >> 16) as u8;
        bytes[le_to_be_address_32(address + 2)] = (self >> 8) as u8;
        bytes[le_to_be_address_32(address + 3)] = self as u8;
    }
}

impl Value for u64 {
    fn read_mem(mem: &[u8], addr: u32) -> u64 {
        u64::from_be_bytes([
            mem[addr as usize],
            mem[addr as usize + 1],
            mem[addr as usize + 2],
            mem[addr as usize + 3],
            mem[addr as usize + 4],
            mem[addr as usize + 5],
            mem[addr as usize + 6],
            mem[addr as usize + 7],
        ])
    }

    fn write_mem(self, mem: &mut [u8], addr: u32) {
        mem[addr as usize] = (self >> 56) as u8;
        mem[addr as usize + 1] = (self >> 48) as u8;
        mem[addr as usize + 2] = (self >> 40) as u8;
        mem[addr as usize + 3] = (self >> 32) as u8;
        mem[addr as usize + 4] = (self >> 24) as u8;
        mem[addr as usize + 5] = (self >> 16) as u8;
        mem[addr as usize + 6] = (self >> 8) as u8;
        mem[addr as usize + 7] = self as u8;
    }

    fn read_reg(regs: &[u32], address: u32) -> u64 {
        let bytes: &[u8] = bytemuck::cast_slice(regs);

        u64::from_be_bytes([
            bytes[le_to_be_address_32(address)],
            bytes[le_to_be_address_32(address + 1)],
            bytes[le_to_be_address_32(address + 2)],
            bytes[le_to_be_address_32(address + 3)],
            bytes[le_to_be_address_32(address + 4)],
            bytes[le_to_be_address_32(address + 5)],
            bytes[le_to_be_address_32(address + 6)],
            bytes[le_to_be_address_32(address + 7)],
        ])
    }

    fn write_reg(self, regs: &mut [u32], address: u32) {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(regs);

        bytes[le_to_be_address_32(address)] = (self >> 56) as u8;
        bytes[le_to_be_address_32(address + 1)] = (self >> 48) as u8;
        bytes[le_to_be_address_32(address + 2)] = (self >> 40) as u8;
        bytes[le_to_be_address_32(address + 3)] = (self >> 32) as u8;
        bytes[le_to_be_address_32(address + 4)] = (self >> 24) as u8;
        bytes[le_to_be_address_32(address + 5)] = (self >> 16) as u8;
        bytes[le_to_be_address_32(address + 6)] = (self >> 8) as u8;
        bytes[le_to_be_address_32(address + 7)] = self as u8;
    }
}
