use core::fmt;
use std::fmt::Display;

use crate::{
    ai::{Ai, AiLocation},
    breakpoints::Breakpoints,
    cart::{Cart, CartLocation},
    controller::Controller,
    cop0::Cop0,
    cop1::Cop1,
    cpu::Cpu,
    dd::Dd,
    dp::{Dp, DpLocation},
    events::{Cycle, EventType, Events},
    exception::Exception,
    location::{Location, MapLocation},
    mi::{Mi, MiLocation},
    openbus,
    pi::{Pi, PiLocation},
    pif::{Pif, PifRamLocation},
    ram::{Ram, RamInterfaceLocation, RamLocation, RamRegsLocation},
    rendering::{audio::AudioRenderer, video::VideoRenderer},
    si::{Si, SiLocation},
    sp::{Sp, SpMemLocation, SpRegsLocation},
    value::Value,
    vi::{self, Vi, ViLocation},
};

// TODO clean up?
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

// TODO rework with a trait + const generics?
#[derive(Clone, Copy)]
pub struct VirtualAddress(pub u32);

#[derive(Clone, Copy)]
pub struct PhysicalAddress(pub u32); // TODO smaller type?

#[derive(Clone, Copy)]
pub enum Address {
    Virtual(VirtualAddress),
    Physical(PhysicalAddress),
}

impl Address {
    pub fn v(addr: u32) -> Self {
        Address::Virtual(VirtualAddress(addr))
    }

    pub fn p(addr: u32) -> Self {
        Address::Physical(PhysicalAddress(addr))
    }

    // pub fn value(&self) -> u32 {
    //     match self {
    //         Address::Virtual(addr) => addr.0,
    //         Address::Physical(addr) => addr.0,
    //     }
    // }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Virtual(addr) => write!(f, "Virtual({:08X})", addr.0),
            Address::Physical(addr) => write!(f, "Physical({:08X})", addr.0),
        }
    }
}

// pub trait Address {
//     fn value(&self) -> u32;
// }

// impl Address for VirtualAddress {
//     fn value(&self) -> u32 {
//         self.0
//     }
// }

// impl Address for PhysicalAddress {
//     fn value(&self) -> u32 {
//         self.0
//     }
// }

pub struct System {
    // Components
    pub cpu: Cpu,
    pub cop0: Cop0,
    pub cop1: Cop1,
    pub ram: Ram,
    pub sp: Sp,
    pub dp: Dp,
    pub mi: Mi,
    pub vi: Vi,
    pub ai: Ai,
    pub pi: Pi,
    pub si: Si,
    pub dd: Dd,
    pub pif: Pif,
    pub cart: Cart,

    // Scheduling
    pub(crate) events: Events,

    // Debugger
    breakpoints: Breakpoints,

    pub controllers: [Controller; 4],

    pub audio_renderer: AudioRenderer,
    pub video_renderer: VideoRenderer,
}

impl System {
    pub fn with_cart(cart: Cart) -> Self {
        let mut s = Self {
            cpu: Cpu::default(),
            cop0: Cop0::default(),
            cop1: Cop1::default(),
            ram: Ram::default(),
            sp: Sp::default(),
            dp: Dp::default(),
            mi: Mi::default(),
            vi: Vi::default(),
            ai: Ai::default(),
            pi: Pi::default(),
            si: Si::default(),
            dd: Dd::default(),
            pif: Pif::default(),
            cart,

            events: Events::default(),

            breakpoints: Breakpoints::default(),

            controllers: [Controller::default(); 4],

            audio_renderer: AudioRenderer::new(),
            video_renderer: VideoRenderer::new(),
        };

        // Load the breakpoints

        match s.load() {
            Ok(()) => {
                log::debug!("Breakpoints loaded");
            }
            Err(e) => {
                log::error!("Failed to load breakpoints: {}", e);
            }
        }

        // s.breakpoints.add(0x80000180);
        // s.breakpoints.add(0x80000100);

        // Schedule the first scanline

        Events::push(
            &mut s,
            EventType::ViScanlineComplete,
            vi::SCANLINE_CPU_CYCLES,
        );

        s.initialize();

        s
    }

    fn initialize(&mut self) {
        // Set the PC to the start of the IPL
        //
        // NOTE: IPL starts at A4000040 and executes the boot sequence, skipped for now

        self.cpu.regs.pc = 0xA4000040;

        // Setup the registers as IPL would have done
        // https://n64.readthedocs.io/index.html#simulating-the-pif-rom

        self.cpu.regs.gpr[11].set(0xA4000040);
        self.cpu.regs.gpr[20].set(0x00000001);
        self.cpu.regs.gpr[22].set(0x0000003F);
        self.cpu.regs.gpr[29].set(0xA4001FF0);

        // Copy the cart's boot code to RAM

        for i in 0..0x1000u32 {
            let byte = Cart::read::<u8>(self, CartLocation::from_relative(i));

            Sp::write_mem(self, Location::from_relative(i), byte);
        }

        // Set the exception code to 11111
        // TODO unclear if expected, makes lemon tests pass

        self.cop0.set_exception_code(u32::MAX);
    }

