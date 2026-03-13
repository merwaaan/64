use arbitrary_int::prelude::*;

use crate::{
    cpu::{self, instructions::Disassembly, opcode::Opcode},
    dp::{Dp, DpLocation},
    inst,
    mi::Interrupt,
    sp::{self, Register, Sp, SpRegsLocation},
    system::System,
};

#[derive(Clone, Copy, Debug)]
pub enum InstructionEffect {
    /// The instruction was a delayed branching.
    /// If the branch was taken, contains the target address.
    DelayedBranching(Option<u12>),
    // TODO SkipDelaySlot
}

pub type InstructionResult = Option<InstructionEffect>;

pub type ExecuteFn = fn(&mut System, Opcode) -> InstructionResult;
pub type DisassembleFn = fn(&System, Opcode) -> Disassembly;
pub type DecodedInstruction = (ExecuteFn, DisassembleFn);

pub fn decode(opcode: Opcode) -> Option<DecodedInstruction> {
    match opcode.group() {
        0b000000 => Some(match opcode.0 & 0x3F {
            0x00 => inst!(sll),
            0x02 => inst!(srl),
            0x03 => inst!(sra),
            0x04 => inst!(sllv),
            0x06 => inst!(srlv),
            0x07 => inst!(srav),
            0x08 => inst!(jr),
            0x09 => inst!(jalr),
            0x0D => inst!(r#break),
            0x20 => inst!(add),
            0x21 => inst!(addu),
            0x22 => inst!(sub),
            0x23 => inst!(subu),
            0x24 => inst!(and),
            0x25 => inst!(or),
            0x26 => inst!(xor),
            0x27 => inst!(nor),
            0x2A => inst!(slt),
            0x2B => inst!(sltu),
            _ => return None, // TODO reserved exception?
        }),
        0b000001 => Some(match opcode.0 & 0x1F_0000 {
            0x00_0000 => inst!(bltz),
            0x01_0000 => inst!(bgez),
            0x10_0000 => inst!(bltzal),
            0x11_0000 => inst!(bgezal),
            _ => return None, // TODO reserved exception?
        }),
        0b010000 => Some(match opcode.0 & 0x03E0_0000 {
            0x000_0000 => inst!(mfc0),
            0x080_0000 => inst!(mtc0),
            _ => return None,
        }),
        0b010010 => Some(inst!(vec)),
        _ => Some(match opcode.group() {
            0x02 => inst!(j),
            0x03 => inst!(jal),
            0x04 => inst!(beq),
            0x05 => inst!(bne),
            0x06 => inst!(blez),
            0x07 => inst!(bgtz),
            0x08 => inst!(addi),
            0x09 => inst!(addiu),
            0x0A => inst!(slti),
            0x0B => inst!(sltiu),
            0x0C => inst!(andi),
            0x0D => inst!(ori),
            0x0E => inst!(xori),
            0x0F => inst!(lui),
            0x20 => inst!(lb),
            0x21 => inst!(lh),
            0x23 => inst!(lw),
            0x24 => inst!(lbu),
            0x25 => inst!(lhu),
            0x27 => inst!(lwu),
            0x28 => inst!(sb),
            0x29 => inst!(sh),
            0x2B => inst!(sw),
            0x32 => inst!(lwc2),
            0x3A => inst!(swc2),
            _ => return None, // TODO reserved exception?
        }),
    }
}

fn offset_addr(s: &System, op: Opcode) -> u12 {
    let base = s.sp.regs2.read(op.base());
    let offset = op.0 & 0xFFFF;
    u12::from_u32(base.wrapping_add(offset) & 0x0FFF)
}

fn branch_target(s: &System, op: Opcode) -> u12 {
    let offset = u12::from_u32((op.0 << 2) & 0x0FFF);

    s.sp.pc.wrapping_add(u12::new(4)).wrapping_add(offset)
}

/// TODO temp
macro_rules! placeholder {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _execute >](_s: &mut System, _op: Opcode) -> Option<InstructionEffect> {
                panic!("Unimplemented instruction: {}", stringify!($name));

            }

            fn [< $name _disassemble >](_s: &System, _op: Opcode) -> Disassembly {
                Disassembly::new(stringify!($name).to_string())
            }
        }
    };
}

/// Helper to reuse the disassembly function from the CPU module.
macro_rules! reuse_cpu_disassembly {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](s: &System, op: Opcode) -> Disassembly {
                cpu::instructions_cpu::[< $name _disassemble >](s, op)
            }
        }
    };
}

