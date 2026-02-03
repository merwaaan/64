use std::fs::File;
use std::io::{BufWriter, Write};

use crate::breakpoints::Breakpoints;
use crate::data::Data;
use crate::events::{Cycle, Events};
use crate::{cart::Cart, cpu::CPU, map::Map};

pub struct System {
    // Components
    pub cart: Cart,
    pub cpu: CPU,
    pub map: Map,

    // Scheduling
    pub cycles: Cycle,
    pub events: Events,
    odd: bool, // TODO temp hack to time CPU

    // Debugger
    // TODO move to external debbuger?
    pub breakpoints: Breakpoints,

    // Debug logging
    pub log_writer: Option<BufWriter<File>>,
    pub log_from: Option<usize>,
    pub log_to: Option<usize>,
}

impl System {
    pub fn new(cart: Cart, log_from: Option<usize>, log_to: Option<usize>) -> Self {
        let log_writer = if log_from.is_some() || log_to.is_some() {
            Some(BufWriter::new(File::create("log1.txt").unwrap()))
        } else {
            None
        };

        Self {
            cart,
            cpu: CPU::default(),
            map: Map::default(),

            cycles: 0,
            events: Events::default(),
            odd: false,

            breakpoints: Breakpoints::default(),

            log_writer,
            log_from,
            log_to,
        }
    }

    // NOTE: IPL starts at A4000040, executes the cart boot sequence, skipped for now
    pub fn skip_ipl(&mut self) {
        self.cpu.regs.pc = 0xA4000040;

        // Setup the registers as IPL would have done

        self.cpu.regs.gpr[11].set(0xA4000040);
        //TODO yes, disabled for diffself.regs.gpr[20].set(0x00000001);
        self.cpu.regs.gpr[22].set(0x0000003F);
        self.cpu.regs.gpr[29].set(0xA4001FF0);

        // TODO cop0 (readthedocs)
        self.cpu.regs.cop0[1].set(0x1F);
        self.cpu.regs.cop0[12].set(0x34000000);
        self.cpu.regs.cop0[15].set(0x00000B00);
        self.cpu.regs.cop0[16].set(0x0006E463);

        // TODO temp p64 match
        self.cpu.regs.gpr[1].set(1);
        self.cpu.regs.gpr[2].set(0xEBDA536);
        self.cpu.regs.gpr[3].set(0xEBDA536);
        self.cpu.regs.gpr[4].set(0xA536);
        self.cpu.regs.gpr[5].set(0xC0F1D859);
        self.cpu.regs.gpr[6].set(0xA4001F0C);
        self.cpu.regs.gpr[7].set(0xA4001F08);
        self.cpu.regs.gpr[8].set(0x000000C0);
        self.cpu.regs.gpr[10].set(0x00000040);
        self.cpu.regs.gpr[11].set(0xA4000040);
        self.cpu.regs.gpr[12].set(0xED10D0B3);
        self.cpu.regs.gpr[13].set(0x1402A4CC);
        self.cpu.regs.gpr[14].set(0x2DE108EA);
        self.cpu.regs.gpr[15].set(0x3103E121);
        self.cpu.regs.gpr[23].set(0x6);
        self.cpu.regs.gpr[25].set(0x9DEBB54F);
        self.cpu.regs.gpr[29].set(0xA4001FF0);
        self.cpu.regs.gpr[31].set(0xA4001554);
        self.cpu.regs.cop0[4].set(0x007FFFF0);
        self.cpu.regs.cop0[8].set(0xFFFFFFFF);
        //self.cpu.regs.cop0[5].set(0x5000);
        self.cpu.regs.cop0[9].set(0x5000);
        self.cpu.regs.cop0[13].set(0x5C);
        self.cpu.regs.cop0[14].set(0xFFFFFFFF);
        self.cpu.regs.cop0[15].set(0x00000B22);
        self.cpu.regs.cop0[16].set(0x7006E463);
        self.cpu.regs.cop0[30].set(0xFFFFFFFF);

        // Copy the cart's boot code to memory

        // TODO which size?
        self.map.rspdmem[0..0x1000].copy_from_slice(&self.cart.data[0..0x1000]);
    }

    pub fn step(&mut self) -> bool {
        // TODO temp logging

        if let Some(ref mut w) = self.log_writer {
            let log_from = self.log_from.unwrap_or(0);
            let log_to = self.log_to.unwrap_or(usize::MAX);

            if self.cpu.step >= log_from && self.cpu.step <= log_to {
                let gpr_hex: String = self
                    .cpu
                    .regs
                    .gpr
                    .iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}={:016X}", i, r.get64()))
                    .collect::<Vec<_>>()
                    .join(" ");

                let cop0_hex: String = self
                    .cpu
                    .regs
                    .cop0
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        format!(
                            "C{}={:016X}",
                            i,
                            if i == 1 || i == 9 { 0 } else { r.get64() }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                writeln!(
                    w,
                    "{:10} {:08X} {} {} H={:016X} L={:016X}",
                    self.cpu.step,
                    self.cpu.regs.pc,
                    gpr_hex,
                    cop0_hex,
                    self.cpu.regs.mult_hi.get64(),
                    self.cpu.regs.mult_lo.get64(),
                )
                .unwrap();

                if self.log_to.is_some() && self.cpu.step == self.log_to.unwrap() {
                    panic!("Reached last log step {}", self.cpu.step);
                }
            }
        }

        // Step the CPU

        CPU::step(self);

        self.cycles += 2; //if self.odd { 2 } else { 1 };
        self.odd = !self.odd;

        // Events
        // TODO how many cycles?

        Events::update(self);

        // Breakpoints

        if self.breakpoints.contains(self.cpu.regs.pc) {
            log::info!("Breakpoint hit at {:08X}", self.cpu.regs.pc);
            true
        } else {
            false
        }
    }

    pub fn read<T: Data>(&self, addr: u32) -> T {
        Map::read(self, addr) // TODO  Map:: really needed??
    }

    pub fn write<T: Data>(&mut self, addr: u32, data: T) {
        Map::write(self, addr, data); // TODO  Map:: really needed???
    }
}
