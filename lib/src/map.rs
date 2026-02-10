use crate::{
    ai::{self, Ai},
    data::Data,
    mi::{self, Mi},
    pi::{self, Pi},
    si::{self, Si},
    system::System,
    vi::{self, Vi},
};

pub struct Map {
    pub rdram: Vec<u8>, // TODO to struct

    // TODO to rsp struct
    pub rspdmem: Vec<u8>,
    pub rspimem: Vec<u8>,
    pub rsp_regs: [u32; 8],

    pub mi: Mi,
    pub vi: Vi,
    pub ai: Ai,
    pub pi: Pi,
    pub si: Si,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            rdram: vec![0; RDRAM_SIZE],

            rspdmem: vec![0; 0x1000],
            rspimem: vec![0; 0x1000],
            rsp_regs: [0; 8],

            mi: Mi::default(),
            vi: Vi::default(),
            ai: Ai::default(),
            pi: Pi::default(),
            si: Si::default(),
        }
    }
}

const RDRAM_START: u32 = 0x0000_0000;
const RDRAM_SIZE: usize = 0x03F0_0000;
const RDRAM_END: u32 = RDRAM_START + RDRAM_SIZE as u32;

const RDRAM_REG_START: u32 = 0x03F0_0000;
const RDRAM_REG_SIZE: usize = 0x0008_0000;
const RDRAM_REG_END: u32 = RDRAM_REG_START + RDRAM_REG_SIZE as u32;

const RDRAM_REG_BROADCAST_START: u32 = 0x03F8_0000;
const RDRAM_REG_BROADCAST_SIZE: usize = 0x0008_0000;
const RDRAM_REG_BROADCAST_END: u32 = RDRAM_REG_BROADCAST_START + RDRAM_REG_BROADCAST_SIZE as u32;

impl Map {
    // fn read_word(addr: u32, buffer: &[u8]) -> u32 {
    //     let word = buffer[addr as usize] as u32;
    //     let word = word << 8 | buffer[addr as usize + 1] as u32;
    //     let word = word << 8 | buffer[addr as usize + 2] as u32;
    //     word << 8 | buffer[addr as usize + 3] as u32
    // }

    // // TODO bad?
    // pub fn read8(s: &mut System, addr: u32) -> u8 {
    //     let addr32 = addr & !3;
    //     let value32 = Self::read(s, addr32);

    //     let byte_offset = addr & 3;

    //     (value32 >> ((3 - byte_offset) * 8)) as u8
    // }

