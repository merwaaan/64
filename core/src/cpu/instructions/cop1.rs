use arbitrary_int::prelude::*;
use rustc_apfloat::{Float, FloatConvert, Status, StatusAnd, ieee::Single};

use crate::{
    check_cop_usable,
    cop1::{self, Cause, Format, Interrupt, RoundingMode},
    cpu::{
        instructions::{Instruction, InstructionEffect, InstructionResult, Reserved},
        opcode::Opcode,
        operands::Operands,
    },
    exception::Exception,
    system::System,
};

#[doc(hidden)]
#[macro_export]
macro_rules! decode_cop1_inst_fmt {
    ($opcode:expr, $m:ident, $ty:path; $($fmt:path),* $(,)?) => {{
        match $opcode.cop1_format() {
            $( Some($fmt) )|* => $m!($ty),
            _ => $crate::cpu::instructions::RESERVED_INSTRUCTION,
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! decode_cop1_inst_fmt_fp {
    ($opcode:expr, $ty:path) => {{
        match $opcode.cop1_format() {
            Some($crate::cop1::Format::Float32) => (<$ty>::execute::<u32>, <$ty>::disassemble),
            Some($crate::cop1::Format::Float64) => (<$ty>::execute::<u64>, <$ty>::disassemble),
            _ => $crate::cpu::instructions::RESERVED_INSTRUCTION,
        }
    }};
}

#[macro_export]
macro_rules! decode_cop1_x {
    ($opcode:expr, $m:ident) => {{
        debug_assert_eq!($opcode.group(), 0x11);

        // TODO can avoid & 1F as they all have the same prefix

        match ($opcode.0 >> 21) & 0x1F {
            0x00 => $m!(crate::cpu::instructions::cop1::Mfc1),
            0x01 => $m!(crate::cpu::instructions::cop1::Dmfc1),
            0x02 => $m!(crate::cpu::instructions::cop1::Cfc1),
            0x04 => $m!(crate::cpu::instructions::cop1::Mtc1),
            0x05 => $m!(crate::cpu::instructions::cop1::Dmtc1),
            0x06 => $m!(crate::cpu::instructions::cop1::Ctc1),
            0x08 => match ($opcode.0 >> 16) & 0x1F {
                0x00 => $m!(crate::cpu::instructions::cop1::Bc1f),
                0x01 => $m!(crate::cpu::instructions::cop1::Bc1t),
                0x02 => $m!(crate::cpu::instructions::cop1::Bc1fl),
                0x03 => $m!(crate::cpu::instructions::cop1::Bc1tl),
                _ => $m!(crate::cpu::instructions::Reserved),
            },
            _ => match $opcode.0 & 0x3F {
                0x00 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Add),
                0x01 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Sub),
                0x02 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Mul),
                0x03 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Div),
                0x04 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Sqrt),
                0x05 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Abs),
                0x06 => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Mov;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x07 => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::Neg),
                0x08 => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Round;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ), // TODO duplicates??
                0x09 => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Trunc;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x0A => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Ceil;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x0B => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Floor;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x0C => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Round;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x0D => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Trunc;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x0E => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Ceil;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x0F => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Floor;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64
                ),
                0x20 | 0x21 | 0x24 | 0x25 => decode_cop1_inst_fmt!(
                    $opcode,
                    $m,
                    $crate::cpu::instructions::cop1::Cvt;
                    $crate::cop1::Format::Float32,
                    $crate::cop1::Format::Float64,
                    $crate::cop1::Format::Int32,
                    $crate::cop1::Format::Int64
                ),
                0x30..=0x3F => decode_cop1_inst_fmt_fp!($opcode, $crate::cpu::instructions::cop1::C),
                _ => $crate::cpu::instructions::RESERVED_INSTRUCTION,
            },
        }
    }};
}

fn set_exception_cause(s: &mut System, cause: Cause) -> Result<(), Exception> {
    s.cop1.fcr31.set_exception_cause(cause);

    if cause.raw_value().value() != 0 {
        let cause = cause.raw_value().value();
        let enabled = s.cop1.fcr31.exception_enabled().raw_value().value() | 0x20u8; // "unimplemented operation" is always enabled
        let flags = s.cop1.fcr31.exception_flags().raw_value().value();

        // Update the flags with the masked exceptions

        let masked = cause & !enabled;

        s.cop1
            .fcr31
            .set_exception_flags(Interrupt::new_with_raw_value(u5::new(flags | masked)));

        // Raise an exception if there are any enabled exception

        let unmasked = cause & enabled;

        if unmasked != 0 {
            return Err(Exception::FloatingPoint);
        }
    }

    Ok(())
}

