use crate::system::System;

#[derive(Debug, Clone, Copy)]
pub enum Exception {
    Interrupt,
    AddressErrorLoad(u32),
    AddressErrorStore(u32),
    CoprocessorUnusable(u32),
    ArithmeticOverflow,
    Trap,
}

impl Exception {
    pub fn raise(&self, s: &mut System) {
        log::warn!("Exception {:?} at {:08X}", self, s.cpu.regs.pc);
        let in_branch_delay = s.cpu.delayed_branching.is_some();

        // Set EXL to prevent nested exceptions

        if !s.cop0.exl() {
            s.cop0.set_exl();

            // Set EPC
            //
            //  Not in a branch delay slot: use current PC
            //  In a branch delay slot: use the PC of the delayed branch instruction (-4)

            let branch_delay_offset = (in_branch_delay as u32) << 2;

            let epc = s.cpu.regs.pc.wrapping_sub(branch_delay_offset);
            log::error!("EPC @ {:08X} (offset: {:08X})", epc, branch_delay_offset);
            s.cop0.set_epc(epc);
        }

        // TODO ERL???

        // Update the CAUSE register

        s.cop0.set_exception_code(self.exception_code());
        s.cop0.set_exception_in_branch_delay_slot(in_branch_delay);

        // TODO temp to please lemmy, should this happen?
        s.cop0.set_coprocessor_error(0);

        match self {
            Exception::AddressErrorLoad(address) => {
                s.cop0.set_bad_address(*address);
            }
            Exception::AddressErrorStore(address) => {
                s.cop0.set_bad_address(*address);
            }
            Exception::CoprocessorUnusable(cop) => {
                s.cop0.set_coprocessor_error(*cop);
            }
            _ => {}
        }

        // Jump to the exception handler

        s.cpu.regs.pc = 0x8000_0180; // TODO others?
    }

    pub fn exception_code(&self) -> u32 {
        match self {
            Exception::Interrupt => 0,
            Exception::AddressErrorLoad(_) => 4,
            Exception::AddressErrorStore(_) => 5,
            Exception::CoprocessorUnusable(_) => 11,
            Exception::ArithmeticOverflow => 12,
            Exception::Trap => 13,
        }
    }
}
