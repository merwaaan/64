use crate::{
    check_cop_usable,
    cop1::{self, Format},
    cpu::{
        instructions::{
            DisassembleFn, Disassembly, ExecuteFn, InstructionEffect, InstructionResult,
        },
        opcode::Opcode,
    },
    exception::Exception,
    inst,
    registers::Registers,
    system::System,
};

pub fn decode(opcode: Opcode) -> Option<(ExecuteFn, DisassembleFn)> {
    debug_assert_eq!(opcode.group(), 0x11);

    // TODO can avoid & 1F as they all have the same prefix

    Some(match (opcode.0 >> 21) & 0x1F {
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
            _ => return None,
        },
        _ => {
            // Expands to `inst!(name)` for the valid formats
            macro_rules! inst_fmt {
                ($name:ident; $($fmt:path),* $(,)?) => {
                    {
                        match opcode.cop1_format() {
                            $( Some($fmt) )|* => inst!($name),
                            _ => return None,
                        }
                    }
                };
            }

            match opcode.0 & 0x3F {
                0x00 => inst_fmt!(add; Format::Float32, Format::Float64),
                0x01 => inst_fmt!(sub; Format::Float32, Format::Float64),
                0x02 => inst_fmt!(mul; Format::Float32, Format::Float64),
                0x03 => inst_fmt!(div; Format::Float32, Format::Float64),
                0x04 => inst_fmt!(sqrt; Format::Float32, Format::Float64),
                0x05 => inst_fmt!(abs; Format::Float32, Format::Float64),
                0x06 => inst_fmt!(mov; Format::Float32, Format::Float64),
                0x07 => inst_fmt!(neg; Format::Float32, Format::Float64),
                0x08 => inst_fmt!(round; Format::Float32, Format::Float64),
                0x09 => inst_fmt!(trunc; Format::Float32, Format::Float64),
                0x0A => inst_fmt!(ceil; Format::Float32, Format::Float64),
                0x0B => inst_fmt!(floor; Format::Float32, Format::Float64),
                0x0C => inst_fmt!(round; Format::Float32, Format::Float64),
                0x0D => inst_fmt!(trunc; Format::Float32, Format::Float64),
                0x0E => inst_fmt!(ceil; Format::Float32, Format::Float64),
                0x0F => inst_fmt!(floor; Format::Float32, Format::Float64),
                0x20 => {
                    inst_fmt!(cvt; Format::Float32, Format::Float64, Format::Int32, Format::Int64)
                }
                0x21 => {
                    inst_fmt!(cvt; Format::Float32, Format::Float64,  Format::Int32, Format::Int64)
                }
                0x24 => {
                    inst_fmt!(cvt;Format::Float32, Format::Float64,  Format::Int32, Format::Int64)
                }
                0x25 => {
                    inst_fmt!(cvt;Format::Float32, Format::Float64,  Format::Int32, Format::Int64)
                }
                0x30 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x31 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x32 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x33 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x34 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x35 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x36 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x37 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x38 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x39 => inst_fmt!(c; Format::Float32, Format::Float64),
                0x3A => inst_fmt!(c; Format::Float32, Format::Float64),
                0x3B => inst_fmt!(c; Format::Float32, Format::Float64),
                0x3C => inst_fmt!(c; Format::Float32, Format::Float64),
                0x3D => inst_fmt!(c; Format::Float32, Format::Float64),
                0x3E => inst_fmt!(c; Format::Float32, Format::Float64),
                0x3F => inst_fmt!(c; Format::Float32, Format::Float64),
                _ => return None,
            }
        }
    })
}

fn abs_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            s.cop1.set32(
                op.fd(),
                f32::from_bits(op.fsv(s)).abs().to_bits(),
                s.cop0.f64(),
            );
        }
        Some(Format::Float64) => {
            s.cop1.set64(
                op.fd(),
                f64::from_bits(op.fsv64(s)).abs().to_bits(),
                s.cop0.f64(),
            );
        }
        _ => unimplemented!("ABS with invalid format {:08X}", op.0),
    }

    s.cop1.fcr31.set_exception_cause(cop1::Cause::default());

    Ok(None)
}

fn abs_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "ABS.{} {},{}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    ))
}

