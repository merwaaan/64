use crate::registers::Reg64;

pub const STATUS_IE_MASK: u32 = 1;
pub const STATUS_EXL_MASK: u32 = 1 << 1;
pub const STATUS_ERL_MASK: u32 = 1 << 2;

#[repr(u32)]
pub enum Register {
    Index,
    Random,
    EntryLo0,
    EntryLo1,
    Context,
    PageMask,
    Wired,
    Rsv7,
    BadVAddr,
    Count,
    EntryHi,
    Compare,
    Status,
    Cause,
    EPC,
    PRId,
    Config,
    LLAddr,
    WatchLo,
    WatchHi,
    XContext,
    Rsv21,
    Rsv22,
    Rsv23,
    Rsv24,
    Rsv25,
    PErr,
    CacheErr,
    TagLo,
    TagHi,
    ErrorEPC,
    Rsv31,
}

#[derive(Clone, Copy)]
pub struct Cop0 {
    regs: [Reg64; 32],
}

impl Default for Cop0 {
    fn default() -> Self {
        let mut regs = [Reg64::default(); 32];

        // Real-world startup values
        // https://n64.readthedocs.io/index.html#simulating-the-pif-rom

        regs[Register::Random as usize].set(0x1F);
        regs[Register::Status as usize].set(0x3400_0000);
        regs[Register::PRId as usize].set(0xB22);
        regs[Register::Config as usize].set(0x7006_E463);

        Self { regs }
    }
}

impl Cop0 {
    pub fn read(&self, reg: usize) -> Reg64 {
        self.regs[reg]
    }

    // WARNING: those writes are masked!

    pub(crate) fn write(&mut self, reg: usize, value: u32) {
        let mask = REG_WRITE_MASK[reg] as u32;
        self.regs[reg].set((self.regs[reg].get() & !mask) | (value & mask));
    }

    pub(crate) fn write64(&mut self, reg: usize, value: u64) {
        let mask = REG_WRITE_MASK[reg] as u64;
        self.regs[reg].set64((self.regs[reg].get64() & !mask) | (value & mask));
    }

    // BadVAddr register

    pub(crate) fn set_bad_address(&mut self, value: u32) {
        self.regs[Register::BadVAddr as usize].set(value);
    }

    // STATUS register

    pub fn ie(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_IE_MASK != 0
    }

    pub fn erl(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_ERL_MASK != 0
    }

    // TODO set???
    pub(crate) fn clear_erl(&mut self) {
        self.regs[Register::Status as usize]
            .set(self.regs[Register::Status as usize].get() & !STATUS_ERL_MASK);
    }

    pub(crate) fn exl(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_EXL_MASK != 0
    }

    pub(crate) fn set_exl(&mut self) {
        self.regs[Register::Status as usize]
            .set(self.regs[Register::Status as usize].get() | STATUS_EXL_MASK);
    }

    pub(crate) fn clear_exl(&mut self) {
        self.regs[Register::Status as usize]
            .set(self.regs[Register::Status as usize].get() & !STATUS_EXL_MASK);
    }

    pub(crate) fn interrupt_mask(&self) -> u32 {
        (self.regs[Register::Status as usize].get() >> 8) & 0xFF
    }

    pub(crate) fn cop1_usable(&self) -> bool {
        self.regs[Register::Status as usize].get() & 0x2000_0000 != 0
    }

    pub(crate) fn cop2_usable(&self) -> bool {
        self.regs[Register::Status as usize].get() & 0x4000_0000 != 0
    }

    // CAUSE register

    pub(crate) fn set_exception_code(&mut self, value: u32) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x7C) | (value << 2));
    }

    pub(crate) fn set_coprocessor_error(&mut self, cop: u32) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x3000_0000) | ((cop & 3) << 28));
    }

    pub(crate) fn exception_in_branch_delay_slot(&self) -> bool {
        self.regs[Register::Cause as usize].get() & 0x8000_0000 != 0
    }

    pub(crate) fn set_exception_in_branch_delay_slot(&mut self, value: bool) {
        self.regs[Register::Cause as usize].set(
            (self.regs[Register::Cause as usize].get() & 0x7FFF_FFFF) | ((value as u32) << 31),
        );
    }

    pub(crate) fn interrupt_pending(&self) -> u32 {
        (self.regs[Register::Cause as usize].get() >> 8) & 0xFF
    }

    pub(crate) fn set_ip2_interrupt(&mut self, value: bool) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x400) | ((value as u32) << 10));
    }

    // EPC register

    pub(crate) fn epc(&self) -> u32 {
        self.regs[Register::EPC as usize].get() // TODO 64/32?
    }

    pub(crate) fn set_epc(&mut self, value: u32) {
        self.regs[Register::EPC as usize].set(value);
    }

    // ErrorEPC register

    pub(crate) fn error_epc(&self) -> u32 {
        self.regs[Register::ErrorEPC as usize].get() // TODO 64/32?
    }

    pub(crate) fn set_error_epc(&mut self, value: u32) {
        self.regs[Register::ErrorEPC as usize].set(value);
    }

    //

    pub(crate) fn f_64(&self) -> bool {
        self.regs[Register::Status as usize].get() & 0x4000_0000 != 0
    }

    // TODO just to_string enum?
    pub fn reg_name(index: usize) -> &'static str {
        const NAMES: [&str; 32] = [
            "Index", "Random", "EntryLo0", "EntryLo1", "Context", "PageMask", "Wired", "Rsv7",
            "BadVAddr", "Count", "EntryHi", "Compare", "Status", "Cause", "EPC", "PRId", "Config",
            "LLAddr", "WatchLo", "WatchHi", "XContext", "Rsv21", "Rsv22", "Rsv23", "Rsv24",
            "Rsv25", "PErr", "CacheErr", "TagLo", "TagHi", "ErrorEPC", "Rsv31",
        ];

        NAMES.get(index).copied().unwrap_or("?") // TODO copied?
    }
}

const WRITABLE: u64 = 0xFFFFFFFF_FFFFFFFF;
const READ_ONLY: u64 = 0;

const REG_WRITE_MASK: [u64; 32] = [
    WRITABLE,
    READ_ONLY, // Random
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    READ_ONLY, // BadVAddr
    WRITABLE,
    WRITABLE,
    WRITABLE,
    0xFFFFFFFF_FFF7FFFF, // Status: bit 19 is read-only
    0x00000000_00000300, // Cause: only bits for IP0-1 are writable
    WRITABLE,
    READ_ONLY, // PrId
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
    WRITABLE,
];
