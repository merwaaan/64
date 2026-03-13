use crate::{
    cop0::{self, Cop0},
    exception::Exception,
    system::{PhysicalAddress, VirtualAddress},
};

/// TLB page
#[derive(Default, Clone, Copy, Debug)]
struct Page {
    p: u32,
    cache: u8,
    writable: bool, // referred to as "dirty" in the CPU user manual
    valid: bool,
}

// TODO optim: identify TLB state that correspond to no TLB and avoid all the matching logic?

/// TLB entry
///
/// An entry is more often evaluated than updated so we store shifted/masked values to avoid bit twiddling in the hot path
/// // TODO really faster??? not sure
#[derive(Default, Clone, Copy, Debug)]
pub struct Entry {
    vpn2: u64, // u64 to preserve the high bits in case they are read back
    global: bool,
    asid: u8,
    mask: u32,
    pages: [Page; 2],
}

impl Entry {
    pub fn from_cop0_regs(cop0: &Cop0) -> Self {
        let hi = cop0.read(cop0::Register::EntryHi as usize).get64();
        let lo0 = cop0.read(cop0::Register::EntryLo0 as usize).get();
        let lo1 = cop0.read(cop0::Register::EntryLo1 as usize).get();
        let mask = cop0.read(cop0::Register::PageMask as usize).get();

        let pages = [
            Page {
                p: (lo0 >> 6) & 0x000F_FFFF,
                cache: ((lo0 >> 3) & 7) as u8,
                writable: lo0 & 4 != 0,
                valid: lo0 & 2 != 0,
            },
            Page {
                p: (lo1 >> 6) & 0x000F_FFFF,
                cache: ((lo1 >> 3) & 7) as u8,
                writable: lo1 & 4 != 0,
                valid: lo1 & 2 != 0,
            },
        ];

        Self {
            vpn2: (hi >> 13) & 0xFFFE0000_07FFFFFF, // clear the fill bits
            global: (lo0 & lo1 & 1) != 0,
            asid: (hi & 0xFF) as u8,
            mask: (mask >> 13) & 0x0000_0FFF,
            pages,
        }
    }

    pub fn to_cop0_regs(&self, cop0: &mut Cop0) {
        cop0.write64(
            cop0::Register::EntryLo0 as usize,
            ((self.pages[0].p << 6)
                | ((self.pages[0].cache as u32) << 3)
                | ((self.pages[0].writable as u32) << 2)
                | ((self.pages[0].valid as u32) << 1)
                | self.global as u32) as u64,
        );

        cop0.write64(
            cop0::Register::EntryLo1 as usize,
            ((self.pages[1].p << 6)
                | ((self.pages[1].cache as u32) << 3)
                | ((self.pages[1].writable as u32) << 2)
                | ((self.pages[1].valid as u32) << 1)
                | self.global as u32) as u64,
        );

        cop0.write64(cop0::Register::PageMask as usize, (self.mask << 13) as u64);

        cop0.write64(
            cop0::Register::EntryHi as usize,
            ((self.vpn2 & !(self.mask as u64)) << 13)
                | ((self.global as u64) << 12)
                | (self.asid as u64),
        );
    }

    pub fn vpn2(&self) -> u64 {
        self.vpn2
    }

    pub fn global(&self) -> bool {
        self.global
    }

    pub fn asid(&self) -> u8 {
        self.asid
    }

    pub fn mask(&self) -> u32 {
        self.mask
    }

    pub fn page_pfn(&self, index: usize) -> u32 {
        debug_assert!(index < self.pages.len());
        self.pages[index].p
    }

    pub fn page_cache(&self, index: usize) -> u8 {
        debug_assert!(index < self.pages.len());
        self.pages[index].cache
    }

    pub fn page_writable(&self, index: usize) -> bool {
        debug_assert!(index < self.pages.len());
        self.pages[index].writable
    }

    pub fn page_valid(&self, index: usize) -> bool {
        debug_assert!(index < self.pages.len());
        self.pages[index].valid
    }
}

/// Translation Lookaside Buffer
#[derive(Default, Clone, Copy, Debug)]
pub struct Tlb {
    entries: [Entry; 32],
}

impl Tlb {
    pub fn read(&self, index: u32) -> Entry {
        debug_assert!(index < 32);

        self.entries[(index & 0x3F) as usize]
    }

    pub fn write(&mut self, index: u32, entry: Entry) {
        debug_assert!(index < 32);

        self.entries[(index & 0x3F) as usize] = entry;
    }

    #[must_use]
    pub fn translate(
        &self,
        addr: VirtualAddress,
        cop0: &Cop0,
        write: bool,
    ) -> Result<PhysicalAddress, Exception> {
        // Extract VPN2 and ASID from the address

        let vpn2 = addr.0 >> 13;

        let asid = cop0.read(cop0::Register::EntryHi as usize).get() as u8;
        // R must match or not??? does for probe

        // Look for a matching entry

        for entry in self.entries {
            let entry_vpn2 = entry.vpn2 as u32;

            // TODO negate mask on write?
            if (entry.global || entry.asid == asid)
                && (entry_vpn2 & !entry.mask) == (vpn2 & !entry.mask)
            {
                // Pick the even or odd page

                let page_size = (entry.mask + 1) * 4096;

                let odd = (addr.0 as u64 & page_size as u64) != 0;

                let page = entry.pages[odd as usize];

                // Check that the page is valid

                if !page.valid {
                    return Err(if write {
                        Exception::TlbInvalidStore
                    } else {
                        Exception::TlbInvalidLoad
                    });
                }

                // Check that the page is writable if we are writing

                if write && !page.writable {
                    return Err(Exception::TlbModification);
                }

                let offset = addr.0 & (page_size as u32 - 1);

                let translated = (page.p << 12) | offset;

                return Ok(PhysicalAddress(translated));
            }
        }

        Err(if write {
            Exception::TlbMissStore
        } else {
            Exception::TlbMissLoad
        })
    }

    /// Returns the index of the first matching entry
    pub fn probe(&self, cop0: &Cop0) -> Option<u8> {
        let hi = cop0.read(cop0::Register::EntryHi as usize).get64();
        let vpn2 = hi >> 13;
        let asid = hi as u8;

        for (index, entry) in self.entries.iter().enumerate() {
            let entry_vpn2 = entry.vpn2;

            // TODO negate mask on write?
            if (entry.global || entry.asid == asid)
                && (entry_vpn2 & !(entry.mask as u64)) == (vpn2 & !(entry.mask as u64))
            {
                return Some(index as u8);
            }
        }

        None
    }
}
