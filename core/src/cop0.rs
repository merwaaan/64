use crate::{registers::Reg64, tlb::Tlb};

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
    EPC, // TODO rename Expect(ion)PC?
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
    ErrorEPC, // TODO rename ErrorPC?
    Rsv31,
}

const WRITABLE: u64 = 0xFFFFFFFF_FFFFFFFF;
const READ_ONLY: u64 = 0;

const REG_WRITE_MASK: [u64; 32] = [
    0x00000000_8000003F, // Index
    READ_ONLY,           // Random
    0x00000000_3FFFFFFF, // EntryLo0
    0x00000000_3FFFFFFF, // EntryLo1
    0xFFFFFFFF_FF800000, // Context
    0xFFFFFFFF_01FFE000, // PageMask
    0x00000000_0000003F, // Wired
    WRITABLE,            // Rsv7TODO?
    READ_ONLY,           // BadVAddr
    WRITABLE,            // Count
    0xC00000FF_FFFFE0FF, // EntryHi
    WRITABLE,            // Compare
    0xFFFFFFFF_FFF7FFFF, // Status: bit 19 is read-only TODO really?
    0x00000000_00000300, // Cause
    WRITABLE,            // EPC
    READ_ONLY,           // PrId
    0x00000000_00000003, // Config
    0x00000000_FFFFFFFF, // LLAddr
    WRITABLE,            // WatchLo
    WRITABLE,            // WatchHi
    0xFFFFFFFE_00000000, // XContext
    WRITABLE,            // Rsv21
    WRITABLE,            // Rsv22
    WRITABLE,            // Rsv23
    WRITABLE,            // Rsv24
    WRITABLE,            // Rsv25
    0x00000000_000000FF, // PErr
    READ_ONLY,           // CacheErr
    WRITABLE,            // TagLo
    WRITABLE,            // TagHi
    WRITABLE,            // ErrorEPC
    WRITABLE,            // Rsv31
];

#[derive(Clone, Copy, Debug)]
pub struct Cop0 {
    regs: [Reg64; 32],

    pub tlb: Tlb, // TODO visibility?
}

// TODO bitfields or something?

impl Default for Cop0 {
    fn default() -> Self {
        let mut regs = [Reg64::default(); 32];

        // Real-world startup values
        // https://n64.readthedocs.io/index.html#simulating-the-pif-rom

        regs[Register::Random as usize].set(0x1F);
        regs[Register::Status as usize].set(0x3400_0000);
        regs[Register::PRId as usize].set(0xB22);
        regs[Register::Config as usize].set(0x7006_E463);

        Self {
            regs,
            tlb: Tlb::default(),
        }
    }
}

impl Cop0 {
    pub fn read(&self, reg: usize) -> Reg64 {
        self.regs[reg]
    }

    // WARNING: those writes are masked!

    pub fn write(&mut self, reg: usize, value: u32) {
        let mask = REG_WRITE_MASK[reg] as u32;

        self.regs[reg].set((self.regs[reg].get() & !mask) | (value & mask));

        match reg {
            // Count
            9 => {
                self.update_cause_register();
            }
            // Compare
            11 => {
                self.set_ip7_interrupt(false);
                self.update_cause_register();
            }
            _ => {}
        }
    }

    // TODO use a single implem with <T: Value>?
    pub fn write64(&mut self, reg: usize, value: u64) {
        let mask = REG_WRITE_MASK[reg];

        self.regs[reg].set64((self.regs[reg].get64() & !mask) | (value & mask));

        match reg {
            // Count
            9 => {
                self.update_cause_register();
            }
            // Compare
            11 => {
                self.set_ip7_interrupt(false);
                self.update_cause_register();
            }
            _ => {}
        }
    }

    pub fn increment_timer(&mut self) {
        self.regs[Register::Count as usize]
            .set(self.regs[Register::Count as usize].get().wrapping_add(1));

        self.update_cause_register();
    }

    /// Updates the CAUSE register, must be called when COUNT or COMPARE change
    fn update_cause_register(&mut self) {
        if self.regs[Register::Count as usize].get() == self.regs[Register::Compare as usize].get()
        {
            self.set_ip7_interrupt(true);
        }
    }

    // BadVAddr register

    pub fn set_bad_virtual_address(&mut self, value: u32) {
        self.regs[Register::BadVAddr as usize].set(value);
    }

    // STATUS register

    pub fn set_status(&mut self, value: u32) {
        self.regs[Register::Status as usize].set(value); // TODO write?
    }

    pub fn ie(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_IE_MASK != 0
    }

    pub fn erl(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_ERL_MASK != 0
    }

    // TODO set???
    pub fn clear_erl(&mut self) {
        self.regs[Register::Status as usize]
            .set(self.regs[Register::Status as usize].get() & !STATUS_ERL_MASK);
    }

    pub fn exl(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_EXL_MASK != 0
    }

    pub fn set_exl(&mut self) {
        self.regs[Register::Status as usize]
            .set(self.regs[Register::Status as usize].get() | STATUS_EXL_MASK);
    }

    pub fn clear_exl(&mut self) {
        self.regs[Register::Status as usize]
            .set(self.regs[Register::Status as usize].get() & !STATUS_EXL_MASK);
    }

    pub fn interrupt_mask(&self) -> u8 {
        ((self.regs[Register::Status as usize].get() >> 8) & 0xFF) as u8
    }

    pub fn cop1_usable(&self) -> bool {
        self.regs[Register::Status as usize].get() & 0x2000_0000 != 0
    }

    pub fn cop2_usable(&self) -> bool {
        self.regs[Register::Status as usize].get() & 0x4000_0000 != 0
    }

    // CAUSE register

    pub fn set_exception_code(&mut self, value: u32) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x7C) | ((value & 0x1F) << 2));
    }

    pub fn set_coprocessor_error(&mut self, cop: u32) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x3000_0000) | ((cop & 3) << 28));
    }

    pub fn exception_in_branch_delay_slot(&self) -> bool {
        self.regs[Register::Cause as usize].get() & 0x8000_0000 != 0
    }

    pub fn set_exception_in_branch_delay_slot(&mut self, value: bool) {
        self.regs[Register::Cause as usize].set(
            (self.regs[Register::Cause as usize].get() & 0x7FFF_FFFF) | ((value as u32) << 31),
        );
    }

    pub fn interrupt_pending(&self) -> u8 {
        ((self.regs[Register::Cause as usize].get() >> 8) & 0xFF) as u8
    }

    pub fn set_ip2_interrupt(&mut self, value: bool) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x400) | ((value as u32) << 10));
    }

    fn set_ip7_interrupt(&mut self, value: bool) {
        self.regs[Register::Cause as usize]
            .set((self.regs[Register::Cause as usize].get() & !0x8000) | ((value as u32) << 15));
    }

    // EPC register

    pub fn exception_pc(&self) -> u32 {
        self.regs[Register::EPC as usize].get() // TODO 64/32?
    }

    pub fn set_exception_pc(&mut self, value: u32) {
        self.regs[Register::EPC as usize].set(value);
    }

    // ErrorEPC register

    pub fn error_pc(&self) -> u32 {
        self.regs[Register::ErrorEPC as usize].get() // TODO 64/32?
    }

    pub fn set_error_pc(&mut self, value: u32) {
        self.regs[Register::ErrorEPC as usize].set(value);
    }

    pub fn f64(&self) -> bool {
        self.regs[Register::Status as usize].get() & 0x0400_0000 != 0
    }

    // LLAddr register

    pub fn set_ll_addr(&mut self, value: u32) {
        self.regs[Register::LLAddr as usize].set(value >> 4);
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
