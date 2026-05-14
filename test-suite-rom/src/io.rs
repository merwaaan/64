use arbitrary_int::prelude::*;

pub fn uncached_ptr(offset: u32) -> *mut u32 {
    (n64_specs::map::Segment::KSEG1 as u32 | offset) as *mut u32
}

pub fn read_uncached(offset: u32) -> u32 {
    unsafe { uncached_ptr(offset).read_volatile() }
}

pub fn write_uncached(offset: u32, value: u32) {
    unsafe { uncached_ptr(offset).write_volatile(value) }
}

pub fn wait_for_pi() {
    // TODO specs
    const PI_STATUS: *const u32 = 0xA460_0010 as *const u32;
    unsafe { while PI_STATUS.read_volatile() & 0x3 != 0 {} }
}

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

pub fn dma_ram_to_rsp(dma: &RspDma) {
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

pub fn wait_until(condition: impl Fn() -> bool) {
    loop {
        if condition() {
            break;
        }
    }
}
