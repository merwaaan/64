use arbitrary_int::prelude::*;
use std::{
    i16,
    simd::{
        cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd},
        num::{SimdInt, SimdUint},
        *,
    },
};

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

pub fn decode(opcode: Opcode) -> DecodedInstruction {
    match opcode.group() {
        0b000000 => match opcode.0 & 0x3F {
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
            _ => RESERVED_INSTRUCTION,
        },
        0b000001 => match opcode.0 & 0x1F_0000 {
            0x00_0000 => inst!(bltz),
            0x01_0000 => inst!(bgez),
            0x10_0000 => inst!(bltzal),
            0x11_0000 => inst!(bgezal),
            _ => RESERVED_INSTRUCTION,
        },
        0b010000 => match opcode.0 & 0x03E0_0000 {
            0x000_0000 => inst!(mfc0),
            0x080_0000 => inst!(mtc0),
            _ => RESERVED_INSTRUCTION,
        },
        0b010010 => match (opcode.0 >> 21) & 0x1F {
            0x00 => inst!(mfc2),
            0x02 => inst!(cfc2),
            0x04 => inst!(mtc2),
            0x06 => inst!(ctc2),
            _ => match opcode.0 & 0x3F {
                0x00 => inst!(vmulf),
                0x01 => inst!(vmulu),
                0x02 => inst!(vrndp),
                0x03 => inst!(vmulq),
                0x04 => inst!(vmudl),
                0x05 => inst!(vmudm),
                0x06 => inst!(vmudn),
                0x07 => inst!(vmudh),
                0x08 => inst!(vmacf),
                0x09 => inst!(vmacu),
                0x0A => inst!(vrndn),
                0x0B => inst!(vmacq),
                0x0C => inst!(vmadl),
                0x0D => inst!(vmadm),
                0x0E => inst!(vmadn),
                0x0F => inst!(vmadh),
                0x10 => inst!(vadd),
                0x11 => inst!(vsub),
                0x13 => inst!(vabs),
                0x14 => inst!(vaddc),
                0x15 => inst!(vsubc),
                0x1D => inst!(vsar),
                0x20 => inst!(vlt),
                0x21 => inst!(veq),
                0x22 => inst!(vne),
                0x23 => inst!(vge),
                0x24 => inst!(vcl),
                0x25 => inst!(vch),
                0x26 => inst!(vcr),
                0x27 => inst!(vmrg),
                0x28 => inst!(vand),
                0x29 => inst!(vnand),
                0x2A => inst!(vor),
                0x2B => inst!(vnor),
                0x2C => inst!(vxor),
                0x2D => inst!(vnxor),
                0x30 => inst!(vrcp),
                0x31 => inst!(vrcpl),
                0x32 => inst!(vrcph),
                0x33 => inst!(vmov),
                0x34 => inst!(vrsq),
                0x35 => inst!(vrsql),
                0x36 => inst!(vrsqh),
                0x37 => inst!(vnop),
                _ => RESERVED_INSTRUCTION,
            },
        },
        _ => match opcode.group() {
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
                _ => RESERVED_INSTRUCTION,
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
                0x0A => inst!(swv),
                0x0B => inst!(stv),
                _ => RESERVED_INSTRUCTION,
            },
            _ => RESERVED_INSTRUCTION,
        },
    }
}

// TODO to opcode struct

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

fn vs(op: Opcode) -> usize {
    ((op.0 >> 11) & 0x1F) as usize
}

fn vd(op: Opcode) -> usize {
    ((op.0 >> 6) & 0x1F) as usize
}

fn vbase(op: Opcode) -> usize {
    ((op.0 >> 21) & 0x1F) as usize
}

fn voffset(op: Opcode, shift: usize) -> usize {
    ((op.0 & 0x7F) as usize) << shift
}

fn broadcast(e: u8, v: i16x8) -> i16x8 {
    debug_assert!(e < 16);

    match e {
        0 | 1 => v,
        // Quarters
        2 => i16x8::from_array([v[0], v[0], v[2], v[2], v[4], v[4], v[6], v[6]]),
        3 => i16x8::from_array([v[1], v[1], v[3], v[3], v[5], v[5], v[7], v[7]]),
        // Halves
        4 => i16x8::from_array([v[0], v[0], v[0], v[0], v[4], v[4], v[4], v[4]]),
        5 => i16x8::from_array([v[1], v[1], v[1], v[1], v[5], v[5], v[5], v[5]]),
        6 => i16x8::from_array([v[2], v[2], v[2], v[2], v[6], v[6], v[6], v[6]]),
        7 => i16x8::from_array([v[3], v[3], v[3], v[3], v[7], v[7], v[7], v[7]]),
        // Singles
        8 => i16x8::splat(v[0]),
        9 => i16x8::splat(v[1]),
        10 => i16x8::splat(v[2]),
        11 => i16x8::splat(v[3]),
        12 => i16x8::splat(v[4]),
        13 => i16x8::splat(v[5]),
        14 => i16x8::splat(v[6]),
        15 => i16x8::splat(v[7]),
        _ => unreachable!(),
    }
}

const ZEROS: i16x8 = i16x8::splat(0);
const ZEROS32: i32x8 = i32x8::splat(0);

const ONES: i16x8 = i16x8::splat(1);
const ONES32: i32x8 = i32x8::splat(1);

const ACC_LO_MASK: i64x8 = i64x8::splat(!0xFFFF);

