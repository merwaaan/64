use std::simd::*;

use arbitrary_int::prelude::*;
use n64_specs as specs;

use crate::{
    blocks::{read_block, write_block},
    events::{EventType, Events},
    location::Location,
    ram::RamLocation,
    sp::{instructions::InstructionEffect, opcode::Opcode},
    system::System,
    value::Value,
};

pub mod instructions;
pub mod opcode;

// TODO split interface and proc?
// TODO timing = 2/3 CPU
// TODO increment clock regs

pub type SpMemLocation = Location<{ specs::rsp::MEMORY_START }, { specs::rsp::MEMORY_END }>;

pub type SpRegsLocation = Location<{ specs::rsp::REGISTERS_START }, { specs::rsp::REGISTERS_END }>;

#[derive(Clone, Copy, Debug)]
pub struct Registers([u32; 32]);

impl Registers {
    pub fn read(&self, offset: usize) -> u32 {
        self.0[offset]
    }

    pub fn write(&mut self, offset: usize, data: u32) {
        // R0 is read-only
        if offset != 0 {
            self.0[offset] = data;
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DmaDirection {
    RamToSp,
    SpToRam,
}

#[derive(Clone, Copy, Debug)]
struct DmaSlot {
    direction: DmaDirection,
    ram_address: u32,
    sp_address: u32,
    length: u32, // TODO just pass reg? instead of dir+length
}

#[derive(Clone)]
pub struct Sp {
    /// Memory, split into:
    /// - DMEM: 0x0000 - 0x0FFF
    /// - IMEM: 0x1000 - 0x1FFF
    pub mem: Vec<u8>, // TODO vis

    /// Control registers
    pub control_regs: specs::rsp::Registers, // TODO vis

    /// Scalar registers
    pub sregs: Registers, //TODO vis

    /// Vector registers
    pub vregs: [i16x8; 32], // TODO vis

    /// 48-bit vector accumulator
    pub vacc: i64x8, // TODO vis

    /// Vector carry-out TODO other half?
    pub vco: u16, // TODO vis

    /// Vector compare code TODO doc
    pub vcc: u16, // TODO vis

    /// Vector compare extension TODO doc
    pub vce: u8, // TODO vis

    /// Partial results for VRCP and similar instructions
    div_in: i32,
    div_out: i32,

    pub pc: u12, // TODO vis

    delayed_branching: Option<u12>,

    active_dma: Option<DmaSlot>,
    pending_dma: Option<DmaSlot>,
}

impl Default for Sp {
    fn default() -> Self {
        let mut control_regs = specs::rsp::Registers::default();
        control_regs.status.set_halted(true);

        Self {
            mem: vec![0; 0x2000],
            control_regs,
            sregs: Registers([0; 32]),
            vregs: [i16x8::splat(0); 32],
            vacc: i64x8::splat(0),
            vcc: 0,
            vco: 0,
            vce: 0,
            div_in: 0,
            div_out: 0,
            pc: u12::ZERO,
            delayed_branching: None,
            active_dma: None,
            pending_dma: None,
        }
    }
}

impl Sp {
    pub fn step(s: &mut System) {
        if s.sp.halted() {
            return;
        }

        // log::debug!("SP: step @ {:08X}", s.sp.pc);

        let instruction = u32::read_mem(&s.sp.mem, 0x1000 | u32::from(s.sp.pc));

        let opcode = Opcode::new_with_raw_value(instruction);

        let (execute, _disassemble) = instructions::decode(opcode);

        let result = execute(s, opcode);

        Self::advance_pc(s);

        if let Some(InstructionEffect::DelayedBranching(target)) = result {
            s.sp.delayed_branching = target;
        }
    }

    fn advance_pc(s: &mut System) {
        if let Some(target) = s.sp.delayed_branching.take() {
            s.sp.pc = target;
        } else {
            s.sp.pc = s.sp.pc.wrapping_add(u12::new(4));
        }
    }

    pub fn halted(&self) -> bool {
        self.control_regs.status.halted()
    }

    pub fn interrupt_on_break(&self) -> bool {
        self.control_regs.status.interrupt_on_break()
    }

    pub fn read_mem<T: Value>(&self, addr: SpMemLocation) -> T {
        T::read_mem(&self.mem, addr.relative() & specs::rsp::MEMORY_MASK)
    }

    pub fn write_mem<T: Value>(s: &mut System, addr: SpMemLocation, data: T) {
        data.write_mem(&mut s.sp.mem, addr.relative() & specs::rsp::MEMORY_MASK);
    }

    pub fn read_block(&self, addr: SpMemLocation, length: usize, callback: impl FnMut(&[u8])) {
        let bank_offset = (addr.relative() & 0x1000) as usize;
        let bank = &self.mem[bank_offset..bank_offset + 0x1000];

        let offset_in_bank = (addr.relative() & 0x0FFF) as usize;
        read_block(bank, offset_in_bank, length, callback);
    }

    // TODO hardware-test mirroring
    pub fn read_reg<T: Value>(&mut self, addr: SpRegsLocation) -> T {
        assert!(T::BYTES == 4, "RSP: read with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::rsp::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "RSP: read from unaligned address {:08X}",
            offset
        );

        // TODO clean up mess

        if addr.relative() < 0x4_0000 {
            let regs_slice = bytemuck::cast_slice(bytemuck::bytes_of(&self.control_regs));
            let data = T::read_reg(regs_slice, offset);

            // Reading the semaphore returns the current value and set it to 1

            if offset == specs::rsp::Register::Semaphore.offset() {
                self.control_regs.semaphore.set_value(true);
            }

            data
        } else if addr.relative() == 0x4_0000 {
            // TODO bad addr?
            if (addr.relative() & 3) != 0 {
                panic!("Unaligned SP PC read: {:08X}", addr.relative());
            }

            let pc = [u32::from(self.pc)];
            T::read_reg(&pc, addr.relative() & 0x0000_0003)

            // TODO use PC reg or not?
        } else {
            panic!("Read SP reg @ {:08X}", addr.relative());
        }
    }

    pub fn write_reg<T: Value>(s: &mut System, addr: SpRegsLocation, data: T) {
        assert!(T::BYTES == 4, "RSP: write with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::rsp::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "RSP: write to unaligned address {:08X}",
            offset
        );

        //log::warn!("write SP reg @ {:08X} {:X}", addr.relative(), data);

        // TODO clean up mess, simplify if always aligned

        if addr.relative() < 0x4_0000 {
            //let reg = ((addr.relative() & n64_specs::rsp::REGISTERS_MASK) >> 2) as u32;

            match offset {
                specs::rsp::DmaRspAddress::OFFSET => {
                    // 11-bit SP address.
                    // Bits 0-2 cannot be written to so the address is always aligned to 8 bytes.
                    // Bit 12 is the "bank" (0 = DMEM, 1 = IMEM).

                    let regs_slice =
                        bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(&mut s.sp.control_regs));
                    data.write_reg(regs_slice, offset);

                    // TODO can't i just write the unmasked part?

                    s.sp.control_regs
                        .dma_rsp_address
                        .set_value(s.sp.control_regs.dma_rsp_address.value() & u13::new(0x1FF8u16));
                }
                specs::rsp::DmaRamAddress::OFFSET => {
                    // 24-bit RAM address.
                    // Bits 0-2 cannot be written to so the address is always aligned to 8 bytes.

                    // TODO reads should return the previous value until DMA starts?

                    let regs_slice =
                        bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(&mut s.sp.control_regs));
                    data.write_reg(regs_slice, offset);

                    s.sp.control_regs.dma_ram_address.set_value(
                        s.sp.control_regs.dma_ram_address.value() & u24::new(0x00FF_FFF8),
                    );
                }
                specs::rsp::DmaReadLength::OFFSET => {
                    // TODO only write on completion if pending? (should read back the ongoing or last DMA)

                    let regs_slice =
                        bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(&mut s.sp.control_regs));
                    data.write_reg(regs_slice, offset);

                    Self::push_dma(
                        s,
                        DmaSlot {
                            direction: DmaDirection::RamToSp,
                            ram_address: s.sp.control_regs.dma_ram_address.raw_value(),
                            sp_address: s.sp.control_regs.dma_rsp_address.raw_value(),
                            length: s.sp.control_regs.dma_read_length.raw_value(),
                        },
                    );
                }
                specs::rsp::DmaWriteLength::OFFSET => {
                    // TODO only write on completion if pending? (should read back the ongoing or last DMA)

                    let regs_slice =
                        bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(&mut s.sp.control_regs));
                    data.write_reg(regs_slice, offset);

                    Self::push_dma(
                        s,
                        DmaSlot {
                            direction: DmaDirection::SpToRam,
                            ram_address: s.sp.control_regs.dma_ram_address.raw_value(),
                            sp_address: s.sp.control_regs.dma_rsp_address.raw_value(),
                            length: s.sp.control_regs.dma_write_length.raw_value(),
                        },
                    );
                }
                specs::rsp::Status::OFFSET => {
                    // TODO simplify and just use struct fields directly?

                    let mut status = s.sp.control_regs.status.raw_value();

                    let mut trigger_bits = [0u32];
                    data.write_reg(&mut trigger_bits, addr.relative() & 3);

                    // Helper to clear/set status bits
                    //
                    // Hardware tests show that setting both clear and set bits causes nothing to happen (n64-systemtest)

                    macro_rules! clear_set {
                        ($clear_mask:expr, $set_mask:expr, $status_mask:expr) => {
                            let clear = trigger_bits[0] & $clear_mask != 0;
                            let set = trigger_bits[0] & $set_mask != 0;

                            if clear && !set {
                                status &= !$status_mask;
                            } else if set && !clear {
                                status |= $status_mask;
                            }
                        };
                    }

                    // HALTED

                    clear_set!(1, 1 << 1, 1);

                    // BROKE

                    let clear_broke = trigger_bits[0] & 4 != 0;

                    if clear_broke {
                        status &= !2;
                    }

                    // INT

                    let set_int = trigger_bits[0] & 16 != 0;
                    let clear_int = trigger_bits[0] & 8 != 0;

                    if clear_int && !set_int {
                        s.mi.clear_pending_interrupt(specs::interrupt::Interrupt::Sp, &mut s.cop0);
                    } else if !clear_int && set_int {
                        s.mi.set_pending_interrupt(specs::interrupt::Interrupt::Sp, &mut s.cop0);
                    }

                    // SSTEP
                    clear_set!(1 << 5, 1 << 6, 1 << 5);

                    // INTBREAK
                    clear_set!(1 << 7, 1 << 8, 1 << 6);

                    // SIGn
                    clear_set!(1 << 9, 1 << 10, 1 << 7); // 0
                    clear_set!(1 << 11, 1 << 12, 1 << 8); // 1
                    clear_set!(1 << 13, 1 << 14, 1 << 9); // 2
                    clear_set!(1 << 15, 1 << 16, 1 << 10); // 3
                    clear_set!(1 << 17, 1 << 18, 1 << 11); // 4
                    clear_set!(1 << 19, 1 << 20, 1 << 12); // 5
                    clear_set!(1 << 21, 1 << 22, 1 << 13); // 6
                    clear_set!(1 << 23, 1 << 24, 1 << 14); // 7

                    s.sp.control_regs.status = specs::rsp::Status::new_with_raw_value(status);
                }
                specs::rsp::Semaphore::OFFSET => {
                    // Writes clear the semaphore
                    s.sp.control_regs.semaphore.set_value(false);
                }
                _ => panic!("Invalid SP register @ {:08X}", offset),
            }
        } else if addr.relative() == 0x4_0000 {
            // TODO bad addr?
            if (addr.relative() & 3) != 0 {
                panic!("Unaligned SP PC write: {:08X}", addr.relative());
            }

            let mut pc = [u32::from(s.sp.pc)];
            data.write_reg(&mut pc, addr.relative() & 3);
            s.sp.pc = u12::from_u32(pc[0] & 0x0FFC);

            // Reset any delayed branching
            s.sp.delayed_branching = None;
        } else {
            panic!("Write SP reg @ {:08X}", addr.relative());
        }
    }

    // pub fn dp_halt(s: &mut System) {
    //     s.mi.set_pending_interrupt(Interrupt::Dp, &mut s.cop0);
    // }

    // TODO Bakuretsu Muteki Bangaioh: huge fishy DMAs that overwrite the SP memory many times

    fn push_dma(s: &mut System, slot: DmaSlot) {
        if s.sp.pending_dma.is_some() {
            // TODO or replace pending?
            log::warn!("SP: DMA transfer already pending");
        }
        // Active DMA transfer: queue
        else if s.sp.active_dma.is_some() {
            s.sp.pending_dma = Some(slot);

            s.sp.control_regs.set_dma_full(true);
        }
        // No active DMA transfer: execute
        else {
            s.sp.active_dma = Some(slot);

            // TODO ENABLED?

            Self::start_dma(s, slot);
        }
    }

    fn start_dma(s: &mut System, slot: DmaSlot) {
        // Number of bytes to copy per "row"
        //
        // Manual: "the lower three bits of the length are ignored and assumed to be all 1's"

        let length = slot.length as usize;

        let bytes_per_row = ((length & 0x0FFF) | 7) + 1;

        // Number of rows to copy

        let rows = ((length >> 12) & 0x00FF) + 1;

        // Number of bytes to skip after each row
        // (only applies to the RAM side!)

        let skips = (length >> 20) & !7;

        let mut ram_addr = (slot.ram_address & 0x007F_FFFF) as usize; // TODO > 0x007F_FFFF possible! libdragon ipl3 uses that to zero DMEM at boot
        let mut sp_addr = slot.sp_address as usize;

        // On the SP side, reads/writes wrap around either IMEM or DMEM

        let sp_bank_offset = sp_addr & 0x1000;
        let sp_bank = &mut s.sp.mem[sp_bank_offset..sp_bank_offset + 0x1000];

        // Transfer

        match slot.direction {
            DmaDirection::RamToSp => {
                // log::info!(
                //     "SP DMA: {:X} bytes * {:X} rows + {:X} skips from RAM {:08X} to SP {:08X}",
                //     bytes_per_row,
                //     rows,
                //     skips,
                //     ram_addr,
                //     sp_addr
                // );

                for _ in 0..rows {
                    s.ram.read_block(
                        RamLocation::from_relative(ram_addr as u32),
                        bytes_per_row,
                        |ram_data| {
                            write_block(ram_data, sp_bank, sp_addr);

                            sp_addr = sp_addr.wrapping_add(bytes_per_row);
                        },
                    );

                    ram_addr = ram_addr.wrapping_add(bytes_per_row).wrapping_add(skips);
                }
            }
            DmaDirection::SpToRam => {
                // log::info!(
                //     "SP DMA: {:X} bytes * {:X} rows + {:X} skips from SP {:08X} to RAM {:08X}",
                //     bytes_per_row,
                //     rows,
                //     skips,
                //     sp_addr,
                //     ram_addr
                // );

                for _ in 0..rows {
                    read_block(sp_bank, sp_addr, bytes_per_row, |sp_data| {
                        s.ram
                            .write_block(RamLocation::from_relative(ram_addr as u32), sp_data);

                        ram_addr = ram_addr.wrapping_add(sp_data.len());
                    });

                    sp_addr = sp_addr.wrapping_add(bytes_per_row);
                }
            }
        }

        // Increment the DMA registers for the next transfer
        // TODO do it on completion?

        let next_sp_address = slot.sp_address.wrapping_add((bytes_per_row * rows) as u32) & 0xFFF
            | (sp_bank_offset as u32);

        s.sp.control_regs
            .dma_rsp_address
            .set_value(u13::new(next_sp_address as u16));

        let next_ram_address = slot
            .ram_address
            .wrapping_add(((bytes_per_row + skips) * rows) as u32);

        s.sp.control_regs
            .dma_ram_address
            .set_value(u24::new(next_ram_address & 0x00FF_FFF8));

        // The length registers always end up being 0xFF8 after a DMA transfer
        // TODO do it on completion?

        s.sp.control_regs
            .dma_read_length
            .set_length(u12::new(0xFF8));

        s.sp.control_regs
            .dma_write_length
            .set_length(u12::new(0xFF8));

        // Update the status

        s.sp.control_regs.set_dma_busy(true);

        // TODO reset count to 0!
        // TODO IO busy?
        // TODO DMA error?

        // Schedule the DMA completion
        //
        // Takes 1 SP cycle per 8 bytes + some overhead

        const CPU_SP_RATIO: f32 = 1.5;
        const OVERHEAD: usize = 9;

        let bytes = rows * bytes_per_row;

        let cycles = ((bytes as f32) / 8.0 * CPU_SP_RATIO) as usize + OVERHEAD;

        Events::push(s, EventType::SpDmaTransferComplete, cycles);
    }

    pub fn dma_completed(s: &mut System) {
        debug_assert!(s.sp.active_dma.is_some(), "SP DMA transfer not in progress");

        // Update the status register

        s.sp.control_regs.set_dma_busy(false);
        s.sp.control_regs.set_dma_full(false);
        // TODO IO busy?

        // Start the pending DMA, if any

        s.sp.active_dma = s.sp.pending_dma.take();

        if let Some(slot) = s.sp.active_dma {
            Self::start_dma(s, slot);
        }

        // No SP interrupt! They are only triggered by the BREAK instruction
    }
}

