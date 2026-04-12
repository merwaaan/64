use arbitrary_int::prelude::*;
use rustc_apfloat::{Float, FloatConvert, Status, StatusAnd, ieee::Single};

use crate::{
    check_cop_usable,
    cop1::{self, Cause, Format, Interrupt, RoundingMode},
    cpu::{
        instructions::{
            DecodedInstruction, InstructionEffect, InstructionResult, RESERVED_INSTRUCTION,
        },
        opcode::Opcode,
    },
    exception::Exception,
    inst,
    registers::Registers,
    system::System,
};

pub fn decode(opcode: Opcode) -> DecodedInstruction {
    debug_assert_eq!(opcode.group(), 0x11);

    // TODO can avoid & 1F as they all have the same prefix

    match (opcode.0 >> 21) & 0x1F {
        0x00 => inst!(mfc1),
        0x01 => inst!(dmfc1),
        0x02 => inst!(cfc1),
        0x04 => inst!(mtc1),
        0x05 => inst!(dmtc1),
        0x06 => inst!(ctc1),
        0x08 => match (opcode.0 >> 16) & 0x1F {
            0x00 => inst!(bc1f),
            0x01 => inst!(bc1t),
            0x02 => inst!(bc1fl),
            0x03 => inst!(bc1tl),
            _ => RESERVED_INSTRUCTION,
        },
        _ => {
            // Expands to `inst!(name)` for the valid formats
            macro_rules! inst_fmt {
                ($name:ident; $($fmt:path),* $(,)?) => {
                    {
                        match opcode.cop1_format() {
                            $( Some($fmt) )|* => inst!($name),
                            _ => RESERVED_INSTRUCTION,
                        }
                    }
                };
            }

            // `Format::Float32` = `name_execute::<u32>`, `Format::Float64` = `name_execute::<u64>`, same disassemble
            macro_rules! inst_fmt_fp {
                ($name:ident) => {
                    match opcode.cop1_format() {
                        Some(Format::Float32) => (
                            paste::paste! { [<$name _execute>]::<u32> },
                            paste::paste! { [<$name _disassemble>] },
                        ),
                        Some(Format::Float64) => (
                            paste::paste! { [<$name _execute>]::<u64> },
                            paste::paste! { [<$name _disassemble>] },
                        ),
                        _ => RESERVED_INSTRUCTION,
                    }
                };
            }

            match opcode.0 & 0x3F {
                0x00 => inst_fmt_fp!(add),
                0x01 => inst_fmt_fp!(sub),
                0x02 => inst_fmt_fp!(mul),
                0x03 => inst_fmt_fp!(div),
                0x04 => inst_fmt_fp!(sqrt),
                0x05 => inst_fmt_fp!(abs),
                0x06 => inst_fmt!(mov; Format::Float32, Format::Float64),
                0x07 => inst_fmt_fp!(neg),
                0x08 => inst_fmt!(round; Format::Float32, Format::Float64), // TODO duplicates??
                0x09 => inst_fmt!(trunc; Format::Float32, Format::Float64),
                0x0A => inst_fmt!(ceil; Format::Float32, Format::Float64),
                0x0B => inst_fmt!(floor; Format::Float32, Format::Float64),
                0x0C => inst_fmt!(round; Format::Float32, Format::Float64),
                0x0D => inst_fmt!(trunc; Format::Float32, Format::Float64),
                0x0E => inst_fmt!(ceil; Format::Float32, Format::Float64),
                0x0F => inst_fmt!(floor; Format::Float32, Format::Float64),
                0x20 | 0x21 | 0x24 | 0x25 => {
                    inst_fmt!(cvt; Format::Float32, Format::Float64, Format::Int32, Format::Int64)
                }
                0x30..=0x3F => inst_fmt_fp!(c),
                _ => RESERVED_INSTRUCTION,
            }
        }
    }
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

fn cfc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // TODO enforce 0-31 in decode?

    check_cop_usable!(1, s);

    match op.fs() {
        0 => s.cpu.regs.gpr[op.rt()].set(s.cop1.fcr0()),
        31 => s.cpu.regs.gpr[op.rt()].set(s.cop1.fcr31.read()),
        _ => unreachable!("CFC1 with invalid fs {}", op.fs()),
    }

    Ok(None)
}

fn cfc1_disassemble(_s: &System, op: Opcode) -> String {
    format!("CFC1 {}, {}", op.rtn(), Registers::fpr_name(op.fs()))
}

