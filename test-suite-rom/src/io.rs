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

// Aligned heap buffer with a fixed capacity.
// Reads and writes go through uncached memory, which is slower but also convenient to avoid caching issues.
pub struct Buffer<T> {
    data: NonNull<T>,
    capacity: usize,
    size: usize,
    layout: Layout,
}

// TODO constructor with iter?
impl<T> Buffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self::with_alignment(capacity, 8)
    }

    pub fn with_alignment(capacity: usize, alignment: usize) -> Self {
        assert!(capacity > 0, "buffer capacity must be > 0");

        assert!(
            alignment.is_power_of_two(),
            "buffer alignment must be a power of two"
        );

        let layout = Layout::array::<T>(capacity)
            .and_then(|l| l.align_to(alignment))
            .and_then(|l| Ok(l.pad_to_align()))
            .expect("invalid buffer layout");

        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        Self {
            data: unsafe { NonNull::new_unchecked(ptr.cast::<T>()) },
            capacity,
            size: 0,
            layout,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn uncached_item(&self, index: usize) -> *mut T {
        let byte_offset = index
            .checked_mul(core::mem::size_of::<T>())
            .expect("buffer index overflow"); // TODO not the actual error

        let physical = physical_addr(self.data.as_ptr() as u32).wrapping_add(byte_offset as u32);

        (n64_specs::map::Segment::KSEG1 as u32 | physical) as *mut T
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.size) }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.uncached_item(0)
    }

    pub fn get(&self, index: usize) -> T {
        unsafe { self.uncached_item(index).read_volatile() }
    }

    fn set(&mut self, index: usize, value: T) {
        assert!(
            index < self.size,
            "buffer index out of bounds ({}/{})",
            index,
            self.size
        );

        unsafe { self.uncached_item(index).write_volatile(value) };
    }

    pub fn push(&mut self, value: T) {
        assert!(
            self.size < self.capacity,
            "buffer capacity exceeded ({})",
            self.capacity
        );

        let old_size = self.size;
        self.size += 1;
        self.set(old_size, value);
    }

    pub fn pop(&mut self) -> T {
        assert!(!self.is_empty(), "buffer is empty");

        self.size -= 1;
        self.get(self.size)
    }

    pub fn clear(&mut self) {
        self.size = 0;
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
