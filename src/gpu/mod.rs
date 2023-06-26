use crate::memory::VRAM_LEN;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const TILE_SET_LEN: usize = 384;

#[derive(Debug, Copy, Clone)]
pub enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

pub type Tile = [[TilePixelValue; 8]; 8];

fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

#[derive(Debug, Clone, Copy)]
pub struct GPU {
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub vram: [u8; VRAM_LEN],
    pub tile_set: [Tile; TILE_SET_LEN],
}

impl GPU {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_LEN],
            tile_set: [empty_tile(); TILE_SET_LEN],
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr]
    }

    pub fn write_vram(&mut self, addr: usize, value: u8) {
        self.vram[addr] = value;
        // If our index is greater than 0x1800, we're not writing to the tile set storage
        // so we can just return.
        if addr >= 0x1800 {
            return;
        }

        // Tiles rows are encoded in two bytes with the first byte always
        // on an even address. Bitwise ANDing the address with 0xffe
        // gives us the address of the first byte.
        // For example: `12 & 0xFFFE == 12` and `13 & 0xFFFE == 12`
        let normalized_addr = addr & 0xFFFE;

        // First we need to get the two bytes that encode the tile row.
        let byte1 = self.vram[normalized_addr];
        let byte2 = self.vram[normalized_addr + 1];

        // A tiles is 8 rows tall. Since each row is encoded with two bytes a tile
        // is therefore 16 bytes in total.
        let tile_index = addr / 16;
        // Every two bytes is a new row
        let row_index = (addr % 16) / 2;

        // Now we're going to loop 8 times to get the 8 pixels that make up a given row.
        for pixel_index in 0..8 {
            // To determine a pixel's value we must first find the corresponding bit that encodes
            // that pixels value:
            // 1111_1111
            // 0123 4567
            //
            // As you can see the bit that corresponds to the nth pixel is the bit in the nth
            // position *from the left*. Bits are normally indexed from the right.
            //
            // To find the first pixel (a.k.a pixel 0) we find the left most bit (a.k.a bit 7). For
            // the second pixel (a.k.a pixel 1) we first the second most left bit (a.k.a bit 6) and
            // so on.
            //
            // We then create a mask with a 1 at that position and 0s everywhere else.
            //
            // Bitwise ANDing this mask with our bytes will leave that particular bit with its
            // original value and every other bit with a 0.
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;

            // If the masked values are not 0 the masked bit must be 1. If they are 0, the masked
            // bit must be 0.
            //
            // Finally we can tell which of the four tile values the pixel is. For example, if the least
            // significant byte's bit is 1 and the most significant byte's bit is also 1, then we
            // have tile value `Three`.
            let value = match (lsb != 0, msb != 0) {
                (true, true) => TilePixelValue::Three,
                (false, true) => TilePixelValue::Two,
                (true, false) => TilePixelValue::One,
                (false, false) => TilePixelValue::Zero,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }
}