fn add_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            let ft = f32::from_bits(op.ftv(s));
            let fs = f32::from_bits(op.fsv(s));
            s.cop1.set32(op.fd(), (ft + fs).to_bits(), s.cop0.f64());
        }
        Some(Format::Float64) => {
            let ft = f64::from_bits(op.ftv64(s));
            let fs = f64::from_bits(op.fsv64(s));
            s.cop1.set64(op.fd(), (ft + fs).to_bits(), s.cop0.f64());
        }
        _ => unimplemented!("ADD with invalid format {:08X}", op.0),
    }

    s.cop1.fcr31.set_exception_cause(cop1::Cause::default());

    Ok(None)
}

fn add_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "ADD.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    ))
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

fn bc1f_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BC1F {:#06X}", op.branch_offset()))
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

fn bc1fl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BC1FL {:#06X}", op.branch_offset()))
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

fn bc1t_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BC1T {:#06X}", op.branch_offset()))
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

fn bc1tl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BC1TL {:#06X}", op.branch_offset()))
}

trait Comparable: PartialOrd + Copy + std::fmt::Display {
    fn is_nan(self) -> bool;
}

impl Comparable for f32 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }
}

impl Comparable for f64 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }
}

fn generic_comparison<T: Comparable>(
    s: &mut System,
    comparison: cop1::Comparison,
    fs: T,
    ft: T,
) -> InstructionResult {
    check_cop_usable!(1, s);

    let unordered = fs.is_nan() || ft.is_nan();

    let (result, signal) = match comparison {
        cop1::Comparison::False => (false, false),
        cop1::Comparison::Unordered => (unordered, false),
        cop1::Comparison::Equal => (fs == ft, false),
        cop1::Comparison::UnorderedOrEqual => (unordered || fs == ft, false),
        cop1::Comparison::OrderedLess => (fs < ft, false),
        cop1::Comparison::UnorderedOrLess => (unordered || fs < ft, false),
        cop1::Comparison::OrderedLessOrEqual => (fs <= ft, false),
        cop1::Comparison::UnorderedOrLessOrEqual => (unordered || fs <= ft, false),
        cop1::Comparison::SignalingFalse => (false, unordered),
        cop1::Comparison::NotGreatherOrLessOrEqual => (unordered, unordered),
        cop1::Comparison::SignalingEqual => (fs == ft, unordered),
        cop1::Comparison::NotGreatherOrLess => (unordered || fs == ft, unordered),
        cop1::Comparison::Less => (fs < ft, unordered),
        cop1::Comparison::NotGreaterOrEqual => (unordered || fs < ft, unordered),
        cop1::Comparison::LessOrEqual => (fs <= ft, unordered),
        cop1::Comparison::NotGreater => (unordered || fs <= ft, unordered),
    };
    // log::error!(
    //     "C: {} {} u={} r={} s={} {:08X}",
    //     fs,
    //     ft,
    //     unordered,
    //     result,
    //     signal,
    //     s.cpu.regs.pc
    // );

    s.cop1.fcr31.set_comparison_result(result);

    s.cop1
        .fcr31
        .set_exception_cause(cop1::Cause::default().with_invalid_operation(signal));

    Ok(None)
}

fn c_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            let fs = f32::from_bits(s.cop1.get32(op.fs(), s.cop0.f64()));
            let ft = f32::from_bits(s.cop1.get32(op.ft(), s.cop0.f64()));

            generic_comparison(s, op.cop1_comparison(), fs, ft)
        }
        Some(Format::Float64) => {
            let fs = f64::from_bits(s.cop1.get64(op.fs(), s.cop0.f64()));
            let ft = f64::from_bits(s.cop1.get64(op.ft(), s.cop0.f64()));

            generic_comparison(s, op.cop1_comparison(), fs, ft)
        }
        _ => unimplemented!("C with invalid format {:08X}", op.0),
    }
}

fn c_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "C.{}.{}",
        op.cop1_comparison(),
        op.cop1_format().unwrap(),
    ))
}

fn ceil_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Ceil>(s, op)
}

fn ceil_disassemble(s: &System, op: Opcode) -> Disassembly {
    generic_rounding_disassemble::<Ceil>(s, op)
}

fn cfc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // TODO enforce 0-31 in decode?

    check_cop_usable!(1, s);

    match op.fs() {
        0 => s.cpu.regs.gpr[op.rt()].set(s.cop1.fcr0),
        31 => s.cpu.regs.gpr[op.rt()].set(s.cop1.fcr31.read()),
        _ => unreachable!("CFC1 with invalid fs {}", op.fs()),
    }

    Ok(None)
}

