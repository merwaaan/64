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

pub fn physical_addr(address: u32) -> u32 {
    address & 0x1FFF_FFFF
}

pub fn uncached_addr(offset: u32) -> u32 {
    physical_addr(offset) | (n64_specs::map::Segment::KSEG1 as u32)
}

pub fn uncached_ptr<T>(offset: u32) -> *mut T {
    uncached_addr(offset) as *mut T
}

pub fn uncached_ptr_of<T>(value: &T) -> *mut T {
    uncached_addr(value as *const T as u32) as *mut T
}

pub fn read_uncached<T>(offset: u32) -> T {
    unsafe { uncached_ptr::<T>(offset).read_volatile() }
}

pub fn write_uncached<T>(offset: u32, value: T) {
    unsafe { uncached_ptr::<T>(offset).write_volatile(value) }
}

/// Aligned heap buffer with a fixed size.
pub struct Buffer<T, const UNCACHED: bool> {
    data: NonNull<T>,
    size: usize,
    layout: Layout,
}

pub type CachedBuffer<T> = Buffer<T, false>;
pub type UncachedBuffer<T> = Buffer<T, true>;

impl<T, const UNCACHED: bool> Buffer<T, UNCACHED> {
    pub fn with_alignment(size: usize, alignment: usize) -> Self {
        assert!(size > 0, "buffer size must be > 0");

        assert!(
            alignment.is_power_of_two(),
            "buffer alignment must be a power of two"
        );

        let actual_alignment = alignment.max(core::mem::align_of::<T>());

        let layout = Layout::array::<T>(size)
            .and_then(|l| l.align_to(actual_alignment))
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

    pub fn from_slice(values: &[T]) -> Self
    where
        T: Copy,
    {
        Self::from_slice_with_alignment(values, 8)
    }

    pub fn from_slice_with_alignment(values: &[T], alignment: usize) -> Self
    where
        T: Copy,
    {
        assert!(!values.is_empty(), "buffer source slice must not be empty");

        let mut buffer = Self::with_alignment(values.len(), alignment);

        for (index, value) in values.iter().copied().enumerate() {
            buffer.set(index, value);
        }

        buffer
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn check_bounds(&self, index: usize) {
        assert!(
            index < self.size,
            "buffer index out of bounds ({}/{})",
            index,
            self.size
        );
    }

    pub fn item_ptr(&self, index: usize) -> *mut T {
        self.check_bounds(index);

        if UNCACHED {
            let byte_offset = index
                .checked_mul(core::mem::size_of::<T>())
                .expect("buffer index overflow");

            let physical =
                physical_addr(self.data.as_ptr() as u32).wrapping_add(byte_offset as u32);

            (n64_specs::map::Segment::KSEG1 as u32 | physical) as *mut T
        } else {
            unsafe { self.data.as_ptr().add(index) }
        }
    }

    pub fn get(&self, index: usize) -> T {
        self.check_bounds(index);

        unsafe { self.item_ptr(index).read_volatile() }
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.check_bounds(index);

        unsafe { self.item_ptr(index).write_volatile(value) };
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.size) }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.item_ptr(0)
    }
}

impl<T, const UNCACHED: bool> Drop for Buffer<T, UNCACHED> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_ptr().cast(), self.layout);
        }
    }
}

// PI

pub fn wait_for_pi() {
    while read_uncached::<u32>(n64_specs::pi::Status::ADDRESS) & 0x3 != 0 {}
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
    pub length: u24, // TODO accept +1?
}

pub fn pi_dma(dma: &PiDma, wait: bool) {
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

    if wait {
        wait_until(|| read_uncached::<u32>(n64_specs::pi::Status::ADDRESS) & 0x1 == 0);
    }
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
