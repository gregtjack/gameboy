use crate::addressable::Addressable;

/// Area 0: 0x9800 - 0x9BFF
/// Area 1: 0x9C00 - 0x9FFF
#[derive(Debug, Clone, Copy)]
pub enum Area {
    Area0,
    Area1,
}

#[derive(Debug, Clone, Copy)]
pub struct TileMap {
    area0: [[u8; 32]; 32],
    area1: [[u8; 32]; 32],
}

impl TileMap {
    pub fn new() -> Self {
        Self {
            area0: [[0; 32]; 32],
            area1: [[0; 32]; 32],
        }
    }

    pub fn get_tile_index(&self, area: Area, x: usize, y: usize) -> u8 {
        match area {
            Area::Area0 => self.area0[y / 8][x / 8],
            Area::Area1 => self.area1[y / 8][x / 8],
        }
    }
}

impl Addressable for TileMap {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x9800..=0x9BFF => {
                let offset = addr as usize - 0x9800;
                self.area0[offset / 32][offset % 32]
            }
            0x9C00..=0x9FFF => {
                let offset = addr as usize - 0x9C00;
                self.area1[offset / 32][offset % 32]
            }
            _ => panic!("invalid address for tile map: {:#06x}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x9800..=0x9BFF => {
                let offset = addr as usize - 0x9800;
                self.area0[offset / 32][offset % 32] = value;
            }
            0x9C00..=0x9FFF => {
                let offset = addr as usize - 0x9C00;
                self.area1[offset / 32][offset % 32] = value;
            }
            _ => panic!("invalid address for tile map: {:#06x}", addr),
        }
    }
}
