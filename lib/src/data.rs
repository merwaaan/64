use std::fmt::{LowerHex, UpperHex};

pub trait Data: Default + LowerHex + UpperHex {
    const SIZE: usize;

    fn read(buffer: &[u8], offset: u32) -> Self;

    fn write(self, buffer: &mut [u8], offset: u32);

    // TODO temp hack to work with 32bits interfaces, clean up in interfaces instead
    fn to_u32(self) -> u32;
    fn from_u32(value: u32) -> Self;
}

impl Data for u8 {
    const SIZE: usize = 1;

    fn read(buffer: &[u8], offset: u32) -> Self {
        buffer[offset as usize]
    }

    fn write(self, buffer: &mut [u8], offset: u32) {
        buffer[offset as usize] = self;
    }

    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_u32(value: u32) -> Self {
        value as u8
    }
}

impl Data for u16 {
    const SIZE: usize = 2;

    fn read(buffer: &[u8], offset: u32) -> Self {
        let offset = offset as usize;

        (buffer[offset] as u16) << 8 | buffer[offset + 1] as u16
    }

    fn write(self, buffer: &mut [u8], offset: u32) {
        let offset = offset as usize;

        buffer[offset] = (self >> 8) as u8;
        buffer[offset + 1] = self as u8;
    }

    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_u32(value: u32) -> Self {
        value as u16
    }
}

impl Data for u32 {
    const SIZE: usize = 4;

    fn read(buffer: &[u8], offset: u32) -> Self {
        let offset = offset as usize;

        (buffer[offset] as u32) << 24
            | (buffer[offset + 1] as u32) << 16
            | (buffer[offset + 2] as u32) << 8
            | buffer[offset + 3] as u32
    }

    fn write(self, buffer: &mut [u8], offset: u32) {
        let offset = offset as usize;

        buffer[offset] = (self >> 24) as u8;
        buffer[offset + 1] = (self >> 16) as u8;
        buffer[offset + 2] = (self >> 8) as u8;
        buffer[offset + 3] = self as u8;
    }

    fn to_u32(self) -> u32 {
        self
    }

    fn from_u32(value: u32) -> Self {
        value
    }
}

impl Data for u64 {
    const SIZE: usize = 8;

    fn read(buffer: &[u8], offset: u32) -> Self {
        let offset = offset as usize;

        (buffer[offset as usize] as u64) << 56
            | (buffer[offset + 1] as u64) << 48
            | (buffer[offset + 2] as u64) << 40
            | (buffer[offset + 3] as u64) << 32
            | (buffer[offset + 4] as u64) << 24
            | (buffer[offset + 5] as u64) << 16
            | (buffer[offset + 6] as u64) << 8
            | buffer[offset + 7] as u64
    }

    fn write(self, buffer: &mut [u8], offset: u32) {
        let offset = offset as usize;

        buffer[offset] = (self >> 56) as u8;
        buffer[offset + 1] = (self >> 48) as u8;
        buffer[offset + 2] = (self >> 40) as u8;
        buffer[offset + 3] = (self >> 32) as u8;
        buffer[offset + 4] = (self >> 24) as u8;
        buffer[offset + 5] = (self >> 16) as u8;
        buffer[offset + 6] = (self >> 8) as u8;
        buffer[offset + 7] = self as u8;
    }

    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_u32(value: u32) -> Self {
        value as u64
    }
}