// TODO u4?
fn velement_offset(op: Opcode) -> u8 {
    ((op.0 >> 7) & 0xF) as u8
}

// TODO u4?
fn velement(op: Opcode) -> u8 {
    ((op.0 >> 21) & 0xF) as u8
}

/// TODO temp
macro_rules! placeholder {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _execute >](_s: &mut System, _op: Opcode) -> Option<InstructionEffect> {
                //panic!("SP: unimplemented {}", stringify!($name));

                None
            }

            fn [< $name _disassemble >](_s: &System, _op: Opcode) -> Disassembly {
                Disassembly::new(stringify!($name).to_string())
            }
        }
    };
}

/// Helper to reuse the disassembly function from the CPU module, as many instructions are shared.
macro_rules! reuse_cpu_disassembly {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](s: &System, op: Opcode) -> Disassembly {
                cpu::instructions_cpu::[< $name _disassemble >](s, op)
            }
        }
    };
}

macro_rules! disassembly_vd_vs_vte {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](_s: &System, op: Opcode) -> Disassembly {
                Disassembly::new(format!(
                    "{} v{}, v{}, v{}[{}]",
                    stringify!($name:upper),
                    vd(op),
                    vs(op),
                    vt(op),
                    velement(op)
                ))
            }
        }
    };
}

fn reserved_execute(_s: &mut System, op: Opcode) -> InstructionResult {
    log::warn!("SP: reserved instruction: {:08X}", op.0);

    None
}

fn reserved_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("<RESERVED {:08X}>", op.0))
}

pub const RESERVED_INSTRUCTION: DecodedInstruction = (reserved_execute, reserved_disassemble);

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
    let e = velement_offset(op) as usize;

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
        velement_offset(op),
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

// TODO unused code for SB into SPMEM - (main CPU SB, not the RSP's!!!)
// // Hardware tests (n64-systemtest) show that SB is broken and clobbers the whole target 32-bit word:
// // - stores high bits to the left of the target address
// // - fills the right bits with zeros
//
// let word = s.sp.regs2.read(op.rt());
//
// let addr = u32::from(offset_addr(s, op));
// let addr_aligned = addr & !3;
//
// let shift = (3 - addr_aligned) * 8;
// let shifted = word << shift;
//
// shifted.write_mem(&mut s.sp.mem, addr_aligned);

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
    let e = velement_offset(op) as usize;

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
        velement_offset(op),
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
    let rs = s.sp.regs2.read(op.rs());
    let imm = op.imm16() as i16 as u32; // sign-extends and then compare unsigned

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

////////////////

fn vnop_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    None
}

fn vnop_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("VNOP".to_string())
}

/*
 * Load & stores
 */

fn lqv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Load data with a 16-bytes alignment:
    // - source: from the effective DMEM address up to the next 16-byte boundary
    // - destination: from byte 0 of the vector register up to the length of the source data, zeroing the right part

    // TODO simplify with iterative approach

    let e = velement_offset(op) as u32;

    let start = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 4) as u32) & 0x0FFF; // TODO mask? or use correct type?
    //let end = start + 16;

    let length = (16 - (start & 0xF)).min(16 - e);

    let mut reg_be8 = s.sp.vregs[vt(op)].to_be_bytes().to_array();

    reg_be8[e as usize..(e + length) as usize]
        .copy_from_slice(&s.sp.mem[start as usize..(start + length) as usize]);

    // TODO from_be_bytes instead of casting?
    let reg_be16: &[i16] = bytemuck::cast_slice(&reg_be8);

    let reg = num::SimdInt::swap_bytes(i16x8::from_slice(reg_be16));

    s.sp.vregs[vt(op)] = reg;

    None
}

fn lqv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LQV v{}[{}], {:X}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

fn lrv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Load data with a 16-bytes alignment:
    // - source: from the previous 16-byte boundary to the effective DMEM address minus one (the exact address is written via LQV)
    // - destination: from byte 16 - length of the vector register to its end

    // TODO simplify with iterative approach

    // TODO manual says it's not used but it is??
    let e = velement_offset(op) as usize;

    let mem_addr =
        (s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 4) as u32) & 0x0FFF) as usize; // TODO mask or use correct type?
    let mem_start = mem_addr & !0xF;
    let mem_length = mem_addr - mem_start;

    let reg_start = 16 - mem_length + e;

    if mem_length != 0 && reg_start < 0x10 {
        let reg_length = 16usize.wrapping_sub(reg_start) & 0xF;

        let mut vreg_be8 = s.sp.vregs[vt(op)].to_be_bytes().to_array();

        vreg_be8[reg_start..reg_start + reg_length]
            .copy_from_slice(&s.sp.mem[mem_start as usize..mem_start as usize + reg_length]);

        // TODO from_be_bytes instead of casting?
        let v_be16: &[i16] = bytemuck::cast_slice(&vreg_be8);

        let v = num::SimdInt::swap_bytes(i16x8::from_slice(v_be16));

        s.sp.vregs[vt(op)] = v;
    }

    None
}

fn lrv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LRV v{}[{}], {:X}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

fn srv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    panic!("SRV not implemented");
}

fn srv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SRV v{}[{}], {:X}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

placeholder!(ltv);
placeholder!(stv);

// fn ltv_execute(s: &mut System, op: Opcode) -> InstructionResult {
//     // TODO

//     None
// }

