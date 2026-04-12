use std::{
    i16,
    simd::{
        cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd},
        num::{SimdInt, SimdUint},
        *,
    },
};

use arbitrary_int::prelude::*;

use crate::{
    blocks::write_block,
    dp::{Dp, DpLocation},
    inst,
    mi::Interrupt,
    sp::{self, Register, Sp, SpRegsLocation, opcode::Opcode},
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
pub type DisassembleFn = fn(&System, Opcode) -> String;
pub type DecodedInstruction = (ExecuteFn, DisassembleFn);

pub fn decode(opcode: Opcode) -> DecodedInstruction {
    match opcode.group().value() {
        // Special group
        0b000000 => match opcode.special_opcode().value() {
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
        // Regimm group
        0b000001 => match opcode.regimm_opcode().value() {
            0x00 => inst!(bltz),
            0x01 => inst!(bgez),
            0x10 => inst!(bltzal),
            0x11 => inst!(bgezal),
            _ => RESERVED_INSTRUCTION,
        },
        // COP0 group
        0b010000 => match opcode.cop_opcode().value() {
            0b00000 => inst!(mfc0),
            0b00100 => inst!(mtc0),
            _ => RESERVED_INSTRUCTION,
        },
        // COP2 group (vector unit)
        0b010010 => match opcode.cop_opcode().value() {
            0x00 => inst!(mfc2),
            0x02 => inst!(cfc2),
            0x04 => inst!(mtc2),
            0x06 => inst!(ctc2),
            _ => match opcode.cop2_opcode().value() {
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
        // Top-level instructions
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
        0x32 => match opcode.cop2_load_store_opcode().value() {
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
        0x3A => match opcode.cop2_load_store_opcode().value() {
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
    }
}

// TODO rm
const ZEROS: i16x8 = i16x8::splat(0);
const ZEROS32: i32x8 = i32x8::splat(0);
const ONES32: i32x8 = i32x8::splat(1);

// TODO split into files

/// TODO temp
macro_rules! placeholder {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _execute >](_s: &mut System, _op: Opcode) -> Option<InstructionEffect> {
                panic!("SP: unimplemented {}", stringify!($name));

                //None
            }

            fn [< $name _disassemble >](_s: &System, _op: Opcode) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

macro_rules! disassembly_rd_rs_rt {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](_s: &System, op: Opcode) -> String {
                format!(
                    "{} r{}, r{}, r{}",
                    stringify!($name:upper), op.rd(), op.rs(), op.rt())
            }
        }
    };
}

macro_rules! disassembly_rt_rs_imm {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](_s: &System, op: Opcode) -> String {
                format!(
                    "{} r{}, r{}, {:#06X}",
                    stringify!($name:upper), op.rt(), op.rs(), op.imm16())
            }
        }
    };
}

macro_rules! disassembly_rs_offset {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](_s: &System, op: Opcode) -> String {
                format!(
                    "{} r{}, {:X}",
                    stringify!($name:upper), op.rs(), op._branch_offset())
            }
        }
    };
}

// TODO offset shift?
macro_rules! disassembly_rt_offset {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](_s: &System, op: Opcode) -> String {
                format!(
                    "{} r{}, {:X}(r{})",
                    stringify!($name:upper), op.rt(), op.offset(0), op.base())
            }
        }
    };
}

macro_rules! disassembly_vd_vs_vte {
    ($name:ident) => {
        paste::paste! {
            fn [< $name _disassemble >](_s: &System, op: Opcode) -> String {
                format!(
                    "{} v{}, v{}, v{}[{}]",
                    stringify!($name:upper),
                    op.vd(),
                    op.vs(),
                    op.vt(),
                    op.element()
                )
            }
        }
    };
}

fn write_acc_lo(s: &mut System, value: i16x8) {
    const MASK: i64x8 = i64x8::splat(!0xFFFF);

    s.sp.vacc = s.sp.vacc & MASK | value.cast::<u16>().cast::<i64>();
}

fn reserved_execute(_s: &mut System, op: Opcode) -> InstructionResult {
    // No exceptions of the RSP so reserved instructions have no effect

    log::warn!("SP: reserved instruction: {:08X}", op.raw_value());

    None
}

fn reserved_disassemble(_s: &System, op: Opcode) -> String {
    format!("<RESERVED {:08X}>", op.raw_value())
}

pub const RESERVED_INSTRUCTION: DecodedInstruction = (reserved_execute, reserved_disassemble);

fn add_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), op.rsv(s).wrapping_add(op.rtv(s)));

    None
}

disassembly_rd_rs_rt!(add);

fn addi_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let imm = op.imm16() as i16 as i32 as u32;

    s.sp.sregs.write(op.rt(), op.rsv(s).wrapping_add(imm));

    None
}

disassembly_rt_rs_imm!(addi);

fn addiu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Same as ADDI since there is no overflow exceptions
    addi_execute(s, op)
}

disassembly_rt_rs_imm!(addiu);
fn addu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Same as ADD since there is no overflow exceptions
    add_execute(s, op)
}

disassembly_rd_rs_rt!(addu);

fn and_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), op.rsv(s) & op.rtv(s));

    None
}

disassembly_rd_rs_rt!(and);

fn andi_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rt(), op.rsv(s) & (op.imm16() as u32));

    None
}

disassembly_rt_rs_imm!(andi);

// ---------
// Branching
// ---------

fn branch(s: &mut System, op: Opcode, condition: bool) -> InstructionResult {
    Some(InstructionEffect::DelayedBranching(if condition {
        Some(op.branch_target(s))
    } else {
        None
    }))
}

fn beq_execute(s: &mut System, op: Opcode) -> InstructionResult {
    branch(s, op, op.rsv(s) == op.rtv(s))
}

fn beq_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "BEQ r{}, r{}, {:#06X}",
        op.rs(),
        op.rt(),
        op.branch_offset()
    )
}

fn bgez_execute(s: &mut System, op: Opcode) -> InstructionResult {
    branch(s, op, (op.rsv(s) as i32) >= 0)
}

disassembly_rs_offset!(bgez);

fn bgtz_execute(s: &mut System, op: Opcode) -> InstructionResult {
    branch(s, op, (op.rsv(s) as i32) > 0)
}

disassembly_rs_offset!(bgtz);

fn bgezal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = op.rsv(s) as i32;

    // The return address is the instruction that follows the delay slot
    s.sp.sregs
        .write(31, s.sp.pc.wrapping_add(u12::new(8)).into());

    branch(s, op, rs >= 0)
}

disassembly_rs_offset!(bgezal);

fn blez_execute(s: &mut System, op: Opcode) -> InstructionResult {
    branch(s, op, (op.rsv(s) as i32) <= 0)
}

disassembly_rs_offset!(blez);

fn bltz_execute(s: &mut System, op: Opcode) -> InstructionResult {
    branch(s, op, (op.rsv(s) as i32) < 0)
}

disassembly_rs_offset!(bltz);

fn bltzal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = op.rsv(s) as i32;

    // The return address is the instruction that follows the delay slot
    s.sp.sregs
        .write(31, s.sp.pc.wrapping_add(u12::new(8)).into());

    branch(s, op, rs < 0)
}

disassembly_rs_offset!(bltzal);

fn bne_execute(s: &mut System, op: Opcode) -> InstructionResult {
    branch(s, op, op.rsv(s) != op.rtv(s))
}

disassembly_rt_rs_imm!(bne); // TODO wrong order

fn j_execute(_s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // The RSP doesn't have exceptions so it just ignores the 2 least significant bits
    // TODO sure? possible to execute misaligned?
    let target = u12::from_u32((op.raw_value() << 2) & 0x0FFC);

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

fn j_disassemble(_s: &System, op: Opcode) -> String {
    format!("J {:#06X}", op.raw_value() & 0x07FF_FFFF)
}

fn jal_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let target = u12::from_u32((op.raw_value() << 2) & 0x0FFC);

    s.sp.sregs
        .write(31, s.sp.pc.wrapping_add(u12::new(8)).into());

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

fn jal_disassemble(_s: &System, op: Opcode) -> String {
    format!("JAL {:#06X}", op.raw_value() & 0x07FF_FFFF)
}

fn jalr_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let target = u12::from_u32(op.rsv(s) & 0x0FFC);

    s.sp.sregs
        .write(op.rd(), s.sp.pc.wrapping_add(u12::new(8)).into());

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

fn jalr_disassemble(_s: &System, op: Opcode) -> String {
    format!("JALR r{}, r{}", op.rd(), op.rs())
}

fn jr_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let target = u12::from_u32(op.rsv(s) & 0x0FFC);

    Some(InstructionEffect::DelayedBranching(Some(target)))
}

