#[repr(u32)]
pub enum Segment {
    /// User segment, TLB mapped
    KUSEG = 0,
    /// Kernel segment 0, directly mapped, cached
    KSEG0 = 0x8000_0000,
    /// Kernel segment 0, directly mapped, uncached
    KSEG1 = 0xA000_0000,
    /// Kernel supervisor segment 2, TLB mapped
    KSEG2 = 0xC000_0000,
    /// Kernel segment 3, TLB mapped
    KSEG3 = 0xE000_0000,
}
