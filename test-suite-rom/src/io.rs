use arbitrary_int::prelude::*;

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
