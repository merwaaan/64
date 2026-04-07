use std::{collections::HashMap, hash::Hasher, sync::Arc};

use bitbybit::bitenum;
use rapidhash::fast::RapidHasher;

use crate::{
    blocks::read_block,
    dp::{SetTile, SetTileSize, rgba5551_to_8888},
};

#[bitenum(u3, exhaustive = true)]
#[derive(Debug)]
pub enum ImageFormat {
    RGBA = 0,
    YUV = 1,
    ColorIndexed = 2,
    IntensityAlpha = 3,
    Intensity = 4,

    // 4+ values also mean Intensity
    Intensity2 = 5,
    Intensity3 = 6,
    Intensity4 = 7,
}

#[bitenum(u2, exhaustive = true)]
#[derive(Debug)]
pub enum TexelSize {
    B4 = 0,
    B8 = 1,
    B16 = 2,
    B32 = 3,
}

impl TexelSize {
    pub fn bits(&self) -> usize {
        match self {
            TexelSize::B4 => 4,
            TexelSize::B8 => 8,
            TexelSize::B16 => 16,
            TexelSize::B32 => 32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub rgba: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

impl Tile {
    /// Hashes a tile to identify it uniquely.
    pub fn hash(tmem: &[u8], tile: SetTile, size: SetTileSize) -> u64 {
        let mut hasher = RapidHasher::default();

        // We hash only the data relevant to the tile to avoid redundant entries
        // (eg. ignore out-of-bound bytes, ignore palettes if not paletted, etc)

        let tile_stride = tile.stride_byte() as u32;

        let top = size.upper_left_y().value();
        let bottom = size.lower_right_y().value();

        debug_assert!(top < bottom);

        let tile_height = (bottom >> 2).wrapping_sub(top >> 2) + 1;

        let tile_bytes = tile_stride as usize * tile_height as usize;

        let tmem_address = tile.tmem_address_byte() as usize;

        hasher.write(&tmem[tmem_address..tmem_address + tile_bytes]);

        match (tile.format(), tile.texel_size()) {
            (ImageFormat::ColorIndexed, TexelSize::B4) => {
                let palette = tile.palette().value() as usize;
                let palette_offset = 0x800 + palette * 0x20;

                hasher.write(&tmem[palette_offset..palette_offset + 0x20]);
            }

            (ImageFormat::ColorIndexed, TexelSize::B8) => {
                hasher.write(&tmem[0x800..0xA00]);
            }

            _ => {}
        }

        // Hash everything that has an effect on the interpretation of the tile

        hasher.write_u8(tile.format().raw_value().value());
        hasher.write_u8(tile.texel_size().raw_value().value());
        hasher.write_u16(tile.line_size().value());
        hasher.write_u16(tile.tmem_address().value());

        hasher.write_u16(size.upper_left_x().value());
        hasher.write_u16(size.upper_left_y().value());
        hasher.write_u16(size.lower_right_x().value());
        hasher.write_u16(size.lower_right_y().value());

        hasher.finish()
    }

    /// Decodes the tile from the TMEM into RGBA format.
    pub fn decode(tmem: &[u8], tile: SetTile, size: SetTileSize) -> Self {
        let left = size.upper_left_x().value();
        let right = size.lower_right_x().value();
        let top = size.upper_left_y().value();
        let bottom = size.lower_right_y().value();

        debug_assert!(left < right);
        debug_assert!(top < bottom);

        let tile_width = ((right >> 2).wrapping_sub(left >> 2) + 1) as usize;
        let tile_height = ((bottom >> 2).wrapping_sub(top >> 2) + 1) as usize;
        let tile_stride = tile.stride_byte();

        let mut rgba: Vec<u8> = Vec::with_capacity(tile_width * tile_height * 4);

        // We copy rows individually to account for the tile's stride which can be different from its width

        // TODO offset start with top left x and y???

        let mut row_address = tile.tmem_address_byte();

        for _row in 0..tile_height {
            match (tile.format(), tile.texel_size()) {
                (ImageFormat::RGBA, TexelSize::B16) => {
                    // 2 bytes per texel: 5 bits red, 5 bits green, 5 bits blue, 1 bit alpha

                    let bytes_per_row = tile_width * 2;

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(
                            tmem_data.chunks_exact(2)
                                .flat_map(|texel| rgba5551_to_8888(texel[0], texel[1])),
                        );
                    });
                }

                (ImageFormat::RGBA, TexelSize::B32) => {
                    // 4 bytes per texel: 8 bits red, 8 bits green, 8 bits blue, 8 bits alpha

                    let bytes_per_row = tile_width * 4;

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend_from_slice(tmem_data);
                    });
                }

                (ImageFormat::ColorIndexed, TexelSize::B4)
                // Some games use unsupported formats: fallback to 4-bit ColorIndexed // TODO is this correct?
                | (ImageFormat::RGBA, TexelSize::B4)
                | (ImageFormat::RGBA, TexelSize::B8) => {
                    // 4 bits per texel: 4-bit color index into one of the 16-bit palettes

                    let bytes_per_row = tile_width.div_ceil(2);

                    let palette_offset =
                        0x800 + (tile.palette().value() as usize) * 16;

                    // TODO optim: convert palettes on LoadTLUT? also when writing tex in case games do crazy hacks?

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(
                            tmem_data.iter()
                                // Split each byte into two 4-bit texels
                                .flat_map(|byte| [byte & 0xF0 >> 4, byte & 0x0F])
                                // Convert each texel to RGBA
                                .flat_map(|color_index| {
                                    let color_offset =
                                        palette_offset + (color_index as usize) * 2;

                                    rgba5551_to_8888(
                                        tmem[color_offset],
                                        tmem[color_offset + 1],
                                    )
                                }),
                        );
                    });

                    // If the tile width is odd, we pushed an extraneous 4-bit entry last, so remove it

                    if tile_width & 1 != 0 {
                        for _ in 0..4 {
                            rgba.pop();
                        }
                    }
                }