fn jr_disassemble(_s: &System, op: Opcode) -> String {
    format!("JR r{}", op.rs())
}

fn break_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    s.sp.cregs[Register::Status as usize] |= sp::STATUS_BROKE | sp::STATUS_HALTED;

    if s.sp.interrupt_on_break() {
        s.mi.set_pending_interrupt(Interrupt::Sp, &mut s.cop0);
    }

    None
}

fn break_disassemble(_s: &System, _op: Opcode) -> String {
    "BREAK".to_string()
}

// -------------
// TODO
// -------------

fn lui_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.sp.sregs.write(op.rt(), (op.imm16() as u32) << 16);

    None
}

fn lui_disassemble(_s: &System, op: Opcode) -> String {
    format!("LUI r{}, {:#06X}", op.rt(), op.imm16())
}

fn lb_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let word = s.sp.mem[op.offset_addr(s)] as i8 as i32 as u32;

    s.sp.sregs.write(op.rt(), word);

    None
}

disassembly_rt_offset!(lb);

fn lbu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let word = s.sp.mem[op.offset_addr(s)] as u32;

    s.sp.sregs.write(op.rt(), word);

    None
}

disassembly_rt_offset!(lbu);

/// Generic vector load
fn generic_load_execute<SIZE>(s: &mut System, op: Opcode) -> InstructionResult {
    let byte_size = size_of::<SIZE>();

    let addr =
        s.sp.sregs
            .read(op.base())
            .wrapping_add(op.offset(byte_size.trailing_zeros() as usize) as u32)
            & 0x0FFF; // TODO mask? or use correct type?

    let vt = op.vt();
    let e = op.element_offset();

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

fn generic_load_disassemble<SIZE>(name: &str, op: Opcode) -> String {
    format!(
        "{} v{}[{}], {:X}({})",
        name,
        op.vt(),
        op.element_offset(),
        op.offset(size_of::<SIZE>()),
        op.base()
    )
}

fn lbv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u8>(s, op)
}

fn lbv_disassemble(_s: &System, op: Opcode) -> String {
    generic_load_disassemble::<u8>("LBV", op)
}

fn lsv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u16>(s, op)
}

fn lsv_disassemble(_s: &System, op: Opcode) -> String {
    generic_load_disassemble::<u16>("LSV", op)
}

fn llv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u32>(s, op)
}

fn llv_disassemble(_s: &System, op: Opcode) -> String {
    generic_load_disassemble::<u32>("LLV", op)
}

fn ldv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_load_execute::<u64>(s, op)
}

fn ldv_disassemble(_s: &System, op: Opcode) -> String {
    generic_load_disassemble::<u64>("LDV", op)
}

fn lh_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);

    let word =
        u16::from_be_bytes([s.sp.mem[addr], s.sp.mem[(addr + 1) & 0xFFF]]) as i16 as i32 as u32;

    s.sp.sregs.write(op.rt(), word);

    None
}

disassembly_rt_offset!(lh);

fn lhu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);

    let word = u16::from_be_bytes([s.sp.mem[addr], s.sp.mem[(addr + 1) & 0xFFF]]) as u32;

    s.sp.sregs.write(op.rt(), word);

    None
}

disassembly_rt_offset!(lhu);

fn lw_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);

    let data = u32::from_be_bytes([
        s.sp.mem[addr],
        s.sp.mem[(addr + 1) & 0x0FFF],
        s.sp.mem[(addr + 2) & 0x0FFF],
        s.sp.mem[(addr + 3) & 0x0FFF],
    ]);

    s.sp.sregs.write(op.rt(), data);

    None
}

disassembly_rt_offset!(lw);

fn lwu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Same as LW since there is no sign extension to 64 bits
    lw_execute(s, op)
}

disassembly_rt_offset!(lwu);

fn mfc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = match op.rd() {
        // SP
        0..=7 => {
            // We use read_reg to trigger side effects (semaphore!)
            s.sp.read_reg(SpRegsLocation::from_relative((op.rd() as u32) * 4))
        }
        // DP
        8..=15 => Dp::read(s, DpLocation::from_relative(((op.rd() - 8) as u32) * 4)),
        _ => panic!("Invalid MFC0 register: {}", op.rd()),
    };

    s.sp.sregs.write(op.rt(), data);

    None
}

fn mfc0_disassemble(_s: &System, op: Opcode) -> String {
    format!("MFC0 r{}, r{}", op.rt(), op.rd())
}

fn mtc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    match op.rd() {
        // SP
        0..=7 => {
            Sp::write_reg(
                s,
                SpRegsLocation::from_relative((op.rd() as u32) * 4),
                op.rtv(s),
            );
        }
        // DP
        8..=15 => {
            Dp::write(
                s,
                DpLocation::from_relative(((op.rd() - 8) as u32) * 4),
                op.rtv(s),
            );
        }
        _ => panic!("Invalid MTC0 register: {}", op.rd()),
    }

    None
}

fn mtc0_disassemble(_s: &System, op: Opcode) -> String {
    format!("MTC0 r{}, r{}", op.rt(), op.rd())
}

fn nor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), !(op.rsv(s) | op.rtv(s)));

    None
}

disassembly_rd_rs_rt!(nor);

fn or_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), op.rsv(s) | op.rtv(s));

    None
}

disassembly_rd_rs_rt!(or);

fn ori_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rt(), op.rsv(s) | (op.imm16() as u32));

    None
}

disassembly_rt_rs_imm!(ori);

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
    let addr = op.offset_addr(s);

    s.sp.mem[addr] = op.rtv(s) as u8;

    None
}

fn sb_disassemble(_s: &System, op: Opcode) -> String {
    // TODO wrong
    format!("SB r{}, r{}, {:#06X}", op.rt(), op.rs(), op.offset(8))
}

/// Generic vector store
fn generic_store_execute<SIZE>(s: &mut System, op: Opcode) -> InstructionResult {
    let byte_size = size_of::<SIZE>();

    let addr =
        s.sp.sregs
            .read(op.base())
            .wrapping_add(op.offset(byte_size.trailing_zeros() as usize) as u32)
            & 0x0FFF; // TODO mask? or use correct type?

    let vt = op.vt();
    let e = op.element_offset();

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
fn generic_store_disassemble<SIZE>(name: &str, op: Opcode) -> String {
    format!(
        "{} v{}[{}], {:X}({})",
        name,
        op.vt(),
        op.element_offset(),
        op.offset(size_of::<SIZE>()),
        op.base()
    )
}

fn sbv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u8>(s, op)
}

fn sbv_disassemble(_s: &System, op: Opcode) -> String {
    generic_store_disassemble::<u8>("SBV", op)
}

fn ssv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u16>(s, op)
}

fn ssv_disassemble(_s: &System, op: Opcode) -> String {
    generic_store_disassemble::<u16>("SSV", op)
}

fn slv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u32>(s, op)
}

fn slv_disassemble(_s: &System, op: Opcode) -> String {
    generic_store_disassemble::<u32>("SLV", op)
}

fn sdv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_store_execute::<u64>(s, op)
}

fn sdv_disassemble(_s: &System, op: Opcode) -> String {
    generic_store_disassemble::<u64>("SDV", op)
}

fn sh_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rt_bytes = u16::to_be_bytes(op.rtv(s) as u16);
    let addr = op.offset_addr(s);

    write_block(&rt_bytes, &mut s.sp.mem[0..0x1000], addr);

    None
}

disassembly_rt_offset!(sh);

fn sll_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), op.rtv(s) << op.shift());

    None
}

disassembly_rd_rs_rt!(sll); // TODO wrong order

fn sllv_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let shift = op.rsv(s) & 0x1F; // TODO sa

    s.sp.sregs.write(op.rd(), op.rtv(s) << shift);

    None
}

disassembly_rd_rs_rt!(sllv); // TODO wrong order

fn sra_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let result = ((op.rtv(s) as i32) >> op.shift() as i32) as u32;

    s.sp.sregs.write(op.rd(), result);

    None
}

disassembly_rd_rs_rt!(sra); // TODO wrong order

fn srav_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let shift = op.rsv(s) & 0x1F; // TODO sa
    let result = ((op.rtv(s) as i32) >> (shift as i32)) as u32;

    s.sp.sregs.write(op.rd(), result);

    None
}