// fn ltv_disassemble(_s: &System, op: Opcode) -> Disassembly {
//     Disassembly::new(format!(
//         "LTV v{}[{}], {:X}({})",
//         vt(op),
//         velement_offset(op),
//         voffset(op, 4),
//         vbase(op)
//     ))
// }

// fn stv_execute(s: &mut System, op: Opcode) -> InstructionResult {
//     let addr = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 4) as u32); // TODO mask? or use correct type?

//     let e = velement_offset(op) as usize;

//     for i in 0..8 {
//         let vt_index = e + i;
//         let vt_lane = vt[vt_index & 7];

//         let vt_shift = if (vt_index & 8) == 0 {
//             EVEN_SHIFT
//         } else {
//             ODD_SHIFT
//         };

//         s.sp.mem[(addr as usize + i) & 0x0FFF] = (vt_lane >> vt_shift) as u8;
//     }

//     None
// }

// fn stv_disassemble(_s: &System, op: Opcode) -> Disassembly {
//     Disassembly::new(format!(
//         "STV v{}[{}], {:X}({})",
//         vt(op),
//         velement_offset(op),
//         voffset(op, 4),
//         vbase(op)
//     ))
// }

placeholder!(lfv);

placeholder!(lhv);
placeholder!(lwv);

fn sqv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Store data with a quadword alignment:
    // - destination: from the effective DMEM address up to the next 16-byte boundary
    // - source: from byte 0 of the vector register up to the length of the destination data

    let start = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 4) as u32) & 0x0FFF; // TODO mask? or use correct type?

    let v_be16 = s.sp.vregs[vt(op)].to_be_bytes().to_array();
    let v_be8 = bytemuck::bytes_of(&v_be16);

    // TODO simpler to copy byte by byte?

    let length = 16 - (start & 0xF);

    let e = velement_offset(op) as u32;
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
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

/// Generic LPV/LUV implementation, only the shift amount differs
fn lxv_execute<const SHIFT: usize>(s: &mut System, op: Opcode) -> InstructionResult {
    // Contrarily to what the manual says, the element specifier offsets the source DMEM bytes.
    // Also, the source data wraps around the 16-bytes segment that starts at the 8-bytes aligned address
    // (eg. address = 0x29 wraps around [0x28, 0x37]).

    let addr = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 3) as u32) as usize; // TODO mask? or use correct type?
    let addr_aligned8 = (addr & !7) as usize;
    let addr_offset8 = (addr & 7) as usize;

    let e = velement_offset(op) as usize;

    let mut reg = ZEROS;

    for i in 0..8usize {
        let byte_addr =
            (addr_aligned8 + ((addr_offset8 + (16 - e + i) & 15) & 0xF) as usize) & 0x0FFF;

        let value = s.sp.mem[byte_addr];

        reg[i as usize] = (value as i16) << SHIFT;
    }

    s.sp.vregs[vt(op)] = reg;

    None
}

fn lpv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    lxv_execute::<8>(s, op)
}

fn lpv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LPV v{}[{}], {}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

fn luv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    lxv_execute::<7>(s, op)
}

fn luv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LUV v{}[{}], {}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

/// Generic SPV/SUV implementation, only the shift amount differs
fn sxv_execute<const EVEN_SHIFT: usize, const ODD_SHIFT: usize>(
    s: &mut System,
    op: Opcode,
) -> InstructionResult {
    // Contrarily to what the manual says, the element specifier offsets the source register bytes.
    // The shift amount actually varies between 8 and 7 depending on the current offset, every 8 bytes.

    let vt = s.sp.vregs[vt(op)];

    let addr = s.sp.regs2.0[vbase(op) as usize].wrapping_add(voffset(op, 3) as u32); // TODO mask? or use correct type?

    let e = velement_offset(op) as usize;

    for i in 0..8 {
        let vt_index = e + i;
        let vt_lane = vt[vt_index & 7];

        let vt_shift = if (vt_index & 8) == 0 {
            EVEN_SHIFT
        } else {
            ODD_SHIFT
        };

        s.sp.mem[(addr as usize + i) & 0x0FFF] = (vt_lane >> vt_shift) as u8;
    }

    None
}

fn spv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    sxv_execute::<8, 7>(s, op)
}

fn spv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SPV v{}[{}], {}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

fn suv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    sxv_execute::<7, 8>(s, op)
}

fn suv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SUV v{}[{}], {}({})",
        vt(op),
        velement_offset(op),
        voffset(op, 4),
        vbase(op)
    ))
}

placeholder!(sfv);
placeholder!(shv);
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
        0 => s.sp.vco as i16 as i32 as u32,
        1 => s.sp.vcc as i16 as i32 as u32,
        2 | 3 => s.sp.vce as u32,
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
        0 => s.sp.vco = value as u16,
        1 => s.sp.vcc = value as u16,
        2 | 3 => s.sp.vce = value as u8,
        _ => unreachable!(),
    };

    None
}

fn ctc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("CTC2 {}, {}", op.rt(), op.rd()))
}

fn mfc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let e = velement_offset(op) as usize;

    let bytes = s.sp.vregs[op.rd()].to_be_bytes();
    let hi = bytes[e & 0xF] as u16;
    let lo = bytes[(e + 1) & 0xF] as u16;
    let data = ((hi << 8) | lo) as i16 as i32 as u32;

    s.sp.regs2.write(op.rt(), data);

    None
}

fn mfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "MFC2 {}, v{}[{}]",
        op.rtn(),
        op.rd(),
        velement_offset(op)
    ))
}

