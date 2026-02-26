use super::{Disassembly, Instruction, InstructionResult, Opcode, System};
use crate::{instruction_struct, instructions::UNKNOWN_};

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x10);

    let instruction: &'static dyn Instruction = match opcode.0 & 0x03E0_0000 {
        0x000_0000 => &MFC0_,
        0x020_0000 => &DMFC0_,
        0x080_0000 => &MTC0_,
        0x0A0_0000 => &DMTC0_,
        // C0 sub-group
        0x200_0000 => match opcode.0 & 0x3F {
            0x01 => &TLBR_,
            0x02 => &TLBWI_,
            0x08 => &TLBP_,
            0x18 => &ERET_,
            _ => &UNKNOWN_,
        },
        _ => &UNKNOWN_,
    };

    if std::ptr::eq(instruction, &UNKNOWN_) {
        None
    } else {
        Some(instruction)
    }
}

instruction_struct!(DMFC0);

impl Instruction for DMFC0 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        s.cpu.regs.gpr[op.rt()].set64(s.cop0.regs[op.rd()].get64());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMFC0 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(DMTC0);

impl Instruction for DMTC0 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        s.cop0.regs[op.rd()].set64(s.cpu.regs.gpr[op.rt()].get64());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMTC0 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(ERET);

impl Instruction for ERET {
    fn execute(&self, s: &mut System, _op: Opcode) -> Option<InstructionResult> {
        if s.cop0.erl() {
            unimplemented!("ERET in ERL mode");
            // s.cpu.regs.pc = s.cop0.error_epc() - 4; // TODO offset???
            // s.cop0.clear_erl();
        } else {
            s.cpu.regs.pc = s.cop0.epc().wrapping_sub(4); // TODO return specific value to signal that instead of adjusting PC?
            s.cop0.clear_exl();
        }

        None
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("ERET".to_string())
    }
}

instruction_struct!(MTC0);

impl Instruction for MTC0 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        let data = s.cpu.regs.gpr[op.rt()].get64();

        // TODO cause: only two last bits can be written! move to reg implem ???
        // TODO not b0-1 but 8-9???? 0x0000_0300
        // if op.rd() == 13 {
        //     data = (data & 3) | (s.cop0.regs[13].get64() & 0xFFFF_FFFF_FFFF_FFFC);
        // }

        //log::info!("MTC0 {}, {:08X}", op.rd0n(), data);

        s.cop0.regs[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTC0 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(MFC0);

impl Instruction for MFC0 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        s.cpu.regs.gpr[op.rt()].set(s.cop0.regs[op.rd()].get());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC0 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(TLBP);

impl Instruction for TLBP {
    fn execute(&self, s: &mut System, _op: Opcode) -> Option<InstructionResult> {
        log::warn!("TLBP @ {:08X}", s.cpu.regs.pc);

        None
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("TLBP".to_string())
    }
}

instruction_struct!(TLBR);

impl Instruction for TLBR {
    fn execute(&self, s: &mut System, _op: Opcode) -> Option<InstructionResult> {
        log::warn!(
            "TLBR @ {:08X} (index={})",
            s.cpu.regs.pc,
            s.cop0.regs[0].get()
        );

        None
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("TLBR".to_string())
    }
}

instruction_struct!(TLBWI);

impl Instruction for TLBWI {
    fn execute(&self, s: &mut System, _op: Opcode) -> Option<InstructionResult> {
        log::warn!(
            "TLBWI @ {:08X} (index={})",
            s.cpu.regs.pc,
            s.cop0.regs[0].get()
        );

        None
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("TLBWI".to_string())
    }
}