fn vec_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    // TODO temp
    //log::error!(" SP: UNIMPLEMENTED VEC {:08X}", op.0);

    None
}

fn vec_disassemble(_s: &System, op: Opcode) -> Disassembly {
    // TODO temp
    Disassembly::new(format!("<UNIMPLEMENTED VEC> {:08X}", op.0))
}

fn add_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), rs.wrapping_add(rt));

    None
}

reuse_cpu_disassembly!(add);

fn addi_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let imm = op.imm16() as i16 as i32 as u32;

    s.sp.regs2.write(op.rt(), rs.wrapping_add(imm));

    None
}

reuse_cpu_disassembly!(addi);

fn addiu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let imm = op.imm16() as i16 as i32 as u32;

    s.sp.regs2.write(op.rt(), rs.wrapping_add(imm));

    None
}

reuse_cpu_disassembly!(addiu);

fn addu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), rs.wrapping_add(rt));

    None
}

reuse_cpu_disassembly!(addu);

fn and_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2
        .write(op.rd(), s.sp.regs2.read(op.rs()) & s.sp.regs2.read(op.rt()));

    None
}

reuse_cpu_disassembly!(and);

fn andi_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2
        .write(op.rt(), s.sp.regs2.read(op.rs()) & (op.imm16() as u32));

    None
}

reuse_cpu_disassembly!(andi);

// TODO generic branching?
fn beq_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(
        if s.sp.regs2.read(op.rs()) == s.sp.regs2.read(op.rt()) {
            Some(branch_target(s, op))
        } else {
            None
        },
    ))
}

reuse_cpu_disassembly!(beq);

fn bgez_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(
        if (s.sp.regs2.read(op.rs()) as i32) >= 0 {
            Some(branch_target(s, op))
        } else {
            None
        },
    ))
}

reuse_cpu_disassembly!(bgez);

fn bgtz_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(
        if (s.sp.regs2.read(op.rs()) as i32) > 0 {
            Some(branch_target(s, op))
        } else {
            None
        },
    ))
}

reuse_cpu_disassembly!(bgtz);

fn bgezal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = s.sp.regs2.read(op.rs()) as i32;

    // The return address is the instruction that follows the delay slot
    s.sp.regs2
        .write(31, s.sp.pc.wrapping_add(u12::new(8)).into());

    Some(InstructionEffect::DelayedBranching(if rs >= 0 {
        Some(branch_target(s, op))
    } else {
        None
    }))
}

reuse_cpu_disassembly!(bgezal);

fn blez_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(
        if (s.sp.regs2.read(op.rs()) as i32) <= 0 {
            Some(branch_target(s, op))
        } else {
            None
        },
    ))
}

reuse_cpu_disassembly!(blez);

fn bltz_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(
        if (s.sp.regs2.read(op.rs()) as i32) < 0 {
            Some(branch_target(s, op))
        } else {
            None
        },
    ))
}

reuse_cpu_disassembly!(bltz);

fn bltzal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = s.sp.regs2.read(op.rs()) as i32;

    // The return address is the instruction that follows the delay slot
    s.sp.regs2
        .write(31, s.sp.pc.wrapping_add(u12::new(8)).into());

    Some(InstructionEffect::DelayedBranching(if rs < 0 {
        Some(branch_target(s, op))
    } else {
        None
    }))
}

reuse_cpu_disassembly!(bltzal);

fn bne_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(
        if s.sp.regs2.read(op.rs()) != s.sp.regs2.read(op.rt()) {
            Some(branch_target(s, op))
        } else {
            None
        },
    ))
}

reuse_cpu_disassembly!(bne);

fn j_execute(_s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // The RSP doesn't have exceptions so it just ignores the 2 least significant bits
    let target = u12::from_u32((op.0 << 2) & 0x0FFC);

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

reuse_cpu_disassembly!(j);

fn jal_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let target = u12::from_u32((op.0 << 2) & 0x0FFC);

    s.sp.regs2
        .write(31, s.sp.pc.wrapping_add(u12::new(8)).into());

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

reuse_cpu_disassembly!(jal);

fn jalr_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let target = u12::from_u32(s.sp.regs2.read(op.rs()) & 0x0FFC);

    s.sp.regs2
        .write(op.rd(), s.sp.pc.wrapping_add(u12::new(8)).into());

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

reuse_cpu_disassembly!(jalr);

fn jr_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let target = u12::from_u32(s.sp.regs2.read(op.rs()) & 0x0FFC);

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

reuse_cpu_disassembly!(jr);

fn break_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    s.sp.regs[Register::Status as usize] |= sp::STATUS_BROKE | sp::STATUS_HALTED;

    if s.sp.interrupt_on_break() {
        s.mi.set_pending_interrupt(Interrupt::Sp, &mut s.cop0);
    }

    None
}

