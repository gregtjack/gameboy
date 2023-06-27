use crate::memory::{Memory, VRAM_LEN};

use self::{lcdc::LCDC, stat::STAT};

mod lcdc;
mod stat;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

#[derive(Debug, Clone, Copy)]
pub struct GPU {
    // Digital image with RGB. Size = 144 * 160 * 3.
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    pub vram: [u8; VRAM_LEN],
    pub lcdc: LCDC,
    pub stat: STAT,
    pub vblank: bool,
    pub hblank: bool,
    // Specifies the position in the 256x256 pixels BG map (32x32 tiles) which is to be displayed at the upper/left LCD
    // display position. Values in range from 0-255 may be used for X/Y each, the video controller automatically wraps
    // back to the upper (left) position in BG map when drawing exceeds the lower (right) border of the BG map area.
    pub sx: u8,
    pub sy: u8,
    // The LY indicates the vertical line to which the present data is transferred to the LCD Driver. The LY can
    // take on any value between 0 through 153. The values between 144 and 153 indicate the V-Blank period. Writing
    // will reset the counter.
    pub ly: u8,
    // The gameboy permanently compares the value of the LYC and LY registers. When both values are identical, the
    // coincident bit in the STAT register becomes set, and (if enabled) a STAT interrupt is requested.
    pub lyc: u8,

    //  This register assigns gray shades to the color numbers of the BG and Window tiles.
    //  Bit 7-6 - Shade for Color Number 3
    //  Bit 5-4 - Shade for Color Number 2
    //  Bit 3-2 - Shade for Color Number 1
    //  Bit 1-0 - Shade for Color Number 0
    pub bgp: u8,
    // This register assigns gray shades for sprite palette 0. It works exactly as BGP ($FF47), except that the lower
    // two bits aren't used because sprite data 00 is transparent.
    pub op0: u8,
    // This register assigns gray shades for sprite palette 1. It works exactly as BGP ($FF47), except that the lower
    // two bits aren't used because sprite data 00 is transparent.
    pub op1: u8,

    // Window x and y
    // Specifies the upper/left positions of the Window area
    pub wx: u8,
    pub wy: u8,
    // Gameboy video controller can display up to 40 sprites either in 8x8 or in 8x16 pixels. Because of a limitation
    // of hardware, only ten sprites can be displayed per scan line. Sprite patterns have the same format as BG tiles,
    // but they are taken from the Sprite Pattern Table located at $8000-8FFF and have unsigned numbering.
    pub oam: [u8; 0xA0],
    dots: u32,
}

impl GPU {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_LEN],
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            vblank: false,
            hblank: false,
            lcdc: lcdc::LCDC::new(),
            stat: stat::STAT::new(),
            sx: 0,
            sy: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            op0: 0,
            op1: 1,
            wx: 0,
            wy: 0,
            oam: [0; 0xA0],
            dots: 0,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self.dots += cycles;
        if self.dots >= 456 {
            self.dots -= 456;
            self.ly = self.ly.wrapping_add(1);
            if self.ly == 144 {
                self.vblank = true;
                self.stat.vblank_interrupt = true;
            } else if self.ly > 153 {
                self.ly = 0;
                self.vblank = false;
                self.stat.vblank_interrupt = false;
            }
        }
    }

    fn render_bg(&mut self) {
        // TODO
    }
}

impl Memory for GPU {
    fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;
        match address {
            0x8000..=0x9FFF => self.vram[address - 0x8000],
            0xFE00..=0xFE9F => self.oam[address - 0xFE00],
            0xFF40 => self.lcdc.into(),
            0xFF41 => self.stat.into(),
            0xFF42 => self.sy,
            0xFF43 => self.sx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.op0,
            0xFF49 => self.op1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!("Invalid vram address: {:X}", address),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;
        match address {
            0x8000..=0x9FFF => self.vram[address - 0x8000] = value,
            0xFE00..=0xFE9F => self.oam[address - 0xFE00] = value,
            0xFF40 => self.lcdc = value.into(),
            0xFF41 => self.stat = value.into(),
            0xFF42 => self.sy = value,
            0xFF43 => self.sx = value,
            0xFF44 => self.ly = value,
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.op0 = value,
            0xFF49 => self.op1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => panic!("Invalid vram address: {:X}", address),
        }
    }
}