// TODO useful for HLE?

// RDP / display list opcodes
// const G_SPNOOP: u8 = 0x00;
// const G_MTX: u8 = 0x01;
// const G_MOVEMEM: u8 = 0x03;
// const G_VTX: u8 = 0x04;
// const G_DL: u8 = 0x06;
// const G_RDPHALF_CONT: u8 = 0xB2;
// const G_RDPHALF_2: u8 = 0xB3;
// const G_RDPHALF_1: u8 = 0xB4;
// const G_CLEARGEOMETRYMODE: u8 = 0xB6;
// const G_SETGEOMETRYMODE: u8 = 0xB7;
// const G_ENDDL: u8 = 0xB8;
// const G_SETOTHERMODE_L: u8 = 0xB9;
// const G_SETOTHERMODE_H: u8 = 0xBA;
// const G_TEXTURE: u8 = 0xBB;
// const G_MOVEWORD: u8 = 0xBC;
// const G_POPMTX: u8 = 0xBD;
// const G_CULLDL: u8 = 0xBE;
// const G_TRI1: u8 = 0xBF;
// const G_NOOP: u8 = 0xC0;
// const G_TEXRECT: u8 = 0xE4;
// const G_TEXRECTFLIP: u8 = 0xE5;
// const G_RDPLOADSYNC: u8 = 0xE6;
// const G_RDPPIPESYNC: u8 = 0xE7;
// const G_RDPTILESYNC: u8 = 0xE8;
// const G_RDPFULLSYNC: u8 = 0xE9;
// const G_SETKEYGB: u8 = 0xEA;
// const G_SETKEYR: u8 = 0xEB;
// const G_SETCONVERT: u8 = 0xEC;
// const G_SETSCISSOR: u8 = 0xED;
// const G_SETPRIMDEPTH: u8 = 0xEE;
// const G_RDPSETOTHERMODE: u8 = 0xEF;
// const G_LOADTLUT: u8 = 0xF0;
// const G_SETTILESIZE: u8 = 0xF2;
// const G_LOADBLOCK: u8 = 0xF3;
// const G_LOADTILE: u8 = 0xF4;
// const G_SETTILE: u8 = 0xF5;
// const G_FILLRECT: u8 = 0xF6;
// const G_SETFILLCOLOR: u8 = 0xF7;
// const G_SETFOGCOLOR: u8 = 0xF8;
// const G_SETBLENDCOLOR: u8 = 0xF9;
// const G_SETPRIMCOLOR: u8 = 0xFA;
// const G_SETENVCOLOR: u8 = 0xFB;
// const G_SETCOMBINE: u8 = 0xFC;
// const G_SETTIMG: u8 = 0xFD;
// const G_SETZIMG: u8 = 0xFE;
// const G_SETCIMG: u8 = 0xFF;

