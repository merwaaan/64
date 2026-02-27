use crate::{cop0, system::System};

#[derive(Debug, Clone, Copy)]
pub enum Exception {
    Interrupt,
    AddressLoad(u32),
    AddressStore(u32),
    CoprocessorUnusable(u32),
    ArithmeticOverflow,
    Trap,
}

impl Exception {
    pub fn raise(&self, s: &mut System) {
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

            s.cop0.set_epc(epc);
        }

        // TODO ERL???

        // Update the CAUSE register

        s.cop0.set_exception_code(self.exception_code());
        s.cop0.set_exception_in_branch_delay_slot(in_branch_delay);

        // TODO temp to please lemmy, should this happen?
        s.cop0.set_coprocessor_error(0);

        match self {
            Exception::AddressLoad(address) => {
                s.cop0.set_bad_address(*address);
            }
            Exception::AddressStore(address) => {
                s.cop0.set_bad_address(*address);
            }
            Exception::CoprocessorUnusable(cop) => {
                s.cop0.set_coprocessor_error(*cop);
            }
            _ => {}
        }

        // Jump to the exception handler

        s.cpu.regs.pc = 0x8000_0180;

        // TODO others?
        // TLB ex 32/64 bits cases
        // BEV

        if s.cop0.read(cop0::Register::Status as usize).get() & 0x0040_0000 != 0 {
            panic!("BEV not supported");
        }
    }

    pub fn exception_code(&self) -> u32 {
        match self {
            Exception::Interrupt => 0,
            Exception::AddressLoad(_) => 4,
            Exception::AddressStore(_) => 5,
            Exception::CoprocessorUnusable(_) => 11,
            Exception::ArithmeticOverflow => 12,
            Exception::Trap => 13,
        }
    }
}
