use crate::addressable::Addressable;

use super::color_id::{ColorBit, ColorId};

pub const VRAM_TILE_DATA_BEGIN: u16 = 0x8000;
pub const VRAM_TILE_DATA_END: u16 = 0x97FF;

#[derive(Debug, Clone, Copy)]
pub enum Area {
    Area0,
    Area1,
}

#[derive(Debug, Clone, Copy)]
pub struct TileData {
    pub tiles: [Tile; 384],
}

impl TileData {
    pub fn new() -> Self {
        Self {
            tiles: [Tile::new(); 384],
        }
    }

    pub fn get_row_pixels(&self, area: Area, index: u8, row: usize) -> [ColorId; 8] {
        let index = match area {
            Area::Area0 => {
                let signed_index = i8::from_ne_bytes(index.to_ne_bytes());
                ((signed_index as i16) + 256) as usize
            }
            Area::Area1 => index as usize,
        };

        self.tiles[index].data[row]
    }

    pub fn get_row(&self, index: u16, row: usize) -> [ColorId; 8] {
        self.tiles[index as usize].get_line(row)
    }
}

impl Addressable for TileData {
    fn read_byte(&self, addr: u16) -> u8 {
        let offset = (addr - VRAM_TILE_DATA_BEGIN) as usize;
        let tile = self.tiles[offset / 16];
        let line = (offset % 16) / 2;
        let hi_or_lo = if offset % 2 == 0 {
            ColorBit::Lo
        } else {
            ColorBit::Hi
        };
        tile.row_to_byte(line, hi_or_lo)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        let offset = (addr - VRAM_TILE_DATA_BEGIN) as usize;
        let tile = &mut self.tiles[offset / 16];
        let line = (offset % 16) / 2;
        let hi_or_lo = if offset % 2 == 0 {
            ColorBit::Lo
        } else {
            ColorBit::Hi
        };
        tile.byte_to_row(line, hi_or_lo, value);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    /// 8 x 8 tiles
    pub data: [[ColorId; 8]; 8],
}

impl Tile {
    pub fn new() -> Self {
        Self {
            data: [[ColorId::Id00; 8]; 8],
        }
    }

    pub fn get_line(&self, row: usize) -> [ColorId; 8] {
        self.data[row]
    }

    pub fn get_pixel(&self, row: usize, col: usize) -> ColorId {
        self.data[row][col]
    }

    pub fn set_pixel(&mut self, row: usize, col: usize, color_id: ColorId) {
        self.data[row][col] = color_id;
    }

    /// For each line, the first byte specifies the least significant bit of the color ID of each
    /// pixel, and the second byte specifies the most significant bit
    pub fn row_to_byte(&self, row: usize, hi_or_lo: ColorBit) -> u8 {
        let mut byte = 0u8;
        for (i, color_id) in self.data[row].iter().enumerate() {
            if color_id.get_bit(hi_or_lo) {
                byte |= 1 << (7 - i);
            }
        }
        byte
    }

    pub fn byte_to_row(&mut self, row: usize, hi_or_lo: ColorBit, data: u8) {
        for col in 0..8 {
            let bit = (data >> (7 - col)) & 1 == 1;
            self.data[row][col].set_bit(hi_or_lo, bit);
        }
    }
}