fn ctc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // TODO enforce 0-31 in decode?

    check_cop_usable!(1, s);

    match op.fs() {
        0 => { /* read-only */ }
        31 => s.cop1.fcr31.write(op.rtv(s)),
        _ => unreachable!("CTC1 with invalid fs {}", op.fs()),
    }

    Ok(None)
}

fn ctc1_disassemble(_s: &System, op: Opcode) -> String {
    format!("CTC1 {}, FCR{}", op.rtn(), op.fs())
}

fn dmfc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let value = s.cop1.get64(op.fs(), s.cop0.f64());

    s.cpu.regs.gpr[op.rt()].set64(value);

    Ok(None)
}

fn dmfc1_disassemble(_s: &System, op: Opcode) -> String {
    format!("DMFC1 {}, {}", op.rtn(), op.fsn())
}

fn dmtc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    s.cop1.set64(op.fs(), op.rtv64(s), s.cop0.f64());

    Ok(None)
}

fn dmtc1_disassemble(_s: &System, op: Opcode) -> String {
    format!("DMTC1 {}, {}", op.rtn(), op.fsn())
}

fn mfc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let value = s.cop1.get32(op.fs(), s.cop0.f64());

    s.cpu.regs.gpr[op.rt()].set(value);

    Ok(None)
}

fn mfc1_disassemble(_s: &System, op: Opcode) -> String {
    format!("MFC1 {}, {}", op.rtn(), op.fsn())
}

fn mov_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => s.cop1.set32(op.fd(), op.fsv(s), s.cop0.f64()),
        Some(Format::Float64) => s.cop1.set64(op.fd(), op.fsv64(s), s.cop0.f64()),
        _ => unimplemented!("MOV with invalid format {:08X}", op.0),
    }

    Ok(None)
}

fn mov_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "MOV.{} {},{}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    )
}

fn mtc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    s.cop1.set32(op.fs(), op.rtv(s), s.cop0.f64());

    Ok(None)
}

fn mtc1_disassemble(_s: &System, op: Opcode) -> String {
    format!("MTC1 {}, {}", op.rtn(), op.fsn())
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

struct Ceil;

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

struct Floor;

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

struct Trunc;

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

struct Round;

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
fn generic_rounding_execute<ROUNDING: Rounding>(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let input_format = op.cop1_format().unwrap();

    let output_format = if (op.0 & ROUNDING::L_MASK) == ROUNDING::L_MASK {
        Format::Int64
    } else {
        Format::Int32
    };

    match (input_format, output_format) {
        (Format::Float32, Format::Int32) => {
            s.cop1.set32(
                op.fd(),
                ROUNDING::apply32(f32::from_bits(op.fsv(s))) as i32 as u32,
                s.cop0.f64(),
            );
        }
        (Format::Float32, Format::Int64) => {
            s.cop1.set64(
                op.fd(),
                ROUNDING::apply32(f32::from_bits(op.fsv(s))) as i64 as u64,
                s.cop0.f64(),
            );
        }
        (Format::Float64, Format::Int32) => {
            s.cop1.set32(
                op.fd(),
                ROUNDING::apply64(f64::from_bits(op.fsv64(s))) as i32 as u32,
                s.cop0.f64(),
            );
        }
        (Format::Float64, Format::Int64) => {
            s.cop1.set64(
                op.fd(),
                ROUNDING::apply64(f64::from_bits(op.fsv64(s))) as i64 as u64,
                s.cop0.f64(),
            );
        }
        _ => unimplemented!("{}.{}.{}", ROUNDING::NAME, output_format, input_format),
    }

    set_exception_cause(s, Cause::default())?;

    Ok(None)
}

fn generic_rounding_disassemble<ROUNDING: Rounding>(_s: &System, op: Opcode) -> String {
    let output_format = if (op.0 & ROUNDING::L_MASK) == ROUNDING::L_MASK {
        Format::Int64
    } else {
        Format::Int32
    };

    format!(
        "{}.{}.{} {},{}",
        ROUNDING::NAME,
        output_format,
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    )
}

fn ceil_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Ceil>(s, op)
}

fn ceil_disassemble(s: &System, op: Opcode) -> String {
    generic_rounding_disassemble::<Ceil>(s, op)
}

fn floor_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Floor>(s, op)
}

fn floor_disassemble(s: &System, op: Opcode) -> String {
    generic_rounding_disassemble::<Floor>(s, op)
}

fn round_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Round>(s, op)
}

