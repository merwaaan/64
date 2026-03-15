use arbitrary_int::prelude::*;
use std::simd::*;

use crate::{
    cpu::{self, instructions::Disassembly, opcode::Opcode},
    dp::{Dp, DpLocation},
    inst,
    mi::Interrupt,
    sp::{self, Register, Sp, SpRegsLocation},
    system::System,
    value::Value,
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

// Create a big jumptable with regs pre-decoded?

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
        0b010010 => Some(match (opcode.0 >> 21) & 0x1F {
            0x00 => inst!(mfc2),
            0x02 => inst!(cfc2),
            0x04 => inst!(mtc2),
            0x06 => inst!(ctc2),
            _ => inst!(vec), // TODO placeholder
        }),
        _ => Some(match opcode.group() {
            // TODO redundant match???
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
            0x32 => match (opcode.0 >> 11) & 0x1F {
                0x00 => inst!(lbv),
                0x01 => inst!(lsv),
                0x02 => inst!(llv),
                0x03 => inst!(ldv),
                0x04 => inst!(lqv),
                0x05 => inst!(lrv),
                0x06 => inst!(lpv),
                0x07 => inst!(luv),
                0x08 => inst!(lhv),
                0x09 => inst!(lfv),
                0x0A => inst!(lwv),
                0x0B => inst!(ltv),
                _ => return None,
            },
            0x3A => match (opcode.0 >> 11) & 0x1F {
                0x00 => inst!(sbv),
                0x01 => inst!(ssv),
                0x02 => inst!(slv),
                0x03 => inst!(sdv),
                0x04 => inst!(sqv),
                0x05 => inst!(srv),
                0x06 => inst!(spv),
                0x07 => inst!(suv),
                0x08 => inst!(shv),
                0x09 => inst!(sfv),
                0x0A => inst!(swv), // TODO ? save as suv in rsp manual
                0x0B => inst!(stv),
                _ => return None,
            },
            _ => return None,
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

fn vt(op: Opcode) -> usize {
    ((op.0 >> 16) & 0x1F) as usize
}

fn vbase(op: Opcode) -> usize {
    ((op.0 >> 21) & 0x1F) as usize
}

fn voffset(op: Opcode, shift: usize) -> usize {
    ((op.0 & 0x7F) as usize) << shift
}

// TODO ux?
fn velement(op: Opcode) -> u8 {
    ((op.0 >> 7) & 0xF) as u8
}

/// TODO temp
macro_rules! placeholder {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _execute >](_s: &mut System, _op: Opcode) -> Option<InstructionEffect> {
                log::error!("Unimplemented instruction: {}", stringify!($name));

                None
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
    // Same as ADDI since there is no overflow exceptions
    addi_execute(s, op)
}

reuse_cpu_disassembly!(addiu);

fn addu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Same as ADD since there is no overflow exceptions
    add_execute(s, op)
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

/// Generic vector load
fn generic_load_execute<SIZE>(s: &mut System, op: Opcode) -> InstructionResult {
    let byte_size = size_of::<SIZE>();

    let addr = s.sp.regs2.0[vbase(op) as usize]
        .wrapping_add(voffset(op, byte_size.trailing_zeros() as usize) as u32)
        & 0x0FFF; // TODO mask? or use correct type?

    let vt = vt(op);
    let e = velement(op) as usize;

    // No wrapping, we're loading up the last lane
    let width = byte_size.min(16 - e);

    for byte_offset in 0..width {
        let byte_index = e + byte_offset;

        let byte = s.sp.mem[((addr as usize) + byte_offset) & 0x0FFF];

        let lane = byte_index >> 1;
        let mut lane_bytes = s.sp.vregs[vt][lane].to_be_bytes();
        lane_bytes[byte_index & 1] = byte;
        s.sp.vregs[vt][lane] = i16::from_be_bytes(lane_bytes);
    }

    None
}

fn generic_load_disassemble<SIZE>(name: &str, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "{} v{}[{}], {:X}({})",
        name,
        vt(op),
        velement(op),
        voffset(op, size_of::<SIZE>()),
        vbase(op)
    ))
}

fn lbv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u8>(s, op)
}