// pub fn halt(s: &mut System) {
//     // BROKE | HALT
//     s.sp.regs[Register::Status as usize] |= 3;

//     if s.sp.regs[Register::Status as usize] & 0x40 != 0 {
//         log::warn!("SP INT");
//         s.mi.set_pending_interrupt(Interrupt::Sp, &mut s.cop0);

//         // TODO hack, clear sigs
//         s.sp.regs[Register::Status as usize] &= !0x7F80;

//         Events::push(s, EventType::DpHalt, 10000);
//     }

//     // TODO temp

//     // https://hack64.net/wiki/doku.php?id=super_mario_64:fast3d_display_list_commands
//     // https://wiki.cloudmodding.com/oot/F3DZEX2/Opcode_Details#0x00_.E2.80.94_G_NOOP

//     //let task_header_addr = 0x0FC0usize;

//     // 1 = graphics command?
//     // if s.sp.mem[task_header_addr + 3] == 1 {
//     //     log::warn!("SP: gfx?");

//     //     let ptr = u32::read_mem(&s.sp.mem, (task_header_addr as u32) + 0x30);
//     //     log::warn!("SP: ptr {:08X}", ptr);

//     //     let mut pc = ptr;
//     //     let mut stack = Vec::new(); // For subroutines (gSPDisplayList)

