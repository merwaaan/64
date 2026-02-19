use crate::{
    ai::{Ai, AiLocation},
    cart::{Cart, CartLocation},
    data::Data,
    dd::Dd,
    dp::{Dp, DpLocation},
    mi::{Mi, MiLocation},
    openbus,
    pi::{Pi, PiLocation},
    pif::{Pif, PifRamLocation},
    rdram::{Rdram, RdramInterfaceLocation, RdramLocation, RdramRegsLocation},
    rsp::{Rsp, RspDmemLocation, RspImemLocation, RspRegsLocation},
    si::{Si, SiLocation},
    system::System,
    vi::{Vi, ViLocation},
};

/// Location in the memory map.
/// Bound by start and end addresses (exclusive).
///
/// Memory map section can define their own location:
///
/// ```
/// pub type RspDmemLocation = Location<0x0000_0000, 0x0401_0000>;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Location<const START: u32, const END: u32>(u32);

impl<const START: u32, const END: u32> Location<START, END> {
    pub const START: u32 = START;
    pub const END: u32 = END;

    pub fn from_relative(addr: u32) -> Self {
        debug_assert!(
            (0..END - START).contains(&addr),
            "Address {:08X} is out of relative range ({}..{})",
            addr,
            START,
            END
        );

        Self(addr)
    }

    pub fn relative(self) -> u32 {
        self.0
    }

    pub fn from_absolute(addr: u32) -> Self {
        debug_assert!(
            (START..END).contains(&addr),
            "Address {:08X} is out of absolute range ({}..{})",
            addr,
            START,
            END
        );

        Self(addr - START)
    }

    pub fn absolute(self) -> u32 {
        START + self.0
    }
}

// TODO store relative addr?
// TODO component should only accept their location?
#[derive(Debug)]
pub enum MapLocation {
    Rdram(RdramLocation),
    RdramRegs(RdramRegsLocation),
    RspDmem(RspDmemLocation),
    RspImem(RspImemLocation),
    RspRegs(RspRegsLocation),
    Dp(DpLocation),
    Mi(MiLocation),
    Vi(ViLocation),
    Ai(AiLocation),
    Pi(PiLocation),
    RdramInterface(RdramInterfaceLocation),
    Si(SiLocation),
    //Dd(DdLocation),
    Cart(CartLocation),
    Pif(PifRamLocation),
    OpenBus(u32),
}

pub struct Map {
    pub rdram: Rdram,
    pub rsp: Rsp,
    pub dp: Dp,
    pub mi: Mi,
    pub vi: Vi,
    pub ai: Ai,
    pub pi: Pi,
    pub si: Si,
    pub dd: Dd,
    pub cart: Cart,
    pub pif: Pif,
}

impl Map {
    pub fn new(cart: Cart) -> Self {
        Self {
            rdram: Rdram::default(),
            rsp: Rsp::default(),
            dp: Dp::default(),
            mi: Mi::default(),
            vi: Vi::default(),
            ai: Ai::default(),
            pi: Pi::default(),
            si: Si::default(),
            dd: Dd::default(),
            cart,
            pif: Pif::default(),
        }
    }