fn cfc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CFC1 {}, {}",
        op.rtn(),
        Registers::fpr_name(op.fs())
    ))
}

fn ctc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // TODO enforce 0-31 in decode?

    check_cop_usable!(1, s);

    match op.fs() {
        0 => {}
        31 => s.cop1.fcr31.write(op.rtv(s)),
        _ => unreachable!("CTC1 with invalid fs {}", op.fs()),
    }

    Ok(None)
}

fn ctc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("CTC1 {}, FCR{}", op.rtn(), op.fs()))
}

fn cvt_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let input_format = op.cop1_format().unwrap();

    let output_format = match op.0 & 0x3F {
        0b100000 => Format::Float32,
        0b100001 => Format::Float64,
        0b100100 => Format::Int32,
        0b100101 => Format::Int64,
        _ => unimplemented!("CVT with output format {}", op.0 & 0x3F),
    };

    match (output_format, input_format) {
        // f32 from f32
        (Format::Float32, Format::Float32) => { /* NOP */ }

        // f32 from f64
        (Format::Float32, Format::Float64) => s.cop1.set32(
            op.fd(),
            (f64::from_bits(op.fsv64(s)) as f32).to_bits(),
            s.cop0.f64(),
        ),

        // f32 from i32
        (Format::Float32, Format::Int32) => {
            s.cop1
                .set32(op.fd(), (op.fsv(s) as i32 as f32).to_bits(), s.cop0.f64())
        }

        // f32 from i64
        (Format::Float32, Format::Int64) => {
            s.cop1
                .set32(op.fd(), (op.fsv64(s) as i64 as f32).to_bits(), s.cop0.f64())
        }

        // f64 from f32
        (Format::Float64, Format::Float32) => s.cop1.set64(
            op.fd(),
            (f32::from_bits(op.fsv(s)) as f64).to_bits(),
            s.cop0.f64(),
        ),

        // f64 from f64
        (Format::Float64, Format::Float64) => { /* NOP */ }

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
        (Format::Int32, Format::Float32) => s.cop1.set64(
            op.fd(),
            f32::from_bits(op.fsv(s)) as i32 as u64,
            s.cop0.f64(),
        ),

        // i32 from f64
        (Format::Int32, Format::Float64) => s.cop1.set64(
            op.fd(),
            f64::from_bits(op.fsv64(s)) as i32 as u64,
            s.cop0.f64(),
        ),

        // i32 from i32
        (Format::Int32, Format::Int32) => { /* NOP */ }

        // i32 from i64
        (Format::Int32, Format::Int64) => {
            // TODO
            log::error!("CVT.W.L not implemented");
        }

        // i64 from f32
        (Format::Int64, Format::Float32) => s.cop1.set64(
            op.fd(),
            f32::from_bits(op.fsv(s)) as i64 as u64,
            s.cop0.f64(),
        ),

        // i64 from f64
        (Format::Int64, Format::Float64) => s.cop1.set64(
            op.fd(),
            f64::from_bits(op.fsv64(s)) as i64 as u64,
            s.cop0.f64(),
        ),

        // i64 from i32
        (Format::Int64, Format::Int32) => {
            // TODO
            log::error!("CVT.L.W not implemented");
        }

        // i64 from i64
        (Format::Int64, Format::Int64) => { /* NOP */ }
    }

    Ok(None)
}

fn cvt_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CVT.{} {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    ))
}

fn div_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            let ft = f32::from_bits(op.ftv(s));
            let fs = f32::from_bits(op.fsv(s));

            s.cop1.set32(op.fd(), (fs / ft).to_bits(), s.cop0.f64());
        }
        Some(Format::Float64) => {
            let ft = f64::from_bits(op.ftv64(s));
            let fs = f64::from_bits(op.fsv64(s));

            s.cop1.set64(op.fd(), (fs / ft).to_bits(), s.cop0.f64());
        }
        _ => unimplemented!("DIV with invalid format {:08X}", op.0),
    }

    Ok(None)
}

fn div_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "DIV.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    ))
}

fn dmfc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let value = s.cop1.get64(op.fs(), s.cop0.f64());

    s.cpu.regs.gpr[op.rt()].set64(value);

    Ok(None)
}

fn dmfc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMFC1 {}, {}", op.rtn(), op.fsn()))
}

fn dmtc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    s.cop1.set64(op.fs(), op.rtv64(s), s.cop0.f64());

    Ok(None)
}