//     //     for _ in 0..100 {
//     //         let w0 = s.read::<u32>(Address::p(pc)).unwrap();
//     //         let w1 = s.read::<u32>(Address::p(pc + 4)).unwrap();

//     //         let opcode = (w0 >> 24) as u8;

//     //         match opcode {
//     //             G_SPNOOP => log::warn!("SP: G_SPNOOP @ {:08X}", pc),
//     //             G_MTX => log::warn!("SP: G_MTX @ {:08X}", pc),
//     //             G_MOVEMEM => log::warn!("SP: G_MOVEMEM @ {:08X}", pc),
//     //             G_VTX => log::warn!("SP: G_VTX @ {:08X}", pc),
//     //             G_DL => {
//     //                 log::warn!("SP: G_DL @ {:08X}", pc);
//     //                 stack.push(pc + 8);
//     //                 pc = s.read::<u32>(Address::p(pc + 4)).unwrap();
//     //                 continue;
//     //             }
//     //             G_RDPHALF_CONT => log::warn!("SP: G_RDPHALF_CONT @ {:08X}", pc),
//     //             G_RDPHALF_2 => log::warn!("SP: G_RDPHALF_2 @ {:08X}", pc),
//     //             G_RDPHALF_1 => log::warn!("SP: G_RDPHALF_1 @ {:08X}", pc),
//     //             G_CLEARGEOMETRYMODE => log::warn!("SP: G_CLEARGEOMETRYMODE @ {:08X}", pc),
//     //             G_SETGEOMETRYMODE => log::warn!("SP: G_SETGEOMETRYMODE @ {:08X}", pc),
//     //             G_ENDDL => {
//     //                 log::warn!("SP: G_ENDDL @ {:08X}", pc);
//     //                 if let Some(return_addr) = stack.pop() {
//     //                     pc = return_addr;
//     //                     continue;
//     //                 } else {
//     //                     break; // Task complete
//     //                 }
//     //             }
//     //             0xDF => {
//     //                 log::warn!("SP: gSPEndDisplayList @ {:08X}", pc);
//     //             }
//     //             G_SETOTHERMODE_L => log::warn!("SP: G_SetOtherMode_L @ {:08X}", pc),
//     //             G_SETOTHERMODE_H => log::warn!("SP: G_SetOtherMode_H @ {:08X}", pc),
//     //             G_TEXTURE => log::warn!("SP: G_TEXTURE @ {:08X}", pc),
//     //             G_MOVEWORD => {
//     //                 let index = (w0 & 0x00FF_0000) >> 16;
//     //                 let offset = w0 & 0x0000_FFFF;
//     //                 let data = w1;

