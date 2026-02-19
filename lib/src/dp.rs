use crate::{data::Data, map::Location, system::System};

const REG_START: u32 = 0x0410_0000;
const REG_END: u32 = 0x0420_0000;

pub type DpLocation = Location<REG_START, REG_END>;

#[derive(Clone)]
pub struct Dp {
    pub regs: [u32; 8],
}

impl Default for Dp {
    fn default() -> Self {
        Self { regs: [0; 8] }
    }
}

impl Dp {
    pub fn read<T: Data>(&self, addr: DpLocation) -> T {
        log::warn!("Read DP register @ {:08X}", addr.relative());

        T::default()
    }

    pub fn write<T: Data>(s: &mut System, addr: DpLocation, data: T) {
        log::warn!(
            "Write DP register @ {:08X} {:X}",
            addr.relative(),
            data.to_u32()
        );
    }

    // fn start_dma(s: &mut System, direction: DmaDirection) {
    //     let length_reg = match direction {
    //         DmaDirection::RamToSp => s.map.rsp.regs[Register::DmaRdLen as usize],
    //         DmaDirection::SpToRam => s.map.rsp.regs[Register::DmaWrLen as usize],
    //     };

    //     // Number of bytes to copy per "row"
    //     // (length < 8 = transfer 8 bytes anyway)

    //     let bytes_per_row = ((length_reg & 0x0FFF) + 1).min(8);

    //     // Number of rows to copy

    //     let rows = ((length_reg >> 12) & 0x00FF) + 1;

    //     // Number of bytes to skip after each rom
    //     // (only applies to the RAM side!)

    //     let skips = (length_reg >> 20) & !7;

    //     let mut ram_addr = s.map.rsp.regs[Register::DmaRamAddr as usize] & 0x00FF_FFF8;
    //     let mut sp_addr = s.map.rsp.regs[Register::DmaSpAddr as usize] & 0x0000_1FF8;

    //     let sp_bank_offset = sp_addr & 0x100;

    //     match direction {
    //         DmaDirection::RamToSp => {
    //             log::info!(
    //                 "SP DMA: {:X} bytes from RAM {:08X} to RSP {:08X} (C={:X}/S={:X})",
    //                 bytes_per_row,
    //                 ram_addr,
    //                 sp_addr,
    //                 rows,
    //                 skips
    //             );

    //             for _ in 0..rows {
    //                 for byte in 0..bytes_per_row {
    //                     let data = s.read::<u8>(ram_addr + byte);

    //                     s.map.rsp.mem[(sp_addr + byte) as usize] = data;
    //                 }

    //                 // The transper wraps around the current bank
    //                 sp_addr = (sp_addr + bytes_per_row) & 0x0FFF + sp_bank_offset;

    //                 ram_addr += bytes_per_row + skips;
    //             }
    //         }
    //         DmaDirection::SpToRam => {
    //             log::info!(
    //                 "SP DMA: {:X} bytes from SP {:08X} to RAM {:08X} (C={:X}/S={:X})",
    //                 bytes_per_row,
    //                 sp_addr,
    //                 ram_addr,
    //                 rows,
    //                 skips
    //             );

    //             for _ in 0..rows {
    //                 for byte in 0..bytes_per_row {
    //                     let data = s.map.rsp.mem[(sp_addr + byte) as usize];

    //                     s.write::<u8>(ram_addr + byte, data);
    //                 }

    //                 sp_addr = (sp_addr + 1) & 0x0FFF + sp_bank_offset;

    //                 ram_addr += bytes_per_row + skips;
    //             }
    //         }
    //     }

    //     2;

    //     // Update the status register

    //     s.map.rsp.regs[Register::Status as usize] |= STATUS_DMA_BUSY;
    //     s.map.rsp.regs[Register::Status as usize] &= !STATUS_DMA_FULL;

    //     // TODO reset count to 0!
    //     // TODO IO busy?
    //     // TODO DMA error? if already busy? queue?

    //     // TODO schedule status update

    //     s.events.push(Event {
    //         id: EventType::RspDmaTransferComplete,
    //         cycle: s.cycles + (bytes_per_row / 8) as usize, // TODO currently just copied from pi
    //     });
    // }

    // pub fn dma_completed(s: &mut System) {
    //     // Update the status register

    //     s.map.rsp.regs[Register::Status as usize] &= !STATUS_DMA_BUSY;
    //     // TODO IO busy?

    //     // Raise the interrupt

    //     s.map.mi.set_pending_interrupt(Interrupt::Sp);
    // }
}