fn mtc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = s.sp.regs2.0[op.rt()] as u16;

    let mut bytes = s.sp.vregs[op.rd()].to_be_bytes();

    let e = velement_offset(op) as usize;

    bytes[e] = (data >> 8) as u8;

    // No wrapping so the LSB is not copied if the element offset is 15

    if e < 15 {
        bytes[e + 1] = data as u8;
    }

    s.sp.vregs[op.rd()] = i16x8::from_be_bytes(bytes);

    None
}

fn mtc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "MTC2 {}, v{}[{}]",
        op.rtn(),
        op.rd(),
        velement_offset(op)
    ))
}

fn vsar_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Accumulator portion indexing:
    // e(0)=8=HI, e(1)=9=MID, e(2)=10=LO, other indices are ignored

    let vd = vd(op);

    match velement(op) {
        // TODO handle sep to avoid computations?
        e @ (8 | 9 | 10) => {
            // Save vd in case it's both the target and the destination
            let vd_value = s.sp.vregs[vd];

            // Write the accumulator portion to vd

            let acc_index = (e - 8) as usize;
            let acc_shift = 32 - 16 * acc_index;
            s.sp.vregs[vd] = (s.sp.vacc >> i64x8::splat(acc_shift as i64)).cast::<i16>();

            // Write vs to the accumulator portion

            // TODO?
            // let acc_mask = i64x8::splat(0xFFFF << acc_shift);
            // s.sp.vacc =
            //     s.sp.vacc & !acc_mask | (vd_value.cast::<u16>().cast::<i64>() << acc_shift as i64);
        }
        _ => {
            log::info!("vsar: e={}", velement(op));

            s.sp.vregs[vd] = ZEROS;

            // TODO write to acc?
        }
    };

    None
}

fn vsar_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "VSAR v{}, v{}, v{}[{}]",
        vd(op),
        vs(op),
        vt(op),
        velement(op)
    ))
}

fn vmov_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // The RSP manual is unclear about VMOV
    //
    // In practice:
    // - vt is broadcasted and goes in acc low
    // - vs is the lane index to write to vd AND to read from in the broadcast

    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let de = vs(op) & 7;
    s.sp.vregs[vd(op)][de] = vt[de];

    s.sp.vacc &= ACC_LO_MASK;
    s.sp.vacc |= vt.cast::<u16>().cast::<i64>();

    None
}

fn vmov_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "VMOV v{}[{}], v{}[{}]",
        vd(op),
        velement(op),
        vt(op),
        velement(op)
    ))
}

/*
 * Logical
 */

// TODO generalize?

fn vand_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let result = vs & vt;

    s.sp.vregs[vd(op)] = vs & vt;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

fn vand_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VAND {}, {}, {}", op.rd(), op.rs(), op.rt()))
}

fn vnand_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let result = !(vs & vt);

    s.sp.vregs[vd(op)] = result;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

fn vnand_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VNAND {}, {}, {}", op.rd(), op.rs(), op.rt()))
}

fn vor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let result = vs | vt;

    s.sp.vregs[vd(op)] = vs | vt;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

fn vor_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VOR {}, {}, {}", op.rd(), op.rs(), op.rt()))
}

fn vnor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let result = !(vs | vt);

    s.sp.vregs[vd(op)] = !(vs | vt);
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

fn vnor_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VNOR {}, {}, {}", op.rd(), op.rs(), op.rt()))
}

fn vxor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let result = vs ^ vt;

    s.sp.vregs[vd(op)] = vs ^ vt;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

fn vxor_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VXOR {}, {}, {}", op.rd(), op.rs(), op.rt()))
}

fn vnxor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let result = !(vs ^ vt);

    s.sp.vregs[vd(op)] = result;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

fn vnxor_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VNXOR {}, {}, {}", op.rd(), op.rs(), op.rt()))
}

// ----------
// Arithmetic
// ----------

fn vabs_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Negate vt's lanes depending on the sign of vs.
    // If vs is zero, set the result to zero.

    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let zeroed = vs.simd_eq(ZEROS).select(ZEROS, vt);

    // The wrapped negated result goes into the accumulator.
    // The saturated negated result goes into the destination register.
    // This matter for 0x8000 which negates to 0x8000 (wrapped) / 0x7FFF (saturated).

    let negated_wrap = vs.simd_lt(ZEROS).select(-zeroed, zeroed);
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | negated_wrap.cast::<u16>().cast::<i64>();

    let negated_sat = vs.simd_lt(ZEROS).select(zeroed.saturating_neg(), zeroed);
    s.sp.vregs[vd(op)] = negated_sat;

    None
}

fn vabs_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("VABS {}, {}, {}", vd(op), vs(op), vt(op)))
}

fn vadd_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Add vt to vs with carry in, store in acc (wrapped) and vd (clamped), clear the carry

    let vs_i32 = s.sp.vregs[vs(op)].cast::<i32>();
    let vt_i32 = broadcast(velement(op), s.sp.vregs[vt(op)]).cast::<i32>();

    let vco = Mask::<i32, 8>::from_bitmask((s.sp.vco & 0xFF) as u64)
        .select(i32x8::splat(1), i32x8::splat(0));

    let wrapped_i32 = vs_i32 + vt_i32 + vco;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | wrapped_i32.cast::<u16>().cast::<i64>();

    let clamped_i32 = wrapped_i32.simd_clamp(i32x8::splat(-32768), i32x8::splat(32767));
    s.sp.vregs[vd(op)] = clamped_i32.cast::<i16>();

    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vadd);