disassembly_rd_rs_rt!(srav); // TODO wrong order

fn slt_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = op.rsv(s) as i32;
    let rt = op.rtv(s) as i32;

    s.sp.sregs.write(op.rd(), (rs < rt) as u32);

    None
}

disassembly_rd_rs_rt!(slt);

fn slti_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv(s) as i32;
    let imm = op.imm16() as i16 as i32;

    s.sp.sregs.write(op.rt(), (rs < imm) as u32);

    None
}

disassembly_rt_rs_imm!(slti);

fn sltiu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = op.rsv(s);
    let imm = op.imm16() as i16 as u32; // sign-extends and then compare unsigned

    s.sp.sregs.write(op.rt(), (rs < imm) as u32);

    None
}

disassembly_rt_rs_imm!(sltiu);

fn sltu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), (op.rsv(s) < op.rtv(s)) as u32);

    None
}

disassembly_rd_rs_rt!(sltu);

fn srl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), op.rtv(s) >> op.shift());

    None
}

disassembly_rd_rs_rt!(srl); // TODO wrong order

fn srlv_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let shift = op.rsv(s) & 0x1F; // TODO sa

    s.sp.sregs.write(op.rd(), op.rtv(s) >> shift);

    None
}

disassembly_rd_rs_rt!(srlv); // TODO wrong order

fn sub_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let rs = op.rsv(s) as i32;
    let rt = op.rtv(s) as i32;

    s.sp.sregs.write(op.rd(), rs.wrapping_sub(rt) as u32);

    None
}

disassembly_rd_rs_rt!(sub);

fn subu_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Same as SUB since there is no overflow exceptions
    sub_execute(s, op)
}

disassembly_rd_rs_rt!(subu);

fn sw_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rt_bytes = u32::to_be_bytes(op.rtv(s));
    let addr = op.offset_addr(s);

    write_block(&rt_bytes, &mut s.sp.mem[0..0x1000], addr);

    None
}

disassembly_rt_offset!(sw);

fn swc2_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    // TODO

    unimplemented!("SWC2");

    None
}

fn swc2_disassemble(_s: &System, op: Opcode) -> String {
    format!("<UNIMPLEMENTED SWC2> {:08X}", op.raw_value())
}

fn xor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rd(), op.rsv(s) ^ op.rtv(s));

    None
}

disassembly_rd_rs_rt!(xor);

fn xori_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    s.sp.sregs.write(op.rt(), op.rsv(s) ^ (op.imm16() as u32));

    None
}

disassembly_rt_rs_imm!(xori);

fn vnop_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    None
}

fn vnop_disassemble(_s: &System, _op: Opcode) -> String {
    "VNOP".to_string()
}

/*
 * Load & stores
 */

fn lqv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Load data with a 16-bytes alignment:
    // - source: from the effective DMEM address up to the next 16-byte boundary
    // - destination: from byte 0 of the vector register up to the length of the source data, zeroing the right part

    // TODO simplify with iterative approach

    let e = op.element_offset() as u32;

    let start = s.sp.sregs.read(op.base()).wrapping_add(op.offset(4) as u32) & 0x0FFF; // TODO mask? or use correct type?
    //let end = start + 16;

    let length = (16 - (start & 0xF)).min(16 - e);

    let mut reg_be8 = op.vtv(s).to_be_bytes().to_array();

    reg_be8[e as usize..(e + length) as usize]
        .copy_from_slice(&s.sp.mem[start as usize..(start + length) as usize]); // TODO unsafe, read_block?

    // TODO from_be_bytes instead of casting?
    let reg_be16: &[i16] = bytemuck::cast_slice(&reg_be8);

    let reg = num::SimdInt::swap_bytes(i16x8::from_slice(reg_be16));

    s.sp.vregs[op.vt()] = reg;

    None
}

fn lqv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "LQV v{}[{}], {:X}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

fn lrv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Load data with a 16-bytes alignment:
    // - source: from the previous 16-byte boundary to the effective DMEM address minus one (the exact address is written via LQV)
    // - destination: from byte 16 - length of the vector register to its end

    // TODO simplify with iterative approach

    // TODO manual says it's not used but it is??
    let e = op.element_offset();

    let mem_addr = (s.sp.sregs.read(op.base()).wrapping_add(op.offset(4) as u32) & 0x0FFF) as usize; // TODO mask or use correct type?
    let mem_start = mem_addr & !0xF;
    let mem_length = mem_addr - mem_start;

    let reg_start = 16 - mem_length + e;

    if mem_length != 0 && reg_start < 0x10 {
        let reg_length = 16usize.wrapping_sub(reg_start) & 0xF;

        let mut vreg_be8 = op.vtv(s).to_be_bytes().to_array();

        vreg_be8[reg_start..reg_start + reg_length]
            .copy_from_slice(&s.sp.mem[mem_start..mem_start + reg_length]); // TODO unsafe, read_block?

        // TODO from_be_bytes instead of casting?
        let v_be16: &[i16] = bytemuck::cast_slice(&vreg_be8);

        let v = num::SimdInt::swap_bytes(i16x8::from_slice(v_be16));

        s.sp.vregs[op.vt()] = v;
    }

    None
}

fn lrv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "LRV v{}[{}], {:X}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

fn srv_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    panic!("SRV not implemented");
}

fn srv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "SRV v{}[{}], {:X}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

fn ltv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // The source is a wrapping chunk of 16 bytes in DMEM, aligned on 8 bytes

    let addr = s.sp.sregs.read(op.base()).wrapping_add(op.offset(4) as u32);

    let addr_base = addr & !7;

    // The destination is a wrapping chunk of 8 vector registers, aligned on 8-vector blocks

    let reg_base = op.vt() & !7;

    // There are some weird indexing rules:
    // - the memory bytes are rotated by 1 if the target address is odd
    // - the memory bytes are rotated by 8 if the target address is in an odd 8-byte chunk

    let start_byte = (op.element_offset() as u32 & 1) + if addr & 8 != 0 { 8 } else { 0 };

    // The starting lane is rotated to the left by the element offset

    let start_lane = (8 - (op.element_offset() >> 1)) & 7; // >> 1 because a lane holds 2 bytes of data

    // Copy diagonally

    for reg_offset in 0..8 {
        let reg = reg_base + reg_offset;

        let lane = (start_lane + reg_offset) & 7;

        let short_addr = start_byte + reg_offset as u32 * 2;

        // Read byte by byte as start_byte could be misaligned and wrapping could occur in the middle
        let hi = s.sp.mem[(addr_base + (short_addr & 15)) as usize & 0x0FFF];
        let lo = s.sp.mem[(addr_base + ((short_addr + 1) & 15)) as usize & 0x0FFF];

        s.sp.vregs[reg][lane] = i16::from_be_bytes([hi, lo]);
    }

    None
}

fn ltv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "LTV v{}[{}], {:X}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

fn stv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // let addr = s.sp.sregs.0[op.base()].wrapping_add(op.offset(4) as u32);

    // let reg_base = op.vt() & !7;
    // let start_lane = op.element_offset() >> 1;
    // let wrap_base = addr & !7;

    // for i in 0..8usize {
    //     let reg = reg_base + i;
    //     let lane = (start_lane + i) & 7;
    //     let byte_offset = ((addr & 7) + (i as u32) * 2) & 0xF;
    //     let mem_addr = ((wrap_base + byte_offset) & 0xFFF) as usize;

    //     let value = s.sp.vregs[reg][lane] as u16;
    //     s.sp.mem[mem_addr] = (value >> 8) as u8;
    //     s.sp.mem[(mem_addr + 1) & 0xFFF] = value as u8;
    // }
    None
}

fn stv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "STV v{}[{}], {:X}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

placeholder!(lfv);

placeholder!(lhv);
placeholder!(lwv);

