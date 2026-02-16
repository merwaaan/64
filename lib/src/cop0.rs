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

#[derive(Default)]
pub struct Cop0 {
    pub regs: [Reg64; 32],
}

impl Cop0 {
    pub fn ie(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_IE_MASK != 0
    }

    pub fn erl(&self) -> bool {
        self.regs[Register::Status as usize].get() & STATUS_ERL_MASK != 0
    }

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

    pub fn epc(&self) -> u32 {
        self.regs[Register::EPC as usize].get() // TODO 64/32?
    }

    pub fn error_epc(&self) -> u32 {
        self.regs[Register::ErrorEPC as usize].get() // TODO 64/32?
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