pub struct Cfc1;

impl Instruction for Cfc1 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        // TODO enforce 0-31 in decode?

        check_cop_usable!(1, s);

        match operands.fs() {
            0 => s.cpu.regs.gpr[operands.rt()].set(s.cop1.fcr0()),
            31 => s.cpu.regs.gpr[operands.rt()].set(s.cop1.fcr31.read()),
            _ => unreachable!("CFC1 with invalid fs {}", operands.fs()),
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("CFC1 {}, {}", operands.rtn(), operands.fsn())
    }
}

pub struct Ctc1;

impl Instruction for Ctc1 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        // TODO enforce 0-31 in decode?

        check_cop_usable!(1, s);

        match operands.fs() {
            0 => { /* read-only */ }
            31 => s.cop1.fcr31.write(operands.rtv(s)),
            _ => unreachable!("CTC1 with invalid fs {}", operands.fs()),
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("CTC1 {}, FCR{}", operands.rtn(), operands.fs())
    }
}

pub struct Dmfc1;

impl Instruction for Dmfc1 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        let value = s.cop1.get64(operands.fs(), s.cop0.f64());

        s.cpu.regs.gpr[operands.rt()].set64(value);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMFC1 {}, {}", operands.rtn(), operands.fsn())
    }
}

pub struct Dmtc1;

impl Instruction for Dmtc1 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        s.cop1.set64(operands.fs(), operands.rtv64(s), s.cop0.f64());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMTC1 {}, {}", operands.rtn(), operands.fsn())
    }
}

pub struct Mfc1;

impl Instruction for Mfc1 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        let value = s.cop1.get32(operands.fs(), s.cop0.f64());

        s.cpu.regs.gpr[operands.rt()].set(value);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MFC1 {}, {}", operands.rtn(), operands.fsn())
    }
}

pub struct Mov;

impl Instruction for Mov {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        match opcode.cop1_format() {
            Some(Format::Float32) => s.cop1.set32(operands.fd(), operands.fsv(s), s.cop0.f64()),
            Some(Format::Float64) => s.cop1.set64(operands.fd(), operands.fsv64(s), s.cop0.f64()),
            _ => unimplemented!("MOV with invalid format {:08X}", opcode.0),
        }

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "MOV.{} {},{}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn()
        )
    }
}

pub struct Mtc1;

impl Instruction for Mtc1 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        s.cop1.set32(operands.fs(), operands.rtv(s), s.cop0.f64());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MTC1 {}, {}", operands.rtn(), operands.fsn())
    }
}

// --------
// Rounding
// --------

trait Rounding {
    const L_MASK: u32;
    const NAME: &'static str;

    fn apply32(value: f32) -> f32;
    fn apply64(value: f64) -> f64;
}

pub(crate) struct Ceil;

impl Rounding for Ceil {
    const L_MASK: u32 = 0b001010;
    const NAME: &'static str = "CEIL";

    fn apply32(value: f32) -> f32 {
        value.ceil()
    }

    fn apply64(value: f64) -> f64 {
        value.ceil()
    }
}

pub(crate) struct Floor;

impl Rounding for Floor {
    const L_MASK: u32 = 0b001011;
    const NAME: &'static str = "FLOOR";

    fn apply32(value: f32) -> f32 {
        value.floor()
    }

    fn apply64(value: f64) -> f64 {
        value.floor()
    }
}

pub(crate) struct Trunc;

impl Rounding for Trunc {
    const L_MASK: u32 = 0b001001;
    const NAME: &'static str = "TRUNC";

    fn apply32(value: f32) -> f32 {
        value.trunc()
    }

    fn apply64(value: f64) -> f64 {
        value.trunc()
    }
}

pub(crate) struct Round;

impl Rounding for Round {
    const L_MASK: u32 = 0b001000;
    const NAME: &'static str = "ROUND";

    fn apply32(value: f32) -> f32 {
        value.round_ties_even()
    }

    fn apply64(value: f64) -> f64 {
        value.round_ties_even()
    }
}

