use std::{
    fmt::{LowerHex, UpperHex},
    mem,
};

// TODO wrapper type BE, NE?

pub trait Value: Sized + Copy + Default + LowerHex + UpperHex + std::fmt::Debug {
    const BYTES: usize = mem::size_of::<Self>();

    fn read_mem(mem: &[u8], offset: u32) -> Self;
    fn write_mem(self, mem: &mut [u8], offset: u32);

    fn read_reg(regs: &[u32], offset: u32) -> Self;
    fn write_reg(self, regs: &mut [u32], offset: u32);
}

impl Value for u8 {
    fn read_mem(from: &[u8], addr: u32) -> u8 {
        from[addr as usize]
    }

    fn write_mem(self, to: &mut [u8], addr: u32) {
        to[addr as usize] = self;
    }

    fn read_reg(regs: &[u32], offset: u32) -> u8 {
        let word = regs[(offset >> 2) as usize];
        word.to_be_bytes()[(offset & 3) as usize]
    }

    fn write_reg(self, regs: &mut [u32], offset: u32) {
        let word = regs[(offset >> 2) as usize];
        let mut bytes = word.to_be_bytes();
        bytes[(offset & 3) as usize] = self;
        regs[(offset >> 2) as usize] = u32::from_be_bytes(bytes);
    }
}

impl Value for u16 {
    fn read_mem(from: &[u8], offset: u32) -> u16 {
        debug_assert!(
            offset & 1 == 0,
            "u16 mem read from unaligned offset: {:08X}",
            offset
        );

        u16::from_be_bytes(
            from[offset as usize..offset as usize + 2]
                .try_into()
                .unwrap(),
        )
    }

    fn write_mem(self, to: &mut [u8], offset: u32) {
        debug_assert!(
            offset & 1 == 0,
            "u16 mem write to unaligned offset: {:08X}",
            offset
        );

        to[offset as usize..offset as usize + 2].copy_from_slice(&self.to_be_bytes());
    }

    fn read_reg(regs: &[u32], offset: u32) -> u16 {
        debug_assert!(
            offset & 1 == 0,
            "u16 reg read from unaligned offset: {:08X}",
            offset
        );

        let word = regs[(offset >> 2) as usize];
        let bytes = word.to_be_bytes();

        let half_index = (offset & 2) as usize;

        u16::from_be_bytes([bytes[half_index], bytes[half_index + 1]])
    }

    fn write_reg(self, regs: &mut [u32], offset: u32) {
        debug_assert!(
            offset & 1 == 0,
            "u16 reg write to unaligned offset: {:08X}",
            offset
        );

        // TODO optim: avoid conversion???

        let word_index = (offset >> 2) as usize;
        let word = regs[word_index];
        let mut bytes = word.to_be_bytes();

        let half_index = (offset & 2) as usize;

        bytes[half_index] = (self >> 8) as u8;
        bytes[half_index + 1] = self as u8;

        regs[word_index] = u32::from_be_bytes(bytes);
    }
}

impl Value for u32 {
    fn read_mem(from: &[u8], addr: u32) -> u32 {
        debug_assert!(
            addr & 3 == 0,
            "u32 mem read from unaligned offset: {:08X}",
            addr
        );

        u32::from_be_bytes(from[addr as usize..addr as usize + 4].try_into().unwrap())
    }

    fn write_mem(self, to: &mut [u8], addr: u32) {
        debug_assert!(
            addr & 3 == 0,
            "u32 mem write to unaligned offset: {:08X}",
            addr
        );

        to[addr as usize..addr as usize + 4].copy_from_slice(&self.to_be_bytes());
    }

    fn read_reg(regs: &[u32], offset: u32) -> u32 {
        debug_assert!(
            offset & 3 == 0,
            "u32 reg read from unaligned offset: {:08X}",
            offset
        );

        regs[(offset >> 2) as usize]
    }

    fn write_reg(self, regs: &mut [u32], offset: u32) {
        debug_assert!(
            offset & 3 == 0,
            "u32 reg write to unaligned offset: {:08X}",
            offset
        );

        regs[(offset >> 2) as usize] = self;
    }
}

impl Value for u64 {
    fn read_mem(from: &[u8], addr: u32) -> u64 {
        debug_assert!(
            addr & 7 == 0,
            "u64 mem read from unaligned offset: {:08X}",
            addr
        );

        u64::from_be_bytes(from[addr as usize..addr as usize + 8].try_into().unwrap())
    }

    fn write_mem(self, to: &mut [u8], addr: u32) {
        debug_assert!(
            addr & 7 == 0,
            "u64 mem write to unaligned offset: {:08X}",
            addr
        );

        to[addr as usize..addr as usize + 8].copy_from_slice(&self.to_be_bytes());
    }

    fn read_reg(regs: &[u32], offset: u32) -> u64 {
        debug_assert!(
            offset & 7 == 0,
            "u64 reg read from unaligned offset: {:08X}",
            offset
        );

        let word_index = (offset >> 2) as usize;

        ((regs[word_index] as u64) << 32) | (regs[word_index + 1] as u64)
    }

    fn write_reg(self, regs: &mut [u32], offset: u32) {
        debug_assert!(
            offset & 7 == 0,
            "u64 reg write to unaligned offset: {:08X}",
            offset
        );

        let word_index = (offset >> 2) as usize;
        regs[word_index] = (self >> 32) as u32;
        regs[word_index + 1] = self as u32;
    }
}
