pub enum Exception {
    Interrupt { cause: u8 },
    TlbModification,
    TlbMissLoad { virtual_address: u32 },
    TlbMissStore { virtual_address: u32 },
    TlbInvalidLoad { virtual_address: u32 },
    TlbInvalidStore { virtual_address: u32 },
    AddressLoad { address: u32 },
    AddressStore { address: u32 },
    Syscall,
    Breakpoint,
    ReservedInstruction,
    CoprocessorUnusable { coprocessor: u32 },
    ArithmeticOverflow,
    Trap,
    FloatingPoint,
}

impl Exception {
    pub fn exception_code(&self) -> u32 {
        match self {
            Exception::Interrupt { .. } => 0,
            Exception::TlbModification => 1,
            Exception::TlbMissLoad { .. } => 2,
            Exception::TlbMissStore { .. } => 3,
            Exception::TlbInvalidLoad { .. } => 2,
            Exception::TlbInvalidStore { .. } => 3,
            Exception::AddressLoad { .. } => 4,
            Exception::AddressStore { .. } => 5,
            Exception::Syscall => 8,
            Exception::Breakpoint => 9,
            Exception::ReservedInstruction => 10,
            Exception::CoprocessorUnusable { .. } => 11,
            Exception::ArithmeticOverflow => 12,
            Exception::Trap => 13,
            Exception::FloatingPoint => 15,
        }
    }
}
