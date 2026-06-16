use crate::{
    check_aligned, check_cop_usable,
    cpu::{
        instructions::{Instruction, InstructionEffect, InstructionResult, branch},
        opcode::Opcode,
        operands::Operands,
    },
    exception::Exception,
    system::{Address, System},
};

// TODO clean up impl, some are not used

#[macro_export]
macro_rules! decode_standard_x {
    ($opcode:expr, $m:ident) => {{
        match $opcode.group() {
            0x02 => $m!(crate::cpu::instructions::standard::J),
            0x03 => $m!(crate::cpu::instructions::standard::Jal),
            0x04 => $m!(crate::cpu::instructions::standard::Beq),
            0x05 => $m!(crate::cpu::instructions::standard::Bne),
            0x06 => $m!(crate::cpu::instructions::standard::Blez),
            0x07 => $m!(crate::cpu::instructions::standard::Bgtz),
            0x08 => $m!(crate::cpu::instructions::standard::Addi),
            0x09 => $m!(crate::cpu::instructions::standard::Addiu),
            0x0A => $m!(crate::cpu::instructions::standard::Slti),
            0x0B => $m!(crate::cpu::instructions::standard::Sltiu),
            0x0C => $m!(crate::cpu::instructions::standard::Andi),
            0x0D => $m!(crate::cpu::instructions::standard::Ori),
            0x0E => $m!(crate::cpu::instructions::standard::Xori),
            0x0F => $m!(crate::cpu::instructions::standard::Lui),
            0x14 => $m!(crate::cpu::instructions::standard::Beql),
            0x15 => $m!(crate::cpu::instructions::standard::Bnel),
            0x16 => $m!(crate::cpu::instructions::standard::Blezl),
            0x17 => $m!(crate::cpu::instructions::standard::Bgtzl),
            0x18 => $m!(crate::cpu::instructions::standard::Daddi),
            0x19 => $m!(crate::cpu::instructions::standard::Daddiu),
            0x1A => $m!(crate::cpu::instructions::standard::Ldl),
            0x1B => $m!(crate::cpu::instructions::standard::Ldr),
            0x20 => $m!(crate::cpu::instructions::standard::Lb),
            0x21 => $m!(crate::cpu::instructions::standard::Lh),
            0x22 => $m!(crate::cpu::instructions::standard::Lwl),
            0x23 => $m!(crate::cpu::instructions::standard::Lw),
            0x24 => $m!(crate::cpu::instructions::standard::Lbu),
            0x25 => $m!(crate::cpu::instructions::standard::Lhu),
            0x26 => $m!(crate::cpu::instructions::standard::Lwr),
            0x27 => $m!(crate::cpu::instructions::standard::Lwu),
            0x28 => $m!(crate::cpu::instructions::standard::Sb),
            0x29 => $m!(crate::cpu::instructions::standard::Sh),
            0x2A => $m!(crate::cpu::instructions::standard::Swl),
            0x2B => $m!(crate::cpu::instructions::standard::Sw),
            0x2C => $m!(crate::cpu::instructions::standard::Sdl),
            0x2D => $m!(crate::cpu::instructions::standard::Sdr),
            0x2E => $m!(crate::cpu::instructions::standard::Swr),
            0x2F => $m!(crate::cpu::instructions::standard::Cache),
            0x30 => $m!(crate::cpu::instructions::standard::Ll),
            0x31 => $m!(crate::cpu::instructions::standard::Lwc1),
            0x34 => $m!(crate::cpu::instructions::standard::Lld),
            0x35 => $m!(crate::cpu::instructions::standard::Ldc1),
            // TODO ldc2, swc2, etc?? or cop2 group???
            0x37 => $m!(crate::cpu::instructions::standard::Ld),
            0x38 => $m!(crate::cpu::instructions::standard::Sc),
            0x39 => $m!(crate::cpu::instructions::standard::Swc1),
            0x3C => $m!(crate::cpu::instructions::standard::Scd),
            0x3D => $m!(crate::cpu::instructions::standard::Sdc1),
            0x3F => $m!(crate::cpu::instructions::standard::Sd),
            _ => $m!(crate::cpu::instructions::Reserved),
        }
    }};
}

// -----------
// Arithmetics
// -----------

pub struct Addi;

impl Instruction for Addi {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv(s) as i32;
        let imm = opcode.imm16() as i16 as i32;