fn sqv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Store data with a quadword alignment:
    // - destination: from the effective DMEM address up to the next 16-byte boundary
    // - source: from byte 0 of the vector register up to the length of the destination data

    let start = s.sp.sregs.read(op.base()).wrapping_add(op.offset(4) as u32) & 0x0FFF; // TODO mask? or use correct type?

    let v_be16 = op.vtv(s).to_be_bytes().to_array();
    let v_be8 = bytemuck::bytes_of(&v_be16);

    // TODO simpler to copy byte by byte?

    let length = 16 - (start & 0xF);

    let e = op.element_offset() as u32;
    let non_wrapped_length = length.min(16 - e);

    s.sp.mem[start as usize..(start + non_wrapped_length) as usize]
        .copy_from_slice(&v_be8[e as usize..(e + non_wrapped_length) as usize]); // TODO unsafe, read_block?

    let wrapped_length = length - non_wrapped_length;

    s.sp.mem[(start + non_wrapped_length) as usize
        ..(start + non_wrapped_length + wrapped_length) as usize]
        .copy_from_slice(&v_be8[0..wrapped_length as usize]); // TODO unsafe, read_block?

    None
}

fn sqv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "SQV v{}[{}], {}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

/// Generic LPV/LUV implementation, only the shift amount differs
fn lxv_execute<const SHIFT: usize>(s: &mut System, op: Opcode) -> InstructionResult {
    // Contrarily to what the manual says, the element specifier offsets the source DMEM bytes.
    // Also, the source data wraps around the 16-bytes segment that starts at the 8-bytes aligned address
    // (eg. address = 0x29 wraps around [0x28, 0x37]).

    let addr = s.sp.sregs.read(op.base()).wrapping_add(op.offset(3) as u32) as usize; // TODO mask? or use correct type?
    let addr_aligned8 = addr & !7;
    let addr_offset8 = addr & 7;

    let e = op.element_offset();

    let mut reg = ZEROS;

    for i in 0..8usize {
        let byte_addr =
            (addr_aligned8 + ((addr_offset8 + (16 - e + i) & 15) & 0xF) as usize) & 0x0FFF;

        let value = s.sp.mem[byte_addr];

        reg[i] = (value as i16) << SHIFT;
    }

    s.sp.vregs[op.vt()] = reg;

    None
}

fn lpv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    lxv_execute::<8>(s, op)
}

fn lpv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "LPV v{}[{}], {}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

fn luv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    lxv_execute::<7>(s, op)
}

fn luv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "LUV v{}[{}], {}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

/// Generic SPV/SUV implementation, only the shift amount differs
fn sxv_execute<const EVEN_SHIFT: usize, const ODD_SHIFT: usize>(
    s: &mut System,
    op: Opcode,
) -> InstructionResult {
    // Contrarily to what the manual says, the element specifier offsets the source register bytes.
    // The shift amount actually varies between 8 and 7 depending on the current offset, every 8 bytes.

    let vt = op.vtv(s);

    let addr = s.sp.sregs.read(op.base()).wrapping_add(op.offset(3) as u32); // TODO mask? or use correct type?

    let e = op.element_offset();

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

fn spv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "SPV v{}[{}], {}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
}

fn suv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    sxv_execute::<7, 8>(s, op)
}

fn suv_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "SUV v{}[{}], {}({})",
        op.vt(),
        op.element_offset(),
        op.offset(4),
        op.base()
    )
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

    let value = match op.rd() & 3 {
        // Only sign-extend from 16 to 32 bits
        0 => s.sp.vco as i16 as i32 as u32,
        1 => s.sp.vcc as i16 as i32 as u32,
        2 | 3 => s.sp.vce as u32,
        _ => unreachable!(),
    };

    s.sp.sregs.write(op.rt(), value);

    None
}

fn cfc2_disassemble(_s: &System, op: Opcode) -> String {
    format!("CFC2 r{}, r{}", op.rt(), op.rd())
}

fn ctc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Same register indices as CFC2

    let value = op.rtv(s);

    match op.rd() & 3 {
        0 => s.sp.vco = value as u16,
        1 => s.sp.vcc = value as u16,
        2 | 3 => s.sp.vce = value as u8,
        _ => unreachable!(),
    };

    None
}

fn ctc2_disassemble(_s: &System, op: Opcode) -> String {
    format!("CTC2 r{}, r{}", op.rt(), op.rd())
}

fn mfc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let e = op.element_offset();

    // TODO vreg index with rd, check???
    let bytes = s.sp.vregs[op.rd()].to_be_bytes();
    let hi = bytes[e & 0xF] as u16;
    let lo = bytes[(e + 1) & 0xF] as u16;
    let data = ((hi << 8) | lo) as i16 as i32 as u32;

    s.sp.sregs.write(op.rt(), data);

    None
}

fn mfc2_disassemble(_s: &System, op: Opcode) -> String {
    format!("MFC2 r{}, v{}[{}]", op.rt(), op.rd(), op.element_offset())
}

fn mtc2_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv(s) as u16;

    // TODO vreg index with rd, check???
    let mut bytes = s.sp.vregs[op.rd()].to_be_bytes();

    let e = op.element_offset();

    bytes[e] = (data >> 8) as u8;

    // No wrapping so the LSB is not copied if the element offset is 15

    if e < 15 {
        bytes[e + 1] = data as u8;
    }

    s.sp.vregs[op.rd()] = i16x8::from_be_bytes(bytes);

    None
}

fn mtc2_disassemble(_s: &System, op: Opcode) -> String {
    format!("MTC2 r{}, v{}[{}]", op.rt(), op.rd(), op.element_offset())
}

fn vsar_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Accumulator portion indexing:
    // e(0)=8=HI, e(1)=9=MID, e(2)=10=LO, other indices are ignored

    let e = op.element();

    match e {
        8..=10 => {
            // Write the accumulator portion to vd

            let acc_index = (e - 8) as usize;
            let acc_shift = 32 - 16 * acc_index;
            s.sp.vregs[op.vd()] = (s.sp.vacc >> i64x8::splat(acc_shift as i64)).cast::<i16>();
        }
        _ => {
            log::warn!("VSAR: e={}", op.element());

            s.sp.vregs[op.vd()] = ZEROS;

            // TODO do something?
        }
    };

    None
}

fn vsar_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VSAR v{}, v{}, v{}[{}]",
        op.vd(),
        op.vs(),
        op.vt(),
        op.element()
    )
}

fn vmov_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // The RSP manual is unclear about VMOV
    //
    // In practice:
    // - vt is broadcast and goes in acc low
    // - vs is the lane index to write to vd AND to read from in the broadcast

    let vt = op.vtv_broadcast(s);

    let de = op.vs() & 7;
    s.sp.vregs[op.vd()][de] = vt[de]; // TODO not what manual says, check

    write_acc_lo(s, vt);
    s.sp.vacc |= vt.cast::<u16>().cast::<i64>();

    None
}

fn vmov_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VMOV v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