fn dmtc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMTC1 {}, {}", op.rtn(), op.fsn()))
}

fn floor_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Floor>(s, op)
}

fn floor_disassemble(s: &System, op: Opcode) -> Disassembly {
    generic_rounding_disassemble::<Floor>(s, op)
}

fn mfc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let value = s.cop1.get32(op.fs(), s.cop0.f64());

    s.cpu.regs.gpr[op.rt()].set(value);

    Ok(None)
}

fn mfc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFC1 {}, {}", op.rtn(), op.fsn()))
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

fn mov_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "MOV.{} {},{}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    ))
}

fn mtc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    s.cop1.set32(op.fs(), op.rtv(s), s.cop0.f64());

    Ok(None)
}

fn mtc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTC1 {}, {}", op.rtn(), op.fsn()))
}

fn mul_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            let ft = f32::from_bits(op.ftv(s));
            let fs = f32::from_bits(op.fsv(s));

            s.cop1.set32(op.fd(), (ft * fs).to_bits(), s.cop0.f64());
        }
        Some(Format::Float64) => {
            let ft = f64::from_bits(op.ftv64(s));
            let fs = f64::from_bits(op.fsv64(s));

            s.cop1.set64(op.fd(), (ft * fs).to_bits(), s.cop0.f64());
        }
        _ => unimplemented!("MUL with invalid format {:08X}", op.0),
    }

    Ok(None)
}

fn mul_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "MUL.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    ))
}

fn neg_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            s.cop1.set32(
                op.fd(),
                (-f32::from_bits(op.fsv(s))).to_bits(),
                s.cop0.f64(),
            );
        }
        Some(Format::Float64) => {
            s.cop1.set64(
                op.fd(),
                (-f64::from_bits(op.fsv64(s))).to_bits(),
                s.cop0.f64(),
            );
        }
        _ => unimplemented!("NEG with invalid format {:08X}", op.0),
    }

    Ok(None)
}

fn neg_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "NEG.{} {},{}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    ))
}

fn round_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Round>(s, op)
}

fn round_disassemble(s: &System, op: Opcode) -> Disassembly {
    generic_rounding_disassemble::<Round>(s, op)
}

fn sqrt_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            s.cop1.set32(
                op.fd(),
                f32::from_bits(op.fsv(s)).sqrt().to_bits(),
                s.cop0.f64(),
            );
        }
        Some(Format::Float64) => {
            s.cop1.set64(
                op.fd(),
                f64::from_bits(op.fsv64(s)).sqrt().to_bits(),
                s.cop0.f64(),
            );
        }
        _ => unimplemented!("SQRT with invalid format {:08X}", op.0),
    }

    Ok(None)
}

fn sqrt_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SQRT.{} {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    ))
}

fn sub_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    match op.cop1_format() {
        Some(Format::Float32) => {
            let ft = f32::from_bits(op.ftv(s));
            let fs = f32::from_bits(op.fsv(s));

            s.cop1.set32(op.fd(), (fs - ft).to_bits(), s.cop0.f64());
        }
        Some(Format::Float64) => {
            let ft = f64::from_bits(op.ftv64(s));
            let fs = f64::from_bits(op.fsv64(s));

            s.cop1.set64(op.fd(), (fs - ft).to_bits(), s.cop0.f64());
        }
        _ => unimplemented!("SUB with invalid format {:08X}", op.0),
    }

    Ok(None)
}

fn sub_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SUB.{} {}, {}, {}",
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn(),
        op.ftn()
    ))
}

fn trunc_execute(s: &mut System, op: Opcode) -> InstructionResult {
    generic_rounding_execute::<Trunc>(s, op)
}

fn trunc_disassemble(s: &System, op: Opcode) -> Disassembly {
    generic_rounding_disassemble::<Trunc>(s, op)
}

// Helpers for the rounding instructions that share the same logic but different rounding behaviors

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

    Ok(None)
}

fn generic_rounding_disassemble<ROUNDING: Rounding>(_s: &System, op: Opcode) -> Disassembly {
    let output_format = if (op.0 & ROUNDING::L_MASK) == ROUNDING::L_MASK {
        Format::Int64
    } else {
        Format::Int32
    };

    Disassembly::new(format!(
        "{}.{}.{} {},{}",
        ROUNDING::NAME,
        output_format,
        op.cop1_format().unwrap(),
        op.fdn(),
        op.fsn()
    ))
}