                (ImageFormat::ColorIndexed, TexelSize::B8) => {
                    // 1 byte per texel: 8-bit color index into the full 16-bit palette

                    let bytes_per_row = tile_width;

                    let palette_offset = 0x800;

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(tmem_data.iter().flat_map(|color_index| {
                            let color_offset =
                                palette_offset + (*color_index as usize) * 2;

                            rgba5551_to_8888(
                                tmem[color_offset],
                                tmem[color_offset + 1],
                            )
                        }));
                    });
                }

                (ImageFormat::IntensityAlpha, TexelSize::B4) => {
                    // 4 bits per texel: 3 bits intensity, 1 bit alpha

                    let bytes_per_row = tile_width.div_ceil(2);

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(
                            tmem_data.iter()
                                // Split each byte into two 4-bit texels
                                .flat_map(|byte| [byte & 0xF0 >> 4, byte & 0x0F])
                                // Convert each texel to RGBA
                                .flat_map(|texel| {
                                    let intensity = ((texel >> 1) & 7) * 255 / 7; // TODO optim?
                                    let alpha = (texel & 1) * 255; // TODO optim?

                                    [intensity, intensity, intensity, alpha]
                                }),
                        );
                    });

                    // If the tile width is odd, we pushed an extraneous 4-bit entry last, so remove it

                    if tile_width & 1 != 0 {
                        for _ in 0..4 {
                            rgba.pop();
                        }
                    }
                }

                (ImageFormat::IntensityAlpha, TexelSize::B8) => {
                    // 1 byte per texel: 4 bits intensity, 4 bits alpha

                    let bytes_per_row = tile_width;

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(tmem_data.iter().flat_map(|texel| {
                            let intensity = (*texel >> 4) * 255 / 15; // TODO optim?
                            let alpha = (*texel & 0x0F) * 255 / 15; // TODO optim?

                            [intensity, intensity, intensity, alpha]
                        }));
                    });
                }

                (ImageFormat::IntensityAlpha, TexelSize::B16) => {
                    // 2 bytes per texel: 8-bit intensity, 8-bit alpha

                    let bytes_per_row = tile_width * 2;

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(tmem_data.chunks_exact(2).flat_map(|texel| {
                            let intensity = texel[0];
                            let alpha = texel[1];

                            [intensity, intensity, intensity, alpha]
                        }));
                    });
                }

                (ImageFormat::Intensity, TexelSize::B4)
                | (ImageFormat::Intensity2, TexelSize::B4)
                | (ImageFormat::Intensity3, TexelSize::B4)
                | (ImageFormat::Intensity4, TexelSize::B4) => {
                    // 4 bits of intensity per texel

                    let bytes_per_row = tile_width.div_ceil(2);

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(
                            tmem_data.iter()
                                // Split each byte into two 4-bit texels
                                .flat_map(|byte| [byte & 0xF0 >> 4, byte & 0x0F])
                                // Convert each texel to RGBA
                                .flat_map(|texel| {
                                    let intensity = (texel << 4) | texel;

                                    [intensity, intensity, intensity, intensity]
                                }),
                        );
                    });

                    // If the tile width is odd, we pushed an extraneous 4-bit entry last, so remove it

                    if tile_width & 1 != 0 {
                        for _ in 0..4 {
                            rgba.pop();
                        }
                    }
                }

                (ImageFormat::Intensity, TexelSize::B8)
                | (ImageFormat::Intensity2, TexelSize::B8)
                | (ImageFormat::Intensity3, TexelSize::B8)
                | (ImageFormat::Intensity4, TexelSize::B8) => {
                    // 1 byte per texel: 8-bit intensity

                    let bytes_per_row = tile_width;

                    read_block(&tmem, row_address, bytes_per_row, |tmem_data| {
                        rgba.extend(tmem_data.iter().flat_map(|intensity| {
                            [*intensity, *intensity, *intensity, *intensity]
                        }));
                    });
                }

                _ => panic!(
                    "Unsupported {:?} / {:?} format",
                    tile.format(),
                    tile.texel_size()
                ),
            }

            row_address += tile_stride;
        }

        debug_assert_eq!(rgba.len(), tile_width * tile_height * 4);

        Self {
            rgba: Arc::new(rgba),
            width: tile_width as u32,
            height: tile_height as u32,
        }
    }
}