// TODO macro: vector_load!("LBV", u8)
// TODO macro: vector_load!("LSV", u16)
// TODO macro: vector_load!("LLV", u32)
// TODO macro: vector_load!("LDV", u64)

fn lbv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_load_disassemble::<u8>("LBV", op)
}

fn lsv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u16>(s, op)
}

fn lsv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_load_disassemble::<u16>("LSV", op)
}

fn llv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u32>(s, op)
}

fn llv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_load_disassemble::<u32>("LLV", op)
}

fn ldv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u64>(s, op)
}

fn ldv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_load_disassemble::<u64>("LDV", op)
}

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

fn lqv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Load data with a quadword alignment:
    // - source: from the effective DMEM address up to the next 16-byte boundary
    // - destination: from byte 0 of the vector register up to the length of the source data, zeroing the right part

    let e = velement(op) as u32;

    let start = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 4) as u32) & 0x0FFF; // TODO mask? or use correct type?
    //let end = start + 16;

    let length = (16 - (start & 0xF)).min(16 - e);

    let mut v_be8 = s.sp.vregs[vt(op)].to_be_bytes().to_array();
    // TODO zero after or not?

    v_be8[e as usize..(e + length) as usize]
        .copy_from_slice(&s.sp.mem[start as usize..(start + length) as usize]);

    // TODO from_be_bytes instead of casting?
    let v_be16: &[i16] = bytemuck::cast_slice(&v_be8);

    let v = num::SimdInt::swap_bytes(i16x8::from_slice(v_be16));

    s.sp.vregs[vt(op)] = v;

    None
}

fn lqv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LQV v{}[{}], {:X}({})",
        vt(op),
        velement(op),
        voffset(op, 4),
        vbase(op)
    ))
}

reuse_cpu_disassembly!(lw);

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

/// Generic vector store
fn generic_store_execute<SIZE>(s: &mut System, op: Opcode) -> InstructionResult {
    let byte_size = size_of::<SIZE>();

    let addr = s.sp.regs2.0[vbase(op) as usize]
        .wrapping_add(voffset(op, byte_size.trailing_zeros() as usize) as u32)
        & 0x0FFF; // TODO mask? or use correct type?

    let vt = vt(op);
    let e = velement(op) as usize;

    for byte_offset in 0..byte_size {
        // Wrap around lanes
        let byte_index = (e + byte_offset) & 0xF;
        let lane = byte_index >> 1;
        let lane_bytes = s.sp.vregs[vt][lane].to_be_bytes();
        let byte = lane_bytes[byte_index & 1];

        s.sp.mem[((addr as usize) + byte_offset) & 0x0FFF] = byte;
    }

    None
}

// TODO use generic_load?
fn generic_store_disassemble<SIZE>(name: &str, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "{} v{}[{}], {:X}({})",
        name,
        vt(op),
        velement(op),
        voffset(op, size_of::<SIZE>()),
        vbase(op)
    ))
}

fn sbv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u8>(s, op)
}

fn sbv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_store_disassemble::<u8>("SBV", op)
}

fn ssv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u16>(s, op)
}

fn ssv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_store_disassemble::<u16>("SSV", op)
}

fn slv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u32>(s, op)
}

fn slv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_store_disassemble::<u32>("SLV", op)
}

fn sdv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u64>(s, op)
}

fn sdv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    generic_store_disassemble::<u64>("SDV", op)
}

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
    // Same as SLTI since there is no overflow exceptions
    slti_execute(s, op)
}

reuse_cpu_disassembly!(sltiu);

fn sltu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = s.sp.regs2.read(op.rs());
    let rt = s.sp.regs2.read(op.rt());

    s.sp.regs2.write(op.rd(), (rs < rt) as u32);

    None
}

