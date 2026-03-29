use crate::rendering::video::Texture;

pub struct Cell {
    pub tile_index: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A packed atlas of tile textures.
pub struct Atlas {
    width: u32,
    height: u32,

    /// Packed cells, in the same order as the source textures.
    pub cells: Vec<Cell>,
}

impl Atlas {
    /// Builds an `width` x `height` atlas from a set of tile textures.
    pub fn build(tile_textures: &[Texture], width: u32, height: u32) -> Self {
        let mut cells = Vec::with_capacity(tile_textures.len());

        // Simple shelf packing:
        // - store from left to right
        // - when the current row is full, move to the next row
        // - use the tallest cell's height as the row height

        let mut current_x = 0;
        let mut current_y = 0;
        let mut current_row_height = 0;

        for (tile_index, tile_texture) in tile_textures.iter().enumerate() {
            if current_x + tile_texture.width > width {
                current_x = 0;
                current_y += current_row_height;
                current_row_height = 0;
            }

            if current_y + tile_texture.height > height {
                panic!(
                    "Texture doesn't fit in atlas: {}x{} @ ({}, {}) > {}x{}",
                    tile_texture.width, tile_texture.height, current_x, current_y, width, height
                );
            }

            cells.push(Cell {
                tile_index,
                x: current_x,
                y: current_y,
                width: tile_texture.width,
                height: tile_texture.height,
            });

            current_x += tile_texture.width;
            current_row_height = current_row_height.max(tile_texture.height);
        }

        Self {
            width,
            height,
            cells,
        }
    }

    /// Returns the packed cells.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Remaps a pair of UV coordinates to the packed atlas UV coordinates.
    pub fn remap_uv(&self, tile_uv: [f32; 2], tile_index: usize) -> [f32; 2] {
        let cell = &self.cells[tile_index];

        [
            (cell.x as f32) / (self.width as f32)
                + tile_uv[0] * (cell.width as f32) / (self.width as f32),
            (cell.y as f32) / (self.height as f32)
                + tile_uv[1] * (cell.height as f32) / (self.height as f32),
        ]
    }
}
