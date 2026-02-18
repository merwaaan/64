use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::{data::Data, map::Location};

pub const ROM_START: u32 = 0x1000_0000;
pub const ROM_END: u32 = 0x1FC0_0000;

pub type CartLocation = Location<ROM_START, ROM_END>;

#[derive(Debug)]
pub struct Cart {
    data: Vec<u8>,
}

impl Cart {
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let mut data = Vec::new(); // TODO just store as u32s?

        let file = File::open(path)?;

        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut data)?;

        // Convert to big-endian, the native N64 format

        let first_word: u32 = u32::from_be_bytes(data[0..4].try_into().unwrap());

        match first_word {
            // Already big-endian
            0x80371240 => {
                log::info!("Big-endian");
            }

            // Byte-swapped
            0x37804012 => {
                log::info!("Byte-swapped");

                for word in data.chunks_exact_mut(2) {
                    word.swap(0, 1);
                }
            }

            // Word-swapped
            0x40123780 => {
                log::info!("Word-swapped");

                todo!("Word-swapped");
            }

            _ => {
                log::warn!("Unknown cart format: {:#08X}", first_word);
            }
        }

        Ok(Self { data })
    }

    pub fn pc(&self) -> u32 {
        u32::from_be_bytes(self.data[0x8..0xC].try_into().unwrap())
    }

    pub fn crc(&self) -> (u32, u32) {
        (
            u32::from_be_bytes(self.data[0x10..0x14].try_into().unwrap()),
            u32::from_be_bytes(self.data[0x14..0x18].try_into().unwrap()),
        )
    }

    pub fn name(&self) -> &[u8] {
        &self.data[0x20..0x34]
    }

    pub fn country(&self) -> u8 {
        self.data[0x3E]
    }

    pub fn version(&self) -> u8 {
        self.data[0x3F]
    }

    pub fn read<T: Data>(&self, loc: CartLocation) -> T {
        T::read(&self.data, loc.relative())
    }
}