// TODO const generics?
fn generic_rounding_execute<ROUNDING: Rounding>(
    s: &mut System,
    opcode: Opcode,
    operands: Operands,
) -> InstructionResult {
    check_cop_usable!(1, s);

    let input_format = opcode.cop1_format().unwrap();

    let output_format = if (opcode.0 & ROUNDING::L_MASK) == ROUNDING::L_MASK {
        Format::Int64
    } else {
        Format::Int32
    };

    match (input_format, output_format) {
        (Format::Float32, Format::Int32) => {
            s.cop1.set32(
                operands.fd(),
                ROUNDING::apply32(f32::from_bits(operands.fsv(s))) as i32 as u32,
                s.cop0.f64(),
            );
        }
        (Format::Float32, Format::Int64) => {
            s.cop1.set64(
                operands.fd(),
                ROUNDING::apply32(f32::from_bits(operands.fsv(s))) as i64 as u64,
                s.cop0.f64(),
            );
        }
        (Format::Float64, Format::Int32) => {
            s.cop1.set32(
                operands.fd(),
                ROUNDING::apply64(f64::from_bits(operands.fsv64(s))) as i32 as u32,
                s.cop0.f64(),
            );
        }
        (Format::Float64, Format::Int64) => {
            s.cop1.set64(
                operands.fd(),
                ROUNDING::apply64(f64::from_bits(operands.fsv64(s))) as i64 as u64,
                s.cop0.f64(),
            );
        }
        _ => unimplemented!("{}.{}.{}", ROUNDING::NAME, output_format, input_format),
    }

    set_exception_cause(s, Cause::default())?;

    Ok(None)
}

fn generic_rounding_disassemble<ROUNDING: Rounding>(
    _s: &System,
    opcode: Opcode,
    operands: Operands,
) -> String {
    let output_format = if (opcode.0 & ROUNDING::L_MASK) == ROUNDING::L_MASK {
        Format::Int64
    } else {
        Format::Int32
    };

    format!(
        "{}.{}.{} {},{}",
        ROUNDING::NAME,
        output_format,
        opcode.cop1_format().unwrap(),
        operands.fdn(),
        operands.fsn()
    )
}

impl Instruction for Ceil {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        generic_rounding_execute::<Ceil>(s, opcode, operands)
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        generic_rounding_disassemble::<Ceil>(s, opcode, operands)
    }
}

impl Instruction for Floor {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        generic_rounding_execute::<Floor>(s, opcode, operands)
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        generic_rounding_disassemble::<Floor>(s, opcode, operands)
    }
}

impl Instruction for Round {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        generic_rounding_execute::<Round>(s, opcode, operands)
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        generic_rounding_disassemble::<Round>(s, opcode, operands)
    }
}

impl Instruction for Trunc {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        generic_rounding_execute::<Trunc>(s, opcode, operands)
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        generic_rounding_disassemble::<Trunc>(s, opcode, operands)
    }
}

// -----------
// Comparisons
// -----------

pub struct C;

impl C {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        check_cop_usable!(1, s);

        let fs = T::read_reg(s, operands.fs());
        let ft = T::read_reg(s, operands.ft());

        let qnan = (fs.is_nan() && !fs.is_signaling()) || (ft.is_nan() && !ft.is_signaling());
        let unordered = fs.is_nan() || ft.is_nan();

        let (result, signal) = match opcode.cop1_comparison() {
            // Non-signalling: raise exceptions on qNaN only
            cop1::Comparison::False => (false, qnan),
            cop1::Comparison::Unordered => (unordered, qnan),
            cop1::Comparison::Equal => (!unordered && fs == ft, qnan),
            cop1::Comparison::UnorderedOrEqual => (unordered || fs == ft, qnan),
            cop1::Comparison::OrderedLess => (!unordered && fs < ft, qnan),
            cop1::Comparison::UnorderedOrLess => (unordered || fs < ft, qnan),
            cop1::Comparison::OrderedLessOrEqual => (!unordered && fs <= ft, qnan),
            cop1::Comparison::UnorderedOrLessOrEqual => (unordered || fs <= ft, qnan),
            // Signalling: raise exceptions on unordered (ie. any NaN)
            cop1::Comparison::SignalingFalse => (false, unordered),
            cop1::Comparison::NotGreaterOrLessOrEqual => (unordered, unordered),
            cop1::Comparison::SignalingEqual => (!unordered && fs == ft, unordered),
            cop1::Comparison::NotGreaterOrLess => (unordered || fs == ft, unordered),
            cop1::Comparison::Less => (!unordered && fs < ft, unordered),
            cop1::Comparison::NotGreaterOrEqual => (unordered || fs < ft, unordered),
            cop1::Comparison::LessOrEqual => (!unordered && fs <= ft, unordered),
            cop1::Comparison::NotGreater => (unordered || fs <= ft, unordered),
        };

