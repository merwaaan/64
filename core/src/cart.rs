use std::{
    ffi::OsStr,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use thiserror::Error;
use zip::ZipArchive;

use crate::{
    blocks::read_block,
    is_supported_rom_file,
    isviewer::{IsViewer, IsViewerBufferLocation, IsViewerControlLocation},
    location::Location,
    system::System,
    value::Value,
};

#[derive(Debug, Error)]
pub enum CartError {
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid ROM extension")]
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

    pub isviewer: IsViewer,
}

impl Cart {
    pub fn load(path: &Path) -> Result<Self, CartError> {
        let mut data = load_file(path)?;

        // Convert to big-endian, the native N64 format

        let first_word: u32 = u32::from_be_bytes(data[0..4].try_into().unwrap());

        match first_word {
            // Already big-endian
            0x8037_1240 => {}

            // Byte-swapped
            0x3780_4012 => {
                for word in data.chunks_exact_mut(2) {
                    word.swap(0, 1);
                }
            }

            // Word-swapped
            0x4012_3780 => {
                todo!("Word-swapped");
            }

            _ => {
                log::warn!("Unknown cart format: {:#08X}", first_word);
            }
        }

        Ok(Self {
            data,
            isviewer: IsViewer::default(),
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

    pub fn read<T: Value>(s: &System, addr: CartLocation) -> T {
        T::read_mem(&s.cart.data, addr.relative() % (s.cart.data.len() as u32))
    }

    pub fn write<T: Value>(s: &mut System, addr: CartLocation, data: T) {
        match addr.absolute() {
            // The IS-Viewer is mapped in the cart's region
            IsViewerControlLocation::START..IsViewerControlLocation::END => {
                s.cart.isviewer.flush();
            }

            IsViewerBufferLocation::START..IsViewerBufferLocation::END => {
                s.cart
                    .isviewer
                    .push(IsViewerBufferLocation::from_absolute(addr.absolute()), data);
            }

            _ => {
                log::warn!("write CART: {:08X} {:X}", addr.relative(), data);
            }
        }
    }

    pub fn read_block(&self, addr: CartLocation, length: usize, callback: impl FnMut(&[u8])) {
        read_block(&self.data, addr.relative() as usize, length, callback);
    }

    // pub fn write_block(&mut self, offset: usize, src: &[u8]) {
    //     write_block(src, &mut self.data, offset)
    // }
}

fn load_file(path: &Path) -> Result<Vec<u8>, CartError> {
    let mut data = Vec::new();

    if is_supported_rom_file(path) {
        let file = File::open(path)?;
        BufReader::new(file).read_to_end(&mut data)?;
    } else if path.extension() == Some(OsStr::new("zip")) {
        let mut archive = ZipArchive::new(File::open(path)?)?;

        let index = (0..archive.len()).find(|&i| {
            let entry = archive.by_index(i).unwrap();
            let name = entry.name().to_lowercase();
            is_supported_rom_file(Path::new(&name))
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

    Ok(data)
}