//     //                 log::warn!(
//     //                     "SP: G_MOVEWORD @ {:08X} {}, {:X}, {:X}",
//     //                     pc,
//     //                     index,
//     //                     offset,
//     //                     data
//     //                 );
//     //             }
//     //             G_POPMTX => log::warn!("SP: G_POPMTX @ {:08X}", pc),
//     //             G_CULLDL => log::warn!("SP: G_CULLDL @ {:08X}", pc),
//     //             G_TRI1 => log::warn!("SP: G_TRI1 @ {:08X}", pc),
//     //             G_NOOP => log::warn!("SP: G_NOOP @ {:08X}", pc),
//     //             G_TEXRECT => log::warn!("SP: G_TEXRECT @ {:08X}", pc),
//     //             G_TEXRECTFLIP => log::warn!("SP: G_TEXRECTFLIP @ {:08X}", pc),
//     //             G_RDPLOADSYNC => log::warn!("SP: G_RDPLOADSYNC @ {:08X}", pc),
//     //             G_RDPPIPESYNC => log::warn!("SP: G_RDPPIPESYNC @ {:08X}", pc),
//     //             G_RDPTILESYNC => log::warn!("SP: G_RDPTILESYNC @ {:08X}", pc),
//     //             G_RDPFULLSYNC => log::warn!("SP: G_RDPFULLSYNC @ {:08X}", pc),
//     //             G_SETKEYGB => log::warn!("SP: G_SETKEYGB @ {:08X}", pc),
//     //             G_SETKEYR => log::warn!("SP: G_SETKEYR @ {:08X}", pc),
//     //             G_SETCONVERT => log::warn!("SP: G_SETCONVERT @ {:08X}", pc),
//     //             G_SETSCISSOR => {
//     //                 let ulx = (w0 & 0x00FF_F000) >> 12;
//     //                 let uly = w0 & 0x0000_0FFF;
//     //                 let lrx = (w1 & 0x00FF_F000) >> 12;
//     //                 let lry = w1 & 0x0000_0FFF;