        set_exception_cause(s, Cause::default().with_invalid_operation(signal))?;

        s.cop1.fcr31.set_comparison_result(result);

        Ok(None)
    }

    pub fn disassemble(_s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!(
            "C.{}.{}",
            opcode.cop1_comparison(),
            opcode.cop1_format().unwrap(),
        )
    }
}

impl Instruction for C {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => C::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => C::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        C::disassemble(s, opcode, operands)
    }
}

pub struct Bc1f;

impl Instruction for Bc1f {
    fn execute(s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        if !s.cop1.fcr31.comparison_result() {
            Ok(Some(InstructionEffect::DelayedBranching(Some(
                opcode.branch_target(s),
            ))))
        } else {
            Ok(None)
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("BC1F {:#06X}", opcode.branch_offset())
    }
}

pub struct Bc1fl;

impl Instruction for Bc1fl {
    fn execute(s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        if !s.cop1.fcr31.comparison_result() {
            Ok(Some(InstructionEffect::DelayedBranching(Some(
                opcode.branch_target(s),
            ))))
        } else {
            // Discard the instruction in the delay slot TODO return special val??
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

            Ok(None)
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("BC1FL {:#06X}", opcode.branch_offset())
    }
}

pub struct Bc1t;

impl Instruction for Bc1t {
    fn execute(s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        if s.cop1.fcr31.comparison_result() {
            Ok(Some(InstructionEffect::DelayedBranching(Some(
                opcode.branch_target(s),
            ))))
        } else {
            Ok(None)
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("BC1T {:#06X}", opcode.branch_offset())
    }
}

pub struct Bc1tl;

impl Instruction for Bc1tl {
    fn execute(s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        if s.cop1.fcr31.comparison_result() {
            Ok(Some(InstructionEffect::DelayedBranching(Some(
                opcode.branch_target(s),
            ))))
        } else {
            // Discard the instruction in the delay slot TODO return special val??
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

            Ok(None)
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("BC1TL {:#06X}", opcode.branch_offset())
    }
}

// ---------------------
// Arithmetic operations
// ---------------------

const QNAN32: u32 = 0x7FBF_FFFF;
const QNAN64: u64 = 0x7FF7_FFFF_FFFF_FFFF;

/// Helper trait to perform arithmetic operations on either u32 or u64
pub trait SoftFloat {
    // TODO simplify constraints

    /// Underlying floating-point type type (eg. u32 -> f32, u64 -> f64)
    type Raw: num_traits::Float;

    /// Software floating point type
    type Soft: rustc_apfloat::Float;

    /// Reads a COP1 register and returns the corresponding soft float
    fn read_reg(s: &System, reg: usize) -> Self::Soft;

    /// Writes a soft float to a COP1 register
    fn write_reg(s: &mut System, reg: usize, value: Self::Soft);

    /// Computes a square root with status flags
    /// (rustc_apfloat doesn't have a sqrt implementation, so we use ieee_apsqrt)
    fn sqrt(
        value: Self::Soft,
        rounding: rustc_apfloat::Round,
    ) -> rustc_apfloat::StatusAnd<Self::Soft>;
}

impl SoftFloat for u32 {
    type Raw = f32;

    type Soft = rustc_apfloat::ieee::Single;

    fn read_reg(s: &System, reg: usize) -> Self::Soft {
        let value = s.cop1.get32(reg, s.cop0.f64());

        rustc_apfloat::ieee::Single::from_bits(value as u128)
    }