fn vaddc_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs_u16 = s.sp.vregs[vs(op)].cast::<u16>();
    let vt_u16 = broadcast(velement(op), s.sp.vregs[vt(op)]).cast::<u16>();

    let result_u16 = vs_u16 + vt_u16;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result_u16.cast::<i64>();
    s.sp.vregs[vd(op)] = result_u16.cast::<i16>();

    let carry_out = result_u16.simd_lt(vs_u16);
    s.sp.vco = carry_out.to_bitmask() as u16;

    None
}

disassembly_vd_vs_vte!(vaddc);

fn clamp_acc_signed<const MIN: i16, const MAX: i16, const MID_OR_LO: bool>(acc: &i64x8) -> i16x8 {
    let hi = (acc >> 32).cast::<i16>();
    let mid = (acc >> 16).cast::<i16>();

    let min = i16x8::splat(MIN);
    let max = i16x8::splat(MAX);

    // TODO optim?

    let underflow =
        hi.simd_lt(ZEROS) & (hi.simd_ne(i16x8::splat(0xFFFFu16 as i16)) | mid.simd_ge(ZEROS));

    let overflow = hi.simd_ge(ZEROS) & (hi.simd_ne(ZEROS) | mid.simd_lt(ZEROS));

    let portion = if MID_OR_LO { mid } else { acc.cast::<i16>() };

    let result = underflow.select(min, overflow.select(max, portion));

    result
}

/// Arithmetic instructions store their 48-bit result in the accumulator with a (relatively) high precision.
/// However, the result of such instructions is written to narrower 16-bit registers.
/// So clamping makes the 48-bit result fit in the 16-bit destination register as faithfully as possible, taking sign into account.
/// Instructions that follow generally use the non-clamped result in the accumulator to preserve precision.
fn clamp_acc_signed_mid(acc: &i64x8) -> i16x8 {
    // let hi_mid = (acc >> 16).cast::<i32>();

    // const MIN: Simd<i32, 8> = i32x8::splat(-0x0000_8000);
    // const MAX: Simd<i32, 8> = i32x8::splat(0x0000_7FFF);

    // hi_mid.simd_clamp(MIN, MAX).cast::<i16>()

    // let hi = (acc >> 32).cast::<i16>();
    // let mid = (acc >> 16).cast::<i16>();
    // let lo = acc.cast::<i16>();

    // let underflow = hi.simd_lt(i16x8::splat(0))
    //     & (hi.simd_ne(i16x8::splat(0xFFFFu16 as i16)) | mid.simd_ge(i16x8::splat(0)));

    // let overflow =
    //     hi.simd_ge(i16x8::splat(0)) & (hi.simd_ne(i16x8::splat(0)) | mid.simd_lt(i16x8::splat(0)));

    // let result = underflow.select(
    //     i16x8::splat(-0x0000_8000),
    //     overflow.select(i16x8::splat(0x0000_7FFF), mid),
    // );

    // result

    clamp_acc_signed::<-0x0000_8000, 0x0000_7FFF, true>(acc)
}

fn clamp_acc_signed_low(acc: &i64x8) -> i16x8 {
    // let hi = (acc >> 32).cast::<i16>();
    // let mid = (acc >> 16).cast::<i16>();
    // let lo = acc.cast::<i16>();

    // let underflow = hi.simd_lt(i16x8::splat(0))
    //     & (hi.simd_ne(i16x8::splat(0xFFFFu16 as i16)) | mid.simd_ge(i16x8::splat(0)));

    // let overflow =
    //     hi.simd_ge(i16x8::splat(0)) & (hi.simd_ne(i16x8::splat(0)) | mid.simd_lt(i16x8::splat(0)));

    // let result = underflow.select(
    //     i16x8::splat(0x0000),
    //     overflow.select(i16x8::splat(0xFFFFu16 as i16), lo),
    // );

    // result

    clamp_acc_signed::<0, -1, false>(acc)
}

/// TODO reuse generic clamp??
/// Unsigned accumulator clamping, similar to the signed version but 0 <= x <= 0xFFFF
fn clamp_acc_unsigned(acc: &i64x8) -> i16x8 {
    let hi_mid = (acc >> 16).cast::<i32>();

    const MIN: Simd<i32, 8> = i32x8::splat(0);
    const MAX: Simd<i32, 8> = i32x8::splat(0x0000_7FFF);
    const MAX_CLAMPED: Simd<i32, 8> = i32x8::splat(0x0000_FFFF);

    let mut result = hi_mid.simd_max(MIN);

    result = result
        .simd_max(MIN)
        .simd_gt(MAX)
        .select(MAX_CLAMPED, result);

    result.cast::<i16>()
}

// fn clamp_acc_unsigned_low(acc: &i64x8) -> i16x8 {
//     let hi_mid = (acc >> 16).cast::<i32>();
//     let low = acc.cast::<i16>();

//     const MIN: Simd<i32, 8> = i32x8::splat(0);
//     const MAX: Simd<i32, 8> = i32x8::splat(0x0000_7FFF);

//     let underflow = hi_mid.simd_lt(MIN);
//     let overflow = hi_mid.simd_gt(MAX);

