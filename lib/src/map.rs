use crate::{
    ai::{self, Ai, AiLocation},
    cart::{self, Cart, CartLocation},
    data::Data,
    mi::{self, Mi, MiLocation},
    pi::{self, Pi, PiLocation},
    rdram::{self, Rdram, RdramInterfaceLocation, RdramLocation, RdramRegsLocation},
    rsp::{self, Rsp, RspDmemLocation, RspImemLocation, RspRegsLocation},
    si::{self, Si, SiLocation},
    system::System,
    vi::{self, Vi, ViLocation},
};

/// Location in the memory map.
#[derive(Debug, Clone, Copy)]
pub struct Location<const START: u32, const END: u32>(u32);

impl<const START: u32, const END: u32> Location<START, END> {
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
    Mi(MiLocation),
    Vi(ViLocation),
    Ai(AiLocation),
    Pi(PiLocation),
    RdramInterface(RdramInterfaceLocation),
    Si(SiLocation),
    //TODODD(u32)
    Cart(CartLocation),
    //Pif(u32),
}

pub struct Map {
    pub rdram: Rdram,
    pub rsp: Rsp,
    pub mi: Mi,
    pub vi: Vi,
    pub ai: Ai,
    pub pi: Pi,
    pub si: Si,
    pub cart: Cart,
}

impl Map {
    pub fn new(cart: Cart) -> Self {
        Self {
            rdram: Rdram::default(),
            rsp: Rsp::default(),
            mi: Mi::default(),
            vi: Vi::default(),
            ai: Ai::default(),
            pi: Pi::default(),
            si: Si::default(),
            cart,
        }
    }

    pub fn decode(addr: u32) -> Option<MapLocation> {
        // TODO future optim: page table?

        let addr = virtual_to_physical_address(addr);

        match addr {
            rdram::DATA_START..rdram::DATA_END => {
                Some(MapLocation::Rdram(RdramLocation::from_absolute(addr)))
            }
            rdram::REG_START..rdram::REG_END => Some(MapLocation::RdramRegs(
                RdramRegsLocation::from_absolute(addr),
            )),
            rsp::DMEM_START..rsp::DMEM_END => {
                Some(MapLocation::RspDmem(RspDmemLocation::from_absolute(addr)))
            }
            rsp::IMEM_START..rsp::IMEM_END => {
                Some(MapLocation::RspImem(RspImemLocation::from_absolute(addr)))
            }
            rsp::REG_START..rsp::REG_END => {
                Some(MapLocation::RspRegs(RspRegsLocation::from_absolute(addr)))
            }
            mi::START..mi::END => Some(MapLocation::Mi(MiLocation::from_absolute(addr))),
            vi::START..vi::END => Some(MapLocation::Vi(ViLocation::from_absolute(addr))),
            ai::START..ai::END => Some(MapLocation::Ai(AiLocation::from_absolute(addr))),
            pi::START..pi::END => Some(MapLocation::Pi(PiLocation::from_absolute(addr))),
            rdram::INTERFACE_START..rdram::INTERFACE_END => Some(MapLocation::RdramInterface(
                RdramInterfaceLocation::from_absolute(addr),
            )),
            si::START..si::END => Some(MapLocation::Si(SiLocation::from_absolute(addr))),
            cart::ROM_START..cart::ROM_END => {
                Some(MapLocation::Cart(CartLocation::from_absolute(addr)))
            }
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
            Some(MapLocation::Mi(addr)) => s.map.mi.read(addr),
            Some(MapLocation::Vi(addr)) => s.map.vi.read(addr),
            Some(MapLocation::Ai(addr)) => s.map.ai.read(addr),
            Some(MapLocation::Pi(addr)) => s.map.pi.read(addr),
            Some(MapLocation::RdramInterface(addr)) => s.map.rdram.read_interface(addr),
            Some(MapLocation::Si(addr)) => s.map.si.read(addr),
            Some(MapLocation::Cart(addr)) => s.map.cart.read(addr),
            None => panic!("Invalid read address: {:08X}", addr),
        }

        // DD
        // 0x0500_0000..=0x05FF_FFFF => {
        //     // Open bus: https://n64brew.dev/wiki/Parallel_Interface#Open_bus_behavior

        //     let lo = addr & 0xFFFF;
        //     T::from_u32((lo << 16) | lo) // TODO weirddd
        // }
    }

    // TODO what if address crosses a boundary?
    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        let location = Self::decode(addr);

        match location {
            Some(MapLocation::Rdram(addr)) => Rdram::write(s, addr, data),
            Some(MapLocation::RdramRegs(addr)) => Rdram::write_reg(s, addr, data),
            Some(MapLocation::RspDmem(addr)) => Rsp::write_dmem(s, addr, data),
            Some(MapLocation::RspImem(addr)) => Rsp::write_imem(s, addr, data),
            Some(MapLocation::RspRegs(addr)) => Rsp::write_reg(s, addr, data),
            Some(MapLocation::Mi(addr)) => Mi::write(s, addr, data),
            Some(MapLocation::Vi(addr)) => Vi::write(s, addr, data),
            Some(MapLocation::Ai(addr)) => Ai::write(s, addr, data),
            Some(MapLocation::Pi(addr)) => Pi::write(s, addr, data),
            Some(MapLocation::RdramInterface(addr)) => Rdram::write_interface(s, addr, data),
            Some(MapLocation::Si(addr)) => Si::write(s, addr, data),
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

// TODO clean up
pub fn address_info(addr: u32) -> Option<&'static str> {
    let location = Map::decode(addr);

    match location {
        Some(MapLocation::RdramRegs(addr)) => Rdram::reg_info(addr),
        Some(MapLocation::RspRegs(addr)) => Rsp::reg_info(addr),
        Some(MapLocation::Mi(addr)) => Mi::reg_info(addr),
        Some(MapLocation::Vi(addr)) => Vi::reg_info(addr),
        Some(MapLocation::Ai(addr)) => Ai::reg_info(addr),
        Some(MapLocation::Pi(addr)) => Pi::address_info(addr),
        Some(MapLocation::RdramInterface(addr)) => Rdram::interface_info(addr),
        Some(MapLocation::Si(addr)) => Si::address_info(addr),
        _ => None,
    }
}
