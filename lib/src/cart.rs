use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use thiserror::Error;
use zip::ZipArchive;

use crate::{data::Data, map::Location, system::System};

#[derive(Debug, Error)]
pub enum CartError {
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid ROM extension (supported: .z64, .v64, .n64, .zip)")]
    InvalidRom,

    #[error("invalid ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),
}

const ROM_START: u32 = 0x1000_0000;
const ROM_END: u32 = 0x1FC0_0000;

pub type CartLocation = Location<ROM_START, ROM_END>;

#[derive(Debug)]
pub struct Cart {
    data: Vec<u8>,

    // TODO name?
    isviewer_buffer: [u8; 0x200],
    isviewer_index: usize,
    isviewer_log: String,
}

impl Cart {
    pub fn load(path: &Path) -> Result<Self, CartError> {
        let mut data = load_rom(path)?;

        // Convert to big-endian, the native N64 format

        let first_word: u32 = u32::from_be_bytes(data[0..4].try_into().unwrap());

        match first_word {
            // Already big-endian
            0x80371240 => {}

            // Byte-swapped
            0x37804012 => {
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

        Ok(Self {
            data,
            isviewer_buffer: [0; 0x200],
            isviewer_index: 0,
            isviewer_log: String::new(),
        })
    }

    // pub fn pc(&self) -> u32 {
    //     u32::from_be_bytes(self.data[0x8..0xC].try_into().unwrap())
    // }

    // pub fn crc(&self) -> (u32, u32) {
    //     (
    //         u32::from_be_bytes(self.data[0x10..0x14].try_into().unwrap()),
    //         u32::from_be_bytes(self.data[0x14..0x18].try_into().unwrap()),
    //     )
    // }

    // pub fn name(&self) -> &[u8] {
    //     &self.data[0x20..0x34]
    // }

    // pub fn country(&self) -> u8 {
    //     self.data[0x3E]
    // }

    // pub fn version(&self) -> u8 {
    //     self.data[0x3F]
    // }

    pub fn read<T: Data>(&self, addr: CartLocation) -> T {
        T::read(&self.data, addr.relative())
    }

    pub fn write<T: Data>(s: &mut System, addr: CartLocation, data: T) {
        log::warn!("write CART: {:08X} {:X}", addr.relative(), data.to_u32());

        if (0x03FF_0020..0x03FF_0220).contains(&addr.relative()) {
            data.write(
                &mut s.map.cart.isviewer_buffer,
                addr.relative() - 0x03FF_0020,
            );

            s.map.cart.isviewer_index += T::SIZE;
        } else if addr.relative() == 0x03FF_0014 {
            let data =
                String::from_utf8_lossy(&s.map.cart.isviewer_buffer[..s.map.cart.isviewer_index]);

            s.map.cart.isviewer_log += &data;
            s.map.cart.isviewer_index = 0;

            log::info!("ISVIEWER: {}", s.map.cart.isviewer_log);
        }
    }
}

const ROM_EXTENSIONS: &[&str] = &[".z64", ".v64", ".n64"];

fn has_rom_extension(ext: &std::ffi::OsStr) -> bool {
    ext.to_str().map_or(false, |e| {
        ROM_EXTENSIONS
            .iter()
            .any(|rom| e.eq_ignore_ascii_case(rom.trim_start_matches('.')))
    })
}

fn load_rom(path: &Path) -> Result<Vec<u8>, CartError> {
    let mut data = Vec::new();

    // Load the file

    let file = File::open(path)?;

    match path.extension() {
        Some(extension) => {
            if has_rom_extension(extension) {
                BufReader::new(file).read_to_end(&mut data)?;
            } else if extension == std::ffi::OsStr::new("zip") {
                let mut archive = ZipArchive::new(file)?;

                let index = (0..archive.len()).find(|&i| {
                    let entry = archive.by_index(i).unwrap();
                    let name = entry.name().to_lowercase();
                    ROM_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
                });

                if let Some(index) = index {
                    let mut entry = archive.by_index(index).unwrap();
                    entry.read_to_end(&mut data)?;
                } else {
                    return Err(CartError::InvalidRom);
                }
            } else {
                return Err(CartError::InvalidRom);
            }
        }
        None => return Err(CartError::InvalidRom),
    }

    Ok(data)
}
