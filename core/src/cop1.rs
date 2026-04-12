use arbitrary_int::prelude::*;
use bitbybit::{bitenum, bitfield};
use strum::Display;

use crate::registers::Reg64;

#[derive(Debug, Display)]
#[bitenum(u5)]
pub enum Format {
    // Single, 32 bits floating point
    #[strum(to_string = "S")]
    Float32 = 0x10,
    // Double, 64 bits floating point
    #[strum(to_string = "D")]
    Float64 = 0x11,
    // Word, 32 bits integer
    #[strum(to_string = "W")]
    Int32 = 0x14,
    // Long, 64 bits integer
    #[strum(to_string = "L")]
    Int64 = 0x15,
}

#[derive(Debug, Display)]
pub enum Condition {
    False,
    True,
    FalseLikely,
    TrueLikely,
}

impl Condition {
    // TODO into/from?

    // fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
    //     match self {
    //         Condition::False => {
    //             return None;
    //         }
    //     }
    // }
}

#[derive(Debug, Display)]
#[bitenum(u4, exhaustive = true)]
pub enum Comparison {
    #[strum(to_string = "F")]
    False = 0x0,
    #[strum(to_string = "UN")]
    Unordered = 0x1,
    #[strum(to_string = "EQ")]
    Equal = 0x2,
    #[strum(to_string = "UEQ")]
    UnorderedOrEqual = 0x3,
    #[strum(to_string = "OLT")]
    OrderedLess = 0x4,
    #[strum(to_string = "ULT")]
    UnorderedOrLess = 0x5,
    #[strum(to_string = "OLE")]
    OrderedLessOrEqual = 0x6,
    #[strum(to_string = "ULE")]
    UnorderedOrLessOrEqual = 0x7,
    #[strum(to_string = "SF")]
    SignalingFalse = 0x8,
    #[strum(to_string = "NGLE")]
    NotGreaterOrLessOrEqual = 0x9,
    #[strum(to_string = "SEQ")]
    SignalingEqual = 0xA,
    #[strum(to_string = "NGL")]
    NotGreaterOrLess = 0xB,
    #[strum(to_string = "LT")]
    Less = 0xC,
    #[strum(to_string = "NGE")]
    NotGreaterOrEqual = 0xD,
    #[strum(to_string = "LE")]
    LessOrEqual = 0xE,
    #[strum(to_string = "NGT")]
    NotGreater = 0xF,
}

// impl Format {
//     pub fn from_bits(value: u32) -> Self {
//         match self {
//             Format::S => f32::from_bits(value),
//             Format::D => f64::from_bits(value),
//             Format::W => f32::from_bits(value),
//             Format::L => f64::from_bits(value),
//             _ => unreachable!(),
//         }
//     }
// }

// TODO clean up doc: current state, seems to work with 64 bits entries split onto halves and skipping odd ones

//////////////

// pub(crate) fn get_reg32(regs: &[Reg64; 32], reg32_index: usize) -> u32 {
//     let reg64_index = reg32_index >> 1;

//     let reg64_shift = (reg32_index & 1) << 5; // Even = 0 to get the low bits, Odd = 32 to get the high bits

//     (regs[reg64_index].get64() >> reg64_shift) as u32
// }

// pub(crate) fn set_reg32(regs: &mut [Reg64; 32], reg32_index: usize, value: u32) {
//     let reg64_index = reg32_index >> 1;

//     let mut reg64 = regs[reg64_index].get64();

//     if reg32_index & 1 == 0 {
//         reg64 &= 0xFFFFFFFF_00000000;
//         reg64 |= value as u64;
//     } else {
//         reg64 &= 0x00000000_FFFFFFFF;
//         reg64 |= (value as u64) << 32;
//     }

//     regs[reg64_index].set64(reg64);
// }

#[bitfield(u6, forbid_overlaps, instrospect, default = 0, debug)]
pub struct Cause {
    #[bit(5, rw)]
    unimplemented_operation: bool,
    #[bit(4, rw)]
    invalid_operation: bool,
    #[bit(3, rw)]
    division_by_zero: bool,
    #[bit(2, rw)]
    overflow: bool,
    #[bit(1, rw)]
    underflow: bool,
    #[bit(0, rw)]
    inexact_operation: bool,
}

#[bitfield(u5, forbid_overlaps, instrospect, default = 0, debug)]
pub struct Interrupt {
    #[bit(4, rw)]
    invalid_operation: bool,
    #[bit(3, rw)]
    division_by_zero: bool,
    #[bit(2, rw)]
    overflow: bool,
    #[bit(1, rw)]
    underflow: bool,
    #[bit(0, rw)]
    inexact_operation: bool,
}

#[derive(Debug)]
#[bitenum(u2, exhaustive = true)]
pub enum RoundingMode {
    Nearest = 0,
    Zero = 1,
    Infinity = 2,
    NegativeInfinity = 3,
}

#[bitfield(u32, forbid_overlaps, introspect, default = 0, debug)]
pub struct Fcr31 {
    #[bit(24, rw)]
    flush_to_zero: bool,

    /// Result of the most recent C instruction.
    /// Checked by BCT/BCF instructions.
    #[bit(23, rw)]
    comparison_result: bool,

