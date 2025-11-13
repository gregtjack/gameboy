use crate::{
    addressable::Addressable,
    mmu::{OAM_BEGIN, OAM_END},
};

#[derive(Debug, Clone, Copy)]
pub struct Oam([Sprite; 40]);

impl Oam {
    pub fn new() -> Self {
        Self([Sprite::new(); 40])
    }

    pub fn get_sprites(&self) -> [Sprite; 40] {
        return self.0;
    }
}

impl Addressable for Oam {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            OAM_BEGIN..=OAM_END => {
                let offset = (addr - OAM_BEGIN) as usize;
                let sprite = self.0[offset / 4];
                match offset % 4 {
                    0 => sprite.y_pos.wrapping_add(16),
                    1 => sprite.x_pos.wrapping_add(8),
                    2 => sprite.tile_index,
                    3 => sprite.get_flags_byte(),
                    _ => panic!("invalid sprite offset"),
                }
            }
            _ => panic!("[oam:read] invalid address"),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            OAM_BEGIN..=OAM_END => {
                let offset = (addr - OAM_BEGIN) as usize;
                let sprite = &mut self.0[offset / 4];
                match offset % 4 {
                    0 => sprite.y_pos = value.wrapping_sub(16),
                    1 => sprite.x_pos = value.wrapping_sub(8),
                    2 => sprite.tile_index = value,
                    3 => sprite.set_flags_from_byte(value),
                    _ => panic!("invalid sprite offset"),
                }
            }
            _ => panic!("[oam:write] invalid address"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    /// byte 0
    pub y_pos: u8,
    /// byte 1
    pub x_pos: u8,
    /// byte 2
    pub tile_index: u8,
    /*
    Byte 3
    +------+----------+--------+--------+-------------+------+-------------+
    |  -   |    7     |   6    |   5    |      4      |  3   |     210     |
    +------+----------+--------+--------+-------------+------+-------------+
    | Attr | priority | y flip | x flip | DMG palette | Bank | CGB palette |
    +------+----------+--------+--------+-------------+------+-------------+
    Priority: 0 = No, 1 = BG and Window colors 1–3 are drawn over this OBJ
    Y flip: 0 = Normal, 1 = Entire OBJ is vertically mirrored
    X flip: 0 = Normal, 1 = Entire OBJ is horizontally mirrored
    DMG palette [Non CGB Mode only]: 0 = OBP0, 1 = OBP1
    Bank [CGB Mode Only]: 0 = Fetch tile from VRAM bank 0, 1 = Fetch tile from VRAM bank 1
    CGB palette [CGB Mode Only]: Which of OBP0–7 to use
    */
    pub priority: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub palette: bool,
    pub bank: bool,
    pub cgb_palette: u8,
}

impl Sprite {
    pub fn new() -> Self {
        Sprite {
            y_pos: 0,
            x_pos: 0,
            tile_index: 0,
            priority: false,
            y_flip: false,
            x_flip: false,
            palette: false,
            bank: false,
            cgb_palette: 0,
        }
    }

    pub fn set_flags_from_byte(&mut self, value: u8) {
        self.priority = ((value >> 7) & 1) != 0;
        self.y_flip = ((value >> 6) & 1) != 0;
        self.x_flip = ((value >> 5) & 1) != 0;
        self.palette = ((value >> 4) & 1) != 0;
        self.bank = ((value >> 3) & 1) != 0;
        self.cgb_palette = value & 0b111;
    }

    pub fn get_flags_byte(&self) -> u8 {
        let mut value = 0;
        value |= (self.priority as u8) << 7;
        value |= (self.y_flip as u8) << 6;
        value |= (self.x_flip as u8) << 5;
        value |= (self.palette as u8) << 4;
        value |= (self.bank as u8) << 3;
        value |= self.cgb_palette & 0b111;
        value
    }
}
