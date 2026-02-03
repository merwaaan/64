use crate::{
    breakpoints::Breakpoints,
    cart::Cart,
    instructions::{DelayedBranching, decode},
    registers::Registers,
};

pub struct CPU {
    pub regs: Registers,

    pub rdram: Vec<u8>,

    pub rspdmem: Vec<u8>,
    pub rspimem: Vec<u8>,
    pub rsp_regs: [u32; 8],

    pub pi_regs: [u32; 5],

    delayed_branching: Option<DelayedBranching>,

    cart: Cart,

    pub breakpoints: Breakpoints,
}

impl CPU {
    pub fn new(cart: Cart) -> Self {
        Self {
            regs: Registers::default(),

            rdram: vec![0; 0x03EF_0000],

            rspdmem: vec![0; 0x1000],
            rspimem: vec![0; 0x1000],
            rsp_regs: [0; 8],

            pi_regs: [0; 5],

            delayed_branching: None,

            cart,

            breakpoints: Breakpoints::default(),
        }
    }

    // NOTE: IPL starts at A4000040, executes the cart boot sequence, skipped for now

    pub fn skip_ipl(&mut self) {
        // Setup the registers as IPL would have done

        self.regs.gpr[11] = 0xA4000040;
        self.regs.gpr[20] = 0x00000001;
        self.regs.gpr[22] = 0x0000003F;
        self.regs.gpr[29] = 0xA4001FF0;

        // TODO cop0 (readthedocs)

        // Copy the cart's boot code to memory

        // TODO which size?
        self.rspdmem[0..0x1000].copy_from_slice(&self.cart.data[0..0x1000]);

        // Start execution

        self.regs.pc = 0xA4000040;
    }

    pub fn step(&mut self) -> bool {
        let instruction = self.read(self.regs.pc);

        if instruction == 0x74027 {
            panic!("PC: {:08X}", self.regs.pc);
        }

        let handler = decode(instruction);

        let next_delayed_branching = handler.execute(self, instruction);

        match self.delayed_branching.take() {
            Some(DelayedBranching(target)) => self.regs.pc = target,
            None => self.regs.pc += 4,
        }

        self.delayed_branching = next_delayed_branching;

        if self.breakpoints.contains(self.regs.pc) {
            log::info!("Breakpoint hit at {:08X}", self.regs.pc);
            true
        } else {
            false
        }
    }

    fn read_word(&self, addr: u32, data: &[u8]) -> u32 {
        let word = data[addr as usize] as u32;
        let word = word << 8 | data[addr as usize + 1] as u32;
        let word = word << 8 | data[addr as usize + 2] as u32;
        word << 8 | data[addr as usize + 3] as u32
    }

    // TODO bad?
    pub fn read8(&self, addr: u32) -> u8 {
        let addr32 = addr & !3;
        let value32 = self.read(addr32);

        let byte_offset = addr & 3;

        (value32 >> ((3 - byte_offset) * 8)) as u8
    }