//     log::info!("underflow: {:?}", underflow);
//     log::info!("overflow: {:?}", overflow);
//     log::info!("hi_mid: {:0X?}", hi_mid);
//     log::info!("hi_mid: {:?}", hi_mid);
//     log::info!("low: {:?}", low);
//     log::info!(
//         "result: {:?}",
//         underflow.select(
//             i16x8::splat(0x0000),
//             overflow.select(i16x8::splat(0xFFFFu16 as i16), low)
//         )
//     );

//     underflow.select(i16x8::splat(0x0000), overflow.select(i16x8::splat(-1), low))

//     // let hi_mid = (acc >> 16).cast::<i32>();
//     // let low = acc.cast::<i16>();

//     // const MIN: Simd<i32, 8> = i32x8::splat(0);
//     // const MAX: Simd<i32, 8> = i32x8::splat(0x0000_7FFF);

//     // let underflow = hi_mid.simd_lt(MIN);
//     // let overflow = hi_mid.simd_gt(MAX);

//     // underflow.select(
//     //     i16x8::splat(0x0000),
//     //     overflow.select(i16x8::splat(0xFFFF), low),
//     // )
// }

fn vsub_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Subtract vt from vs with borrow in, store in acc (wrapped) and vd (clamped), clear the carry

    let vs_i32 = s.sp.vregs[vs(op)].cast::<i32>();
    let vt_i32 = broadcast(velement(op), s.sp.vregs[vt(op)]).cast::<i32>();

    let vco_i32 = Mask::<i32, 8>::from_bitmask(s.sp.vco as u64).select(ONES32, ZEROS32);

    let wrapped_i32 = vs_i32 - vt_i32 - vco_i32;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | wrapped_i32.cast::<u16>().cast::<i64>();

    let clamped_i32 = wrapped_i32.simd_clamp(i32x8::splat(-32768), i32x8::splat(32767));
    s.sp.vregs[vd(op)] = clamped_i32.cast::<i16>();

    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vsub);

fn vsubc_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Subtract vt from vs, store in acc and vd (both wrapped), update VCO

    let vs_u16 = s.sp.vregs[vs(op)].cast::<u16>();
    let vt_u16 = broadcast(velement(op), s.sp.vregs[vt(op)]).cast::<u16>();

    let result_u16 = vs_u16 - vt_u16;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result_u16.cast::<i64>();
    s.sp.vregs[vd(op)] = result_u16.cast::<i16>();

    let not_equal = result_u16.simd_ne(u16x8::splat(0)).to_bitmask() as u8;
    let carry_out = vt_u16.simd_gt(vs_u16).to_bitmask() as u8; // unsigned comparison!
    s.sp.vco = ((not_equal as u16) << 8) | (carry_out as u16);

    None
}

disassembly_vd_vs_vte!(vsubc);

// TODO MERGE WITH OTHER MULTS?
// Generic vector multiplication base for vmulf, vmacf, vmulu, vmacu
//
// Multiplies vs * vt as 1.15 fixed-point values.
// Shifts by 1 to convert the 2.30 product back to 1.15.
// Store result in acc, store clamped result in vd.
//
// ADD_ACC variants (vmacf, macu) add the product to the accumulator, otherwise replace it and add rounding.
// CLAMP_SIGNED variants (vmulf, vmacf) store the signed clamped result in vd, otherwise it's unsigned.
fn vmul_generic<const ADD_ACC: bool, const CLAMP_SIGNED: bool>(
    s: &mut System,
    op: Opcode,
) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<i32>();

    let product = (vs32 * vt32).cast::<i64>() << 1;

    if ADD_ACC {
        s.sp.vacc += product;
    } else {
        s.sp.vacc = product + i64x8::splat(0x8000);
    }

    s.sp.vregs[vd(op)] = if CLAMP_SIGNED {
        clamp_acc_signed_mid(&s.sp.vacc)
    } else {
        clamp_acc_unsigned(&s.sp.vacc)
    };

    None
}

fn vmulf_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    vmul_generic::<false, true>(s, op)
}

disassembly_vd_vs_vte!(vmulf);

fn vmacf_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    vmul_generic::<true, true>(s, op)
}

disassembly_vd_vs_vte!(vmacf);

fn vmulu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    vmul_generic::<false, false>(s, op)
}

disassembly_vd_vs_vte!(vmulu);

fn vmacu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    vmul_generic::<true, false>(s, op)
}

disassembly_vd_vs_vte!(vmacu);

fn vmudl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<u16>().cast::<u32>();
    let vt32 = vt.cast::<u16>().cast::<u32>();

    s.sp.vacc = ((vs32 * vt32) >> 16).cast::<i64>();

    s.sp.vregs[vd(op)] = s.sp.vacc.cast::<i16>();

    None
}

disassembly_vd_vs_vte!(vmudl);

fn vmadl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<u16>().cast::<u32>();
    let vt32 = vt.cast::<u16>().cast::<u32>();

    s.sp.vacc += ((vs32 * vt32) >> 16).cast::<i64>();

    s.sp.vregs[vd(op)] = clamp_acc_signed_low(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadl);

fn vmudn_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<u16>().cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc = (vs32 * vt32).cast::<i64>();

    s.sp.vregs[vd(op)] = clamp_acc_signed_low(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmudn);

fn vmadn_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<u16>().cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc += (vs32 * vt32).cast::<i64>();

    s.sp.vregs[vd(op)] = clamp_acc_signed_low(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadn);