    /// Result of the most recently executed instruction.
    #[bits(12..=17, rw)]
    exception_cause: Cause,

    /// Enabled exceptions.
    /// If both a cause bit and the corresponding enabled bit are set, the exception is raised.
    #[bits(7..=11, rw)]
    exception_enabled: Interrupt,

    /// Exceptions that were raised.
    /// "Sticky", so bits stay set until cleared by software using CTC1.
    #[bits(2..=6, rw)]
    exception_flags: Interrupt,

    /// Global rounding mode applied to all floating-point operations.
    #[bits(0..=1, rw)]
    rounding_mode: RoundingMode,
}

impl Fcr31 {
    pub fn read(&self) -> u32 {
        self.raw_value
    }

    pub fn write(&mut self, value: u32) {
        *self = Fcr31::new_with_raw_value(value & 0x0183_FFFF);
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Cop1 {
    // Floating-point registers
    //
    // Provides:
    // - 16 64-bit registers when the FR bit in the Status register is 0
    // - 32 64-bit registers when the FR bit in the Status register is 1
    fpr: [Reg64; 32],

    // Floating-point control register
    pub fcr31: Fcr31,
}

impl Cop1 {
    /// FCR0: read-only implementation/revision register
    pub fn fcr0(&self) -> u32 {
        0x0000_0A00
    }

    /// Converts a 64-bits/32-bits mode register index to the "physical" register index that actually contains the data.
    ///
    /// 64-bits mode:
    /// - 32 64-bits registers are available.
    /// - Internally, we have 32 u64 entries, one for each 64-bits register.
    /// - So the 64-bits mode index is the same as the entry index.
    ///
    /// 32-bits mode:
    /// - 32 32-bits registers are available.
    /// - Internally, we store two 32-bits registers in each of the first 16 u64 entries (high bits = N+1, low bits = N).
    /// - So the 32-bits mode index is the index of the entry that contains the target 32-bits register in its pair.
    ///
    /// This matches the actual hardware that leaks that internal pairing when mixing DMTC/DMFC and switching between 64-bits and 32-bits modes.
    ///
    /// Expected behavior:
    /// - Switch to 32-bits mode
    /// - MTC1 0, 0xAABBCCDD -> Entry 0 contains 0x00000000_AABBCCDD
    /// - MTC1 1, 0x12345678 -> Entry 0 contains 0x12345678_AABBCCDD
    /// - Switch to 64-bits mode
    /// - DMFC1 0 -> gives 0x12345678_AABBCCDD
    /// - DMFC1 1 -> gives 0x12345678_AABBCCDD
    /// - MFC1 0 -> gives 0xAABBCCDD
    /// - MFC1 1 -> gives 0x12345678
    pub fn get32(&self, index: usize, f64_mode: bool) -> u32 {
        if f64_mode {
            self.get32_64mode(index)
        } else {
            self.get32_32mode(index)
        }
    }

    pub fn set32(&mut self, index: usize, value: u32, f64_mode: bool) {
        if f64_mode {
            self.set32_64mode(index, value)
        } else {
            self.set32_32mode(index, value)
        }
    }

    pub fn get64(&self, index: usize, f64_mode: bool) -> u64 {
        if f64_mode {
            self.get64_64mode(index)
        } else {
            self.get64_32mode(index)
        }
    }

    pub fn set64(&mut self, index: usize, value: u64, f64_mode: bool) {
        if f64_mode {
            self.set64_64mode(index, value)
        } else {
            self.set64_32mode(index, value)
        }
    }

    //

    fn get64_64mode(&self, index: usize) -> u64 {
        self.fpr[index].get64()
    }

    fn get32_64mode(&self, index: usize) -> u32 {
        self.fpr[index].get()
    }

    fn set64_64mode(&mut self, reg64_index: usize, value: u64) {
        self.fpr[reg64_index].set64(value);
    }

    fn set32_64mode(&mut self, index: usize, value: u32) {
        let reg64 = self.fpr[index].get64();
        self.fpr[index].set64((reg64 & 0xFFFFFFFF_00000000) | (value as u64));
    }

    fn get64_32mode(&self, index: usize) -> u64 {
        let reg64_index = (index >> 1) * 2;

        self.fpr[reg64_index].get64()
    }

    fn get32_32mode(&self, index: usize) -> u32 {
        let reg64_index = (index >> 1) * 2;

        if index & 1 == 0 {
            self.fpr[reg64_index].get()
        } else {
            (self.fpr[reg64_index].get64() >> 32) as u32
        }
    }

    fn set64_32mode(&mut self, index: usize, value: u64) {
        let reg64_index = (index >> 1) * 2;

        self.fpr[reg64_index].set64(value);
    }

    fn set32_32mode(&mut self, index: usize, value: u32) {
        let reg64_index = (index >> 1) * 2;
        let reg64 = self.fpr[reg64_index].get64();

        if index & 1 == 0 {
            self.fpr[reg64_index].set64((reg64 & 0xFFFFFFFF_00000000) | (value as u64));
        } else {
            self.fpr[reg64_index].set64((reg64 & 0x00000000_FFFFFFFF) | ((value as u64) << 32));
        }
    }
}
