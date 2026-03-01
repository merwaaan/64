use strum::Display;

use crate::registers::Reg64;

#[derive(Debug, Display)]
pub enum Format {
    // 32 bits floating point
    S,
    // 64 bits floating point
    D,
    // 32 bits integer
    W,
    // 64 bits integer
    L,
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

// TODO as function of Cop0

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
pub(crate) fn translate_register_index(index: usize, f64_mode: bool) -> usize {
    index >> ((!f64_mode) as usize)
}

pub(crate) fn get64_64mode(regs: &[Reg64; 32], index: usize) -> u64 {
    regs[index].get64()
}

pub(crate) fn get32_64mode(regs: &[Reg64; 32], index: usize) -> u32 {
    regs[index].get()
}

pub(crate) fn set64_64mode(regs: &mut [Reg64; 32], reg64_index: usize, value: u64) {
    regs[reg64_index].set64(value);
}

pub(crate) fn set32_64mode(regs: &mut [Reg64; 32], index: usize, value: u32) {
    let reg64 = regs[index].get64();
    regs[index].set64((reg64 & 0xFFFFFFFF_00000000) | (value as u64));
}

pub(crate) fn get64_32mode(regs: &[Reg64; 32], index: usize) -> u64 {
    let reg64_index = (index >> 1) * 2;

    regs[reg64_index].get64()
}

pub(crate) fn set64_32mode(regs: &mut [Reg64; 32], index: usize, value: u64) {
    let reg64_index = (index >> 1) * 2;

    regs[reg64_index].set64(value);
}

pub(crate) fn get32_32mode(regs: &[Reg64; 32], index: usize) -> u32 {
    let reg64_index = (index >> 1) * 2;

    if index & 1 == 0 {
        regs[reg64_index].get() as u32
    } else {
        (regs[reg64_index].get64() >> 32) as u32
    }
}

pub(crate) fn set32_32mode(regs: &mut [Reg64; 32], index: usize, value: u32) {
    let reg64_index = (index >> 1) * 2;
    let reg64 = regs[reg64_index].get64();

    if index & 1 == 0 {
        regs[reg64_index].set64((reg64 & 0xFFFFFFFF_00000000) | (value as u64));
    } else {
        regs[reg64_index].set64((reg64 & 0x00000000_FFFFFFFF) | ((value as u64) << 32));
    }
}

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
