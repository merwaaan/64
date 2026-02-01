use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[derive(Debug)]
pub struct Cart {
    pub data: Vec<u8>,
}

impl Cart {
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let mut data = Vec::new();

        let file = File::open(path)?;

        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut data)?;

        // Swap bytes
        // TODO depends on header?

        for word in data.chunks_exact_mut(2) {
            word.swap(0, 1);
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
}
