use crate::{
    ai::AiLocation,
    cart::CartLocation,
    dp::DpLocation,
    mi::MiLocation,
    pi::PiLocation,
    pif::PifRamLocation,
    ram::{RamInterfaceLocation, RamLocation, RamRegsLocation},
    si::SiLocation,
    sp::{SpMemLocation, SpRegsLocation},
    vi::ViLocation,
};

/// Location in the memory map.
/// Bound by start (inclusive) and end (exclusive) addresses.
///
/// Memory map sections can define their own locations:
///
/// ```
/// type RspDmemLocation = Location<0x0400_0000, 0x0400_1000>;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Location<const START: u32, const END: u32>(u32);

impl<const START: u32, const END: u32> Location<START, END> {
    pub const START: u32 = START;
    pub const END: u32 = END;

    pub fn from_relative(addr: u32) -> Self {
        debug_assert!(
            (0..END - START).contains(&addr),
            "Address {:08X} is out of relative range ({:08X}..{:08X})",
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
            "Address {:08X} is out of absolute range ({:08X}..{:08X})",
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
    Ram(RamLocation),
    RamRegs(RamRegsLocation),
    SpMem(SpMemLocation),
    SpRegs(SpRegsLocation),
    Dp(DpLocation),
    Mi(MiLocation),
    Vi(ViLocation),
    Ai(AiLocation),
    Pi(PiLocation),
    RamInterface(RamInterfaceLocation),
    Si(SiLocation),
    //Dd(DdLocation),
    Cart(CartLocation),
    Pif(PifRamLocation),
    OpenBus(u32),
}