    pub fn read(&self, addr: u32) -> u32 {
        // TODO just mask?
        let physical_addr = match addr {
            0x0000_0000..=0x7FFF_FFFF => addr,
            0x8000_0000..=0x9FFFFFFF => addr - 0x8000_0000,
            0xA000_0000..=0xBFFFFFFF => addr - 0xA000_0000,
            _ => panic!("Invalid address: {:#06X}", addr),
        };

        match physical_addr {
            // RDRAM
            0..=0x03EF_FFFF => self.read_word(physical_addr, &self.rdram),

            // RDRAM registers
            0x03F0_0000..=0x03F7_FFFF => {
                log::warn!("Reading from RDRAM registers: {:08X}", physical_addr);
                0
            }

            // TODO RDRAM registers broadcast?

            // RSP DMEM
            0x0400_0000..=0x0400_0FFF => {
                let rspdmem_addr = physical_addr - 0x0400_0000;
                self.read_word(rspdmem_addr, &self.rspdmem)
            }

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => {
                let rspimem_addr = physical_addr - 0x0400_1000;
                self.read_word(rspimem_addr, &self.rspimem)
            }

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                log::warn!("Read RSP REGS: {:08X}", physical_addr);
                let rsp_regs = (physical_addr >> 2) & 3;
                self.rsp_regs[rsp_regs as usize]
            }

            // MI interface
            0x0430_0000..=0x043F_FFFF => {
                let mips_reg = ((physical_addr >> 2) & 7) as usize;

                match mips_reg {
                    0 => {
                        log::warn!("read MI_MODE");
                        0
                    }
                    1 => {
                        log::warn!("read MI_VERSION");
                        0
                    }
                    2 => {
                        log::warn!("read MI_INTERRUPT");
                        0
                    }
                    3 => {
                        log::warn!("read MI_MASK");
                        0
                    }
                    _ => panic!("Invalid MIPS register: {:08X}", mips_reg),
                }
            }

            // PI interface
            0x0460_0000..=0x046FFFFF => {
                log::warn!("Reading from PI interface: {:08X}", physical_addr);
                0
            }

            // RDRAM interface
            0x0470_0000..=0x047D_DFFF => {
                log::warn!("Reading from RDRAM interface: {:08X}", physical_addr);

                if physical_addr == 0x0470_000C {
                    log::warn!("Reading from RDRAM interface 0x14: {:08X}", physical_addr);
                    0x14
                } else {
                    0
                }
            }

            // SI registers
            0x0480_0000..=0x048F_FFFF => {
                let si_reg = ((physical_addr >> 2) & 7) as usize;

                match si_reg {
                    0 => {
                        log::warn!("read SI_DRAM_ADDR");
                        0
                    }
                    1 => {
                        log::warn!("read SI_PIF_AD_RD64B");
                        0
                    }
                    2 => {
                        log::warn!("read SI_PIF_AD_WR4B");
                        0
                    }
                    4 => {
                        log::warn!("read SI_PIF_AD_WR64B");
                        0
                    }
                    5 => {
                        log::warn!("read SI_PIF_AD_RD4B");
                        0
                    }
                    6 => {
                        log::warn!("read SI_STATUS");
                        0
                    }
                    _ => panic!("Invalid SI register: {:08X}", si_reg),
                }
            }

            // Cartridge
            0x1000_0000..=0x1FBFFFFF => {
                let cart_addr = physical_addr - 0x1000_0000;
                self.read_word(cart_addr, &self.cart.data)
            }

            // PIF RAM
            0x1FC0_07C0..=0x1FC0_07FF => {
                log::warn!("read PIF RAM: {:08X}", physical_addr);
                0
            }

            _ => panic!("Invalid read address: {:032X}", physical_addr),
        }
    }

    // TODO bad?
    pub fn write8(&mut self, addr: u32, data: u8) {
        let addr32 = addr & !3;
        let mut value32 = self.read(addr32);

        let byte_offset = addr & 3;

        value32 = value32 & !(0xFF << ((3 - byte_offset) * 8))
            | ((data as u32) << ((3 - byte_offset) * 8));

        self.write(addr32, value32);
    }

    pub fn write(&mut self, addr: u32, data: u32) {
        // TODO assert aligned? read too?

        let physical_addr = match addr {
            0..=0x7FFF_FFFF => addr,
            0x8000_0000..=0x9FFFFFFF => addr - 0x8000_0000,
            0xA000_0000..=0xBFFFFFFF => addr - 0xA000_0000,
            0xC000_0000..=0xDFFFFFFF => panic!("Invalid address: {:08X}", addr),
            0xE000_0000..=0xFFFFFFFF => panic!("Invalid address: {:08X}", addr),
        };

        match physical_addr {
            // RDRAM
            0..=0x03EF_FFFF => {
                self.rdram[physical_addr as usize] = (data >> 24) as u8;
                self.rdram[physical_addr as usize + 1] = (data >> 16) as u8;
                self.rdram[physical_addr as usize + 2] = (data >> 8) as u8;
                self.rdram[physical_addr as usize + 3] = (data & 0xFF) as u8;
            }

            // RSP DMEM
            0x0400_0000..=0x0400_0FFF => {
                log::warn!("Write {:x} to RSP DMEM {:08X}", data, physical_addr);
            }

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => {
                log::warn!("Write {:x} to RSP IMEM {:08X}", data, physical_addr);
            }

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                let rsp_reg = ((physical_addr >> 2) & 7) as usize;

                match rsp_reg {
                    0 => {
                        log::warn!("write SP_DMA_SPADDR {:x}", data);
                    }
                    1 => {
                        log::warn!("write SP_DMA_RAMADDR {:x}", data);
                    }
                    2 => {
                        log::warn!("write SP_DMA_RDLEN {:x}", data);
                    }
                    3 => {
                        log::warn!("write SP_DMA_WRLEN {:x}", data);
                    }
                    4 => {
                        log::warn!("write SP_STATUS {:x}", data);
                    }
                    5 => {
                        log::warn!("write SP_DMA_FULL {:x}", data);
                    }
                    6 => {
                        log::warn!("write SP_DMA_BUSY {:x}", data);
                    }
                    7 => {
                        log::warn!("write SP_SEMAPHORE {:x}", data);
                    }
                    _ => panic!("Invalid RSP register: {:08X}", rsp_reg),
                }
            }

            // MIPS registers
            0x0430_0000..=0x043F_FFFF => {
                let mips_reg = ((physical_addr >> 2) & 7) as usize;

                match mips_reg {
                    0 => {
                        log::warn!("write MI_MODE {:x}", data);
                    }
                    1 => {
                        log::warn!("write MI_VERSION {:x}", data);
                    }
                    2 => {
                        log::warn!("write MI_INTERRUPT {:x}", data);
                    }
                    3 => {
                        log::warn!("write MI_MASK {:x}", data);
                    }
                    _ => panic!("Invalid MIPS register: {:08X}", mips_reg),
                }
            }

            // AI registers
            0x0450_0000..=0x045F_FFFF => {
                let ai_reg = ((physical_addr >> 2) & 7) as usize;

                match ai_reg {
                    0 => {
                        log::warn!("write AI_DRAM_ADDR {:x}", data);
                    }
                    1 => {
                        log::warn!("write AI_LENGTH  {:x}", data);
                    }
                    2 => {
                        log::warn!("write AI_STATUS   {:x}", data);
                    }
                    3 => {
                        log::warn!("write AI_DACRATE  {:x}", data);
                    }
                    4 => {
                        log::warn!("write AI_BITRATE   {:x}", data);
                    }
                    _ => panic!("Invalid AI register: {:08X}", ai_reg),
                }
            }

            // PI interface
            0x0460_0000..=0x046FFFFF => {
                // TODO just mask and index?

                match physical_addr {
                    // DRAM_ADDR
                    0x0460_0000 => {
                        log::warn!("write PI_DRAM_ADDR {:x}", data);
                        self.pi_regs[0] = data & 0x00FF_FFFE;
                    }
                    // CART_ADDR
                    0x0460_0004 => {
                        log::warn!("write CART_ADDR {:x}", data);
                        self.pi_regs[1] = data & 0xFFFF_FFFE;
                    }
                    // RD_LEN
                    // TODO
                    // WR_LEN
                    0x0460_000C => {
                        log::warn!("write WR_LEN {:x}", data);
                        self.pi_regs[3] = data & 0x00FF_FFFF;

                        // TODO proper DMA transfer

                        log::warn!(
                            "PI DMA transfer: {:08X} from 0x{:08X} to 0x{:08X}",
                            self.pi_regs[3],
                            self.pi_regs[1],
                            self.pi_regs[0],
                        );

                        for offset in 0..=self.pi_regs[3] {
                            self.write(
                                self.pi_regs[0] + offset,
                                self.read(self.pi_regs[1] + offset),
                            );
                        }
                    }
                    // PI_STATUS
                    0x0460_0010 => {
                        log::warn!("write PI_STATUS {:x}", data);
                        self.pi_regs[4] = data & 0xFFFFFFFE;
                    }
                    _ => panic!("Invalid PI interface address: {:08X}", physical_addr),
                }
                //log::warn!("Writing to PI interface: {:08X}", physical_addr);
            }

            // SI registers
            0x0480_0000..=0x048F_FFFF => {
                let si_reg = ((physical_addr >> 2) & 7) as usize;

                match si_reg {
                    0 => {
                        log::warn!("write SI_DRAM_ADDR  {:x}", data);
                    }
                    1 => {
                        log::warn!("write SI_PIF_AD_RD64B   {:x}", data);
                    }
                    2 => {
                        log::warn!("write SI_PIF_AD_WR4B    {:x}", data);
                    }
                    4 => {
                        log::warn!("write SI_PIF_AD_WR64B   {:x}", data);
                    }
                    5 => {
                        log::warn!("write SI_PIF_AD_RD4B    {:x}", data);
                    }
                    6 => {
                        log::warn!("write SI_STATUS     {:x}", data);
                    }
                    _ => panic!("Invalid SI register: {:08X}", si_reg),
                }
            }

            // PIF RAM
            0x1FC0_07C0..=0x1FC0_07FF => {
                log::warn!("write PIF RAM: {:X}", data);
            }

            _ => panic!("Invalid write address: {:032X}", addr),
        }
    }
}