        match rs.checked_add(imm) {
            Some(result) => {
                s.cpu.regs.gpr[operands.rt()].set(result as u32);
                Ok(None)
            }
            None => Err(Exception::ArithmeticOverflow),
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "ADDI {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Addiu;

impl Instruction for Addiu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let imm = (opcode.imm16() as i16 as i32) as u32;

        s.cpu.regs.gpr[operands.rt()].set(operands.rsv(s).wrapping_add(imm));

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "ADDIU {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Andi;

impl Instruction for Andi {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()].set64(operands.rsv64(s) & (opcode.imm16() as u64));

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "ANDI {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

// ---------
// Branching
// ---------

pub struct Beq;

impl Instruction for Beq {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<false>(s, opcode, operands.rsv64(s) == operands.rtv64(s))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "BEQ {}, {}, {:#06X}",
            operands.rsn(),
            operands.rtn(),
            opcode.branch_offset()
        )
    }
}

pub struct Beql;

impl Instruction for Beql {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, operands.rsv64(s) == operands.rtv64(s))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "BEQL {}, {}, {:#06X}",
            operands.rsn(),
            operands.rtn(),
            opcode.branch_offset()
        )
    }
}

pub struct Bgtz;

impl Instruction for Bgtz {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<false>(s, opcode, (operands.rsv64(s) as i64) > 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BGTZ {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bgtzl;

impl Instruction for Bgtzl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, (operands.rsv64(s) as i64) > 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BGTZL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Blez;

impl Instruction for Blez {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<false>(s, opcode, (operands.rsv64(s) as i64) <= 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BLEZ {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Blezl;

impl Instruction for Blezl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, (operands.rsv64(s) as i64) <= 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BLEZL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bltzl;

impl Instruction for Bltzl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, (operands.rsv64(s) as i64) < 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BLTZL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bne;

impl Instruction for Bne {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<false>(s, opcode, operands.rsv64(s) != operands.rtv64(s))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "BNE {}, {}, {:#X}",
            operands.rsn(),
            operands.rtn(),
            opcode.branch_offset()
        )
    }
}

pub struct Bnel;

impl Instruction for Bnel {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, operands.rsv64(s) != operands.rtv64(s))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "BNEL {}, {}, {:#X}",
            operands.rsn(),
            operands.rtn(),
            opcode.branch_offset()
        )
    }
}

pub struct Cache;

impl Instruction for Cache {
    fn execute(_s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        // According to the MIPS manual, the CACHE instruction causes a reserved instruction exception if COP0 is disabled.
        // However, on the N64, COP0 cannot be disabled.

        //TODO log::debug!("CACHE {:08X}", op.0);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "CACHE {}, {}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Daddi;

impl Instruction for Daddi {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv64(s) as i64;
        let imm = opcode.imm16() as i16 as i64;

        match rs.checked_add(imm) {
            Some(result) => {
                s.cpu.regs.gpr[operands.rt()].set64(result as u64);
                Ok(None)
            }
            None => Err(Exception::ArithmeticOverflow),
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "DADDI {}, {}, {}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Daddiu;

impl Instruction for Daddiu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let res = operands
            .rsv64(s)
            .wrapping_add(opcode.imm16() as i16 as i64 as u64);

        s.cpu.regs.gpr[operands.rt()].set64(res);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "DADDIU {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

// -----
// Jumps
// -----

fn j_target(pc: u32, op: Opcode) -> u32 {
    let hi = pc.wrapping_add(4) & 0xF000_0000;
    let lo = (op.0 & 0x03FF_FFFF) << 2;
    hi | lo
}

pub struct J;

impl Instruction for J {
    fn execute(s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        Ok(Some(InstructionEffect::DelayedBranching(Some(j_target(
            s.cpu.regs.pc,
            opcode,
        )))))
    }

    fn disassemble(s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("J {:#06X}", j_target(s.cpu.regs.pc, opcode))
    }
}

pub struct Jal;

impl Instruction for Jal {
    fn execute(s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        Ok(Some(InstructionEffect::DelayedBranching(Some(j_target(
            s.cpu.regs.pc,
            opcode,
        )))))
    }

    fn disassemble(s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("JAL {:#06X}", j_target(s.cpu.regs.pc, opcode))
    }
}

// ----------
// Load/Store
// ----------

pub struct Lb;

impl Instruction for Lb {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let data = s.read::<u8>(Address::v(addr))? as i8 as i32 as u32;
        s.cpu.regs.gpr[operands.rt()].set(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LB {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lbu;

impl Instruction for Lbu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let data = s.read::<u8>(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set(data as u32);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LBU {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Ld;

impl Instruction for Ld {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 7);

        let data = s.read(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LD {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Ldc1;

impl Instruction for Ldc1 {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 7);

        let data = s.read(Address::v(addr))?;
        s.cop1.set64(operands.ft(), data, s.cop0.f64());

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LDC1 {}, {}({})",
            operands.ftn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Ldl;

impl Instruction for Ldl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let addr_base = addr & !7;
        let addr_offset = addr & 7;

        let mut data = s.read(Address::v(addr_base))?;

        if addr_offset != 0 {
            data <<= addr_offset * 8;
            data |= operands.rtv64(s) & !(u64::MAX << (8 * addr_offset));
        }

        s.cpu.regs.gpr[operands.rt()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LDL {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Ldr;

impl Instruction for Ldr {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let base = addr & !7;
        let offset = addr & 7;

        let mut data = s.read(Address::v(base))?;

        if offset != 7 {
            data >>= (7 - offset) * 8;
            data |= operands.rtv64(s) & (u64::MAX << (8 * (offset + 1)));
        }

        s.cpu.regs.gpr[operands.rt()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LDR {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lh;

impl Instruction for Lh {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 1);

        let data = s.read::<u16>(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set(data as i16 as i32 as u32);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LH {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lhu;

impl Instruction for Lhu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 1);

        let data = s.read::<u16>(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set(data as u32);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LHU {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Ll;

impl Instruction for Ll {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 3);

        s.cop0.set_ll_addr(addr);
        s.cpu.regs.load_linked_bit = true;

        let data = s.read(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LL {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lld;

impl Instruction for Lld {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 7);

        s.cop0.set_ll_addr(addr);
        s.cpu.regs.load_linked_bit = true;

        let data = s.read(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LDD {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lui;

impl Instruction for Lui {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()].set((opcode.imm16() as u32) << 16);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("LUI {}, {:#04X}", operands.rtn(), opcode.imm16())
    }
}

pub struct Lw;

impl Instruction for Lw {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 3);

        let data = s.read(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set(data);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LW {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lwc1;

impl Instruction for Lwc1 {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 3);

        let data = s.read(Address::v(addr))?;
        s.cop1.set32(operands.rt(), data, s.cop0.f64());

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LWC1 {}, {}({})",
            operands.ftn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Lwl;

impl Instruction for Lwl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let addr_base = addr & !3;
        let addr_offset = addr & 3;

        let data = s.read(Address::v(addr_base))?;

        let word = if addr_offset == 0 {
            data
        } else {
            let mut word = s.cpu.regs.gpr[operands.rt()].get();
            word &= 0xFFFF_FFFF >> (32 - 8 * addr_offset);
            word |= data << (8 * addr_offset);
            word
        };

        s.cpu.regs.gpr[operands.rt()].set(word);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LWL {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

// TODO move partial shift stuff to helpers!

pub struct Lwr;

impl Instruction for Lwr {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let addr_base = addr & !3;
        let addr_offset = addr & 3;

        let data = s.read(Address::v(addr_base))?;

        let word = if addr_offset == 3 {
            data
        } else {
            let mut word = s.cpu.regs.gpr[operands.rt()].get();
            word &= !(0xFFFF_FFFF >> (24 - 8 * addr_offset));
            word |= data >> (24 - 8 * addr_offset);
            word
        };

        s.cpu.regs.gpr[operands.rt()].set(word);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LWR {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Lwu;

impl Instruction for Lwu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(load, addr, 3);

        let data = s.read::<u32>(Address::v(addr))?;
        s.cpu.regs.gpr[operands.rt()].set64(data as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "LWU {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Ori;

impl Instruction for Ori {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()].set64(operands.rsv64(s) | opcode.imm16() as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "ORI {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Xori;

impl Instruction for Xori {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()].set64(operands.rsv64(s) ^ opcode.imm16() as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "XORI {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Sb;

impl Instruction for Sb {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        s.write(Address::v(opcode.offset_addr(s)), operands.rtv(s) as u8)?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SB {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Sc;

impl Instruction for Sc {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 3);

        s.cpu.regs.gpr[operands.rt()].set(s.cpu.regs.load_linked_bit as u32);

        if s.cpu.regs.load_linked_bit {
            s.write(Address::v(addr), operands.rtv(s))?;
        }

        s.cpu.regs.load_linked_bit = false;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SC {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Scd;

impl Instruction for Scd {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 3);

        s.cpu.regs.gpr[operands.rt()].set64(s.cpu.regs.load_linked_bit as u64);

        if s.cpu.regs.load_linked_bit {
            s.write(Address::v(addr), operands.rtv64(s))?;
        }

        s.cpu.regs.load_linked_bit = false;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SCD {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Sd;

impl Instruction for Sd {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 7);

        s.write(Address::v(addr), s.cpu.regs.gpr[operands.rt()].get64())?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SD {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Sdc1;

impl Instruction for Sdc1 {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 7);

        s.write(Address::v(addr), s.cop1.get64(operands.ft(), s.cop0.f64()))?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SDC1 {}, {:#06X}({})",
            operands.ftn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Sdl;

impl Instruction for Sdl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let base = addr & !7;
        let offset = addr & 7;

        let dword = if offset == 0 {
            operands.rtv64(s)
        } else {
            let mut dword = s.read(Address::v(base))?;
            dword &= 0xFFFFFFFF_FFFFFFFF << (64 - 8 * offset);
            dword |= operands.rtv64(s) >> (8 * offset);
            dword
        };

        s.write(Address::v(base), dword)?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SDL {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Sdr;

impl Instruction for Sdr {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let base = addr & !7;
        let offset = addr & 7;

        let data = if offset == 7 {
            operands.rtv64(s)
        } else {
            let mut dword = s.read(Address::v(base))?;
            dword &= 0xFFFFFFFF_FFFFFFFF >> (8 * (offset + 1));
            dword |= operands.rtv64(s) << (56 - 8 * offset);
            dword
        };

        s.write(Address::v(base), data)?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SDR {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Sh;

impl Instruction for Sh {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 1);

        s.write(Address::v(addr), operands.rtv(s) as u16)?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SH {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Slti;

impl Instruction for Slti {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()]
            .set64(((operands.rsv64(s) as i64) < (opcode.imm16() as i16 as i64)) as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SLTI {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Sltiu;

impl Instruction for Sltiu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let imm = opcode.imm16() as i16 as i64 as u64;

        s.cpu.regs.gpr[operands.rt()].set64((operands.rsv64(s) < imm) as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SLTIU {}, {}, {:#06X}",
            operands.rtn(),
            operands.rsn(),
            opcode.imm16()
        )
    }
}

pub struct Sw;

impl Instruction for Sw {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 3);

        s.write(Address::v(addr), operands.rtv(s))?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SW {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Swc1;

impl Instruction for Swc1 {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        let addr = opcode.offset_addr(s);
        check_aligned!(store, addr, 3);

        s.write(Address::v(addr), s.cop1.get32(operands.ft(), s.cop0.f64()))?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SWC1 {}, {:#06X}({})",
            operands.ftn(),
            opcode.imm16(),
            operands.basen()
        )
    }
}

pub struct Swl;

impl Instruction for Swl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let addr_base = addr & !3;
        let addr_offset = addr & 3;

        let word = if addr_offset == 0 {
            operands.rtv(s)
        } else {
            let mut word = s.read(Address::v(addr_base))?;
            word &= 0xFFFF_FFFF << (32 - 8 * addr_offset);
            word |= operands.rtv(s) >> (8 * addr_offset);
            word
        };

        s.write(Address::v(addr_base), word)?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SWL {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}

pub struct Swr;

impl Instruction for Swr {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        let addr = opcode.offset_addr(s);
        let base = addr & !3;
        let offset = addr & 3;

        let word = if offset == 3 {
            operands.rtv(s)
        } else {
            let mut word = s.read(Address::v(base))?;
            word &= 0xFFFF_FFFF >> (8 * (offset + 1));
            word |= operands.rtv(s) << (24 - 8 * offset);
            word
        };

        s.write(Address::v(base), word)?;

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SWR {}, {:#06X}({})",
            operands.rtn(),
            opcode.imm16(),
            operands.rsn()
        )
    }
}