//     //                 log::warn!(
//     //                     "SP: G_SETSCISSOR @ {:08X} ({}, {}) ({}, {})",
//     //                     pc,
//     //                     ulx,
//     //                     uly,
//     //                     lrx,
//     //                     lry
//     //                 );
//     //             }
//     //             G_SETPRIMDEPTH => log::warn!("SP: G_SETPRIMDEPTH @ {:08X}", pc),
//     //             G_RDPSETOTHERMODE => log::warn!("SP: G_RDPSetOtherMode @ {:08X}", pc),
//     //             G_LOADTLUT => log::warn!("SP: G_LOADTLUT @ {:08X}", pc),
//     //             G_SETTILESIZE => {
//     //                 let uls = w0.get_bits(12..=24);
//     //                 let ult = w0.get_bits(0..=11);
//     //                 let t = w1.get_bits(24..=27);
//     //                 let lrs = w1.get_bits(12..=24);
//     //                 let lrt = w1.get_bits(0..=11);

//     //                 log::warn!(
//     //                     "SP: G_SETTILESIZE @ {:08X} ({}, {}) ({}, {}) {:X}",
//     //                     pc,
//     //                     uls,
//     //                     ult,
//     //                     lrs,
//     //                     lrt,
//     //                     t
//     //                 );
//     //             }
//     //             G_LOADBLOCK => log::warn!("SP: G_LOADBLOCK @ {:08X}", pc),
//     //             G_LOADTILE => log::warn!("SP: G_LOADTILE @ {:08X}", pc),
//     //             G_SETTILE => {
//     //                 let format = w0.get_bits(21..=23);
//     //                 let pixel_bits = w0.get_bits(19..=20);
//     //                 //TODOlet line = w0.get_bits(19..=17);
//     //                 let tile = w1.get_bits(24..=26);
//     //                 let palette = w1.get_bits(20..=23);
//     //                 let clamp_mirror_t = w1.get_bits(18..=19);
//     //                 let mask_t = w1.get_bits(14..=17);
//     //                 let shift_t = w1.get_bits(10..=13);
//     //                 let clamp_mirror_s = w1.get_bits(8..=9);
//     //                 let mask_s = w1.get_bits(4..=7);
//     //                 let shift_s = w1.get_bits(0..=3);