    fn write_reg(s: &mut System, reg: usize, value: Self::Soft) {
        // Convert any NaN output to the MIPS canonical qNaN

        let value_u32 = if value.is_nan() {
            QNAN32
        } else {
            // Flush to zero

            if s.cop1.fcr31.flush_to_zero() && value.is_denormal() {
                let flushed = match s.cop1.fcr31.rounding_mode() {
                    RoundingMode::Zero | RoundingMode::Nearest => {
                        if value.is_negative() {
                            -0.0
                        } else {
                            0.0
                        }
                    }
                    RoundingMode::Infinity => {
                        if value.is_negative() {
                            -0.0
                        } else {
                            f32::MIN_POSITIVE
                        }
                    }
                    RoundingMode::NegativeInfinity => {
                        if value.is_negative() {
                            -f32::MIN_POSITIVE
                        } else {
                            0.0
                        }
                    }
                };

                flushed.to_bits()
            } else {
                value.to_bits() as u32
            }
        };

        s.cop1.set32(reg, value_u32, s.cop0.f64());
    }

    fn sqrt(
        value: Self::Soft,
        rounding: rustc_apfloat::Round,
    ) -> rustc_apfloat::StatusAnd<Self::Soft> {
        let (result, _iterations) = ieee_apsqrt::sqrt_accurate(value.to_bits() as u32, rounding);
        // TODO try fast

        rustc_apfloat::StatusAnd {
            value: Self::Soft::from_bits(result.value as u128),
            status: result.status,
        }
    }
}

impl SoftFloat for u64 {
    type Raw = f64;

    type Soft = rustc_apfloat::ieee::Double;

    fn read_reg(s: &System, reg: usize) -> Self::Soft {
        let value = s.cop1.get64(reg, s.cop0.f64());

        rustc_apfloat::ieee::Double::from_bits(value as u128)
    }

    fn write_reg(s: &mut System, reg: usize, value: Self::Soft) {
        // Convert any NaN output to the MIPS qNaN

        let value_u64 = if value.is_nan() {
            QNAN64
        } else {
            // Flush to zero

            if s.cop1.fcr31.flush_to_zero() && value.is_denormal() {
                let flushed = match s.cop1.fcr31.rounding_mode() {
                    RoundingMode::Zero | RoundingMode::Nearest => {
                        if value.is_negative() {
                            -0.0
                        } else {
                            0.0
                        }
                    }
                    RoundingMode::Infinity => {
                        if value.is_negative() {
                            -0.0
                        } else {
                            f64::MIN_POSITIVE
                        }
                    }
                    RoundingMode::NegativeInfinity => {
                        if value.is_negative() {
                            -f64::MIN_POSITIVE
                        } else {
                            0.0
                        }
                    }
                };

                flushed.to_bits()
            } else {
                value.to_bits() as u64
            }
        };

        s.cop1.set64(reg, value_u64, s.cop0.f64());
    }