// https://emudev.org/2020/03/28/RSP
const RECIPROCAL_TABLE: [u16; 512] = [
    0xFFFF, 0xFF00, 0xFE01, 0xFD04, 0xFC07, 0xFB0C, 0xFA11, 0xF918, 0xF81F, 0xF727, 0xF631, 0xF53B,
    0xF446, 0xF352, 0xF25F, 0xF16D, 0xF07C, 0xEF8B, 0xEE9C, 0xEDAE, 0xECC0, 0xEBD3, 0xEAE8, 0xE9FD,
    0xE913, 0xE829, 0xE741, 0xE65A, 0xE573, 0xE48D, 0xE3A9, 0xE2C5, 0xE1E1, 0xE0FF, 0xE01E, 0xDF3D,
    0xDE5D, 0xDD7E, 0xDCA0, 0xDBC2, 0xDAE6, 0xDA0A, 0xD92F, 0xD854, 0xD77B, 0xD6A2, 0xD5CA, 0xD4F3,
    0xD41D, 0xD347, 0xD272, 0xD19E, 0xD0CB, 0xCFF8, 0xCF26, 0xCE55, 0xCD85, 0xCCB5, 0xCBE6, 0xCB18,
    0xCA4B, 0xC97E, 0xC8B2, 0xC7E7, 0xC71C, 0xC652, 0xC589, 0xC4C0, 0xC3F8, 0xC331, 0xC26B, 0xC1A5,
    0xC0E0, 0xC01C, 0xBF58, 0xBE95, 0xBDD2, 0xBD10, 0xBC4F, 0xBB8F, 0xBACF, 0xBA10, 0xB951, 0xB894,
    0xB7D6, 0xB71A, 0xB65E, 0xB5A2, 0xB4E8, 0xB42E, 0xB374, 0xB2BB, 0xB203, 0xB14B, 0xB094, 0xAFDE,
    0xAF28, 0xAE73, 0xADBE, 0xAD0A, 0xAC57, 0xABA4, 0xAAF1, 0xAA40, 0xA98E, 0xA8DE, 0xA82E, 0xA77E,
    0xA6D0, 0xA621, 0xA574, 0xA4C6, 0xA41A, 0xA36E, 0xA2C2, 0xA217, 0xA16D, 0xA0C3, 0xA01A, 0x9F71,
    0x9EC8, 0x9E21, 0x9D79, 0x9CD3, 0x9C2D, 0x9B87, 0x9AE2, 0x9A3D, 0x9999, 0x98F6, 0x9852, 0x97B0,
    0x970E, 0x966C, 0x95CB, 0x952B, 0x948B, 0x93EB, 0x934C, 0x92AD, 0x920F, 0x9172, 0x90D4, 0x9038,
    0x8F9C, 0x8F00, 0x8E65, 0x8DCA, 0x8D30, 0x8C96, 0x8BFC, 0x8B64, 0x8ACB, 0x8A33, 0x899C, 0x8904,
    0x886E, 0x87D8, 0x8742, 0x86AD, 0x8618, 0x8583, 0x84F0, 0x845C, 0x83C9, 0x8336, 0x82A4, 0x8212,
    0x8181, 0x80F0, 0x8060, 0x7FD0, 0x7F40, 0x7EB1, 0x7E22, 0x7D93, 0x7D05, 0x7C78, 0x7BEB, 0x7B5E,
    0x7AD2, 0x7A46, 0x79BA, 0x792F, 0x78A4, 0x781A, 0x7790, 0x7706, 0x767D, 0x75F5, 0x756C, 0x74E4,
    0x745D, 0x73D5, 0x734F, 0x72C8, 0x7242, 0x71BC, 0x7137, 0x70B2, 0x702E, 0x6FA9, 0x6F26, 0x6EA2,
    0x6E1F, 0x6D9C, 0x6D1A, 0x6C98, 0x6C16, 0x6B95, 0x6B14, 0x6A94, 0x6A13, 0x6993, 0x6914, 0x6895,
    0x6816, 0x6798, 0x6719, 0x669C, 0x661E, 0x65A1, 0x6524, 0x64A8, 0x642C, 0x63B0, 0x6335, 0x62BA,
    0x623F, 0x61C5, 0x614B, 0x60D1, 0x6058, 0x5FDF, 0x5F66, 0x5EED, 0x5E75, 0x5DFD, 0x5D86, 0x5D0F,
    0x5C98, 0x5C22, 0x5BAB, 0x5B35, 0x5AC0, 0x5A4B, 0x59D6, 0x5961, 0x58ED, 0x5879, 0x5805, 0x5791,
    0x571E, 0x56AC, 0x5639, 0x55C7, 0x5555, 0x54E3, 0x5472, 0x5401, 0x5390, 0x5320, 0x52AF, 0x5240,
    0x51D0, 0x5161, 0x50F2, 0x5083, 0x5015, 0x4FA6, 0x4F38, 0x4ECB, 0x4E5E, 0x4DF1, 0x4D84, 0x4D17,
    0x4CAB, 0x4C3F, 0x4BD3, 0x4B68, 0x4AFD, 0x4A92, 0x4A27, 0x49BD, 0x4953, 0x48E9, 0x4880, 0x4817,
    0x47AE, 0x4745, 0x46DC, 0x4674, 0x460C, 0x45A5, 0x453D, 0x44D6, 0x446F, 0x4408, 0x43A2, 0x433C,
    0x42D6, 0x4270, 0x420B, 0x41A6, 0x4141, 0x40DC, 0x4078, 0x4014, 0x3FB0, 0x3F4C, 0x3EE8, 0x3E85,
    0x3E22, 0x3DC0, 0x3D5D, 0x3CFB, 0x3C99, 0x3C37, 0x3BD6, 0x3B74, 0x3B13, 0x3AB2, 0x3A52, 0x39F1,
    0x3991, 0x3931, 0x38D2, 0x3872, 0x3813, 0x37B4, 0x3755, 0x36F7, 0x3698, 0x363A, 0x35DC, 0x357F,
    0x3521, 0x34C4, 0x3467, 0x340A, 0x33AE, 0x3351, 0x32F5, 0x3299, 0x323E, 0x31E2, 0x3187, 0x312C,
    0x30D1, 0x3076, 0x301C, 0x2FC2, 0x2F68, 0x2F0E, 0x2EB4, 0x2E5B, 0x2E02, 0x2DA9, 0x2D50, 0x2CF8,
    0x2C9F, 0x2C47, 0x2BEF, 0x2B97, 0x2B40, 0x2AE8, 0x2A91, 0x2A3A, 0x29E4, 0x298D, 0x2937, 0x28E0,
    0x288B, 0x2835, 0x27DF, 0x278A, 0x2735, 0x26E0, 0x268B, 0x2636, 0x25E2, 0x258D, 0x2539, 0x24E5,
    0x2492, 0x243E, 0x23EB, 0x2398, 0x2345, 0x22F2, 0x22A0, 0x224D, 0x21FB, 0x21A9, 0x2157, 0x2105,
    0x20B4, 0x2063, 0x2012, 0x1FC1, 0x1F70, 0x1F1F, 0x1ECF, 0x1E7F, 0x1E2E, 0x1DDF, 0x1D8F, 0x1D3F,
    0x1CF0, 0x1CA1, 0x1C52, 0x1C03, 0x1BB4, 0x1B66, 0x1B17, 0x1AC9, 0x1A7B, 0x1A2D, 0x19E0, 0x1992,
    0x1945, 0x18F8, 0x18AB, 0x185E, 0x1811, 0x17C4, 0x1778, 0x172C, 0x16E0, 0x1694, 0x1648, 0x15FD,
    0x15B1, 0x1566, 0x151B, 0x14D0, 0x1485, 0x143B, 0x13F0, 0x13A6, 0x135C, 0x1312, 0x12C8, 0x127F,
    0x1235, 0x11EC, 0x11A3, 0x1159, 0x1111, 0x10C8, 0x107F, 0x1037, 0x0FEF, 0x0FA6, 0x0F5E, 0x0F17,
    0x0ECF, 0x0E87, 0x0E40, 0x0DF9, 0x0DB2, 0x0D6B, 0x0D24, 0x0CDD, 0x0C97, 0x0C50, 0x0C0A, 0x0BC4,
    0x0B7E, 0x0B38, 0x0AF2, 0x0AAD, 0x0A68, 0x0A22, 0x09DD, 0x0998, 0x0953, 0x090F, 0x08CA, 0x0886,
    0x0842, 0x07FD, 0x07B9, 0x0776, 0x0732, 0x06EE, 0x06AB, 0x0668, 0x0624, 0x05E1, 0x059E, 0x055C,
    0x0519, 0x04D6, 0x0494, 0x0452, 0x0410, 0x03CE, 0x038C, 0x034A, 0x0309, 0x02C7, 0x0286, 0x0245,
    0x0204, 0x01C3, 0x0182, 0x0141, 0x0101, 0x00C0, 0x0080, 0x0040,
];