//     //                 log::warn!("SP: G_SETTILE @ {:08X}", pc,);
//     //             }
//     //             G_FILLRECT => {
//     //                 let lrx = w0.get_bits(12..=24);
//     //                 let lry = w0.get_bits(0..=11);
//     //                 let ulx = w1.get_bits(12..=24);
//     //                 let uly = w1.get_bits(0..=11);

//     //                 log::warn!(
//     //                     "SP: G_FILLRECT @ {:08X} ({}, {}) ({}, {})",
//     //                     pc,
//     //                     lrx,
//     //                     lry,
//     //                     ulx,
//     //                     uly,
//     //                 );
//     //             }
//     //             G_SETFILLCOLOR => {
//     //                 let color = w1;
//     //                 log::warn!("SP: G_SETFILLCOLOR @ {:08X} {:X}", pc, color);
//     //             }
//     //             G_SETFOGCOLOR => {
//     //                 let color = w1;
//     //                 log::warn!("SP: G_SETFOGCOLOR @ {:08X} {:X}", pc, color);
//     //             }
//     //             G_SETBLENDCOLOR => {
//     //                 let color = w1;
//     //                 log::warn!("SP: G_SETBLENDCOLOR @ {:08X} {:X}", pc, color);
//     //             }
//     //             G_SETPRIMCOLOR => {
//     //                 let color = w1;
//     //                 log::warn!("SP: G_SETPRIMCOLOR @ {:08X} {:X}", pc, color);
//     //             }
//     //             G_SETENVCOLOR => {
//     //                 let color = w1;
//     //                 log::warn!("SP: G_SETENVCOLOR @ {:08X} {:X}", pc, color);
//     //             }
//     //             G_SETCOMBINE => log::warn!("SP: G_SETCOMBINE @ {:08X}", pc),
//     //             G_SETTIMG => {
//     //                 let width = (w0 & 0x0FFF) + 1;
//     //                 let addr = w1;
//     //                 log::warn!("SP: G_SETTIMG @ {:08X} {}, {:X}", pc, width, addr);
//     //             }
//     //             G_SETZIMG => {
//     //                 let addr = w1;
//     //                 log::warn!("SP: G_SETZIMG @ {:08X}  {:X}", pc, addr);
//     //             }
//     //             G_SETCIMG => {
//     //                 let width = (w0 & 0x0FFF) + 1;
//     //                 let addr = w1;
//     //                 log::warn!("SP: G_SETCIMG @ {:08X} {}, {:X}", pc, width, addr);
//     //             }
//     //             _ => log::warn!("SP: Unknown opcode: {:02x} @ {:08X}", opcode, pc),
//     //         }
//     //         pc += 8;
//     //     }
//     // } else {
//     //     log::warn!(
//     //         "SP:  TASK other {:X} {:X} {:X} {:X}",
//     //         s.sp.mem[task_header_addr],
//     //         s.sp.mem[task_header_addr + 1],
//     //         s.sp.mem[task_header_addr + 2],
//     //         s.sp.mem[task_header_addr + 3]
//     //     );
//     // }
// }