    fn sqrt(
        value: Self::Soft,
        rounding: rustc_apfloat::Round,
    ) -> rustc_apfloat::StatusAnd<Self::Soft> {
        let (result, _iterations) = ieee_apsqrt::sqrt_accurate(value.to_bits() as u64, rounding);
        // TODO try fast

        rustc_apfloat::StatusAnd {
            value: Self::Soft::from_bits(result.value as u128),
            status: result.status,
        }
    }
}

pub fn apfloat_rounding(s: &System) -> rustc_apfloat::Round {
    match s.cop1.fcr31.rounding_mode() {
        RoundingMode::Nearest => rustc_apfloat::Round::NearestTiesToEven,
        RoundingMode::Zero => rustc_apfloat::Round::TowardZero,
        RoundingMode::Infinity => rustc_apfloat::Round::TowardPositive,
        RoundingMode::NegativeInfinity => rustc_apfloat::Round::TowardNegative,
    }
}

/// Helper for catching the error cases of two-operand arithmetic operations
fn base_arithmetic_op<
    T: SoftFloat,
    F: FnOnce(
        &mut System,
        T::Soft,
        T::Soft,
        rustc_apfloat::Round,
    ) -> rustc_apfloat::StatusAnd<T::Soft>,
>(
    s: &mut System,
    operands: Operands,
    f: F,
) -> InstructionResult {
    check_cop_usable!(1, s);

    // Any sNaN or subnormal input: unimplemented operation

    let ft = T::read_reg(s, operands.ft());
    let fs = T::read_reg(s, operands.fs());

    if fs.is_signaling() || ft.is_signaling() || fs.is_denormal() || ft.is_denormal() {
        set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
    }

    // Perform the operation with the global rounding mode

    let result = f(s, fs, ft, apfloat_rounding(s));

    // Denormal results and underflow can result in unimplemented operation exceptions under certain conditions:
    // - The result is denormal OR underflow occurred AND
    // - flushing to zero is disabled OR underflow/inexact exception are enabled

    if (result.value.is_denormal() || result.status.contains(rustc_apfloat::Status::UNDERFLOW))
        && (!s.cop1.fcr31.flush_to_zero()
            || (s.cop1.fcr31.exception_enabled().underflow()
                || s.cop1.fcr31.exception_enabled().inexact_operation()))
    {
        set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
    }

    // Other errors
    // (Flushing to zero also causes underflow + inexact operation)

    let flush = result.value.is_denormal() && s.cop1.fcr31.flush_to_zero();

    let inexact = result.status.contains(rustc_apfloat::Status::INEXACT) | flush;

    let underflow = result.status.contains(rustc_apfloat::Status::UNDERFLOW) | flush;

    let invalid =
        result.status.contains(rustc_apfloat::Status::INVALID_OP) || result.value.is_nan();

    let overflow = result.status.contains(rustc_apfloat::Status::OVERFLOW);

    let division_by_zero = result.status.contains(rustc_apfloat::Status::DIV_BY_ZERO);

    set_exception_cause(
        s,
        Cause::default()
            .with_inexact_operation(inexact)
            .with_invalid_operation(invalid)
            .with_overflow(overflow)
            .with_underflow(underflow)
            .with_division_by_zero(division_by_zero),
    )?;

    // Write the result if no exception was raised

    T::write_reg(s, operands.fd(), result.value);

    Ok(None)
}

pub struct Add;

impl Add {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        base_arithmetic_op::<T, _>(s, operands, |_s, fs, ft, rounding| fs.add_r(ft, rounding))
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "ADD.{} {}, {}, {}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn(),
            operands.ftn()
        )
    }
}

impl Instruction for Add {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Add::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Add::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Add::disassemble(s, opcode, operands)
    }
}

pub struct Sub;

impl Sub {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        base_arithmetic_op::<T, _>(s, operands, |_s, fs, ft, rounding| fs.sub_r(ft, rounding))
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SUB.{} {}, {}, {}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn(),
            operands.ftn()
        )
    }
}

impl Instruction for Sub {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Sub::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Sub::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Sub::disassemble(s, opcode, operands)
    }
}

pub struct Mul;

impl Mul {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        base_arithmetic_op::<T, _>(s, operands, |_s, fs, ft, rounding| fs.mul_r(ft, rounding))
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "MUL.{} {}, {}, {}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn(),
            operands.ftn()
        )
    }
}

impl Instruction for Mul {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Mul::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Mul::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Mul::disassemble(s, opcode, operands)
    }
}

pub struct Div;

impl Div {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        base_arithmetic_op::<T, _>(s, operands, |_s, fs, ft, rounding| fs.div_r(ft, rounding))
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "DIV.{} {}, {}, {}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn(),
            operands.ftn()
        )
    }
}

impl Instruction for Div {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Div::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Div::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Div::disassemble(s, opcode, operands)
    }
}

pub struct Sqrt;

impl Sqrt {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        check_cop_usable!(1, s);

        // sNaN or subnormal input: unimplemented operation

        let fs: <T as SoftFloat>::Soft = T::read_reg(s, operands.fs());

        set_exception_cause(
            s,
            Cause::default().with_unimplemented_operation(fs.is_signaling() || fs.is_denormal()),
        )?;

        let result = T::sqrt(fs, apfloat_rounding(s));

        let invalid =
            result.status.contains(rustc_apfloat::Status::INVALID_OP) || result.value.is_nan();

        let inexact = result.status.contains(rustc_apfloat::Status::INEXACT);

        set_exception_cause(
            s,
            Cause::default()
                .with_invalid_operation(invalid)
                .with_inexact_operation(inexact),
        )?;

        T::write_reg(s, operands.fd(), result.value);

        Ok(None)
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "SQRT.{} {}, {}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn()
        )
    }
}

impl Instruction for Sqrt {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Sqrt::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Sqrt::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Sqrt::disassemble(s, opcode, operands)
    }
}

pub struct Abs;

impl Abs {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        check_cop_usable!(1, s);

        // sNaN or subnormal input: unimplemented operation
        // qNaN input: invalid operation