fn vmudm_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<u16>().cast::<i32>();

    s.sp.vacc = (vs32 * vt32).cast::<i64>();

    s.sp.vregs[vd(op)] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmudm);

fn vmadm_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<u16>().cast::<i32>();

    s.sp.vacc += (vs32 * vt32).cast::<i64>();

    s.sp.vregs[vd(op)] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadm);

fn vmudh_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc = (vs32 * vt32).cast::<i64>() << 16;

    s.sp.vregs[vd(op)] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmudh);

fn vmadh_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc += (vs32 * vt32).cast::<i64>() << 16;

    s.sp.vregs[vd(op)] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadh);

placeholder!(vmacq);
placeholder!(vmulq);

/// First part of a clipping test for clamping a vector to a range, ie. -VT <= VS <= VT.
/// Sets VCO, VCC, VCE depending on various comparisons for VCL to continue with.
///
/// VCC:
/// - high bits -> is the value "too high" (VS >= VT)?
/// - low bits -> is the value "too low" (VS <= -VT)?
///
/// VCO:
/// - high-bits: ??? TODO
/// - low-bits: sign mismatch between VS and VT
///
/// VCE:
/// - flags if the value exactly matches the lower bound (VS == -VT - 1)
///
/// Depending if the sign are the same or not, the arithmetic operations are adjusted.
fn vch_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    const MINUS_ONE: i16x8 = i16x8::splat(-1);

    let diff_sign = (vs ^ vt).simd_lt(ZEROS);
    let sum = vs + vt;
    let diff = vs - vt;

    let ge = diff_sign.select(vt.simd_lt(ZEROS), diff.simd_ge(ZEROS));
    let le = diff_sign.select(sum.simd_le(ZEROS), vt.simd_lt(ZEROS));
    let ne = diff_sign.select(sum.simd_ne(ZEROS) & vs.simd_ne(!vt), diff.simd_ne(ZEROS)); // TODO not sure here? !vt? more complex on the right?
    let vce = diff_sign.select(sum.simd_eq(MINUS_ONE), Mask::from_bitmask(0));

    s.sp.vcc = ((ge.to_bitmask() as u8 as u16) << 8) | (le.to_bitmask() as u8 as u16);
    s.sp.vco = ((ne.to_bitmask() as u8 as u16) << 8) | (diff_sign.to_bitmask() as u8 as u16);
    s.sp.vce = vce.to_bitmask() as u8;

    let result = diff_sign.select(
        sum.simd_le(ZEROS).select(-vt, vs),
        diff.simd_ge(ZEROS).select(vt, vs),
    );

    s.sp.vregs[vd(op)] = result;
    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();

    None
}

disassembly_vd_vs_vte!(vch);