reuse_cpu_disassembly!(break);

fn lui_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.sp.regs2.write(op.rt(), (op.imm16() as u32) << 16);

    None
}

reuse_cpu_disassembly!(lui);

fn lb_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));
    let word = s.sp.mem[addr as usize] as i8 as i32 as u32;

    s.sp.regs2.write(op.rt(), word);

    None
}

reuse_cpu_disassembly!(lb);

fn lbu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));
    let word = s.sp.mem[addr as usize] as u32;

    s.sp.regs2.write(op.rt(), word);

    None
}

reuse_cpu_disassembly!(lbu);

fn lh_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));

    let word = u16::from_be_bytes([
        s.sp.mem[addr as usize],
        s.sp.mem[(addr as usize + 1) & 0xFFF],
    ]) as i16 as i32 as u32;

    s.sp.regs2.write(op.rt(), word);

    None
}

reuse_cpu_disassembly!(lh);

fn lhu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));

    let word = u16::from_be_bytes([
        s.sp.mem[addr as usize],
        s.sp.mem[(addr as usize + 1) & 0xFFF],
    ]) as u32;

    s.sp.regs2.write(op.rt(), word);

    None
}

reuse_cpu_disassembly!(lhu);

fn lw_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));

    let data = u32::from_be_bytes([
        s.sp.mem[addr as usize],
        s.sp.mem[(addr as usize + 1) & 0x0FFF],
        s.sp.mem[(addr as usize + 2) & 0x0FFF],
        s.sp.mem[(addr as usize + 3) & 0x0FFF],
    ]);

    s.sp.regs2.write(op.rt(), data);

    None
}

reuse_cpu_disassembly!(lw);

fn lwc2_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    // TODO
    //log::error!(" SP: UNIMPLEMENTED LWC2 {:08X}", op.0);

    None
}

fn lwc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("<UNIMPLEMENTED LWC2> {:08X}", op.0))
}

fn lwu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Same as LW since there is no sign extension to 64 bits
    lw_execute(s, op)
}

reuse_cpu_disassembly!(lwu);

fn mfc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = match op.rd() {
        // SP
        // We use read_reg to trigger side effects (semaphore!)
        0..=7 => {
            s.sp.read_reg(SpRegsLocation::from_relative((op.rd() as u32) * 4))
        }
        // DP
        8..=15 => Dp::read(s, DpLocation::from_relative(((op.rd() - 8) as u32) * 4)),
        _ => panic!("Invalid MFC0 register: {}", op.rd()),
    };

    s.sp.regs2.write(op.rt(), data);

    None
}

fn mfc0_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFC0 {}, {}", op.rtn(), op.rd())) // TODO rd name
}

fn mtc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    match op.rd() {
        // SP
        0..=7 => {
            // TODO weird, just use a read_reg func?
            Sp::write_reg(
                s,
                SpRegsLocation::from_relative((op.rd() as u32) * 4),
                s.sp.regs2.read(op.rt()),
            );
        }
        // DP
        8..=15 => {
            // TODO weird, just use a read_reg func?
            Dp::write(
                s,
                DpLocation::from_relative(((op.rd() - 8) as u32) * 4),
                s.sp.regs2.read(op.rt()),
            );
        }
        _ => panic!("Invalid MTC0 register: {}", op.rd()),
    }

    None
}

fn mtc0_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTC0 {}, {}", op.rtn(), op.rd())) // TODO rd name
}

fn nor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2.write(
        op.rd(),
        !(s.sp.regs2.read(op.rs()) | s.sp.regs2.read(op.rt())),
    );

    None
}

reuse_cpu_disassembly!(nor);

fn or_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2
        .write(op.rd(), s.sp.regs2.read(op.rs()) | s.sp.regs2.read(op.rt()));

    None
}

reuse_cpu_disassembly!(or);