fn round_disassemble(s: &System, op: Opcode) -> String {
    generic_rounding_disassemble::<Round>(s, op)
}

fn trunc_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Trunc>(s, op)
}

fn trunc_disassemble(s: &System, op: Opcode) -> String {
    generic_rounding_disassemble::<Trunc>(s, op)
}

// -----------
// Comparisons
// -----------

fn c_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let fs = T::read_reg(s, op.fs());
    let ft = T::read_reg(s, op.ft());

    let qnan = (fs.is_nan() && !fs.is_signaling()) || (ft.is_nan() && !ft.is_signaling());
    let unordered = fs.is_nan() || ft.is_nan();

    let (result, signal) = match op.cop1_comparison() {
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

fn c_disassemble(_s: &System, op: Opcode) -> String {
    format!("C.{}.{}", op.cop1_comparison(), op.cop1_format().unwrap(),)
}

fn bc1f_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    if !s.cop1.fcr31.comparison_result() {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        Ok(None)
    }
}

fn bc1f_disassemble(_s: &System, op: Opcode) -> String {
    format!("BC1F {:#06X}", op.branch_offset())
}

fn bc1fl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    if !s.cop1.fcr31.comparison_result() {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

fn bc1fl_disassemble(_s: &System, op: Opcode) -> String {
    format!("BC1FL {:#06X}", op.branch_offset())
}

fn bc1t_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    if s.cop1.fcr31.comparison_result() {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        Ok(None)
    }
}

fn bc1t_disassemble(_s: &System, op: Opcode) -> String {
    format!("BC1T {:#06X}", op.branch_offset())
}

fn bc1tl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    if s.cop1.fcr31.comparison_result() {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

fn bc1tl_disassemble(_s: &System, op: Opcode) -> String {
    format!("BC1TL {:#06X}", op.branch_offset())
}

// ---------------------
// Arithmetic operations
// ---------------------

const QNAN32: u32 = 0x7FBF_FFFF;
const QNAN64: u64 = 0x7FF7_FFFF_FFFF_FFFF;

/// Helper trait to perform arithmetic operations on either u32 or u64
trait SoftFloat {
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
    op: Opcode,
    f: F,
) -> InstructionResult {
    check_cop_usable!(1, s);

    // Any sNaN or subnormal input: unimplemented operation

    let ft = T::read_reg(s, op.ft());
    let fs = T::read_reg(s, op.fs());

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

    T::write_reg(s, op.fd(), result.value);

    Ok(None)
}

fn add_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    base_arithmetic_op::<T, _>(s, op, |_s, fs, ft, rounding| fs.add_r(ft, rounding))
}

fn add_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "ADD.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    )
}

fn sub_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    base_arithmetic_op::<T, _>(s, op, |_s, fs, ft, rounding| fs.sub_r(ft, rounding))
}

fn sub_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "SUB.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    )
}

fn mul_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    base_arithmetic_op::<T, _>(s, op, |_s, fs, ft, rounding| fs.mul_r(ft, rounding))
}

fn mul_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "MUL.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    )
}

fn div_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    base_arithmetic_op::<T, _>(s, op, |_s, fs, ft, rounding| fs.div_r(ft, rounding))
}

fn div_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "DIV.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    )
}

fn sqrt_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    // sNaN or subnormal input: unimplemented operation

    let fs: <T as SoftFloat>::Soft = T::read_reg(s, op.fs());

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

    T::write_reg(s, op.fd(), result.value);

    Ok(None)
}

fn sqrt_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "SQRT.{} {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    )
}

fn abs_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    // sNaN or subnormal input: unimplemented operation
    // qNaN input: invalid operation

    let fs = T::read_reg(s, op.fs());

    set_exception_cause(
        s,
        Cause::default()
            .with_unimplemented_operation(fs.is_signaling() || fs.is_denormal())
            .with_invalid_operation(fs.is_nan() && !fs.is_signaling()),
    )?;

    T::write_reg(s, op.fd(), fs.abs());

    Ok(None)
}

fn abs_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "ABS.{} {},{}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    )
}

fn neg_execute<T: SoftFloat>(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    // sNaN or subnormal input: unimplemented operation
    // qNaN input: invalid operation

    let fs = T::read_reg(s, op.fs());

    set_exception_cause(
        s,
        Cause::default()
            .with_unimplemented_operation(fs.is_signaling() || fs.is_denormal())
            .with_invalid_operation(fs.is_nan() && !fs.is_signaling()),
    )?;

    T::write_reg(s, op.fd(), -fs);

    Ok(None)
}