const SQRT_TABLE: [u16; 512] = [
    0x6A09, 0xFFFF, 0x6955, 0xFF00, 0x68A1, 0xFE02, 0x67EF, 0xFD06, 0x673E, 0xFC0B, 0x668D, 0xFB12,
    0x65DE, 0xFA1A, 0x6530, 0xF923, 0x6482, 0xF82E, 0x63D6, 0xF73B, 0x632B, 0xF648, 0x6280, 0xF557,
    0x61D7, 0xF467, 0x612E, 0xF379, 0x6087, 0xF28C, 0x5FE0, 0xF1A0, 0x5F3A, 0xF0B6, 0x5E95, 0xEFCD,
    0x5DF1, 0xEEE5, 0x5D4E, 0xEDFF, 0x5CAC, 0xED19, 0x5C0B, 0xEC35, 0x5B6B, 0xEB52, 0x5ACB, 0xEA71,
    0x5A2C, 0xE990, 0x598F, 0xE8B1, 0x58F2, 0xE7D3, 0x5855, 0xE6F6, 0x57BA, 0xE61B, 0x5720, 0xE540,
    0x5686, 0xE467, 0x55ED, 0xE38E, 0x5555, 0xE2B7, 0x54BE, 0xE1E1, 0x5427, 0xE10D, 0x5391, 0xE039,
    0x52FC, 0xDF66, 0x5268, 0xDE94, 0x51D5, 0xDDC4, 0x5142, 0xDCF4, 0x50B0, 0xDC26, 0x501F, 0xDB59,
    0x4F8E, 0xDA8C, 0x4EFE, 0xD9C1, 0x4E6F, 0xD8F7, 0x4DE1, 0xD82D, 0x4D53, 0xD765, 0x4CC6, 0xD69E,
    0x4C3A, 0xD5D7, 0x4BAF, 0xD512, 0x4B24, 0xD44E, 0x4A9A, 0xD38A, 0x4A10, 0xD2C8, 0x4987, 0xD206,
    0x48FF, 0xD146, 0x4878, 0xD086, 0x47F1, 0xCFC7, 0x476B, 0xCF0A, 0x46E5, 0xCE4D, 0x4660, 0xCD91,
    0x45DC, 0xCCD6, 0x4558, 0xCC1B, 0x44D5, 0xCB62, 0x4453, 0xCAA9, 0x43D1, 0xC9F2, 0x434F, 0xC93B,
    0x42CF, 0xC885, 0x424F, 0xC7D0, 0x41CF, 0xC71C, 0x4151, 0xC669, 0x40D2, 0xC5B6, 0x4055, 0xC504,
    0x3FD8, 0xC453, 0x3F5B, 0xC3A3, 0x3EDF, 0xC2F4, 0x3E64, 0xC245, 0x3DE9, 0xC198, 0x3D6E, 0xC0EB,
    0x3CF5, 0xC03F, 0x3C7C, 0xBF93, 0x3C03, 0xBEE9, 0x3B8B, 0xBE3F, 0x3B13, 0xBD96, 0x3A9C, 0xBCED,
    0x3A26, 0xBC46, 0x39B0, 0xBB9F, 0x393A, 0xBAF8, 0x38C5, 0xBA53, 0x3851, 0xB9AE, 0x37DD, 0xB90A,
    0x3769, 0xB867, 0x36F6, 0xB7C5, 0x3684, 0xB723, 0x3612, 0xB681, 0x35A0, 0xB5E1, 0x352F, 0xB541,
    0x34BF, 0xB4A2, 0x344F, 0xB404, 0x33DF, 0xB366, 0x3370, 0xB2C9, 0x3302, 0xB22C, 0x3293, 0xB191,
    0x3226, 0xB0F5, 0x31B9, 0xB05B, 0x314C, 0xAFC1, 0x30DF, 0xAF28, 0x3074, 0xAE8F, 0x3008, 0xADF7,
    0x2F9D, 0xAD60, 0x2F33, 0xACC9, 0x2EC8, 0xAC33, 0x2E5F, 0xAB9E, 0x2DF6, 0xAB09, 0x2D8D, 0xAA75,
    0x2D24, 0xA9E1, 0x2CBC, 0xA94E, 0x2C55, 0xA8BC, 0x2BEE, 0xA82A, 0x2B87, 0xA799, 0x2B21, 0xA708,
    0x2ABB, 0xA678, 0x2A55, 0xA5E8, 0x29F0, 0xA559, 0x298B, 0xA4CB, 0x2927, 0xA43D, 0x28C3, 0xA3B0,
    0x2860, 0xA323, 0x27FD, 0xA297, 0x279A, 0xA20B, 0x2738, 0xA180, 0x26D6, 0xA0F6, 0x2674, 0xA06C,
    0x2613, 0x9FE2, 0x25B2, 0x9F59, 0x2552, 0x9ED1, 0x24F2, 0x9E49, 0x2492, 0x9DC2, 0x2432, 0x9D3B,
    0x23D3, 0x9CB4, 0x2375, 0x9C2F, 0x2317, 0x9BA9, 0x22B9, 0x9B25, 0x225B, 0x9AA0, 0x21FE, 0x9A1C,
    0x21A1, 0x9999, 0x2145, 0x9916, 0x20E8, 0x9894, 0x208D, 0x9812, 0x2031, 0x9791, 0x1FD6, 0x9710,
    0x1F7B, 0x968F, 0x1F21, 0x960F, 0x1EC7, 0x9590, 0x1E6D, 0x9511, 0x1E13, 0x9492, 0x1DBA, 0x9414,
    0x1D61, 0x9397, 0x1D09, 0x931A, 0x1CB1, 0x929D, 0x1C59, 0x9221, 0x1C01, 0x91A5, 0x1BAA, 0x9129,
    0x1B53, 0x90AF, 0x1AFC, 0x9034, 0x1AA6, 0x8FBA, 0x1A50, 0x8F40, 0x19FA, 0x8EC7, 0x19A5, 0x8E4F,
    0x1950, 0x8DD6, 0x18FB, 0x8D5E, 0x18A7, 0x8CE7, 0x1853, 0x8C70, 0x17FF, 0x8BF9, 0x17AB, 0x8B83,
    0x1758, 0x8B0D, 0x1705, 0x8A98, 0x16B2, 0x8A23, 0x1660, 0x89AE, 0x160D, 0x893A, 0x15BC, 0x88C6,
    0x156A, 0x8853, 0x1519, 0x87E0, 0x14C8, 0x876D, 0x1477, 0x86FB, 0x1426, 0x8689, 0x13D6, 0x8618,
    0x1386, 0x85A7, 0x1337, 0x8536, 0x12E7, 0x84C6, 0x1298, 0x8456, 0x1249, 0x83E7, 0x11FB, 0x8377,
    0x11AC, 0x8309, 0x115E, 0x829A, 0x1111, 0x822C, 0x10C3, 0x81BF, 0x1076, 0x8151, 0x1029, 0x80E4,
    0x0FDC, 0x8078, 0x0F8F, 0x800C, 0x0F43, 0x7FA0, 0x0EF7, 0x7F34, 0x0EAB, 0x7EC9, 0x0E60, 0x7E5E,
    0x0E15, 0x7DF4, 0x0DCA, 0x7D8A, 0x0D7F, 0x7D20, 0x0D34, 0x7CB6, 0x0CEA, 0x7C4D, 0x0CA0, 0x7BE5,
    0x0C56, 0x7B7C, 0x0C0C, 0x7B14, 0x0BC3, 0x7AAC, 0x0B7A, 0x7A45, 0x0B31, 0x79DE, 0x0AE8, 0x7977,
    0x0AA0, 0x7911, 0x0A58, 0x78AB, 0x0A10, 0x7845, 0x09C8, 0x77DF, 0x0981, 0x777A, 0x0939, 0x7715,
    0x08F2, 0x76B1, 0x08AB, 0x764D, 0x0865, 0x75E9, 0x081E, 0x7585, 0x07D8, 0x7522, 0x0792, 0x74BF,
    0x074D, 0x745D, 0x0707, 0x73FA, 0x06C2, 0x7398, 0x067D, 0x7337, 0x0638, 0x72D5, 0x05F3, 0x7274,
    0x05AF, 0x7213, 0x056A, 0x71B3, 0x0526, 0x7152, 0x04E2, 0x70F2, 0x049F, 0x7093, 0x045B, 0x7033,
    0x0418, 0x6FD4, 0x03D5, 0x6F76, 0x0392, 0x6F17, 0x0350, 0x6EB9, 0x030D, 0x6E5B, 0x02CB, 0x6DFD,
    0x0289, 0x6DA0, 0x0247, 0x6D43, 0x0206, 0x6CE6, 0x01C4, 0x6C8A, 0x0183, 0x6C2D, 0x0142, 0x6BD1,
    0x0101, 0x6B76, 0x00C0, 0x6B1A, 0x0080, 0x6ABF, 0x0040, 0x6A64,
];

fn vrcp_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vt = op.vtv(s);

    let input = vt[(op.element() & 7) as usize] as i32;

    // Zero as input is a special case that produces the maximum signed value
    let result = if input == 0 {
        i32::MAX as u32
    } else {
        // Compute the absolute value

        let abs = input.unsigned_abs();

        // Shift the absolute value so that the MSB is at bit 15

        let shift = abs.leading_zeros();
        let shifted = abs << shift;

        // Extract the index into the reciprocal table, ie. the 9 bits right after the MSB

        let index = (shifted >> 22) & 0x1FF;

        // Get the result from the lookup table, shift it back, restore the implicit MSB

        let table_entry = RECIPROCAL_TABLE[index as usize] as u32;

        let result = (0x4000_0000 | (table_entry << 14)) >> (shift ^ 31);

        // Restore the input sign

        if input < 0 { !result } else { result }
    };

    // The accumulator is loaded with broadcast vt (before updating it with the result!)

    write_acc_lo(s, op.vtv_broadcast(s));

    // Store the full 32-bit result in DIV OUT

    s.sp.div_out = result as i32;

    // Store the low bits of the result in vd

    s.sp.vregs[op.vd()][op.de() & 7] = result as i16;

    // TODO dp?

    None
}

