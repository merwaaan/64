use alloc::alloc::{alloc, dealloc, handle_alloc_error};
use arbitrary_int::prelude::*;
use core::{alloc::Layout, ptr::NonNull};

pub fn wait_until(condition: impl Fn() -> bool) {
    loop {
        if condition() {
            break;
        }
    }
}

// Uncached memory access

pub fn physical(address: u32) -> u32 {
    address & 0x1FFF_FFFF
}

pub fn uncached_ptr(offset: u32) -> *mut u32 {
    (n64_specs::map::Segment::KSEG1 as u32 | offset) as *mut u32
}

pub fn read_uncached(offset: u32) -> u32 {
    unsafe { uncached_ptr(offset).read_volatile() }
}

pub fn write_uncached(offset: u32, value: u32) {
    unsafe { uncached_ptr(offset).write_volatile(value) }
}

//

pub struct Buffer<T> {
    data: NonNull<T>,
    size: usize,
    layout: Layout,
}

impl<T> Buffer<T> {
    pub fn new(size: usize) -> Self {
        Self::with_alignment(size, 8)
    }

    pub fn with_alignment(size: usize, alignment: usize) -> Self {
        assert!(size > 0, "buffer size must be > 0");

        assert!(
            alignment.is_power_of_two(),
            "buffer alignment must be a power of two"
        );

        let layout = Layout::array::<T>(size)
            .and_then(|l| l.align_to(alignment))
            .and_then(|l| Ok(l.pad_to_align()))
            .expect("invalid buffer layout");

        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        Self {
            data: unsafe { NonNull::new_unchecked(ptr.cast::<T>()) },
            size,
            layout,
        }
    }

    // TODO also byte_len?

    pub fn len(&self) -> usize {
        self.size
    }

    fn uncached_item(&self, index: usize) -> *mut T {
        let byte_offset = index
            .checked_mul(core::mem::size_of::<T>())
            .expect("buffer index overflow");

        let physical = physical(self.data.as_ptr() as u32).wrapping_add(byte_offset as u32);

        (n64_specs::map::Segment::KSEG1 as u32 | physical) as *mut T
    }

    pub fn as_ptr(&self) -> *mut T {
        self.uncached_item(0)
    }

    pub fn get(&self, index: usize) -> T {
        unsafe { self.uncached_item(index).read_volatile() }
    }

    pub fn set(&mut self, index: usize, value: T) {
        unsafe { self.uncached_item(index).write_volatile(value) };
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_ptr().cast(), self.layout);
        }
    }
}

// PI

pub fn wait_for_pi() {
    while read_uncached(n64_specs::pi::Status::ADDRESS) & 0x3 != 0 {}
    // TODO timeout?
}

pub enum PiDmaDirection {
    RamToPi,
    PiToRam,
}

pub struct PiDma {
    pub direction: PiDmaDirection,
    pub ram_address: u24,
    pub pi_address: u32,
    pub length: u24,
}

// TODO wait option?
pub fn pi_dma(dma: &PiDma) {
    // TODO wait for other DMA

    write_uncached(
        n64_specs::pi::DmaRamAddress::ADDRESS,
        dma.ram_address.value(),
    );

    write_uncached(n64_specs::pi::DmaPiAddress::ADDRESS, dma.pi_address);

    let start_reg_address = match dma.direction {
        PiDmaDirection::RamToPi => n64_specs::pi::DmaReadLength::ADDRESS,
        PiDmaDirection::PiToRam => n64_specs::pi::DmaWriteLength::ADDRESS,
    };

    write_uncached(start_reg_address, dma.length.value());
}

// RSP

pub enum RspDmaDirection {
    RamToRsp,
    RspToRam,
}

pub struct RspDma {
    pub direction: RspDmaDirection,
    pub source_address: u32,
    pub destination_address: u32,
    pub rows: u8,
    pub length: u12,
    pub skip: u12,
}

// TODO wait option?
pub fn rsp_dma(dma: &RspDma) {
    // TODO wait for other DMA

    write_uncached(
        n64_specs::rsp::DmaRspAddress::ADDRESS,
        dma.destination_address,
    );

    write_uncached(n64_specs::rsp::DmaRamAddress::ADDRESS, dma.source_address);

    let start_reg_address = match dma.direction {
        RspDmaDirection::RamToRsp => n64_specs::rsp::DmaReadLength::ADDRESS,
        RspDmaDirection::RspToRam => n64_specs::rsp::DmaWriteLength::ADDRESS,
    };

    write_uncached(
        start_reg_address,
        n64_specs::rsp::DmaReadLength::default()
            .with_rows(dma.rows)
            .with_length(dma.length)
            .with_skip(dma.skip)
            .raw_value(),
    );
}