        let fs = T::read_reg(s, operands.fs());

        set_exception_cause(
            s,
            Cause::default()
                .with_unimplemented_operation(fs.is_signaling() || fs.is_denormal())
                .with_invalid_operation(fs.is_nan() && !fs.is_signaling()),
        )?;

        T::write_reg(s, operands.fd(), fs.abs());

        Ok(None)
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "ABS.{} {},{}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn()
        )
    }
}

impl Instruction for Abs {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Abs::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Abs::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Abs::disassemble(s, opcode, operands)
    }
}

pub struct Neg;

impl Neg {
    pub fn execute<T: SoftFloat>(
        s: &mut System,
        _opcode: Opcode,
        operands: Operands,
    ) -> InstructionResult {
        check_cop_usable!(1, s);

        // sNaN or subnormal input: unimplemented operation
        // qNaN input: invalid operation

        let fs = T::read_reg(s, operands.fs());

        set_exception_cause(
            s,
            Cause::default()
                .with_unimplemented_operation(fs.is_signaling() || fs.is_denormal())
                .with_invalid_operation(fs.is_nan() && !fs.is_signaling()),
        )?;

        T::write_reg(s, operands.fd(), -fs);

        Ok(None)
    }

    pub fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "NEG.{} {},{}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn()
        )
    }
}

impl Instruction for Neg {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        match opcode.cop1_format() {
            Some(Format::Float32) => Neg::execute::<u32>(s, opcode, operands),
            Some(Format::Float64) => Neg::execute::<u64>(s, opcode, operands),
            _ => <Reserved as Instruction>::execute(s, opcode, operands),
        }
    }

    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String {
        Neg::disassemble(s, opcode, operands)
    }
}

// -----------
// Conversions
// -----------

pub struct Cvt;

impl Instruction for Cvt {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        check_cop_usable!(1, s);

        set_exception_cause(s, Cause::default())?;

        let input_format = opcode.cop1_format().unwrap();

        let output_format = match opcode.0 & 0x3F {
            0b100000 => Format::Float32,
            0b100001 => Format::Float64,
            0b100100 => Format::Int32,
            0b100101 => Format::Int64,
            _ => unimplemented!("CVT with output format {}", opcode.0 & 0x3F),
        };

        let rounding = apfloat_rounding(s);

