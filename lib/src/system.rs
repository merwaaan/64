use crate::breakpoints::{Breakpoint, Breakpoints};
use crate::cart::CartLocation;
use crate::cop0::Cop0;
use crate::data::Data;
use crate::events::{Cycle, Event, EventType, Events};
use crate::map::Location;
use crate::rsp::Rsp;
use crate::{cart::Cart, cpu::CPU, map::Map};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub struct System {
    // Components
    pub cpu: CPU,
    pub cop0: Cop0,
    pub map: Map,

    // Scheduling
    pub cycles: Cycle,
    pub events: Events,
    odd: bool, // TODO temp hack to time CPU

    // Debugger
    breakpoints: Breakpoints,

    broken: bool,
}

impl System {
    pub fn new(cart: Cart) -> Self {
        let mut s = Self {
            cpu: CPU::default(),
            cop0: Cop0::default(),
            map: Map::new(cart),

            cycles: 0,
            events: Events::default(),
            odd: false,

            breakpoints: Breakpoints::default(),

            broken: false,
        };

        s.events.push(Event {
            id: EventType::ViScanlineComplete,
            cycle: 1000, // TODO!!!
        });

        match s.load() {
            Ok(()) => {
                log::info!("Breakpoints loaded");
            }
            Err(e) => {
                log::error!("Failed to load breakpoints: {}", e);
            }
        }

        s
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
        self.cop0.regs[1].set(0x1F);
        self.cop0.regs[12].set(0x34000000);
        self.cop0.regs[15].set(0x00000B00);
        self.cop0.regs[16].set(0x0006E463);

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
        self.cop0.regs[4].set(0x007FFFF0);
        self.cop0.regs[8].set(0xFFFFFFFF);
        //self.cop0.regs[5].set(0x5000);
        self.cop0.regs[9].set(0x5000);
        self.cop0.regs[13].set(0x5C);
        self.cop0.regs[14].set(0xFFFFFFFF);
        self.cop0.regs[15].set(0x00000B22);
        self.cop0.regs[16].set(0x7006E463);
        self.cop0.regs[30].set(0xFFFFFFFF);

        // Copy the cart's boot code to memory

        // TODO which size?
        // TODO rel???
        for i in 0..0x1000u32 {
            Rsp::write_dmem(
                self,
                Location::from_relative(i),
                self.map.cart.read::<u8>(CartLocation::from_relative(i)),
            );
        }
    }

    pub fn step(&mut self) -> bool {
        if self.broken {
            return false;
        }

        // Step the CPU

        let ok = CPU::step(self);

        if !ok {
            log::warn!("BROKEN at {:08X}", self.cpu.regs.pc);
            self.broken = true;
            return false;
        }

        self.cycles += 2; //if self.odd { 2 } else { 1 };
        self.odd = !self.odd;

        // Events
        // TODO how many cycles?

        Events::update(self);

        // Check for pending interrupts

        // TODO mask?
        // TODO raise int if cause b0-1 set?

        // if self.map.mi.has_interrupt() {
        //     log::error!(
        //         "has_interrupt: {} {} {}",
        //         self.cop0.ie(),
        //         self.cop0.exl(),
        //         self.cop0.erl()
        //     );
        // }
        // if self.map.mi.has_interrupt() {
        //     log::error!(
        //         "has_interrupt: {} {} {}",
        //         self.cop0.ie(),
        //         self.cop0.exl(),
        //         self.cop0.erl()
        //     );
        // }
        if self.cop0.ie() && !self.cop0.exl() && !self.cop0.erl() && self.map.mi.has_interrupt() {
            // EPC
            // Cause (BD/ExcCode)
            self.cop0.regs[13].set(0x400); // TODO tmp

            self.cop0.set_exl();

            self.cpu.regs.pc = 0x8000_0180; // TODO others?

            log::warn!("TODOOOOOO INTERRUPT")
        }

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

    pub fn breakpoints(&self) -> &Breakpoints {
        &self.breakpoints
    }

    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.add(breakpoint);

        self.save().unwrap();
    }

    pub fn remove_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.remove(breakpoint);

        self.save().unwrap();
    }

    fn save(&self) -> Result<(), LoadError> {
        let breakpoints_json = serde_json::to_string(&self.breakpoints)?;
        std::fs::write("breakpoints.json", breakpoints_json)?;
        Ok(())
    }

    fn load(&mut self) -> Result<(), LoadError> {
        let breakpoints_json = std::fs::read_to_string("breakpoints.json")?;
        let breakpoints: Breakpoints = serde_json::from_str(&breakpoints_json)?;
        self.breakpoints = breakpoints;
        Ok(())
    }
}
