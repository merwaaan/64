use std::simd::i16x8;

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::system::System;

/// Helper to decode opcodes
#[bitfield(u32, instrospect, debug)]

pub struct Opcode {
    /// Group (special, regimm, etc, or just top-level instructions)
    #[bits(26..=31, r)]
    group: u6,

    /// Opcode within the Special group
    #[bits(0..=5, r)]
    special_opcode: u6,

    /// Opcode within the Regimm group
    #[bits(16..=20, r)]
    regimm_opcode: u5,

    /// Opcode within the COP0/COP2 groups
    #[bits(21..=25, r)]
    cop_opcode: u5,

    /// Sub-opcode within the COP2 group
    #[bits(0..=5, r)]
    cop2_opcode: u6,

    /// Sub-opcode within the COP2 load & store group
    #[bits(11..=15, r)]
    cop2_load_store_opcode: u5,

    /// rs register index
    #[bits(21..=25, r)]
    _rs: u5,

    /// rt register index
    #[bits(16..=20, r)]
    _rt: u5,

    /// rd register index
    #[bits(11..=15, r)]
    _rd: u5,

    /// vt register index
    #[bits(16..=20, r)]
    _vt: u5,

    /// vs register index
    #[bits(11..=15, r)]
    _vs: u5,

    /// vd register index
    #[bits(6..=10, r)]
    _vd: u5,

    /// Destination element
    #[bits(11..=15, r)]
    _de: u5,

    /// Immediate 16-bits value
    #[bits(0..=15, r)]
    imm16: u16,

    /// Shift amount
    #[bits(6..=10, r)]
    _shift: u5,

    /// Branch offset
    #[bits(0..=15, r)]
    _branch_offset: u16,

    /// Base
    #[bits(21..=25, r)]
    _base: u5,

    /// Offset
    #[bits(0..=6, r)]
    _offset: u7,

    /// Element offset
    #[bits(7..=10, r)]
    _element_offset: u4,

    /// Element
    #[bits(21..=24, r)]
    _element: u4,
}

impl Opcode {
    pub fn rs(&self) -> usize {
        self._rs().value() as usize
    }

    pub fn rsv(&self, s: &System) -> u32 {
        s.sp.sregs.read(self.rs())
    }

    pub fn rt(&self) -> usize {
        self._rt().value() as usize
    }

    pub fn rtv(&self, s: &System) -> u32 {
        s.sp.sregs.read(self.rt())
    }

    pub fn rd(&self) -> usize {
        self._rd().value() as usize
    }

    pub fn vt(&self) -> usize {
        self._vt().value() as usize
    }

    pub fn vtv(&self, s: &System) -> i16x8 {
        s.sp.vregs[self.vt()]
    }

    pub fn vs(&self) -> usize {
        self._vs().value() as usize
    }

    pub fn vsv(&self, s: &System) -> i16x8 {
        s.sp.vregs[self.vs()]
    }

    pub fn vd(&self) -> usize {
        self._vd().value() as usize
    }

    pub fn de(&self) -> usize {
        self._de().value() as usize
    }

    pub fn shift(&self) -> usize {
        self._shift().value() as usize
    }

    pub fn branch_offset(&self) -> u32 {
        (self._branch_offset() as i16 as i32 as u32) << 2
    }

    pub fn branch_target(&self, s: &System) -> u12 {
        let target = u32::from(s.sp.pc)
            .wrapping_add(4)
            .wrapping_add(self.branch_offset());

        u12::from_u32(target & 0x0FFF)
    }

    pub fn base(&self) -> usize {
        self._base().value() as usize
    }

    pub fn offset(&self, shift: usize) -> usize {
        (self._offset().value() as usize) << shift
    }

    pub fn offset_addr(&self, s: &System) -> usize {
        let offset = self.raw_value & 0xFFFF;

        (s.sp.sregs.read(self.base()).wrapping_add(offset) & 0x0FFF) as usize
    }

    pub fn element_offset(&self) -> usize {
        self._element_offset().value() as usize
    }

    pub fn element(&self) -> u8 {
        self._element().value()
    }

    pub fn vtv_broadcast(&self, s: &System) -> i16x8 {
        let vt = s.sp.vregs[self.vt()];

        match self.element() {
            0 | 1 => vt,

            // Quarters
            2 => i16x8::from_array([vt[0], vt[0], vt[2], vt[2], vt[4], vt[4], vt[6], vt[6]]),
            3 => i16x8::from_array([vt[1], vt[1], vt[3], vt[3], vt[5], vt[5], vt[7], vt[7]]),

            // Halves
            4 => i16x8::from_array([vt[0], vt[0], vt[0], vt[0], vt[4], vt[4], vt[4], vt[4]]),
            5 => i16x8::from_array([vt[1], vt[1], vt[1], vt[1], vt[5], vt[5], vt[5], vt[5]]),
            6 => i16x8::from_array([vt[2], vt[2], vt[2], vt[2], vt[6], vt[6], vt[6], vt[6]]),
            7 => i16x8::from_array([vt[3], vt[3], vt[3], vt[3], vt[7], vt[7], vt[7], vt[7]]),

            // Singles
            8 => i16x8::splat(vt[0]),
            9 => i16x8::splat(vt[1]),
            10 => i16x8::splat(vt[2]),
            11 => i16x8::splat(vt[3]),
            12 => i16x8::splat(vt[4]),
            13 => i16x8::splat(vt[5]),
            14 => i16x8::splat(vt[6]),
            15 => i16x8::splat(vt[7]),

            _ => unreachable!(),
        }
    }
}