        match (output_format, input_format) {
            // f32 from f32
            (Format::Float32, Format::Float32) => {
                set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
            }

            // f32 from f64
            (Format::Float32, Format::Float64) => {
                let fs_f64 = u64::read_reg(s, operands.fs());

                if fs_f64.is_signaling() || fs_f64.is_denormal() {
                    set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
                }

                let mut inexact = false;
                let fs_f32: StatusAnd<Single> = fs_f64.convert_r(rounding, &mut inexact);

                // Denormal results and underflow can result in unimplemented operation exceptions under certain conditions:
                // - The result is denormal OR underflow occurred AND
                // - flushing to zero is disabled OR underflow/inexact exception are enabled

                // TODO share with base_arithmetic_op
                if (fs_f32.value.is_denormal()
                    || fs_f32.status.contains(rustc_apfloat::Status::UNDERFLOW))
                    && (!s.cop1.fcr31.flush_to_zero()
                        || (s.cop1.fcr31.exception_enabled().underflow()
                            || s.cop1.fcr31.exception_enabled().inexact_operation()))
                {
                    set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
                }

                // (Flushing to zero also causes underflow + inexact operation)

                let flush = fs_f32.value.is_denormal() && s.cop1.fcr31.flush_to_zero();

                set_exception_cause(
                    s,
                    Cause::default()
                        .with_invalid_operation(fs_f32.value.is_nan())
                        .with_inexact_operation(fs_f32.status.contains(Status::INEXACT) | flush)
                        .with_overflow(fs_f32.status.contains(Status::OVERFLOW))
                        .with_underflow(fs_f32.status.contains(Status::UNDERFLOW) | flush),
                )?;

                u32::write_reg(s, operands.fd(), fs_f32.value);
            }

            // f32 from i32
            (Format::Float32, Format::Int32) => {
                let fs_i32 = operands.fsv(s) as i32;

                let fs_f32 = Single::from_i128_r(fs_i32 as i128, rounding);

                set_exception_cause(
                    s,
                    Cause::default()
                        .with_inexact_operation(fs_f32.status.contains(Status::INEXACT)),
                )?;

                u32::write_reg(s, operands.fd(), fs_f32.value);
            }

            // f32 from i64
            (Format::Float32, Format::Int64) => {
                let fs_u64 = operands.fsv64(s);

                let fs_i64 = fs_u64 as i64;

                let fs_f32 = Single::from_i128_r(fs_i64 as i128, rounding);

                // "if any of bits 53 to 62 of the result of conversion from a floating-point format to a fixed-point format is 1,
                // an unimplemented operation exception will occur" (VR4300 manual)
                // TODO
                // set_exception_cause(
                //     s,
                //     Cause::default().with_unimplemented_operation(fs_f32.value.to_bits() >> 53 != 0),
                // )?;

                set_exception_cause(
                    s,
                    Cause::default()
                        .with_inexact_operation(fs_f32.status.contains(Status::INEXACT)),
                )?;

                u32::write_reg(s, operands.fd(), fs_f32.value);
            }

            // f64 from f32
            (Format::Float64, Format::Float32) => {
                s.cop1.set64(
                    operands.fd(),
                    (f32::from_bits(operands.fsv(s)) as f64).to_bits(),
                    s.cop0.f64(),
                );
            }

            // f64 from f64
            (Format::Float64, Format::Float64) => {
                set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
            }

            // f64 from i32
            (Format::Float64, Format::Int32) => s.cop1.set64(
                operands.fd(),
                (operands.fsv(s) as i32 as f64).to_bits(),
                s.cop0.f64(),
            ),

            // f64 from i64
            (Format::Float64, Format::Int64) => s.cop1.set64(
                operands.fd(),
                (operands.fsv64(s) as i64 as f64).to_bits(),
                s.cop0.f64(),
            ),

            // i32 from f32
            (Format::Int32, Format::Float32) => {
                let fs32 = f32::from_bits(operands.fsv(s));

                set_exception_cause(
                    s,
                    Cause::default().with_inexact_operation(fs32.fract() != 0.0),
                )?;

                s.cop1
                    .set64(operands.fd(), fs32 as i32 as u64, s.cop0.f64())
            }

            // i32 from f64
            (Format::Int32, Format::Float64) => {
                let fs64 = f64::from_bits(operands.fsv64(s));
                let result = fs64 as i32 as u64;

                // set_exception_cause(
                //     s,
                //     Cause::default().with_inexact_operation(fs64.fract() != 0.0),
                // )?;
                set_exception_cause(
                    s,
                    Cause::default().with_inexact_operation((result as i32 as f64) != fs64),
                )?;

                s.cop1.set64(operands.fd(), result, s.cop0.f64())
            }

            // i32 from i32
            (Format::Int32, Format::Int32) => {
                set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
            }

            // i32 from i64
            (Format::Int32, Format::Int64) => {
                set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
            }

            // i64 from f32
            (Format::Int64, Format::Float32) => {
                let fs32 = f32::from_bits(operands.fsv(s));
                let result = fs32 as i64 as u64;
                // set_exception_cause(
                //     s,
                //     Cause::default().with_inexact_operation(fs32.fract() != 0.0),
                // )?;
                set_exception_cause(
                    s,
                    Cause::default().with_inexact_operation((result as i64 as f32) != fs32),
                )?;

                s.cop1
                    .set64(operands.fd(), fs32 as i64 as u64, s.cop0.f64())
            }

            // i64 from f64
            (Format::Int64, Format::Float64) => {
                let fs64 = f64::from_bits(operands.fsv64(s));
                let result = fs64 as i64 as u64;
                // set_exception_cause(
                //     s,
                //     Cause::default().with_inexact_operation(fs64.fract() != 0.0),
                // )?;

                set_exception_cause(
                    s,
                    Cause::default().with_inexact_operation((result as i64 as f64) != fs64),
                )?;

                s.cop1.set64(operands.fd(), result, s.cop0.f64())
            }

            // i64 from i32
            (Format::Int64, Format::Int32) => {
                set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
            }

            // i64 from i64
            (Format::Int64, Format::Int64) => { /* NOP */ }
        }

        Ok(None)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "CVT.{} {}, {}",
            opcode.cop1_format().unwrap(),
            operands.fdn(),
            operands.fsn()
        )
    }
}