    pub fn decode(addr: u32) -> Option<MapLocation> {
        // TODO future optim: page table?

        let addr = virtual_to_physical_address(addr);

        match addr {
            RdramLocation::START..RdramLocation::END => {
                Some(MapLocation::Rdram(RdramLocation::from_absolute(addr)))
            }
            RdramRegsLocation::START..RdramRegsLocation::END => Some(MapLocation::RdramRegs(
                RdramRegsLocation::from_absolute(addr),
            )),
            RspDmemLocation::START..RspDmemLocation::END => {
                Some(MapLocation::RspDmem(RspDmemLocation::from_absolute(addr)))
            }
            RspImemLocation::START..RspImemLocation::END => {
                Some(MapLocation::RspImem(RspImemLocation::from_absolute(addr)))
            }
            RspRegsLocation::START..RspRegsLocation::END => {
                Some(MapLocation::RspRegs(RspRegsLocation::from_absolute(addr)))
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
            RdramInterfaceLocation::START..RdramInterfaceLocation::END => Some(
                MapLocation::RdramInterface(RdramInterfaceLocation::from_absolute(addr)),
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
            PifRamLocation::START..PifRamLocation::END => {
                Some(MapLocation::Pif(PifRamLocation::from_absolute(addr)))
            }
            0x1FD00000..0x80000000 => Some(MapLocation::OpenBus(addr)),
            _ => None,
        }
    }

    pub fn read<T: Data>(s: &System, addr: u32) -> T {
        let location = Self::decode(addr);

        match location {
            Some(MapLocation::Rdram(addr)) => s.map.rdram.read(addr),
            Some(MapLocation::RdramRegs(addr)) => s.map.rdram.read_reg(addr),
            Some(MapLocation::RspDmem(addr)) => s.map.rsp.read_dmem(addr),
            Some(MapLocation::RspImem(addr)) => s.map.rsp.read_imem(addr),
            Some(MapLocation::RspRegs(addr)) => s.map.rsp.read_reg(addr),
            Some(MapLocation::Dp(addr)) => s.map.dp.read(addr),
            Some(MapLocation::Mi(addr)) => s.map.mi.read(addr),
            Some(MapLocation::Vi(addr)) => s.map.vi.read(addr),
            Some(MapLocation::Ai(addr)) => s.map.ai.read(addr),
            Some(MapLocation::Pi(addr)) => s.map.pi.read(addr),
            Some(MapLocation::RdramInterface(addr)) => s.map.rdram.read_interface(addr),
            Some(MapLocation::Si(addr)) => s.map.si.read(addr),
            //Some(MapLocation::Dd(addr)) => s.map.dd.read(addr),
            Some(MapLocation::Cart(addr)) => s.map.cart.read(addr),
            Some(MapLocation::Pif(addr)) => s.map.pif.read(addr),
            Some(MapLocation::OpenBus(addr)) => openbus::read(addr),
            None => panic!("Invalid read address: {:08X}", addr),
        }
    }

    // TODO what if address crosses a boundary?
    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        let location = Self::decode(addr);
        //log::warn!("write {:08X} {:X}", addr, data.to_u32());
        match location {
            Some(MapLocation::Rdram(addr)) => Rdram::write(s, addr, data),
            Some(MapLocation::RdramRegs(addr)) => Rdram::write_reg(s, addr, data),
            Some(MapLocation::RspDmem(addr)) => Rsp::write_dmem(s, addr, data),
            Some(MapLocation::RspImem(addr)) => Rsp::write_imem(s, addr, data),
            Some(MapLocation::RspRegs(addr)) => Rsp::write_reg(s, addr, data),
            Some(MapLocation::Dp(addr)) => Dp::write(s, addr, data),
            Some(MapLocation::Mi(addr)) => Mi::write(s, addr, data),
            Some(MapLocation::Vi(addr)) => Vi::write(s, addr, data),
            Some(MapLocation::Ai(addr)) => Ai::write(s, addr, data),
            Some(MapLocation::Pi(addr)) => Pi::write(s, addr, data),
            Some(MapLocation::RdramInterface(addr)) => Rdram::write_interface(s, addr, data),
            Some(MapLocation::Si(addr)) => Si::write(s, addr, data),
            Some(MapLocation::Cart(addr)) => Cart::write(s, addr, data),
            Some(MapLocation::Pif(addr)) => Pif::write(s, addr, data),
            Some(MapLocation::OpenBus(addr)) => openbus::write(addr, data),
            _ => panic!("Invalid write address: {:08X}", addr),
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
    match Map::decode(addr) {
        Some(MapLocation::RdramRegs(addr)) => Rdram::reg_info(addr),
        Some(MapLocation::RspRegs(addr)) => Rsp::reg_info(addr),
        Some(MapLocation::Mi(addr)) => Mi::reg_info(addr),
        Some(MapLocation::Vi(addr)) => Vi::reg_info(addr),
        Some(MapLocation::Ai(addr)) => Ai::reg_info(addr),
        Some(MapLocation::Pi(addr)) => Pi::reg_info(addr),
        Some(MapLocation::RdramInterface(addr)) => Rdram::interface_info(addr),
        Some(MapLocation::Si(addr)) => Si::reg_info(addr),
        _ => None,
    }
}
