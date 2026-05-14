#[derive(Debug, strum::Display, strum::EnumIter)]
#[repr(u32)]
pub enum Register {
    Index,
    Random,
    EntryLo0,
    EntryLo1,
    Context,
    PageMask,
    Wired,
    Unused7,
    BadVAddr,
    Count,
    EntryHi,
    Compare,
    Status,
    Cause,
    ExceptionPC,
    PRId,
    Config,
    LLAddr,
    WatchLo,
    WatchHi,
    XContext,
    Unused21,
    Unused22,
    Unused23,
    Unused24,
    Unused25,
    PErr,
    CacheErr,
    TagLo,
    TagHi,
    ErrorPC,
    Unused31,
}

impl Register {
    pub const fn index(&self) -> u32 {
        match self {
            _ => todo!(),
        }
    }
}