fn vrcp_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VRCP v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

fn vrcph_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // VRCPH does not calculate anything:
    // - stores the high bits of the last result (DIV OUT) into vd
    // - loads the next input value into DIV IN
    // - broadcasts the next input value into the accumulator

    let input = op.vtv(s)[(op.element() & 7) as usize];

    s.sp.div_in = (input as i32) << 16;

    write_acc_lo(s, op.vtv_broadcast(s));

    s.sp.vregs[op.vd()][op.de() & 7] = (s.sp.div_out >> 16) as i16;

    // TODO dp???

    None
}

fn vrcph_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VRCPH v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

// TODO generic func for vrcp/vrcpl
fn vrcpl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // TODO should be at least a bit different for vrcp?!

    let vt = op.vtv(s);

    let input = vt[(op.element() & 7) as usize] as i32;

    // Zero as input is a special case that produces the maximum signed value
    let result = if input == 0 {
        i32::MAX as u32
    } else {
        // Compute the absolute value

        let abs = input.unsigned_abs();

        // Shift the absolute value so that the MSB is at bit 15

        let shift = abs.leading_zeros();
        let shifted = abs << shift;

        // Extract the index into the reciprocal table, ie. the 9 bits right after the MSB

        let index = (shifted >> 22) & 0x1FF;

        // Get the result from the lookup table, shift it back, restore the implicit MSB

        let table_entry = RECIPROCAL_TABLE[index as usize] as u32;

        let result = (0x4000_0000 | (table_entry << 14)) >> (shift ^ 31);

        // Restore the input sign

        if input < 0 { !result } else { result }
    };

    // The accumulator is loaded with broadcast vt (before updating it with the result!)

    write_acc_lo(s, op.vtv_broadcast(s));

    // Store the full 32-bit result in DIV OUT

    s.sp.div_out = result as i32;

    // Store the low bits of the result in vd

    s.sp.vregs[op.vd()][op.de() & 7] = result as i16;

    // TODO dp?

    None
}

fn vrcpl_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VRCPL v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

// TODO clarify doc
fn vrsq_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vt = op.vtv(s);

    let input = vt[(op.element() & 7) as usize] as i32;

    // Zero as input is a special case that produces the maximum signed value
    let result = if input == 0 {
        i32::MAX as u32
    }
    // i16::MIN as input is another special case
    else if input == -32768 {
        0xFFFF_0000u32
    } else {
        // Compute the absolute value

        let abs = input.unsigned_abs();

        // Shift the absolute value so that the MSB is at bit 15

        let left_shift = abs.leading_zeros();
        let shifted = abs << left_shift;

        // Extract the index into the reciprocal table, ie. the 9 bits right after the MSB

        let index = (shifted >> 22) & 0x1FE | (left_shift & 1); // TODO when generalizing 1fE is important!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

        // Get the result from the lookup table, shift it back, restore the implicit MSB

        let table_entry = SQRT_TABLE[index as usize] as u32;

        let right_shift = (left_shift ^ 31) >> 1;

        let unshifted = (0x4000_0000 | (table_entry << 14)) >> right_shift;

        // Restore the input sign

        if input < 0 { !unshifted } else { unshifted }
    };

    // The accumulator is loaded with broadcast vt (before updating it with the result!)

    write_acc_lo(s, op.vtv_broadcast(s));

    // Store the full 32-bit result in DIV OUT

    s.sp.div_out = result as i32;

    // Store the low bits of the result in vd

    s.sp.vregs[op.vd()][op.de() & 7] = result as i16;

    // TODO dp?

    None
}

fn vrsq_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VRSQ v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

fn vrsqh_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let input = op.vtv(s)[(op.element() & 7) as usize];

    s.sp.div_in = (input as i32) << 16;

    write_acc_lo(s, op.vtv_broadcast(s));

    s.sp.vregs[op.vd()][op.de() & 7] = (s.sp.div_out >> 16) as i16;

    // TODO dp???

    None
}

fn vrsqh_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VRSQH v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

fn vrsql_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vt = op.vtv(s);

    let input = vt[(op.element() & 7) as usize] as i32;

    // Zero as input is a special case that produces the maximum signed value
    let result = if input == 0 {
        i32::MAX as u32
    }
    // i16::MIN as input is another special case
    else if input == -32768 {
        0xFFFF_0000u32
    } else {
        // Compute the absolute value

        let abs = input.unsigned_abs();

        // Shift the absolute value so that the MSB is at bit 15

        let left_shift = abs.leading_zeros();
        let shifted = abs << left_shift;

        // Extract the index into the reciprocal table, ie. the 9 bits right after the MSB

        let index = (shifted >> 22) & 0x1FE | (left_shift & 1); // TODO when generalizing 1fE is important!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

        // Get the result from the lookup table, shift it back, restore the implicit MSB

        let table_entry = SQRT_TABLE[index as usize] as u32;

        let right_shift = (left_shift ^ 31) >> 1;

        let unshifted = (0x4000_0000 | (table_entry << 14)) >> right_shift;

        // Restore the input sign

        if input < 0 { !unshifted } else { unshifted }
    };

    // The accumulator is loaded with broadcast vt (before updating it with the result!)

    write_acc_lo(s, op.vtv_broadcast(s));

    // Store the full 32-bit result in DIV OUT

    s.sp.div_out = result as i32;

    // Store the low bits of the result in vd

    s.sp.vregs[op.vd()][op.de() & 7] = result as i16;

    // TODO dp?

    None
}

fn vrsql_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "VRSQL v{}[{}], v{}[{}]",
        op.vd(),
        op.de(),
        op.vt(),
        op.element()
    )
}

/*
 * Logical
 */

// TODO generalize?

fn vand_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let result = vs & vt;

    s.sp.vregs[op.vd()] = vs & vt;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vand);

fn vnand_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let result = !(vs & vt);

    s.sp.vregs[op.vd()] = result;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vnand);

fn vor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let result = vs | vt;

    s.sp.vregs[op.vd()] = vs | vt;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vor);

fn vnor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let result = !(vs | vt);

    s.sp.vregs[op.vd()] = !(vs | vt);
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vnor);

fn vxor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let result = vs ^ vt;

    s.sp.vregs[op.vd()] = vs ^ vt;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vxor);

fn vnxor_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let result = !(vs ^ vt);

    s.sp.vregs[op.vd()] = result;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vnxor);

// ----------
// Arithmetic
// ----------

fn vabs_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Negate vt's lanes depending on the sign of vs.
    // If vs is zero, set the result to zero.

    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let zeroed = vs.simd_eq(ZEROS).select(ZEROS, vt);

    // The wrapped negated result goes into the accumulator.
    // The saturated negated result goes into the destination register.
    // This matter for 0x8000 which negates to 0x8000 (wrapped) / 0x7FFF (saturated).

    let negated_wrap = vs.simd_lt(ZEROS).select(-zeroed, zeroed);
    write_acc_lo(s, negated_wrap);

    let negated_sat = vs.simd_lt(ZEROS).select(zeroed.saturating_neg(), zeroed);
    s.sp.vregs[op.vd()] = negated_sat;

    None
}

disassembly_vd_vs_vte!(vabs);

fn vadd_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Add vt to vs with carry in, store in acc (wrapped) and vd (clamped), clear the carry

    let vs_i32 = op.vsv(s).cast::<i32>();
    let vt_i32 = op.vtv_broadcast(s).cast::<i32>();

    let vco = Mask::<i32, 8>::from_bitmask((s.sp.vco & 0xFF) as u64)
        .select(i32x8::splat(1), i32x8::splat(0));

    let wrapped_i32 = vs_i32 + vt_i32 + vco;
    write_acc_lo(s, wrapped_i32.cast::<i16>());

    let clamped_i32 = wrapped_i32.simd_clamp(i32x8::splat(-32768), i32x8::splat(32767));
    s.sp.vregs[op.vd()] = clamped_i32.cast::<i16>();

    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vadd);

fn vaddc_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs_u16 = op.vsv(s).cast::<u16>();
    let vt_u16 = op.vtv_broadcast(s).cast::<u16>();

    let result_u16 = vs_u16 + vt_u16;
    write_acc_lo(s, result_u16.cast::<i16>());
    s.sp.vregs[op.vd()] = result_u16.cast::<i16>();

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

    underflow.select(min, overflow.select(max, portion))
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

