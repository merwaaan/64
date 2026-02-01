use crate::{cart::Cart, registers::Registers};

pub struct CPU {
    pub regs: Registers,

    pub rdram: Vec<u8>,
    pub rspdmem: Vec<u8>,
    pub rspimem: Vec<u8>,

    delayed_branching: Option<DelayedBranching>,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),

            rdram: vec![0; 0x03F_0000],
            rspdmem: vec![0; 0x1000],
            rspimem: vec![0; 0x1000],

            delayed_branching: None,
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

    pub fn step(&mut self) {
        println!("----------\nPC: {:08x}", self.regs.pc);

        let instruction = self.read(self.regs.pc as u32);
        println!("Instruction: {:08x}", instruction);

        let next_delayed_branching = self.execute(instruction);

        match self.delayed_branching.take() {
            Some(DelayedBranching(target)) => self.regs.pc = target,
            None => self.regs.pc += 4,
        }

        self.delayed_branching = next_delayed_branching;

        println!("GPRs: {:X?}", self.regs.gpr);
    }

    fn read(&self, addr: u32) -> u32 {
        println!("Reading from address: {:08x}", addr);

        match addr {
            0..=0x7FFF_FFFF => panic!("Invalid address: {:08x}", addr),
            0x8000_0000..=0x9FFFFFFF => self.read_physical(addr - 0x8000_0000),
            0xA000_0000..=0xBFFFFFFF => self.read_physical(addr - 0xA000_0000),
            0xC000_0000..=0xDFFFFFFF => panic!("Invalid address: {:08x}", addr),
            0xE000_0000..=0xFFFFFFFF => panic!("Invalid address: {:08x}", addr),
        }
    }

    fn read_physical(&self, addr: u32) -> u32 {
        println!("Reading from physical address: {:08x}", addr);

        match addr {
            // RDRAM
            0..=0x03EF_FFFF => {
                let word = self.rdram[addr as usize] as u32;
                let word = word << 8 | self.rdram[addr as usize + 1] as u32;
                let word = word << 8 | self.rdram[addr as usize + 2] as u32;
                let word = word << 8 | self.rdram[addr as usize + 3] as u32;
                word
            }

            // RDRAM registers
            0x03F0_0000..=0x03F7_FFFF => {
                println!("WARN: Reading from RDRAM registers: {:08x}", addr);
                0
            }

            // TODO RDRAM registers broadcast?

            // RSP DMEM
            0x0400_0000..=0x0400_0FFF => {
                let rspdmem_addr = addr - 0x0400_0000;
                let word = self.rspdmem[rspdmem_addr as usize] as u32;
                let word = word << 8 | self.rspdmem[rspdmem_addr as usize + 1] as u32;
                let word = word << 8 | self.rspdmem[rspdmem_addr as usize + 2] as u32;
                let word = word << 8 | self.rspdmem[rspdmem_addr as usize + 3] as u32;
                word
            }

            // RSP IMEM
            0x0400_1000..=0x0400_1FFF => {
                panic!("WARN: Reading from RSP IMEM: {:08x}", addr);
                0
            }

            // RSP registers
            0x0404_0000..=0x040B_FFFF => {
                panic!("WARN: Reading from RSP REGS: {:08x}", addr);
                0
            }

            // TODO others

            // RDRAM interface
            0x0470_0000..=0x047D_DFFF => {
                println!("WARN: Reading from RDRAM interface: {:08x}", addr);

                if addr == 0x0470_000C {
                    println!("WARN: Reading from RDRAM interface 0x14: {:08x}", addr);
                    0x14
                } else {
                    0
                }
            }

            _ => panic!("Invalid address: {:032x}", addr),
        }
    }

    fn execute(&mut self, instruction: u32) -> Option<DelayedBranching> {
        println!("Executing instruction: {:032b}", instruction);

        let opcode = instruction >> 26;

        match opcode {
            // Special block
            0x00 => match instruction & 0x3F {
                0x00 => {
                    println!("SLL");

                    let shift = ((instruction >> 6) & 0x1F) as usize;
                    let rd = ((instruction >> 11) & 0x1F) as usize;
                    let rt = ((instruction >> 16) & 0x1F) as usize;

                    self.regs.gpr[rd] = self.regs.gpr[rt] << shift;

                    // TODO 64 bit mode
                }

                0x24 => {
                    println!("AND");

                    let rd = ((instruction >> 11) & 0x1F) as usize;
                    let rt = ((instruction >> 16) & 0x1F) as usize;
                    let rs = ((instruction >> 21) & 0x1F) as usize;

                    self.regs.gpr[rd] = self.regs.gpr[rs] & self.regs.gpr[rt];
                }

                0x25 => {
                    println!("OR");

                    let rd = ((instruction >> 11) & 0x1F) as usize;
                    let rt = ((instruction >> 16) & 0x1F) as usize;
                    let rs = ((instruction >> 21) & 0x1F) as usize;

                    self.regs.gpr[rd] = self.regs.gpr[rs] | self.regs.gpr[rt];
                }

                0x26 => {
                    println!("XOR");

                    let rd = ((instruction >> 11) & 0x1F) as usize;
                    let rt = ((instruction >> 16) & 0x1F) as usize;
                    let rs = ((instruction >> 21) & 0x1F) as usize;

                    self.regs.gpr[rd] = self.regs.gpr[rs] ^ self.regs.gpr[rt];
                }

                0x2B => {
                    println!("SLTU");

                    let rd = ((instruction >> 11) & 0x1F) as usize;
                    let rt = ((instruction >> 16) & 0x1F) as usize;
                    let rs = ((instruction >> 21) & 0x1F) as usize;

                    self.regs.gpr[rd] = (self.regs.gpr[rs] < self.regs.gpr[rt]) as u64;
                }
                _ => panic!("Unknown opcode: {:06b}", opcode),
            },

            0x02 => {
                println!("J");
                unimplemented!();
            }
            0x03 => {
                println!("JAL");
                unimplemented!();
            }
            0x04 => {
                println!("BEQ");
                unimplemented!();
            }
            0x05 => {
                println!("BNE");

                let rt = ((instruction >> 16) & 0x1F) as usize;
                let rs = ((instruction >> 21) & 0x1F) as usize;

                if self.regs.gpr[rs] != self.regs.gpr[rt] {
                    let offset = (((instruction & 0xFFFF) as u16) << 2) as i16 as u64;

                    let future_pc = self.regs.pc.wrapping_add(offset).wrapping_add(4);

                    return Some(DelayedBranching(future_pc));
                }

                // TODO sign extend?
                // TODO delay 1 instruction?
            }
            0x06 => {
                println!("BLEZ");
                unimplemented!();
            }
            0x07 => {
                println!("BGTZ");
                unimplemented!();
            }
            0x08 => {
                println!("ADDI");
                unimplemented!();
            }
            0x09 => {
                println!("ADDIU");

                let imm = (instruction & 0xFFFF) as i16 as u64;
                let rt = ((instruction >> 16) & 0x1F) as usize;
                let rs = ((instruction >> 21) & 0x1F) as usize;
                self.regs.gpr[rt] = self.regs.gpr[rs].wrapping_add(imm);

                // TODO 64 mode: sign extend?
                // TODO no overflow?
            }

            // COP 0
            0x10 => match (instruction >> 21) & 0x1F {
                4 => {
                    println!("MTC0 (todo)")
                }
                _ => panic!("Unknown opcode: {:06b}", opcode),
            },

            0x0A => {
                println!("SLTI");
                unimplemented!();
            }
            0x0B => {
                println!("SLTIU???");
                unimplemented!();
            }
            0x0C => {
                println!("ANDI");
                unimplemented!();
            }
            0x0D => {
                println!("ORI");
                unimplemented!();
            }
            0x0E => {
                println!("XORI");
                unimplemented!();
            }
            0x0F => {
                println!("LUI");

                let imm = instruction & 0xFFFF;
                let rt = ((instruction >> 16) & 0x1F) as usize;
                self.regs.gpr[rt] = (imm << 16) as i32 as u64; // Sign-extend
            }
            0x23 => {
                println!("LW");

                let offset = (instruction & 0xFFFF) as i16 as u32;
                let rt = ((instruction >> 16) & 0x1F) as usize;
                let base = ((instruction >> 21) & 0x1F) as usize;
                let addr = self.regs.gpr[base] as u32 + offset;
                self.regs.gpr[rt] = self.read(addr) as i32 as u64;
            }
            0x24 => {
                println!("LBU");
                unimplemented!();
            }
            0x25 => {
                println!("LHU");
                unimplemented!();
            }
            0x28 => {
                println!("SB");
                unimplemented!();
            }
            0x29 => {
                println!("SH");
                unimplemented!();
            }
            0x2B => {
                println!("SW");
                unimplemented!();
            }
            0x2F => {
                // https://hack64.net/docs/VR43XX.pdf page 404
                println!("CACHE");

                let base = ((instruction >> 21) & 0x1F) as usize;
                let op = (instruction >> 16) & 0x1F;

                let target = op & 3;
                let op2 = (op >> 2) & 7;
                print!("CACHE  {:08x} {:08x} {:08x} ", op, target, op2);

                // match op {
                //     // CACHE manual = 274
                //     0 => println!("CACHE HIT INVALIDATE"),
                //     1 => println!("CACHE HIT WRITEBACK"),
                //     2 => println!("CACHE HIT WRITEBACK INVALIDATE"),
                //     3 => println!("CACHE HIT WRITEBACK INVALIDATE"),
                //     4 => println!("CACHE HIT WRITEBACK INVALIDATE"),
                //     5 => println!("CACHE HIT WRITEBACK INVALIDATE"),
                //     _ => panic!("Unknown cache op: {:06b}", opcode),
                // }

                // unimplemented!();
            }

            _ => panic!("Unknown opcode: {:06b}", opcode),
        }

        None
    }
}

struct DelayedBranching(u64);
