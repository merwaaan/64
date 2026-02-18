use crate::{data::Data, map::Location, system::System};

pub const DMEM_START: u32 = 0x0000_0000;
pub const DMEM_END: u32 = 0x0401_0000;
pub const DMEM_MASK: u32 = 0x0FFF;

pub type RspDmemLocation = Location<DMEM_START, DMEM_END>;

pub const IMEM_START: u32 = DMEM_END;
pub const IMEM_END: u32 = 0x0402_0000;
pub const IMEM_MASK: u32 = 0x0FFF;

pub type RspImemLocation = Location<IMEM_START, IMEM_END>;

pub const REG_START: u32 = 0x0404_0000;
pub const REG_END: u32 = 0x040C_0000;
pub const REG_MASK: u32 = 0x1F;

pub type RspRegsLocation = Location<REG_START, REG_END>;

pub struct Rsp {
    pub dmem: Vec<u8>, // TODO not pub
    pub imem: Vec<u8>,
    pub regs: [u32; 8],
}

impl Default for Rsp {
    fn default() -> Self {
        Self {
            dmem: vec![0; 0x1000],
            imem: vec![0; 0x1000],
            regs: [0; 8],
        }
    }
}

impl Rsp {
    pub fn read_dmem<T: Data>(&self, addr: RspDmemLocation) -> T {
        T::read(&self.dmem, addr.relative() & DMEM_MASK)
    }

    pub fn write_dmem<T: Data>(s: &mut System, addr: RspDmemLocation, data: T) {
        data.write(&mut s.map.rsp.dmem, addr.relative() & DMEM_MASK);
    }
    pub fn read_imem<T: Data>(&self, addr: RspImemLocation) -> T {
        T::read(&self.imem, addr.relative() & IMEM_MASK)
    }

    pub fn write_imem<T: Data>(s: &mut System, addr: RspImemLocation, data: T) {
        data.write(&mut s.map.rsp.imem, addr.relative() & IMEM_MASK);
    }

    pub fn read_reg<T: Data>(&self, addr: RspRegsLocation) -> T {
        log::warn!("Read RSP reg UNIMPLEMENTED: {:08X}", addr.relative());

        let reg = ((addr.relative() & REG_MASK) >> 2) as usize;

        T::from_u32(self.regs[reg])
    }

    pub fn write_reg<T: Data>(s: &mut System, addr: RspRegsLocation, data: T) {
        log::warn!(
            "Write RSP reg UNIMPLEMENTED: {:08X} {:X}",
            addr.relative(),
            data.to_u32()
        );

        let reg = ((addr.relative() & REG_MASK) >> 2) as usize;

        match reg {
            0 => {
                log::warn!("write SP_DMA_SPADDR {:X}", data);
            }
            1 => {
                log::warn!("write SP_DMA_RAMADDR {:X}", data);
            }
            2 => {
                log::warn!("write SP_DMA_RDLEN {:X}", data);
                log::error!("SP DMA NOT IMPLEMENTED");
            }
            3 => {
                log::warn!("write SP_DMA_WRLEN {:X}", data);
                log::error!("SP DMA NOT IMPLEMENTED");
            }
            4 => {
                log::warn!("write SP_STATUS {:X}", data);
            }
            5 => {
                log::warn!("write SP_DMA_FULL {:X}", data);
            }
            6 => {
                log::warn!("write SP_DMA_BUSY {:X}", data);
            }
            7 => {
                log::warn!("write SP_SEMAPHORE {:X}", data);
            }
            _ => panic!("Invalid RSP register: {:08X}", reg),
        }

        s.map.rsp.regs[reg] = data.to_u32();
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
}