fn vsub_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Subtract vt from vs with borrow in, store in acc (wrapped) and vd (clamped), clear the carry

    let vs_i32 = op.vsv(s).cast::<i32>();
    let vt_i32 = op.vtv_broadcast(s).cast::<i32>();

    let vco_i32 = Mask::<i32, 8>::from_bitmask(s.sp.vco as u64).select(ONES32, ZEROS32);

    let wrapped_i32 = vs_i32 - vt_i32 - vco_i32;
    write_acc_lo(s, wrapped_i32.cast::<i16>());

    let clamped_i32 = wrapped_i32.simd_clamp(i32x8::splat(-32768), i32x8::splat(32767));
    s.sp.vregs[op.vd()] = clamped_i32.cast::<i16>();

    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vsub);

fn vsubc_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Subtract vt from vs, store in acc and vd (both wrapped), update VCO

    let vs_u16 = op.vsv(s).cast::<u16>();
    let vt_u16 = op.vtv_broadcast(s).cast::<u16>();

    let result_u16 = vs_u16 - vt_u16;
    write_acc_lo(s, result_u16.cast::<i16>());
    s.sp.vregs[op.vd()] = result_u16.cast::<i16>();

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
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<i32>();

    let product = (vs32 * vt32).cast::<i64>() << 1;

    if ADD_ACC {
        s.sp.vacc += product;
    } else {
        s.sp.vacc = product + i64x8::splat(0x8000);
    }

    s.sp.vregs[op.vd()] = if CLAMP_SIGNED {
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
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<u16>().cast::<u32>();
    let vt32 = vt.cast::<u16>().cast::<u32>();

    s.sp.vacc = ((vs32 * vt32) >> 16).cast::<i64>();

    s.sp.vregs[op.vd()] = s.sp.vacc.cast::<i16>();

    None
}

disassembly_vd_vs_vte!(vmudl);

fn vmadl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<u16>().cast::<u32>();
    let vt32 = vt.cast::<u16>().cast::<u32>();

    s.sp.vacc += ((vs32 * vt32) >> 16).cast::<i64>();

    s.sp.vregs[op.vd()] = clamp_acc_signed_low(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadl);

fn vmudn_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<u16>().cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc = (vs32 * vt32).cast::<i64>();

    s.sp.vregs[op.vd()] = clamp_acc_signed_low(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmudn);

fn vmadn_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<u16>().cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc += (vs32 * vt32).cast::<i64>();

    s.sp.vregs[op.vd()] = clamp_acc_signed_low(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadn);

fn vmudm_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<u16>().cast::<i32>();

    s.sp.vacc = (vs32 * vt32).cast::<i64>();

    s.sp.vregs[op.vd()] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmudm);

fn vmadm_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<u16>().cast::<i32>();

    s.sp.vacc += (vs32 * vt32).cast::<i64>();

    s.sp.vregs[op.vd()] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmadm);

fn vmudh_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc = (vs32 * vt32).cast::<i64>() << 16;

    s.sp.vregs[op.vd()] = clamp_acc_signed_mid(&s.sp.vacc);

    None
}

disassembly_vd_vs_vte!(vmudh);

fn vmadh_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vs32 = vs.cast::<i32>();
    let vt32 = vt.cast::<i32>();

    s.sp.vacc += (vs32 * vt32).cast::<i64>() << 16;

    s.sp.vregs[op.vd()] = clamp_acc_signed_mid(&s.sp.vacc);

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
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

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

    s.sp.vregs[op.vd()] = result;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vch);

fn vcl_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    // let vsval = broadcast(velement(op), s.sp.vregs[op.vt()]);
    // let vtval = s.sp.vregs[op.vs()];
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

    write_acc_lo(s, result.cast::<i16>());
    s.sp.vco = 0;
    s.sp.vce = 0;

    // const ZERO: i16x8 = i16x8::splat(0);

    // let mut ge: Mask<i16, 8> = Mask::from_bitmask(s.sp.vcc as u64 >> 8);
    // let mut le: Mask<i16, 8> = Mask::from_bitmask(s.sp.vcc as u64 & 0xFF);
    // let eq: Mask<i16, 8> = !Mask::from_bitmask(s.sp.vco as u64 >> 8);
    // let diff_sign: Mask<i16, 8> = Mask::from_bitmask(s.sp.vco as u64 & 0xFF);
    // let vce: Mask<i16, 8> = Mask::from_bitmask(s.sp.vce as u64);

    // let sum = vs.cast::<u16>() + vt.cast::<u16>();

    // let carry = sum.simd_lt(vs.cast::<u16>());
    // // let carry = (vs.cast::<i32>() + vt.cast::<i32>())
    // //     .simd_ne(sum.cast::<i32>())
    // //     .cast::<i16>();

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

    // s.sp.vregs[op.vd()] = result;
    // s.sp.vacc = s.sp.vacc & i64x8::splat(!0xFFFF) | result.cast::<u16>().cast::<i64>();

    None
}

disassembly_vd_vs_vte!(vcl);

fn vcr_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let diff_sign = (vs ^ vt).simd_lt(ZEROS);
    let sum = vs + vt;
    let diff = vs - vt;

    let ge = diff_sign.select(vt.simd_lt(ZEROS), diff.simd_ge(ZEROS));
    let le = diff_sign.select(sum.simd_lt(ZEROS), vt.simd_lt(ZEROS));

    s.sp.vcc = ((ge.to_bitmask() as u8 as u16) << 8) | (le.to_bitmask() as u8 as u16);
    s.sp.vco = 0;
    s.sp.vce = 0;

    let vc = diff_sign.select(!vt, vt);
    let result = diff_sign.select(le.select(vc, vs), ge.select(vc, vs));

    s.sp.vregs[op.vd()] = result;
    write_acc_lo(s, result);

    None
}

disassembly_vd_vs_vte!(vcr);

fn vmrg_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let mask = Mask::<i16, 8>::from_bitmask(s.sp.vcc as u64);

    let result = mask.select(vs, vt);

    write_acc_lo(s, result);
    s.sp.vregs[op.vd()] = result;

    s.sp.vco = 0; // Manual error: VCO is actuallycleared

    None
}

disassembly_vd_vs_vte!(vmrg);

placeholder!(vrndn);
placeholder!(vrndp);

// -----------
// Comparisons
// -----------

fn veq_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Compare vs and vt taking VCO into account, store the result in acc and vd, clear VCO.
    // The manual states that "VCO and VCE are used as input" but the bit about VCE looks like an error.

    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let equal = vs.simd_eq(vt) & !Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64);

    let result = equal.select(vs, vt);

    write_acc_lo(s, result);
    s.sp.vregs[op.vd()] = result;

    s.sp.vcc = equal.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(veq);

fn vne_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Similar to VEQ, VCE seems unused

    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let not_equal = vs.simd_ne(vt) | Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64);

    let result = not_equal.select(vs, vt);

    write_acc_lo(s, result);
    s.sp.vregs[op.vd()] = result;

    s.sp.vcc = not_equal.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vne);

fn vge_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Similar to VEQ and VNE, still no VCE in sight, the condition takes both halves of VCO into account

    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vco_mask = Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64)
        & Mask::<i16, 8>::from_bitmask(s.sp.vco as u64);

    let ge = vs.simd_gt(vt) | vs.simd_eq(vt) & !vco_mask;

    let result = ge.select(vs, vt);

    write_acc_lo(s, result);
    s.sp.vregs[op.vd()] = result;

    s.sp.vcc = ge.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vge);

fn vlt_execute(s: &mut System, op: Opcode) -> Option<InstructionEffect> {
    // Similar to VGE

    let vs = op.vsv(s);
    let vt = op.vtv_broadcast(s);

    let vco_mask = Mask::<i16, 8>::from_bitmask((s.sp.vco >> 8) as u64)
        & Mask::<i16, 8>::from_bitmask(s.sp.vco as u64);

    let lt = vs.simd_lt(vt) | vs.simd_eq(vt) & vco_mask;

    let result = lt.select(vs, vt);

    write_acc_lo(s, result);
    s.sp.vregs[op.vd()] = result;

    s.sp.vcc = lt.to_bitmask() as u16;
    s.sp.vco = 0;

    None
}

disassembly_vd_vs_vte!(vlt);
