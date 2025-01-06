use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::{
        interrupts::{InterruptFlag, Interrupts},
        OAM_BEGIN, OAM_END, UNDEFINED, VRAM_BEGIN, VRAM_END, VRAM_LEN,
    },
    utils::addressable::Addressable,
};

use self::{
    lcdc::Lcdc,
    stat::{Mode, Stat},
};

mod lcdc;
mod stat;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const TILESET_FIRST_BEGIN_ADDRESS: u16 = 0x8000;
const TILESET_SECOND_BEGIN_ADDRESS: u16 = 0x9000;
const BGMAP_FIRST_BEGIN_ADDRESS: u16 = 0x9800;
const BGMAP_SECOND_BEGIN_ADDRESS: u16 = 0x9C00;

const CYCLES_OAM: u32 = 80;
const CYCLES_VRAM: u32 = 172;
const CYCLES_HBLANK: u32 = 204;
const CYCLES_VBLANK: u32 = 456;

const SCANLINES_DISPLAY: u8 = 143;
const MAX_SCANLINES: u8 = 153;

const PALETTE: [[u8; 4]; 4] = [
    [255, 255, 255, 255],
    [192, 192, 192, 255],
    [96, 96, 96, 255],
    [0, 0, 0, 255],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Color {
    pub fn rgba(&self) -> [u8; 4] {
        match self {
            Color::White => PALETTE[0],
            Color::LightGray => PALETTE[1],
            Color::DarkGray => PALETTE[2],
            Color::Black => PALETTE[3],
        }
    }
}

pub type Screen = [[Color; SCREEN_HEIGHT]; SCREEN_WIDTH];

#[derive(Debug)]
pub struct Gpu {
    // Digital image with RGB. Size = 144 * 160 * 3.
    frame_buffer: Screen,
    vram: [u8; 0x2000],
    lcdc: Lcdc,
    stat: Stat,
    v_blank: bool,
    h_blank: bool,
    int: Rc<RefCell<Interrupts>>,
    // Specifies the position in the 256x256 pixels BG map (32x32 tiles) which is to be displayed at the upper/left LCD
    // display position.
    sx: u8,
    sy: u8,
    // The LY indicates the vertical line to which the present data is transferred to the LCD Driver
    ly: u8,
    // The gameboy permanently compares the value of the LYC and LY registers. When both values are identical, the
    // coincident bit in the STAT register becomes set, and (if enabled) a STAT interrupt is requested.
    lyc: u8,
    //  This register assigns gray shades to the color numbers of the BG and Window tiles.
    //  Bit 7-6 - Shade for Color Number 3
    //  Bit 5-4 - Shade for Color Number 2
    //  Bit 3-2 - Shade for Color Number 1
    //  Bit 1-0 - Shade for Color Number 0
    bgp: u8,
    // This register assigns gray shades for sprite palette 0. It works exactly as BGP ($FF47), except that the lower
    // two bits aren't used because sprite data 00 is transparent.
    op0: u8,
    // This register assigns gray shades for sprite palette 1. It works exactly as BGP ($FF47), except that the lower
    // two bits aren't used because sprite data 00 is transparent.
    op1: u8,
    // Window x and y
    // Specifies the upper/left positions of the Window area
    wx: u8,
    wy: u8,
    // Gameboy video controller can display up to 40 sprites either in 8x8 or in 8x16 pixels. Because of a limitation
    // of hardware, only ten sprites can be displayed per scan line. Sprite patterns have the same format as BG tiles,
    // but they are taken from the Sprite Pattern Table located at $8000-8FFF and have unsigned numbering.
    oam: [u8; 0xA0],
    clock: u32,
}

impl Gpu {
    pub fn new(intf: Rc<RefCell<Interrupts>>) -> Self {
        Self {
            vram: [0; VRAM_LEN],
            frame_buffer: [[Color::White; SCREEN_HEIGHT]; SCREEN_WIDTH],
            int: intf,
            v_blank: false,
            h_blank: false,
            lcdc: Lcdc::from(0x91),
            stat: Stat::new(),
            sx: 0,
            sy: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            op0: 0xFF,
            op1: 0xFF,
            wx: 0,
            wy: 0,
            oam: [0; 0xA0],
            clock: 0,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        let mcycles = cycles / 4;
        if !self.lcdc.lcd_enable {
            return;
        }

        self.clock += cycles;

        let one_line_cycles = CYCLES_OAM + CYCLES_VRAM + CYCLES_HBLANK; // (* 114 *)

        match self.stat.mode {
            Mode::OAM => {
                if self.clock >= CYCLES_OAM {
                    self.set_mode(Mode::VRAM);
                    self.clock %= CYCLES_OAM;
                }
            }
            Mode::VRAM => {
                if self.clock >= CYCLES_VRAM {
                    self.clock %= CYCLES_VRAM;
                    self.render_scanline();
                    self.set_mode(Mode::HBlank);
                }
            }
            Mode::HBlank => {
                if self.clock >= CYCLES_HBLANK {
                    self.clock %= CYCLES_HBLANK;

                    if self.ly >= SCANLINES_DISPLAY {
                        self.set_mode(Mode::VBlank);
                        // render screen
                        self.int.borrow_mut().set_flag(InterruptFlag::VBlank);
                    } else {
                        self.set_scanline(self.ly + 1);
                        self.set_mode(Mode::OAM);
                    }
                }
            }
            Mode::VBlank => {
                if self.clock >= CYCLES_VBLANK {
                    self.set_scanline(self.ly + 1);
                    self.clock %= CYCLES_VBLANK;
                    if self.ly >= MAX_SCANLINES {
                        self.set_mode(Mode::OAM);
                        self.set_scanline(0);
                    }
                }
            }
        }
    }

    pub fn get_frame_buffer(&self) -> Screen {
        self.frame_buffer
    }

    fn compare_lyc(&mut self) {
        self.stat.coincidence_flag = false;
        if self.lyc == self.ly {
            self.stat.coincidence_flag = true;
            if self.stat.coincidence_interrupt {
                self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
            }
        }
    }

    fn set_scanline(&mut self, value: u8) {
        self.ly = value;
        self.compare_lyc();
    }

    fn set_mode(&mut self, mode: Mode) {
        self.stat.mode = mode;
        match self.stat.mode {
            Mode::OAM => {
                if self.stat.oam_interrupt {
                    self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                }
            }
            Mode::HBlank => {
                if self.stat.hblank_interrupt {
                    self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                }
            }
            Mode::VBlank => {
                if self.stat.vblank_interrupt {
                    self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                }
            }
            _ => {}
        }
    }

    fn render_scanline(&mut self) {
        if self.lcdc.bg_display {
            self.render_bg_line();
        }

        if self.lcdc.obj_display_enable {
            self.render_obj_line();
        }
    }

    fn render_bg_line(&mut self) {
        let y_bg = self.ly.wrapping_add(self.sy);
        let line_is_window = self.lcdc.window_display_enable && self.ly >= self.wy;

        for x in 0..SCREEN_WIDTH as u8 {
            let x_bg = x.wrapping_add(self.sx);
            let column_is_window = self.lcdc.window_display_enable && x >= self.wx.wrapping_sub(7);
            let is_window = line_is_window && column_is_window;

            let tile_address = if is_window {
                self.get_window_address(self.ly, x)
            } else {
                self.get_bgmap_address(y_bg, x_bg)
            };

            let tile = self.read_byte(tile_address);

            let tile_begin = if self.lcdc.bg_window_tile_data_select {
                // Use first tileset, tile_number interpreted as unsigned
                TILESET_FIRST_BEGIN_ADDRESS + tile as u16 * 16
            } else {
                // Use second tileset, tile_number interpreted as signed
                TILESET_SECOND_BEGIN_ADDRESS.wrapping_add(((tile as i8) as u16).wrapping_mul(16))
            };

            let y_tile_addr_offset = if is_window {
                (self.ly - self.wy) % 8 * 2
            } else {
                y_bg % 8 * 2
            } as u16;

            let tile_data_address = tile_begin + y_tile_addr_offset;

            let tile_data = self.read_byte(tile_data_address);
            // The color data is placed one byte after the pixel data
            let tile_color = self.read_byte(tile_data_address + 1);

            let pixel_index = if is_window {
                self.wx.wrapping_sub(x) % 8
            } else {
                7 - (x_bg % 8)
            };

            // Draw bg pixel to screen
            let color_index = Self::get_color_index(tile_data, tile_color, pixel_index);
        }
    }

    fn render_obj_line(&mut self) {}

    fn get_window_address(&self, y: u8, x: u8) -> u16 {
        let addr = if self.lcdc.window_tile_map_display_select {
            BGMAP_SECOND_BEGIN_ADDRESS
        } else {
            BGMAP_FIRST_BEGIN_ADDRESS
        };

        let y_offset = y.wrapping_sub(self.wy);
        let x_offset = x.wrapping_sub(self.wx.wrapping_sub(7));

        addr + (y_offset as u16 / 8 * 32) + (x_offset as u16 / 8)
    }

    fn get_bgmap_address(&self, y: u8, x: u8) -> u16 {
        let addr = if self.lcdc.bg_tile_map_display_select {
            BGMAP_SECOND_BEGIN_ADDRESS
        } else {
            BGMAP_FIRST_BEGIN_ADDRESS
        };

        addr + (y as u16 / 8 * 32) + (x as u16 / 8)
    }

    fn get_color_index(tile_data: u8, tile_color: u8, pixel_index: u8) -> u8 {
        (if tile_data & (1 << pixel_index) > 0 {
            1
        } else {
            0
        }) | (if tile_color & (1 << pixel_index) > 0 {
            1
        } else {
            0
        }) << 1
    }
}

impl Addressable for Gpu {
    fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;
        match address {
            VRAM_BEGIN..=VRAM_END => self.vram[address - VRAM_BEGIN],
            OAM_BEGIN..=OAM_END => self.oam[address - OAM_BEGIN],
            0xFF40 => self.lcdc.into(),
            0xFF41 => self.stat.into(),
            0xFF42 => self.sy,
            0xFF43 => self.sx,
            0xFF44 => 0x90,
            0xFF45 => self.lyc,
            0xFF46 => UNDEFINED,
            0xFF47 => self.bgp,
            0xFF48 => self.op0,
            0xFF49 => self.op1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!("[gpu] read: invalid address {:02x}", address),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;
        match address {
            VRAM_BEGIN..=VRAM_END => self.vram[address - VRAM_BEGIN] = value,
            OAM_BEGIN..=OAM_END => self.oam[address - OAM_BEGIN] = value,
            0xFF40 => self.lcdc = value.into(),
            0xFF41 => self.stat = value.into(),
            0xFF42 => self.sy = value,
            0xFF43 => self.sx = value,
            // Writing to this register resets the scanline
            0xFF44 => self.ly = 0,
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.op0 = value,
            0xFF49 => self.op1 = value,
            0xFF4A => self.wy = value,
            0xFF4B if value >= 7 => self.wx = value,
            _ => panic!("[gpu] write: invalid address {:02x}", address),
        }
    }
}
