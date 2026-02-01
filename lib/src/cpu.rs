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

    delayed_branching: Option<DelayedBranching>,

    pub breakpoints: Breakpoints,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),

            rdram: vec![0; 0x03F_0000],
            rspdmem: vec![0; 0x1000],
            rspimem: vec![0; 0x1000],

            delayed_branching: None,

            breakpoints: Breakpoints::new(),
        }
    }

    // NOTE: IPL starts at A4000040, executes the cart boot sequence, skipped for now

    pub fn skip_ipl(&mut self, cart: &Cart) {
        // Setup the registers as IPL would have done

        self.regs.gpr[11] = 0xFFFFFFFF_A4000040;
        self.regs.gpr[20] = 0x00000000_00000001;
        self.regs.gpr[22] = 0x00000000_0000003F;
        self.regs.gpr[29] = 0xFFFFFFFF_A4001FF0;

        // TODO cop0 (readthedocs)

        // Copy the cart's boot code to memory

        // TODO which size?
        self.rspdmem[0..0x1000].copy_from_slice(&cart.data[0..0x1000]);

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
            log::info!("Breakpoint hit at {:08x}", self.regs.pc);
            true
        } else {
            false
        }
    }

    pub fn read(&self, addr: u32) -> u32 {
        // TODO just mask?
        let physical_addr = match addr {
            0x8000_0000..=0x9FFFFFFF => addr - 0x8000_0000,
            0xA000_0000..=0xBFFFFFFF => addr - 0xA000_0000,
            _ => panic!("Invalid address: {:08x}", addr),
        };

        match physical_addr {
            // RDRAM
            0..=0x03EF_FFFF => {
                let word = self.rdram[physical_addr as usize] as u32;
                let word = word << 8 | self.rdram[physical_addr as usize + 1] as u32;
                let word = word << 8 | self.rdram[physical_addr as usize + 2] as u32;
                let word = word << 8 | self.rdram[physical_addr as usize + 3] as u32;
                word
            }

            // RDRAM registers
            0x03F0_0000..=0x03F7_FFFF => {
                log::warn!("Reading from RDRAM registers: {:08x}", physical_addr);
                0
            }

            // TODO RDRAM registers broadcast?

            // RSP DMEM
            0x0400_0000..=0x0400_0FFF => {
                let rspdmem_addr = physical_addr - 0x0400_0000;
                let word = self.rspdmem[rspdmem_addr as usize] as u32;
                let word = word << 8 | self.rspdmem[rspdmem_addr as usize + 1] as u32;
                let word = word << 8 | self.rspdmem[rspdmem_addr as usize + 2] as u32;
                let word = word << 8 | self.rspdmem[rspdmem_addr as usize + 3] as u32;
                word
            }

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => {
                panic!("WARN: Reading from RSP IMEM: {:08x}", physical_addr);
            }

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                panic!("WARN: Reading from RSP REGS: {:08x}", physical_addr);
            }

            // TODO others

            // RDRAM interface
            0x0470_0000..=0x047D_DFFF => {
                log::warn!("Reading from RDRAM interface: {:08x}", physical_addr);

                if physical_addr == 0x0470_000C {
                    log::warn!("Reading from RDRAM interface 0x14: {:08x}", physical_addr);
                    0x14
                } else {
                    0
                }
            }

            _ => panic!("Invalid read address: {:032x}", physical_addr),
        }
    }

    pub fn write(&mut self, addr: u32, data: u32) {
        let physical_addr = match addr {
            0..=0x7FFF_FFFF => panic!("Invalid address: {:08x}", addr),
            0x8000_0000..=0x9FFFFFFF => addr - 0x8000_0000,
            0xA000_0000..=0xBFFFFFFF => addr - 0xA000_0000,
            0xC000_0000..=0xDFFFFFFF => panic!("Invalid address: {:08x}", addr),
            0xE000_0000..=0xFFFFFFFF => panic!("Invalid address: {:08x}", addr),
        };

        match physical_addr {
            // RDRAM
            0..=0x03EF_FFFF => {
                self.rdram[physical_addr as usize] = data as u8;
                self.rdram[physical_addr as usize + 1] = (data >> 8) as u8;
                self.rdram[physical_addr as usize + 2] = (data >> 16) as u8;
                self.rdram[physical_addr as usize + 3] = (data >> 24) as u8;
            }

            _ => panic!("Invalid write address: {:032x}", addr),
        }
    }
}