    pub fn step(&mut self) -> bool {
        // Step the processors

        Cpu::step(self);
        Sp::step(self); // TODO 2 SP cycles for 3 CPU cycles?

        // Increment the timer every 2 CPU cycles

        if self.cpu.cycles().is_multiple_of(2) {
            // TODO ok right now, but not robust if some instructions take more than 2 cycles!
            self.cop0.increment_timer();
        }

        // Process scheduled events

        Events::update(self);

        // Breakpoints

        if self.breakpoints.should_break(self.cpu.regs.pc) {
            log::info!("Breakpoint hit at {:08X}", self.cpu.regs.pc);
            true
        } else {
            false
        }
    }

    // TODO or Option<Result<Loc, Exc>>?
    #[must_use]
    fn decode(&self, addr: Address, write: bool) -> Result<Option<MapLocation>, Exception> {
        let physical_addr = match addr {
            Address::Virtual(addr) => match addr.0 {
                0x0000_0000..=0x7FFF_FFFF => self.cop0.tlb.translate(addr, &self.cop0, write)?,
                0x8000_0000..=0x9FFF_FFFF => PhysicalAddress(addr.0),
                0xA000_0000..=0xBFFF_FFFF => PhysicalAddress(addr.0),
                0xC000_0000..=0xDFFF_FFFF => self.cop0.tlb.translate(addr, &self.cop0, write)?,
                0xE000_0000..=0xFFFF_FFFF => self.cop0.tlb.translate(addr, &self.cop0, write)?,
            },
            Address::Physical(addr) => addr,
        };

        let addr = physical_addr.0 & 0x1FFF_FFFF;

        Ok(match addr {
            RamLocation::START..RamLocation::END => {
                Some(MapLocation::Ram(RamLocation::from_absolute(addr)))
            }
            RamRegsLocation::START..RamRegsLocation::END => {
                Some(MapLocation::RamRegs(RamRegsLocation::from_absolute(addr)))
            }
            SpMemLocation::START..SpMemLocation::END => {
                Some(MapLocation::SpMem(SpMemLocation::from_absolute(addr)))
            }
            SpRegsLocation::START..SpRegsLocation::END => {
                Some(MapLocation::SpRegs(SpRegsLocation::from_absolute(addr)))
            }
            DpLocation::START..DpLocation::END => {
                Some(MapLocation::Dp(DpLocation::from_absolute(addr)))
            }
            MiLocation::START..MiLocation::END => {
                Some(MapLocation::Mi(MiLocation::from_absolute(addr)))
            }
            ViLocation::START..ViLocation::END => {
                Some(MapLocation::Vi(ViLocation::from_absolute(addr)))
            }
            AiLocation::START..AiLocation::END => {
                Some(MapLocation::Ai(AiLocation::from_absolute(addr)))
            }
            PiLocation::START..PiLocation::END => {
                Some(MapLocation::Pi(PiLocation::from_absolute(addr)))
            }
            RamInterfaceLocation::START..RamInterfaceLocation::END => Some(
                MapLocation::RamInterface(RamInterfaceLocation::from_absolute(addr)),
            ),
            SiLocation::START..SiLocation::END => {
                Some(MapLocation::Si(SiLocation::from_absolute(addr)))
            }
            // DdLocation::START..DdLocation::END => {
            //     Some(MapLocation::Dd(DdLocation::from_absolute(addr)))
            // }
            0x0500_0000..0x1000_0000 => Some(MapLocation::OpenBus(addr)),
            CartLocation::START..CartLocation::END => {
                Some(MapLocation::Cart(CartLocation::from_absolute(addr)))
            }
            // TODO actually not openbus? ignore writes? what about reads?
            0x1FC0_0000..PifRamLocation::START => Some(MapLocation::OpenBus(addr)),
            PifRamLocation::START..PifRamLocation::END => {
                Some(MapLocation::Pif(PifRamLocation::from_absolute(addr)))
            }
            0x1FD00000..0x80000000 => Some(MapLocation::OpenBus(addr)),
            _ => None,
        })
    }