fn neg_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "NEG.{} {},{}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    )
}

// -----------
// Conversions
// -----------

fn cvt_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    set_exception_cause(s, Cause::default())?;

    let input_format = op.cop1_format().unwrap();

    let output_format = match op.0 & 0x3F {
        0b100000 => Format::Float32,
        0b100001 => Format::Float64,
        0b100100 => Format::Int32,
        0b100101 => Format::Int64,
        _ => unimplemented!("CVT with output format {}", op.0 & 0x3F),
    };

    let rounding = apfloat_rounding(s);

    match (output_format, input_format) {
        // f32 from f32
        (Format::Float32, Format::Float32) => {
            set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
        }

        // f32 from f64
        (Format::Float32, Format::Float64) => {
            let fs_f64 = u64::read_reg(s, op.fs());

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

            u32::write_reg(s, op.fd(), fs_f32.value);
        }

        // f32 from i32
        (Format::Float32, Format::Int32) => {
            let fs_i32 = op.fsv(s) as i32;

            let fs_f32 = Single::from_i128_r(fs_i32 as i128, rounding);

            set_exception_cause(
                s,
                Cause::default().with_inexact_operation(fs_f32.status.contains(Status::INEXACT)),
            )?;

            u32::write_reg(s, op.fd(), fs_f32.value);
        }

        // f32 from i64
        (Format::Float32, Format::Int64) => {
            let fs_u64 = op.fsv64(s);

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
                Cause::default().with_inexact_operation(fs_f32.status.contains(Status::INEXACT)),
            )?;

            u32::write_reg(s, op.fd(), fs_f32.value);
        }

        // f64 from f32
        (Format::Float64, Format::Float32) => {
            s.cop1.set64(
                op.fd(),
                (f32::from_bits(op.fsv(s)) as f64).to_bits(),
                s.cop0.f64(),
            );
        }

        // f64 from f64
        (Format::Float64, Format::Float64) => {
            set_exception_cause(s, Cause::default().with_unimplemented_operation(true))?;
        }

        // f64 from i32
        (Format::Float64, Format::Int32) => {
            s.cop1
                .set64(op.fd(), (op.fsv(s) as i32 as f64).to_bits(), s.cop0.f64())
        }

        // f64 from i64
        (Format::Float64, Format::Int64) => {
            s.cop1
                .set64(op.fd(), (op.fsv64(s) as i64 as f64).to_bits(), s.cop0.f64())
        }

        // i32 from f32
        (Format::Int32, Format::Float32) => {
            let fs32 = f32::from_bits(op.fsv(s));

            set_exception_cause(
                s,
                Cause::default().with_inexact_operation(fs32.fract() != 0.0),
            )?;

            s.cop1.set64(op.fd(), fs32 as i32 as u64, s.cop0.f64())
        }

        // i32 from f64
        (Format::Int32, Format::Float64) => {
            let fs64 = f64::from_bits(op.fsv64(s));
            let result = fs64 as i32 as u64;

            // set_exception_cause(
            //     s,
            //     Cause::default().with_inexact_operation(fs64.fract() != 0.0),
            // )?;
            set_exception_cause(
                s,
                Cause::default().with_inexact_operation((result as i32 as f64) != fs64),
            )?;

            s.cop1.set64(op.fd(), result, s.cop0.f64())
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
            let fs32 = f32::from_bits(op.fsv(s));
            let result = fs32 as i64 as u64;
            // set_exception_cause(
            //     s,
            //     Cause::default().with_inexact_operation(fs32.fract() != 0.0),
            // )?;
            set_exception_cause(
                s,
                Cause::default().with_inexact_operation((result as i64 as f32) != fs32),
            )?;

            s.cop1.set64(op.fd(), fs32 as i64 as u64, s.cop0.f64())
        }

        // i64 from f64
        (Format::Int64, Format::Float64) => {
            let fs64 = f64::from_bits(op.fsv64(s));
            let result = fs64 as i64 as u64;
            // set_exception_cause(
            //     s,
            //     Cause::default().with_inexact_operation(fs64.fract() != 0.0),
            // )?;

            set_exception_cause(
                s,
                Cause::default().with_inexact_operation((result as i64 as f64) != fs64),
            )?;

            s.cop1.set64(op.fd(), result, s.cop0.f64())
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

fn cvt_disassemble(_s: &System, op: Opcode) -> String {
    format!(
        "CVT.{} {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    )
}