reuse_cpu_disassembly!(sltu);

fn sqv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Store data with a quadword alignment:
    // - destination: from the effective DMEM address up to the next 16-byte boundary
    // - source: from byte 0 of the vector register up to the length of the destination data

    let start = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 4) as u32) & 0x0FFF; // TODO mask? or use correct type?

    let v_be16 = s.sp.vregs[vt(op)].to_be_bytes().to_array();
    let v_be8 = bytemuck::bytes_of(&v_be16);

    // TODO simpler to copy byte by byte?

    let length = 16 - (start & 0xF);

    let e = velement(op) as u32;
    let non_wrapped_length = length.min(16 - e);

    s.sp.mem[start as usize..(start + non_wrapped_length) as usize]
        .copy_from_slice(&v_be8[e as usize..(e + non_wrapped_length) as usize]);

    let wrapped_length = length - non_wrapped_length;

    s.sp.mem[(start + non_wrapped_length) as usize
        ..(start + non_wrapped_length + wrapped_length) as usize]
        .copy_from_slice(&v_be8[0..wrapped_length as usize]);

    None
}

fn sqv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SQV v{}[{}], {}({})",
        vt(op),
        velement(op),
        voffset(op, 4),
        vbase(op)
    ))
}

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
    // Same as SUB since there is no overflow exceptions
    sub_execute(s, op)
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

placeholder!(lfv);
placeholder!(lrv);
placeholder!(lpv);
placeholder!(luv);
placeholder!(lhv);
placeholder!(ltv);
placeholder!(lwv);

placeholder!(sfv);
placeholder!(srv);
placeholder!(spv);
placeholder!(suv);
placeholder!(shv);
placeholder!(stv);
placeholder!(swv);

fn cfc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // CTC2/CFC2 copy from/to the VCO/VCC/VCE registers
    //
    // 0: VCO
    // 1: VCC
    // 2: VCE
    // 3: VCE again (weird but confirmed by n64-systemtest)

    s.sp.regs2.0[op.rt()] = match op.rd() & 3 {
        // Only sign-extend from 16 to 32 bits
        0 => s.sp.vcarry as i16 as i32 as u32,
        1 => s.sp.vcomparecode as i16 as i32 as u32,
        2 | 3 => s.sp.vcompareext as u32,
        _ => unreachable!(),
    };

    None
}

fn cfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("CFC2 {}, {}", op.rt(), op.rd()))
}

fn ctc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Same register indices as CFC2

    let value = s.sp.regs2.0[op.rt()];

    match op.rd() & 3 {
        0 => s.sp.vcarry = value as u16,
        1 => s.sp.vcomparecode = value as u16,
        2 | 3 => s.sp.vcompareext = value as u8,
        _ => unreachable!(),
    };

    None
}

fn ctc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("CTC2 {}, {}", op.rt(), op.rd()))
}

fn mfc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let e = velement(op) as usize;

    let bytes = s.sp.vregs[op.rd()].to_be_bytes();
    let hi = bytes[e & 0xF] as u16;
    let lo = bytes[(e + 1) & 0xF] as u16;
    let data = ((hi << 8) | lo) as i16 as i32 as u32;

    s.sp.regs2.write(op.rt(), data);

    None
}

fn mfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFC2 {}, v{}[{}]", op.rtn(), op.rd(), velement(op)))
}

fn mtc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = s.sp.regs2.0[op.rt()] as u16;

    let mut bytes = s.sp.vregs[op.rd()].to_be_bytes();

    let e = velement(op) as usize;

    bytes[e] = (data >> 8) as u8;

    // No wrapping so the LSB is not copied if the element offset is 15

    if e < 15 {
        bytes[e + 1] = data as u8;
    }

    s.sp.vregs[op.rd()] = i16x8::from_be_bytes(bytes);

    None
}

fn mtc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTC2 {}, v{}[{}]", op.rtn(), op.rd(), velement(op)))
}
