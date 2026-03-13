use crate::{cop0, system::System};

#[derive(Debug, Clone, Copy)]
pub enum Exception {
    Interrupt(u8), // u8 = interrupt bits
    TlbModification,
    TlbMissLoad,
    TlbMissStore,
    TlbInvalidLoad,
    TlbInvalidStore,
    AddressLoad(u32),
    AddressStore(u32),
    Syscall,
    Breakpoint,
    // TODO ReservedInstruction
    CoprocessorUnusable(u32),
    ArithmeticOverflow,
    Trap,
}

impl Exception {
    pub fn raise(&self, s: &mut System) {
        // if !matches!(self, Exception::Interrupt(8)) && !matches!(self, Exception::Interrupt(4)) {
        //     log::warn!("Exception: {:?}", self);
        // }

        let in_branch_delay = s.cpu.in_branch_delay_slot();

        // Set EXL to prevent nested exceptions

        if !s.cop0.exl() {
            s.cop0.set_exl();

            // Set EPC
            //
            //  Not in a branch delay slot: use current PC
            //  In a branch delay slot: use the PC of the delayed branch instruction (-4)

            let branch_delay_offset = (in_branch_delay as u32) << 2;

            let epc = s.cpu.regs.pc.wrapping_sub(branch_delay_offset);

            s.cop0.set_exception_pc(epc);
        }

        // TODO ERL???

        // Update the CAUSE register

        s.cop0.set_exception_code(self.exception_code());
        s.cop0.set_exception_in_branch_delay_slot(in_branch_delay);

        // TODO temp to please lemmy, should this happen?
        s.cop0.set_coprocessor_error(0);

        match self {
            Exception::AddressLoad(address) => {
                s.cop0.set_bad_virtual_address(*address);
            }
            Exception::AddressStore(address) => {
                s.cop0.set_bad_virtual_address(*address);
            }
            Exception::CoprocessorUnusable(cop) => {
                s.cop0.set_coprocessor_error(*cop);
            }
            _ => {}
        }

        // Jump to the exception handler

        s.cpu.regs.pc = match self {
            Exception::TlbMissLoad | Exception::TlbMissStore => 0x8000_0080,
            _ => 0x8000_0180,
        };

        s.cpu.regs.load_linked_bit = false; // TODO not documented anywhere???

        // TODO others?
        // TLB ex 32/64 bits cases
        // BEV

        if s.cop0.read(cop0::Register::Status as usize).get() & 0x0040_0000 != 0 {
            panic!("BEV not supported");
        }
    }

    pub fn exception_code(&self) -> u32 {
        match self {
            Exception::Interrupt(_) => 0,
            Exception::TlbModification => 1,
            Exception::TlbMissLoad => 2,
            Exception::TlbMissStore => 3,
            Exception::TlbInvalidLoad => 2,
            Exception::TlbInvalidStore => 3,
            Exception::AddressLoad(_) => 4,
            Exception::AddressStore(_) => 5,
            Exception::Syscall => 8,
            Exception::Breakpoint => 9,
            Exception::CoprocessorUnusable(_) => 11,
            Exception::ArithmeticOverflow => 12,
            Exception::Trap => 13,
        }
    }

    /// Raises pending interrupts ready to be serviced
    pub fn check_interrupts(s: &mut System) -> bool {
        // We can only raise interrupts if:
        // - Interrupts are globally enabled
        // - We are not currently handling an exception
        // - We are not currently handling an error exception

        if s.cop0.ie() && !s.cop0.exl() && !s.cop0.erl() {
            // Combine the interrupt mask (Cause register) and the pending bits (Status register).
            // if any pending interrupt is unmasked, raise it.
            //
            // Common interrupts sources:
            // - Software interrupts (set by programs via MTC0)
            // - MI (should have set ip2 when its internal pending interrupts are unmasked)
            // - Timer aka Count/Compare (should have set ip7 when the timer ends)

            // TODO document other interrupt bits?

            let mask = s.cop0.interrupt_mask();
            let pending = s.cop0.interrupt_pending();

            let interrupts = mask & pending;

            if interrupts != 0 {
                Exception::Interrupt(interrupts).raise(s);

                return true;
            }
        }

        false
    }
}

/// Checks address alignment.
/// Returns an exception if unaligned, convenient for instruction implementations.
#[macro_export]
macro_rules! check_aligned {
    (load, $addr:expr, $mask:expr) => {
        if ($addr & $mask) != 0 {
            return Err(Exception::AddressLoad($addr));
        }
    };
    (store, $addr:expr, $mask:expr) => {
        if ($addr & $mask) != 0 {
            return Err(Exception::AddressStore($addr));
        }
    };
}

/// Checks coprocessor usability.
/// Returns an exception if unusable, convenient for instruction implementations.
#[macro_export]
macro_rules! check_cop_usable {
    ($cop:literal, $s:expr) => {
        paste::paste! {
            if !$s.cop0.[<cop $cop _usable>]() {
                return Err(Exception::CoprocessorUnusable($cop));
            }
        }
    };
}