    pub fn read<T: Data>(s: &System, addr: u32) -> T {
        let addr = virtual_to_physical_address(addr);

        match addr {
            // RDRAM
            RDRAM_START..RDRAM_END => T::read(&s.map.rdram, addr - RDRAM_START),

            // RDRAM registers
            RDRAM_REG_START..RDRAM_REG_END => {
                panic!("Reading from RDRAM registers: {:08X}", addr);
                log::warn!("Reading from RDRAM registers: {:08X}", addr);
                T::default()
            }

            // TODO RDRAM registers broadcast?

            // RSP DMEM
            0x0400_0000..=0x0400_0FFF => T::read(&s.map.rspdmem, addr - 0x0400_0000),

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => T::read(&s.map.rspimem, addr - 0x0400_1000),

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                log::warn!("Read RSP REGS: {:08X}", addr);
                let rsp_reg = (addr >> 2) & 3;
                T::from_u32(s.map.rsp_regs[rsp_reg as usize]) // TODO weirddd
            }

            mi::START..mi::END => s.map.mi.read(addr),
            vi::START..vi::END => s.map.vi.read(addr),
            ai::START..ai::END => s.map.ai.read(addr),
            pi::START..pi::END => s.map.pi.read(addr),

            // RDRAM interface
            0x0470_0000..=0x047D_DFFF => {
                log::warn!("Reading from RDRAM interface: {:08X}", addr);

                if addr == 0x0470_000C {
                    log::warn!("Reading from RDRAM interface 0x14: {:08X}", addr);
                    T::from_u32(0x14)
                } else {
                    T::default()
                }
            }

            si::START..si::END => s.map.si.read(addr),

            // DD
            0x0500_0000..=0x05FF_FFFF => {
                // Open bus: https://n64brew.dev/wiki/Parallel_Interface#Open_bus_behavior

                let lo = addr & 0xFFFF;
                T::from_u32((lo << 16) | lo) // TODO weirddd
            }

            // Cartridge
            0x1000_0000..=0x1FBFFFFF => {
                T::read(&s.cart.data, addr - 0x1000_0000) // TODO
            }

            // PIF RAM
            0x1FC0_07C0..=0x1FC0_07FF => {
                log::warn!("read PIF RAM: {:08X}", addr);
                T::default()
            }

            _ => panic!(
                "Invalid read address: {:032X} @ {:08X}",
                addr, s.cpu.regs.pc
            ),
        }
    }

    // fn write_word(offset: u32, data: u32, buffer: &mut [u8]) {
    //     buffer[offset as usize] = (data >> 24) as u8;
    //     buffer[offset as usize + 1] = (data >> 16) as u8;
    //     buffer[offset as usize + 2] = (data >> 8) as u8;
    //     buffer[offset as usize + 3] = (data & 0xFF) as u8;
    // }

    // // TODO bad?
    // pub fn write8(s: &mut System, addr: u32, data: u8) {
    //     let addr32 = addr & !3;
    //     let mut value32 = s.read(addr32);

    //     let byte_offset = addr & 3;

    //     value32 = value32 & !(0xFF << ((3 - byte_offset) * 8))
    //         | ((data as u32) << ((3 - byte_offset) * 8));

    //     s.write(addr32, value32);
    // }

    // pub fn write16(s: &mut System, addr: u32, data: u16) {
    //     let addr32 = addr & !3;
    //     let mut value32 = s.read(addr32);

    //     let byte_offset = addr & 3;

    //     value32 = value32 & !(0xFF << ((3 - byte_offset) * 8))
    //         | ((data as u32) << ((3 - byte_offset) * 8));

    //     s.write(addr32, value32);
    // }

    // TODO what if address crosses a boundary?
    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        if (RDRAM_REG_BROADCAST_START..RDRAM_REG_BROADCAST_END).contains(&addr) {
            panic!(
                "Writing to RDRAM registers: {:08X} @ {:08X}",
                addr, s.cpu.regs.pc
            );
        }

        let addr = virtual_to_physical_address(addr);

        match addr {
            RDRAM_START..RDRAM_END => {
                data.write(&mut s.map.rdram, addr - RDRAM_START);
            }

            RDRAM_REG_START..RDRAM_REG_END => {
                log::warn!("write RDRAM_REG_0 {:X}", data); // TODO
            }

            RDRAM_REG_BROADCAST_START..RDRAM_REG_BROADCAST_END => {
                log::warn!("write RDRAM_REG_BROADCAST {:X}", data); // TODO
            }

            // RSP DMEM
            0x0400_0000..=0x0400_0FFF => {
                data.write(&mut s.map.rspdmem, addr - 0x0400_0000);
            }

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => {
                data.write(&mut s.map.rspimem, addr - 0x0400_1000);
            }

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                let rsp_reg = ((addr >> 2) & 7) as usize;

                match rsp_reg {
                    0 => {
                        log::warn!("write SP_DMA_SPADDR {:X}", data);
                    }
                    1 => {
                        log::warn!("write SP_DMA_RAMADDR {:X}", data);
                    }
                    2 => {
                        log::warn!("write SP_DMA_RDLEN {:X}", data);
                        log::warn!("SP DMA------------------------------------------");
                    }
                    3 => {
                        log::warn!("write SP_DMA_WRLEN {:X}", data);
                        log::warn!("SP DMA------------------------------------------");
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
                    _ => panic!("Invalid RSP register: {:08X}", rsp_reg),
                }
            }

            mi::START..mi::END => Mi::write(s, addr, data),
            vi::START..vi::END => Vi::write(s, addr, data),
            ai::START..ai::END => Ai::write(s, addr, data),

            pi::START..pi::END => Pi::write(s, addr, data),
            si::START..si::END => Si::write(s, addr, data),

            // PIF RAM
            0x1FC0_07C0..=0x1FC0_07FF => {
                log::warn!("write PIF RAM: {:X}", data);
            }

            _ => panic!("Invalid write address: {:032X}", addr),
        }
    }
}

