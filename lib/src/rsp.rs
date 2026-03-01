use strum::{Display, EnumIter};

use crate::{
    data::Value,
    events::{EventType, Events},
    map::Location,
    mi::Interrupt,
    system::System,
};

// TODO separate i/dmem really needed?

const MEM_START: u32 = 0x0400_0000;
const MEM_END: u32 = 0x0404_0000;
const MEM_MASK: u32 = 0x1FFF;

pub type RspMemLocation = Location<MEM_START, MEM_END>;

// const IMEM_START: u32 = DMEM_END;
// const IMEM_END: u32 = 0x0400_2000;
// const IMEM_MASK: u32 = 0x0FFF;

// pub type RspImemLocation = Location<IMEM_START, IMEM_END>;

const REG_START: u32 = MEM_END;
const REG_END: u32 = 0x040C_0000;
const REG_MASK: u32 = 0x1F;

pub type RspRegsLocation = Location<REG_START, REG_END>;

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Register {
    DmaSpAddr,
    DmaRamAddr,
    DmaRdLen,
    DmaWrLen,
    Status,
    DmaFull,
    DmaBusy,
    Semaphore,
}

// TODOrm
//const STATUS_HALTED_MASK: u32 = 1;
//const STATUS_BROKE: u32 = 1 << 1;
const STATUS_DMA_BUSY: u32 = 1 << 2;
const STATUS_DMA_FULL: u32 = 1 << 3;
//const STATUS_IO_BUSY: u32 = 1 << 4;
//const STATUS_SINGLE_STEP_MODE: u32 = 1 << 5;
//const STATUS_INTERRUPT_ON_BREAK: u32 = 1 << 6;
// TODO others?

#[derive(Debug)]
enum DmaDirection {
    RamToSp,
    SpToRam,
}

#[derive(Clone)]
pub struct Rsp {
    // DMEM: 0x0000 - 0x0FFF
    // IMEM: 0x1000 - 0x1FFF
    mem: Vec<u8>,

    pub regs: [u32; 8],

    pub pc: u32,
}

impl Default for Rsp {
    fn default() -> Self {
        let mut regs = [0; 8];

        regs[Register::Status as usize] = 0x0000_0001; // TODO for lemmy

        Self {
            mem: vec![0; 0x2000],
            regs,
            pc: 0,
        }
    }
}

impl Rsp {
    pub fn read_mem<T: Value>(&self, addr: RspMemLocation) -> T {
        T::read_mem(&self.mem, addr.relative() & MEM_MASK)
    }

    pub fn write_mem<T: Value>(s: &mut System, addr: RspMemLocation, data: T) {
        data.write_mem(&mut s.map.rsp.mem, addr.relative() & MEM_MASK);
    }

    pub fn read_reg<T: Value>(&self, addr: RspRegsLocation) -> T {
        if addr.relative() < 0x4_0000 {
            T::read_reg(&self.regs, addr.relative() & REG_MASK)
        } else {
            if (addr.relative() & 3) != 0 {
                panic!("Unaligned RSP PC read: {:08X}", addr.relative());
            }

            T::default() // TODO PC
        }
    }

    pub fn write_reg<T: Value>(s: &mut System, addr: RspRegsLocation, data: T) {
        if addr.relative() < 0x4_0000 {
            let reg = ((addr.relative() & REG_MASK) >> 2) as usize;

            match reg {
                0 => {
                    log::warn!("write SP_DMA_SPADDR {:X}", data);

                    // 11-bit SP address.
                    // Bits 0-2 cannot be written to so the address is always aligned to 8 bytes.
                    // Bit 12 is the "bank" (O = DMEM, 1 = IMEM).

                    data.write_reg(&mut s.map.rsp.regs, addr.relative() & REG_MASK);

                    s.map.rsp.regs[Register::DmaSpAddr as usize] &= 0x0000_1FF8;
                }
                1 => {
                    log::warn!("write SP_DMA_RAMADDR {:X}", data);

                    // 24-bit RAM address.
                    // Bits 0-2 cannot be written to so the address is always aligned to 8 bytes.

                    // TODO reads should return the previous value until DMA starts?

                    data.write_reg(&mut s.map.rsp.regs, addr.relative() & REG_MASK);

                    s.map.rsp.regs[Register::DmaRamAddr as usize] &= 0x00FF_FFF8;
                }
                2 => {
                    log::warn!("write SP_DMA_RDLEN {:X}", data);

                    data.write_reg(&mut s.map.rsp.regs, addr.relative() & REG_MASK);

                    Self::start_dma(s, DmaDirection::RamToSp);
                }
                3 => {
                    log::warn!("write SP_DMA_WRLEN {:X}", data);

                    data.write_reg(&mut s.map.rsp.regs, addr.relative() & REG_MASK);

                    Self::start_dma(s, DmaDirection::SpToRam);
                }
                4 => {
                    log::error!("write SP_STATUS {:X}", data);

                    // TODO!

                    let mut trigger_bits = [0u32];
                    data.write_reg(&mut trigger_bits, addr.relative() & 3);

                    //let status_reg = &mut s.map.rsp.regs[Register::Status as usize];

                    let set_int = trigger_bits[0] & 16 != 0;
                    let clear_int = trigger_bits[0] & 8 != 0;

                    if clear_int && !set_int {
                        s.map.mi.clear_pending_interrupt(Interrupt::Sp, &mut s.cop0);
                    } else if !clear_int && set_int {
                        s.map.mi.set_pending_interrupt(Interrupt::Sp, &mut s.cop0);
                    }
                }
                5 => {
                    log::error!("write SP_DMA_FULL {:X}", data);
                }
                6 => {
                    log::error!("write SP_DMA_BUSY {:X}", data);
                }
                7 => {
                    log::error!("write SP_SEMAPHORE {:X}", data);
                }
                _ => panic!("Invalid RSP register: {:08X}", reg),
            }
        } else {
            if (addr.relative() & 3) != 0 {
                panic!("Unaligned RSP PC write: {:08X}", addr.relative());
            }

            let mut pc = [s.map.rsp.pc];
            data.write_reg(&mut pc, addr.relative() & 0x0000_0003);
            s.map.rsp.pc = pc[0]; // TODO mask?
        }
    }

