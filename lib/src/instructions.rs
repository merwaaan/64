use crate::cpu::CPU;
use crate::registers::Registers;

/// Result of a branch/jump: target PC to use after the delay slot.
#[derive(Clone, Copy, Debug)]
pub struct DelayedBranching(pub u64);

/// Instruction trait.
pub trait Instruction {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching>;
    fn disassemble(&self, cpu: &CPU, instruction: u32) -> String;
}

// Helpers to extract fields from the opcode

fn rs(opcode: u32) -> usize {
    ((opcode >> 21) & 0x1F) as usize
}

fn rt(opcode: u32) -> usize {
    ((opcode >> 16) & 0x1F) as usize
}

fn rd(opcode: u32) -> usize {
    ((opcode >> 11) & 0x1F) as usize
}

fn sa(opcode: u32) -> u32 {
    (opcode >> 6) & 0x1F
}

fn funct(opcode: u32) -> u32 {
    opcode & 0x3F
}

fn imm16(opcode: u32) -> u16 {
    (opcode & 0xFFFF) as u16
}

fn imm16_sext_u64(opcode: u32) -> u64 {
    (imm16(opcode) as i16 as i32) as u64
}

fn reg(i: usize) -> String {
    format!("{}/{:X}", Registers::gpr_name(i), i)
}

/// Returns the instruction for the given opcode
pub fn decode(opcode: u32) -> &'static dyn Instruction {
    let op = opcode >> 26;

    match op {
        0x00 => {
            let f = funct(opcode);
            match f {
                0x00 => &SLL,
                0x08 => &JR,
                0x24 => &AND,
                0x25 => &OR,
                0x26 => &XOR,
                0x2B => &SLTU,
                _ => &UNKNOWN,
            }
        }
        0x05 => &BNE,
        0x09 => &ADDIU,
        0x0F => &LUI,
        0x23 => &LW,
        0x2B => &SW,
        0x10 => &COP0,
        0x2F => &CACHE,
        _ => &UNKNOWN,
    }
}

struct Unknown;

impl Instruction for Unknown {
    fn execute(&self, _cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        panic!("Unknown opcode: {:08X}", opcode)
    }
    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!("<UNKNOWN {:08X}>", opcode)
    }
}

struct Addiu;

impl Instruction for Addiu {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = imm16_sext_u64(opcode);
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs].wrapping_add(imm);

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "ADDIU {}, {}, {:#x}",
            reg(rt(opcode)),
            reg(rs(opcode)),
            imm16(opcode) as i16
        )
    }
}

struct And;

impl Instruction for And {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rs(opcode)] & cpu.regs.gpr[rt(opcode)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "AND {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

struct Bne;

impl Instruction for Bne {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rs = rs(opcode);
        let rt = rt(opcode);

        if cpu.regs.gpr[rs] != cpu.regs.gpr[rt] {
            let offset = (imm16(opcode) as i16 as i32 as i64) << 2; // TODO less casts??

            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(offset as u64);

            return Some(DelayedBranching(future_pc));
        }

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = (imm16(opcode) as i16 as i32) << 2;

        format!(
            "BNE {}, {}, {:#x}",
            reg(rs(opcode)),
            reg(rt(opcode)),
            offset
        )
    }
}

struct Cache;

impl Instruction for Cache {
    fn execute(&self, _cpu: &mut CPU, _instruction: u32) -> Option<DelayedBranching> {
        None
    }
    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let op = (opcode >> 16) & 0x1F;
        let base = rs(opcode);

        format!("CACHE {}, {}({})", op, imm16(opcode) as i16, reg(base))
    }
}

struct Cop0;

impl Instruction for Cop0 {
    fn execute(&self, _cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let sub = (opcode >> 21) & 0x1F;
        match sub {
            4 => {} // MTC0 todo
            _ => panic!("Unknown COP0 sub: {:02x}", sub),
        }
        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let sub = (opcode >> 21) & 0x1F;

        // TODO

        format!("<COP0 {:02x}>", sub)
    }
}

struct Jr;

impl Instruction for Jr {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rs = rs(opcode);
        let addr = cpu.regs.gpr[rs];

        Some(DelayedBranching(addr))
    }

    fn disassemble(&self, cpu: &CPU, opcode: u32) -> String {
        let rs = rs(opcode);
        let addr = cpu.regs.gpr[rs];

        format!("JR {}={:04X}", reg(rs), addr)
    }
}

struct Lui;

impl Instruction for Lui {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = (opcode & 0xFFFF) as u32; // TODO imm()
        let rt = rt(opcode);

        cpu.regs.gpr[rt] = (imm << 16) as i32 as u64;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let imm = (opcode & 0xFFFF) as u32; // TODO imm()

        format!("LUI {}, {:#x}", reg(rt(opcode)), imm)
    }
}

struct Lw;

impl Instruction for Lw {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let offset = imm16(opcode) as i16 as i32 as u32; // TODO less casts??
        let rt = rt(opcode);
        let base = rs(opcode);

        let addr = cpu.regs.gpr[base] as u32 + offset;

        cpu.regs.gpr[rt] = cpu.read(addr) as i32 as u64;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = imm16(opcode) as i16;

        format!("LW {}, {}({})", reg(rt(opcode)), offset, reg(rs(opcode)))
    }
}

struct Or;

impl Instruction for Or {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rs(opcode)] | cpu.regs.gpr[rt(opcode)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "OR {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

struct Sll;

impl Instruction for Sll {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let shift = (opcode >> 6) & 0x1F;

        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rt(opcode)] << shift;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let (rd, rt, sa) = (rd(opcode), rt(opcode), sa(opcode));

        if rd == 0 && rt == 0 && sa == 0 {
            "NOP".to_string()
        } else {
            format!("SLL {}, {}, {}", reg(rd), reg(rt), sa)
        }
    }
}

struct Sltu;

impl Instruction for Sltu {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = (cpu.regs.gpr[rs(opcode)] < cpu.regs.gpr[rt(opcode)]) as u64;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "SLTU {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

struct Sw;

impl Instruction for Sw {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let offset = imm16(opcode) as i16 as i32 as u32;
        let rt = rt(opcode);
        let base = rs(opcode); // TODO weird, impl base

        let addr = cpu.regs.gpr[base] as u32 + offset;
        cpu.write(addr, cpu.regs.gpr[rt] as u32);

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = imm16(opcode) as i16;
        let base = rs(opcode);
        let rt = rt(opcode);

        format!("SW {}, {:04X}({})", reg(rt), offset, reg(base))
    }
}

struct Xor;

impl Instruction for Xor {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rs(opcode)] ^ cpu.regs.gpr[rt(opcode)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "XOR {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

// Static instances for decode()
static UNKNOWN: Unknown = Unknown;
static ADDIU: Addiu = Addiu;
static AND: And = And;
static BNE: Bne = Bne;
static CACHE: Cache = Cache;
static COP0: Cop0 = Cop0;
static JR: Jr = Jr;
static LUI: Lui = Lui;
static LW: Lw = Lw;
static OR: Or = Or;
static SLL: Sll = Sll;
static SLTU: Sltu = Sltu;
static SW: Sw = Sw;
static XOR: Xor = Xor;