fn vcl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    // let vsval = broadcast(velement(op), s.sp.vregs[vt(op)]);
    // let vtval = s.sp.vregs[vs(op)];
    // let vs = vtval;
    // let vt = vsval;

    // let mut ge = s.sp.vcc as u64 >> 8;
    // let mut le = s.sp.vcc as u64 & 0xFF;
    // let eq = !(s.sp.vco as u64 >> 8);
    // let diff_sign = s.sp.vco as u64 & 0xFF;
    // let vce = s.sp.vce as u64;

    let mut result = u16x8::splat(0);

    for i in 0..8 {
        let vco_low = ((s.sp.vco >> i) & 1) != 0;
        let vco_high = ((s.sp.vco >> (i + 8)) & 1) != 0;
        let mut vcc_low = ((s.sp.vcc >> i) & 1) != 0;
        let mut vcc_high = ((s.sp.vcc >> (i + 8)) & 1) != 0;
        let vce = ((s.sp.vce >> i) & 1) != 0;

        let vs_lane = vs[i] as u16;
        let vt_lane = vt[i] as u16;

        let r = if vco_low {
            let (sum, carry) = vs_lane.overflowing_add(vt_lane);

            if !vco_high {
                // if vce {
                //     vcc_low = (sum == 0) || !carry;
                // } else {
                //     vcc_low = (sum == 0) && !carry;
                // }
                vcc_low = ((sum == 0) && !carry) || (vce && ((sum == 0) || !carry));
            }

            if vcc_low {
                -(vt_lane as i16) as u16
            } else {
                vs_lane
            }
        } else {
            if !vco_high {
                // if vce {
                //     vcc_high = true;
                // } else {
                //     vcc_high = vs_lane >= vt_lane;
                // }
                vcc_high = vs_lane >= vt_lane;
            }

            if vcc_high { vt_lane } else { vs_lane }
        };

        // result[i] = if vco_low {
        //     if vcc_low {
        //         (vt_lane as i16).saturating_neg() as u16
        //     } else {
        //         vs_lane
        //     }
        // } else {
        //     if vcc_high { vt_lane } else { vs_lane }
        // };
        result[i] = r;

        s.sp.vcc &= !(1 << i);
        s.sp.vcc |= (vcc_low as u16) << i;
        s.sp.vcc &= !(1 << (i + 8));
        s.sp.vcc |= (vcc_high as u16) << (i + 8);
    }

    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u64>().cast::<i64>();
    s.sp.vco = 0;
    s.sp.vce = 0;

    // const ZERO: i16x8 = i16x8::splat(0);

    // let mut ge: Mask<i16, 8> = Mask::from_bitmask(s.sp.vcc as u64 >> 8);
    // let mut le: Mask<i16, 8> = Mask::from_bitmask(s.sp.vcc as u64 & 0xFF);
    // let eq: Mask<i16, 8> = !Mask::from_bitmask(s.sp.vco as u64 >> 8);
    // let diff_sign: Mask<i16, 8> = Mask::from_bitmask(s.sp.vco as u64 & 0xFF);
    // let vce: Mask<i16, 8> = Mask::from_bitmask(s.sp.vce as u64);

    // log::info!("--------------------------------------");
    // log::info!("vs: {:0X?} {:?}", vs, vs);
    // log::info!("vt: {:0X?} {:?}", vt, vt);
    // log::info!("s.sp.vcc: {:08b}", s.sp.vcc);
    // log::info!("ge: {:?}", ge);
    // log::info!("le: {:?}", le);
    // log::info!("s.sp.vco: {:08b}", s.sp.vco);
    // log::info!("diff_sign: {:?}", diff_sign);
    // log::info!("eq: {:?}", eq);
    // log::info!("s.sp.vce: {:08b}", s.sp.vce);
    // log::info!("vce: {:?}", vce);
    // log::info!("===");

    // let sum = vs.cast::<u16>() + vt.cast::<u16>();

    // let carry = sum.simd_lt(vs.cast::<u16>());
    // // let carry = (vs.cast::<i32>() + vt.cast::<i32>())
    // //     .simd_ne(sum.cast::<i32>())
    // //     .cast::<i16>();

    // log::info!("sum: {:0X?}", sum);
    // log::info!("carry: {:?}", carry);

    // // le = diff_sign.select(
    // //     eq.select(
    // //         (!vce & sum.simd_eq(u16x8::splat(0)) & !carry)
    // //             | (vce & (sum.simd_eq(u16x8::splat(0)) | !carry)),
    // //         le,
    // //     ),
    // //     le,
    // // );
    // le = diff_sign.select(
    //     eq.select(
    //         sum.simd_eq(u16x8::splat(0)) & !carry | (vce & (sum.simd_eq(u16x8::splat(0)) | !carry)),
    //         le,
    //     ),
    //     le,
    // );
    // //le = (diff_sign & eq).select(!carry, le);

    // ge = diff_sign.select(ge, eq.select(vs.simd_ge(vt), ge));

    // s.sp.vcc = ((ge.to_bitmask() as u8 as u16) << 8) | (le.to_bitmask() as u8 as u16);
    // s.sp.vco = 0;
    // s.sp.vce = 0;

    // let result = diff_sign.select(le.select(-vt, vs), ge.select(vt, vs));

    // log::info!("ge END: {:?}", ge);
    // log::info!("le END: {:?}", le);
    // log::info!("result: {:0X?} {:?}", result, result);

    // s.sp.vregs[vd(op)] = result;
    // s.sp.vacc = s.sp.vacc & i64x8::splat(!0xFFFF) | result.cast::<u16>().cast::<i64>();

    None
}

disassembly_vd_vs_vte!(vcl);

fn vmrg_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let mask = Mask::<i16, 8>::from_bitmask(s.sp.vcc as u64);

    let result = mask.select(vs, vt);

    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();
    s.sp.vregs[vd(op)] = result;

    s.sp.vco = 0; // Manual error: VCO is actuallycleared

    None
}

disassembly_vd_vs_vte!(vmrg);

placeholder!(vcr);
placeholder!(vrcp);
placeholder!(vrcpl);
placeholder!(vrcph);
placeholder!(vrsq);
placeholder!(vrsql);
placeholder!(vrsqh);
placeholder!(vrndn);
placeholder!(vrndp);

// -----------
// Comparisons
// -----------

// TODO note about manual being super inaccurate
fn veq_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Compare vs and vt taking VCO into account, store the result in acc and vd, clear VCO.
    // The manual states that "VCO and VCE are used as input" but the bit about VCE looks like an error.

    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let equal = vs.simd_eq(vt) & !Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64);

    let result = equal.select(vs, vt);

    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();
    s.sp.vregs[vd(op)] = result;

    s.sp.vcc = equal.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(veq);

fn vne_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Similar to VEQ, VCE seems unused

    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let not_equal = vs.simd_ne(vt) | Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64);

    let result = not_equal.select(vs, vt);

    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();
    s.sp.vregs[vd(op)] = result;

    s.sp.vcc = not_equal.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vne);

fn vge_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Similar to VEQ and VNE, still no VCE in sight, the condition takes both halves of VCO into account

    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vco_mask = Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64)
        & Mask::<i16, 8>::from_bitmask(s.sp.vco as u64);

    let ge = vs.simd_gt(vt) | vs.simd_eq(vt) & !vco_mask;

    let result = ge.select(vs, vt);

    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();
    s.sp.vregs[vd(op)] = result;

    s.sp.vcc = ge.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vge);

fn vlt_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Similar to VGE

    let vs = s.sp.vregs[vs(op)];
    let vt = broadcast(velement(op), s.sp.vregs[vt(op)]);

    let vco_mask = Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64)
        & Mask::<i16, 8>::from_bitmask(s.sp.vco as u64);

    let lt = vs.simd_lt(vt) | vs.simd_eq(vt) & vco_mask;

    let result = lt.select(vs, vt);

    s.sp.vacc = s.sp.vacc & ACC_LO_MASK | result.cast::<u16>().cast::<i64>();
    s.sp.vregs[vd(op)] = result;

    s.sp.vcc = lt.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vlt);