#[derive(Default)]
pub struct TileCache {
    tiles: HashMap<u64, Tile>,

    i: u64,
}

impl TileCache {
    /// Returns the tile in decoded RGBA format from the cache.
    /// Decodes it if it doesn't exist yet.
    pub fn get(&mut self, tmem: &[u8], tile: SetTile, size: SetTileSize) -> (&Tile, u64) {
        let hash = Tile::hash(tmem, tile, size);

        let tile = self
            .tiles
            .entry(hash)
            .or_insert_with(|| Tile::decode(tmem, tile, size));

        (tile, hash)

        // let tile = self
        //     .tiles
        //     .entry(self.i)
        //     .or_insert_with(|| Tile::decode(tmem, tile, size));

        // self.i += 1;

        // (tile, self.i)
    }
}

#[cfg(test)]
mod test {
    use arbitrary_int::prelude::*;

    use super::*;

    fn setup_tmem() -> [u8; 0x1000] {
        std::array::from_fn(|i| i as u8)
    }

    fn setup_tile(
        tmem_addr: u16,
        width: u16,
        height: u16,
        format: ImageFormat,
        texel_size: TexelSize,
    ) -> (SetTile, SetTileSize) {
        let stride = ((((width * texel_size.bits() as u16) + 7) & !7) / 8).next_multiple_of(8);

        (
            SetTile::default()
                .with_format(format)
                .with_texel_size(texel_size)
                .with_line_size(u9::new(stride >> 3))
                .with_tmem_address(u9::new(tmem_addr >> 3)),
            SetTileSize::default()
                .with_upper_left_x(u12::new(0))
                .with_upper_left_y(u12::new(0))
                .with_lower_right_x(u12::new((width - 1) << 2))
                .with_lower_right_y(u12::new((height - 1) << 2)),
        )
    }