    pub fn reg_info(addr: RspRegsLocation) -> Option<&'static str> {
        match addr.relative() & REG_MASK {
            0 => Some("RSP_DMA_SPADDR"),
            1 => Some("RSP_DMA_RAMADDR"),
            2 => Some("RSP_DMA_RDLEN"),
            3 => Some("RSP_DMA_WRLEN"),
            4 => Some("RSP_STATUS"),
            5 => Some("RSP_DMA_FULL"),
            6 => Some("RSP_DMA_BUSY"),
            7 => Some("RSP_SEMAPHORE"),
            _ => None,
        }
    }

    fn start_dma(s: &mut System, direction: DmaDirection) {
        let length_reg = match direction {
            DmaDirection::RamToSp => s.map.rsp.regs[Register::DmaRdLen as usize],
            DmaDirection::SpToRam => s.map.rsp.regs[Register::DmaWrLen as usize],
        };

        // Number of bytes to copy per "row"
        // (length < 8 = transfer 8 bytes anyway)

        let bytes_per_row = ((length_reg & 0x0FFF) | 7) + 1;
        //let bytes_per_row = ((length_reg & 0x0FFF) + 1).min(8);

        //panic!("----- {:X}", bytes_per_row);
        // Number of rows to copy

        let rows = ((length_reg >> 12) & 0x00FF) + 1;

        // Number of bytes to skip after each rom
        // (only applies to the RAM side!)

        let skips = (length_reg >> 20) & !7;

        let mut ram_addr = s.map.rsp.regs[Register::DmaRamAddr as usize];
        let mut sp_addr = s.map.rsp.regs[Register::DmaSpAddr as usize];

        let sp_bank_offset = sp_addr & 0x1000;

        match direction {
            DmaDirection::RamToSp => {
                log::info!(
                    "SP DMA: {:X} bytes * {:X} rows + {:X} skips from RAM {:08X} to RSP {:08X}",
                    bytes_per_row,
                    rows,
                    skips,
                    ram_addr,
                    sp_addr
                );

                for _ in 0..rows {
                    for byte in 0..bytes_per_row {
                        let data = s.read::<u8>(ram_addr + byte);

                        // The transfer wraps around the current bank
                        let wrapping_sp_addr = ((sp_addr + byte) & 0x0FFF) | sp_bank_offset;

                        s.map.rsp.mem[wrapping_sp_addr as usize] = data;
                    }

                    sp_addr = sp_addr.wrapping_add(bytes_per_row);

                    ram_addr = ram_addr.wrapping_add(bytes_per_row).wrapping_add(skips);
                }
            }
            DmaDirection::SpToRam => {
                log::info!(
                    "SP DMA: {:X} bytes * {:X} rows + {:X} skips from SP {:08X} to RAM {:08X}",
                    bytes_per_row,
                    rows,
                    skips,
                    sp_addr,
                    ram_addr
                );

                for _ in 0..rows {
                    for byte in 0..bytes_per_row {
                        let wrapping_sp_addr = ((sp_addr + byte) & 0x0FFF) | sp_bank_offset;

                        let data = s.map.rsp.mem[wrapping_sp_addr as usize];

                        s.write::<u8>(ram_addr + byte, data);
                    }

                    sp_addr = sp_addr.wrapping_add(bytes_per_row);

                    ram_addr = ram_addr.wrapping_add(bytes_per_row).wrapping_add(skips);
                }
            }
        }

        // Update the status register

        s.map.rsp.regs[Register::Status as usize] |= STATUS_DMA_BUSY;
        s.map.rsp.regs[Register::Status as usize] &= !STATUS_DMA_FULL;

        // TODO reset count to 0!
        // TODO IO busy?
        // TODO DMA error? if already busy? queue?

        // TODO schedule status update

        Events::push(
            s,
            EventType::RspDmaTransferComplete,
            0, //(bytes_per_row / 8) as usize, // TODO currently just copied from pi
        );
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.map.rsp.regs[Register::Status as usize] &= !STATUS_DMA_BUSY;
        // TODO IO busy?

        // Raise the interrupt

        s.map.mi.set_pending_interrupt(Interrupt::Sp, &mut s.cop0);
    }
}
