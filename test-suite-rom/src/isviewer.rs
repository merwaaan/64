use alloc::fmt;
use core::fmt::Arguments;

// TODO use specs
const LENGTH_REG: *mut u32 = 0xB3FF0014u32 as *mut u32;
const BUF_START: *mut u32 = 0xB3FF0020u32 as *mut u32;
const BUF_SIZE: usize = 0x200;

pub fn write_fmt(args: Arguments) {
    write_raw(fmt::format(args));
}

// TODO why not byte by byte?
pub fn write_raw<T: AsRef<[u8]>>(data: T) {
    //TODO add critical section here
    // Credit to Lemmy for algorithm: https://github.com/lemmy-64/n64-systemtest/blob/main/src/isviewer.rs
    for chunk in data.as_ref().chunks(BUF_SIZE) {
        let mut value = 0;
        let mut shift = 24u32;
        let mut i = 0;
        for byte in chunk {
            value |= (*byte as u32) << shift;
            if shift == 0 {
                push(i, value);
                i += 1;
                shift = 24;
                value = 0;
            } else {
                shift -= 8;
            }
        }
        if shift < 24 {
            push(i, value);
        }
        flush(chunk.len());
    }
}

#[inline(always)]
fn push(word_count: usize, word: u32) {
    unsafe {
        BUF_START.add(word_count).write_volatile(word);
    }
}

#[inline(always)]
fn flush(byte_count: usize) {
    unsafe {
        LENGTH_REG.write_volatile(byte_count as u32);
    }
}