    const FORMATS: [(ImageFormat, TexelSize); 9] = [
        (ImageFormat::RGBA, TexelSize::B16),
        (ImageFormat::RGBA, TexelSize::B32),
        (ImageFormat::ColorIndexed, TexelSize::B4),
        (ImageFormat::ColorIndexed, TexelSize::B8),
        (ImageFormat::IntensityAlpha, TexelSize::B4),
        (ImageFormat::IntensityAlpha, TexelSize::B8),
        (ImageFormat::IntensityAlpha, TexelSize::B16),
        (ImageFormat::Intensity, TexelSize::B4),
        (ImageFormat::Intensity, TexelSize::B8),
    ];

    #[test]
    fn tile_hash_constant() {
        for (format, texel_size) in FORMATS {
            let tmem = setup_tmem();

            let (tile, size) = setup_tile(0, 32, 32, format, texel_size);

            let hashes = (0..100)
                .map(|_| Tile::hash(&tmem, tile, size))
                .collect::<Vec<_>>();

            assert!(hashes.iter().all(|hash| *hash == hashes[0]));
        }
    }

    #[test]
    fn tile_hash_depends_on_texels() {
        for (format, texel_size) in FORMATS {
            let mut tmem = setup_tmem();

            let (tile, size) = setup_tile(128, 30, 5, format, texel_size);

            let length = match texel_size {
                TexelSize::B4 => 80,
                TexelSize::B8 => 160,
                TexelSize::B16 => 320,
                TexelSize::B32 => 600,
            };

            let stop = if matches!(format, ImageFormat::ColorIndexed) {
                0x800
            } else {
                0x1000
            };

            let initial = Tile::hash(&tmem, tile, size);

            // Before

            for i in 0..128 {
                tmem[i] = !tmem[i];
                assert_eq!(initial, Tile::hash(&tmem, tile, size));
                tmem[i] = !tmem[i];
            }

            // Texels

            for i in 128..128 + length {
                tmem[i] = !tmem[i];
                assert_ne!(initial, Tile::hash(&tmem, tile, size));
                tmem[i] = !tmem[i];
            }

            // After

            for i in 128 + length..stop {
                tmem[i] = !tmem[i];
                assert_eq!(initial, Tile::hash(&tmem, tile, size));
                tmem[i] = !tmem[i];
            }
        }
    }

    #[test]
    fn tile_hash_depends_on_full_palette_if_paletted32() {
        let mut tmem = setup_tmem();

        let (tile, size) = setup_tile(0, 32, 32, ImageFormat::ColorIndexed, TexelSize::B8);

        let initial = Tile::hash(&tmem, tile, size);

        // Before palette

        for i in 0x400..0x800 {
            tmem[i] = !tmem[i];
            assert_eq!(initial, Tile::hash(&tmem, tile, size));
            tmem[i] = !tmem[i];
        }

        // Full palette

        for i in 0x800..0xA00 {
            tmem[i] = !tmem[i];
            assert_ne!(initial, Tile::hash(&tmem, tile, size));
            tmem[i] = !tmem[i];
        }

        // After palette

        for i in 0xA00..0x1000 {
            tmem[i] = !tmem[i];
            assert_eq!(initial, Tile::hash(&tmem, tile, size));
            tmem[i] = !tmem[i];
        }
    }

    #[test]
    fn tile_hash_depends_on_sub_palette_if_paletted16() {
        let mut tmem = setup_tmem();

        let (mut tile, size) = setup_tile(0, 32, 32, ImageFormat::ColorIndexed, TexelSize::B4);
        tile.set_palette(u4::new(3));

        let initial = Tile::hash(&tmem, tile, size);

        // Before + previous palettes

        for i in 0x200..0x860 {
            tmem[i] = !tmem[i];
            assert_eq!(initial, Tile::hash(&tmem, tile, size));
            tmem[i] = !tmem[i];
        }

        // Used palette

        for i in 0x860..0x880 {
            tmem[i] = !tmem[i];
            assert_ne!(initial, Tile::hash(&tmem, tile, size));
            tmem[i] = !tmem[i];
        }

        // Next palettes + end of TMEM

        for i in 0x880..0x1000 {
            tmem[i] = !tmem[i];
            assert_eq!(initial, Tile::hash(&tmem, tile, size));
            tmem[i] = !tmem[i];
        }
    }
}
