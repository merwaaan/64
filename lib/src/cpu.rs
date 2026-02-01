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

    pub pi_regs: [u32; 5],

    delayed_branching: Option<DelayedBranching>,

    cart: Cart,

    pub breakpoints: Breakpoints,
}

impl CPU {
    pub fn new(cart: Cart) -> Self {
        Self {
            regs: Registers::new(),

            rdram: vec![0; 0x03F_0000],
            rspdmem: vec![0; 0x1000],
            rspimem: vec![0; 0x1000],

            pi_regs: [0; 5],

            delayed_branching: None,

            cart,

            breakpoints: Breakpoints::new(),
        }
    }

    // NOTE: IPL starts at A4000040, executes the cart boot sequence, skipped for now

    pub fn skip_ipl(&mut self) {
        // Setup the registers as IPL would have done

        self.regs.gpr[11] = 0xFFFFFFFF_A4000040;
        self.regs.gpr[20] = 0x00000000_00000001;
        self.regs.gpr[22] = 0x00000000_0000003F;
        self.regs.gpr[29] = 0xFFFFFFFF_A4001FF0;

        // TODO cop0 (readthedocs)

        // Copy the cart's boot code to memory

        // TODO which size?
        self.rspdmem[0..0x1000].copy_from_slice(&self.cart.data[0..0x1000]);

        // Start execution

        self.regs.pc = 0xA4000040;
    }

    pub fn step(&mut self) -> bool {
        let instruction = self.read(self.regs.pc as u32);
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
        let word = word << 8 | data[addr as usize + 3] as u32;
        word
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
                panic!("Reading from RSP IMEM: {:08X}", physical_addr);
            }

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                panic!("Reading from RSP REGS: {:08X}", physical_addr);
            }

            // TODO others

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

            0x1000_0000..=0x1FBFFFFF => {
                let cart_addr = physical_addr - 0x1000_0000;
                self.read_word(cart_addr, &self.cart.data)
            }

            _ => panic!("Invalid read address: {:032X}", physical_addr),
        }
    }

    pub fn write(&mut self, addr: u32, data: u32) {
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

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => {
                log::warn!("Write to RSP IMEM {:08X}", physical_addr);
            }

            // PI interface
            0x0460_0000..=0x046FFFFF => {
                // TODO just mask and index?

                match physical_addr {
                    // DRAM_ADDR
                    0x0460_0000 => self.pi_regs[0] = data & &0xFFFFFFFE,
                    // CART_ADDR
                    0x0460_0004 => self.pi_regs[1] = data & &0xFFFFFFFE,
                    // RD_LEN
                    // TODO
                    // WR_LEN
                    0x0460_000C => {
                        self.pi_regs[3] = data & &0x00FFFFFF;

                        // TODO proper DMA transfer

                        log::info!(
                            "PI DMA transfer: {:08X} from 0x{:08X} to 0x{:08X}",
                            self.pi_regs[3],
                            self.pi_regs[1],
                            self.pi_regs[0],
                        );

                        // TODO SM64: @ 8000 0050 -> PI_WR_LEN written as FFFFF but ref emu stores 7F???

                        // TODO value is minus one!!!!!!!!!!!! just +1?

                        for offset in 0..self.pi_regs[3] {
                            self.write(
                                self.pi_regs[0] + offset,
                                self.read(self.pi_regs[1] + offset),
                            );
                        }
                    }
                    _ => panic!("Invalid PI interface address: {:08X}", physical_addr),
                }
                //log::warn!("Writing to PI interface: {:08X}", physical_addr);
            }

            _ => panic!("Invalid write address: {:032X}", addr),
        }
    }
}