    #[must_use]
    pub fn read<T: Value>(&mut self, addr: Address) -> Result<T, Exception> {
        self.decode(addr, false).map(|location| match location {
            Some(MapLocation::Ram(addr)) => Ram::read(self, addr),
            Some(MapLocation::RamRegs(addr)) => self.ram.read_reg(addr),
            Some(MapLocation::SpMem(addr)) => self.sp.read_mem(addr),
            Some(MapLocation::SpRegs(addr)) => self.sp.read_reg(addr),
            Some(MapLocation::Dp(addr)) => Dp::read(self, addr),
            Some(MapLocation::Mi(addr)) => Mi::read(self, addr),
            Some(MapLocation::Vi(addr)) => Vi::read(self, addr),
            Some(MapLocation::Ai(addr)) => Ai::read(self, addr),
            Some(MapLocation::Pi(addr)) => Pi::read(self, addr),
            Some(MapLocation::RamInterface(addr)) => self.ram.read_interface(addr),
            Some(MapLocation::Si(addr)) => Si::read(self, addr),
            //Some(MapLocation::Dd(addr)) => s.dd.read(addr),
            Some(MapLocation::Cart(addr)) => Cart::read(self, addr),
            Some(MapLocation::Pif(addr)) => self.pif.read(&self.controllers, addr),
            Some(MapLocation::OpenBus(addr)) => openbus::read(addr),
            None => {
                log::warn!("Invalid read address: {}", addr);
                T::default()
            }
        })
    }

    /// Reads without side effects, for debugging
    pub fn peek<T: Value>(&mut self, addr: Address) -> Option<T> {
        self.read(addr).ok()
    }

    // TODO what if address crosses a boundary?
    #[must_use]
    pub fn write<T: Value>(&mut self, addr: Address, data: T) -> Result<(), Exception> {
        let location = self.decode(addr, true)?;

        match location {
            Some(MapLocation::Ram(addr)) => Ram::write(self, addr, data),
            Some(MapLocation::RamRegs(addr)) => Ram::write_reg(self, addr, data),
            Some(MapLocation::SpMem(addr)) => Sp::write_mem(self, addr, data),
            Some(MapLocation::SpRegs(addr)) => Sp::write_reg(self, addr, data),
            Some(MapLocation::Dp(addr)) => Dp::write(self, addr, data),
            Some(MapLocation::Mi(addr)) => Mi::write(self, addr, data),
            Some(MapLocation::Vi(addr)) => Vi::write(self, addr, data),
            Some(MapLocation::Ai(addr)) => Ai::write(self, addr, data),
            Some(MapLocation::Pi(addr)) => Pi::write(self, addr, data),
            Some(MapLocation::RamInterface(addr)) => Ram::write_interface(self, addr, data),
            Some(MapLocation::Si(addr)) => Si::write(self, addr, data),
            Some(MapLocation::Cart(addr)) => Cart::write(self, addr, data),
            Some(MapLocation::Pif(addr)) => self.pif.write(addr, data),
            Some(MapLocation::OpenBus(addr)) => openbus::write(addr, data),
            _ => log::warn!("Invalid write address: {}", addr),
        };

        Ok(())
    }

    // fn address_info(addr: u32) -> Option<&'static str> {
    //     match Map::decode(addr) {
    //         Some(MapLocation::RdramRegs(addr)) => Rdram::reg_info(addr),
    //         Some(MapLocation::RspRegs(addr)) => Rsp::reg_info(addr),
    //         Some(MapLocation::Mi(addr)) => Mi::reg_info(addr),
    //         Some(MapLocation::Vi(addr)) => Vi::reg_info(addr),
    //         Some(MapLocation::Ai(addr)) => Ai::reg_info(addr),
    //         Some(MapLocation::Pi(addr)) => Pi::reg_info(addr),
    //         Some(MapLocation::RdramInterface(addr)) => Rdram::interface_info(addr),
    //         Some(MapLocation::Si(addr)) => Si::reg_info(addr),
    //         _ => None,
    //     }
    // }

    // Events

    pub fn pending_events(&self) -> Vec<(EventType, Cycle)> {
        self.events.snapshot()
    }

    // Breakpoints
    // TODO move to emulator?

    pub fn breakpoints(&self) -> &Breakpoints {
        &self.breakpoints
    }

    pub fn add_breakpoint(&mut self, address: u32) {
        self.breakpoints.add(address);

        self.save().unwrap();
    }

    pub fn remove_breakpoint(&mut self, address: u32) {
        self.breakpoints.remove(address);

        self.save().unwrap();
    }

    pub fn toggle_breakpoint(&mut self, address: u32) {
        self.breakpoints.toggle(address);

        self.save().unwrap();
    }

    // TODO move saving to client code? just serialize in core

    #[must_use]
    fn save(&self) -> Result<(), LoadError> {
        let breakpoints_json = serde_json::to_string(&self.breakpoints)?;
        std::fs::write("breakpoints.json", breakpoints_json)?;
        Ok(())
    }

    #[must_use]
    fn load(&mut self) -> Result<(), LoadError> {
        let path = std::path::Path::new("breakpoints.json");

        if path.exists() {
            let breakpoints_json = std::fs::read_to_string(path)?;
            let breakpoints: Breakpoints = serde_json::from_str(&breakpoints_json)?;
            self.breakpoints = breakpoints;
        }

        Ok(())
    }
}