fn ori_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2
        .write(op.rt(), s.sp.regs2.read(op.rs()) | (op.imm16() as u32));

    None
}

reuse_cpu_disassembly!(ori);

fn sb_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));
    let byte = s.sp.regs2.read(op.rt()) as u8;

    s.sp.mem[addr as usize] = byte;

    None
}

reuse_cpu_disassembly!(sb);

fn sh_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));
    let bytes = u16::to_be_bytes(s.sp.regs2.read(op.rt()) as u16);

    s.sp.mem[addr as usize] = bytes[0];
    s.sp.mem[(addr as usize + 1) & 0x0FFF] = bytes[1];

    None
}

reuse_cpu_disassembly!(sh);

fn sll_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), rt << op.shift());

    None
}

reuse_cpu_disassembly!(sll);

fn sllv_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rt = s.sp.regs2.read(op.rt());
    let shift = s.sp.regs2.read(op.rs()) & 0x1F;

    s.sp.regs2.write(op.rd(), rt << shift);

    None
}

reuse_cpu_disassembly!(sllv);

fn sra_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rt = s.sp.regs2.read(op.rt());
    let result = ((rt as i32) >> op.shift() as i32) as u32;

    s.sp.regs2.write(op.rd(), result);

    None
}

reuse_cpu_disassembly!(sra);

fn srav_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rt = s.sp.regs2.read(op.rt());
    let rs = s.sp.regs2.read(op.rs());
    let shift = rs & 0x1F;
    let result = ((rt as i32) >> (shift as i32)) as u32;

    s.sp.regs2.write(op.rd(), result);

    None
}

reuse_cpu_disassembly!(srav);

fn slt_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs()) as i32;
    let rt = s.sp.regs2.read(op.rt()) as i32;

    s.sp.regs2.write(op.rd(), (rs < rt) as u32);

    None
}

reuse_cpu_disassembly!(slt);

fn slti_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = s.sp.regs2.read(op.rs()) as i32;
    let imm = op.imm16() as i16 as i32;

    s.sp.regs2.write(op.rt(), (rs < imm) as u32);

    None
}

reuse_cpu_disassembly!(slti);

fn sltiu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let imm = op.imm16() as i16 as i32 as u32;

    s.sp.regs2.write(op.rt(), (rs < imm) as u32);

    None
}

reuse_cpu_disassembly!(sltiu);

fn sltu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), (rs < rt) as u32);

    None
}

reuse_cpu_disassembly!(sltu);

fn srl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), rt >> op.shift());

    None
}

reuse_cpu_disassembly!(srl);

fn srlv_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rt = s.sp.regs2.read(op.rt());
    let shift = s.sp.regs2.read(op.rs()) & 0x1F;

    s.sp.regs2.write(op.rd(), rt >> shift);

    None
}

reuse_cpu_disassembly!(srlv);

fn sub_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs()) as i32;
    let rt = s.sp.regs2.read(op.rt()) as i32;

    s.sp.regs2.write(op.rd(), rs.wrapping_sub(rt) as u32);

    None
}

reuse_cpu_disassembly!(sub);

fn subu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), rs.wrapping_sub(rt));

    None
}

reuse_cpu_disassembly!(subu);

fn sw_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = u32::from(offset_addr(s, op));
    let bytes = u32::to_be_bytes(s.sp.regs2.read(op.rt()));

    s.sp.mem[addr as usize] = bytes[0];
    s.sp.mem[(addr as usize + 1) & 0x0FFF] = bytes[1]; // TODO write mem helper with u12 to avoid repeating mask?
    s.sp.mem[(addr as usize + 2) & 0x0FFF] = bytes[2];
    s.sp.mem[(addr as usize + 3) & 0x0FFF] = bytes[3];

    None
}

reuse_cpu_disassembly!(sw);

fn swc2_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    // TODO
    //log::error!(" SP: UNIMPLEMENTED SWC2 {:08X}", op.0);

    None
}

fn swc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("<UNIMPLEMENTED SWC2> {:08X}", op.0))
}

fn xor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2
        .write(op.rd(), s.sp.regs2.read(op.rs()) ^ s.sp.regs2.read(op.rt()));

    None
}

reuse_cpu_disassembly!(xor);

fn xori_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.regs2
        .write(op.rt(), s.sp.regs2.read(op.rs()) ^ (op.imm16() as u32));

    None
}

reuse_cpu_disassembly!(xori);