pub fn virtual_to_physical_address(addr: u32) -> u32 {
    // TODO TLB

    match addr {
        0x0000_0000..=0x7FFF_FFFF => addr,

        // TOD just mask below?
        0x8000_0000..=0x9FFF_FFFF => addr - 0x8000_0000,
        0xA000_0000..=0xBFFF_FFFF => addr - 0xA000_0000,
        0xC000_0000..=0xDFFF_FFFF => addr - 0xC000_0000,
        0xE000_0000..=0xFFFF_FFFF => addr - 0xE000_0000,
    }
}

pub fn address_info(addr: u32) -> Option<&'static str> {
    let addr = virtual_to_physical_address(addr);

    // TODO delegate!!!!!
    // TODO check masks!
    // TODO normalize strings

    let s = match addr {
        0x03F0_0000..=0x03F7_FFFF => match addr & 0x3F {
            0x00 => "RDRAM device type",
            0x04 => "RDRAM device ID",
            0x08 => "RDRAM delay",
            0x0C => "RDRAM mode",
            0x10 => "RDRAM RefInterval",
            0x14 => "RDRAM RefRow",
            0x18 => "RDRAM RasInterval",
            0x1C => "RDRAM MinInterval ",
            0x20 => "RDRAM AddressSelect  ",
            0x24 => "RDRAM DeviceManufacturer  ",
            _ => "",
        },

        // TODO rdram write only?
        0x0400_0000..=0x048F_FFFF => match addr {
            0x0400_0000..=0x0400_0FFF => "RSP DMEM",
            0x0400_1000..=0x0400_1FFF => "RSP IMEM",
            0x0404_0000..=0x040B_FFFF => match addr & 0x3F {
                0x00 => "SP_DMA_SPADDR",
                0x04 => "SP_DMA_RAMADDR",
                0x08 => "SP_DMA_RDLEN",
                0x0C => "SP_DMA_WRLEN",
                0x10 => "SP_STATUS",
                0x14 => "SP_DMA_FULL",
                0x18 => "SP_DMA_BUSY",
                0x1C => "SP_SEMAPHORE",
                _ => "",
            },

            0x0410_0000..=0x042F_FFFF => "RDP command registers TODO",

            0x0430_0000..=0x043F_FFFF => match addr & 0x3F {
                0x00 => "MI_MODE",
                0x04 => "MI_VERSION",
                0x08 => "MI_INTERRUPT",
                0x0C => "MI_MASK",
                _ => "",
            },

            0x0440_0000..=0x044F_FFFF => "VI TODO",

            0x0450_0000..=0x045F_FFFF => "AI TODO",

            0x0460_0000..=0x046F_FFFF => match addr & 0x3F {
                0x00 => "PI_DRAM_ADDR",
                0x04 => "PI_CART_ADDR",
                0x08 => "PI_RD_LEN",
                0x0C => "PI_WR_LEN",
                0x10 => "PI_STATUS",
                0x14 => "PI_BSD_DOM1_LAT",
                0x18 => "PI_BSD_DOM1_PWD",
                0x20 => "PI_BSD_DOM1_RLS",
                0x24 => "PI_BSD_DOM2_LAT",
                0x28 => "PI_BSD_DOM2_PWD",
                0x1C => "PI_BSD_DOM1_PGS",
                0x2C => "PI_BSD_DOM2_PGS",
                0x30 => "PI_BSD_DOM2_RLS",
                _ => "",
            },

            0x0470_0000..=0x047F_FFFF => match addr & 0x3F {
                0x00 => "RI_MODE",
                0x04 => "RI_CONFIG",
                0x08 => "RI_CURRENT_LOAD",
                0x0C => "RI_SELECT",
                0x10 => "RI_REFRESH",
                0x14 => "RI_LATENCY",
                0x18 => "RI_ERROR",
                0x1C => "RI_BANK_STATUS",
                _ => "",
            },

            0x0480_0000..=0x048F_FFFF => match addr & 0x3F {
                0x00 => "SI_DRAM_ADDR",
                0x04 => "SI_PIF_AD_RD64B",
                0x08 => "SI_PIF_AD_WR4B",
                0x10 => "SI_PIF_AD_WR64B",
                0x14 => "SI_PIF_AD_RD4B",
                0x18 => "SI_STATUS",
                _ => "",
            },

            _ => "",
        },

        // TODO others
        _ => "",
    };

    if s.is_empty() { None } else { Some(s) }
}
