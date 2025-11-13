use std::{cell::RefCell, rc::Rc};

use oam::{Oam, Sprite};
use palette::Palette;
use tile_data::{TileData, VRAM_TILE_DATA_BEGIN, VRAM_TILE_DATA_END};
use tile_map::TileMap;

use crate::{
    addressable::Addressable,
    mmu::{
        interrupts::{InterruptFlag, Interrupts},
        OAM_BEGIN, OAM_END, UNDEFINED,
    },
};

use self::{
    lcdc::Lcdc,
    stat::{Mode, Stat},
};

mod color_id;
mod lcdc;
mod oam;
mod palette;
mod stat;
mod tile_data;
mod tile_map;

pub use palette::{Color, Theme, ThemeManager};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const CYCLES_OAM: u32 = 80;
const CYCLES_VRAM: u32 = 172;
const CYCLES_HBLANK: u32 = 204;
const CYCLES_VBLANK: u32 = CYCLES_OAM + CYCLES_VRAM + CYCLES_HBLANK;

const SCANLINES_DISPLAY: u8 = 143;
const MAX_SCANLINES: u8 = 153;

pub type Screen = [[palette::Color; SCREEN_HEIGHT]; SCREEN_WIDTH];

#[derive(Debug)]
pub struct Gpu {
    // Digital image with RGB. Size = 144 * 160 * 3.
    frame_buffer: Screen,
    tile_data: TileData,
    tile_map: TileMap,
    lcdc: Lcdc,
    stat: Stat,
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
    bgp: Palette,
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
    oam: Oam,
    clock: u32,
    theme_manager: ThemeManager,
}

impl Gpu {
    pub fn new(intf: Rc<RefCell<Interrupts>>) -> Self {
        let stat = Stat::new();
        let image = [[palette::Color::White; SCREEN_HEIGHT]; SCREEN_WIDTH];

        Self {
            tile_data: TileData::new(),
            tile_map: TileMap::new(),
            frame_buffer: image,
            int: intf,
            // LCD starts disabled - BIOS will enable it
            lcdc: Lcdc::from(0x00),
            stat,
            sx: 0,
            sy: 0,
            ly: 0,
            lyc: 0,
            bgp: Palette::new(),
            op0: 0xFF,
            op1: 0xFF,
            wx: 0,
            wy: 0,
            oam: Oam::new(),
            clock: 0,
            theme_manager: ThemeManager::new(),
        }
    }

    pub fn step(&mut self, cycles: u32) {
        if !self.lcdc.lcd_enable {
            // When LCD is disabled, reset GPU state
            if !matches!(self.stat.mode, Mode::VBlank) || self.ly != 0 {
                self.ly = 0;
                self.clock = 0;
                self.stat.mode = Mode::VBlank;
            }
            return;
        }

        self.clock += cycles;

        match self.stat.mode {
            Mode::OAM => {
                if self.clock >= CYCLES_OAM {
                    self.clock %= CYCLES_OAM;
                    self.set_mode(Mode::VRAM);
                }
            }
            Mode::VRAM => {
                if self.clock >= CYCLES_VRAM {
                    self.clock %= CYCLES_VRAM;
                    if self.stat.hblank_interrupt {
                        self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                    }
                    self.render_line();
                    self.set_mode(Mode::HBlank);
                }
            }
            Mode::HBlank => {
                if self.clock >= CYCLES_HBLANK {
                    self.clock %= CYCLES_HBLANK;
                    self.set_scanline(self.ly + 1);

                    if self.ly > SCANLINES_DISPLAY {
                        self.set_mode(Mode::VBlank);
                        // render screen
                        if self.stat.vblank_interrupt {
                            self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                        }
                        self.int.borrow_mut().set_flag(InterruptFlag::VBlank);
                    } else {
                        self.set_mode(Mode::OAM);
                        if self.stat.oam_interrupt {
                            self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                        }
                    }
                }
            }
            Mode::VBlank => {
                if self.clock >= CYCLES_VBLANK {
                    self.clock %= CYCLES_VBLANK;
                    self.set_scanline(self.ly + 1);
                    if self.ly > MAX_SCANLINES {
                        self.set_scanline(0);
                        self.set_mode(Mode::OAM);
                        if self.stat.oam_interrupt {
                            self.int.borrow_mut().set_flag(InterruptFlag::LCDStat);
                        }
                    }
                }
            }
        }
    }

    pub fn get_frame_buffer(&self) -> Screen {
        self.frame_buffer
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme_manager.set_theme(theme);
    }

    pub fn get_theme(&self) -> Theme {
        self.theme_manager.get_theme()
    }

    pub fn get_color_rgba(&self, color: palette::Color) -> [u8; 4] {
        self.theme_manager.get_color_rgba(color)
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

    fn render_line(&mut self) {
        // Always render background first (even if bg_display is false, we still need to fill the screen)
        // When bg_display is false, the background should show as white/transparent
        if self.lcdc.bg_display {
            self.render_bg_line();
            self.render_window_line();
        }

        if self.lcdc.obj_display_enable {
            self.render_obj_line();
        }
    }

    fn render_bg_line(&mut self) {
        let y = self.ly.wrapping_add(self.sy);
        let bg_tile_map_area = if self.lcdc.bg_tile_map_display_select {
            tile_map::Area::Area1
        } else {
            tile_map::Area::Area0
        };

        let tile_data_area = if self.lcdc.bg_window_tile_data_select {
            tile_data::Area::Area1
        } else {
            tile_data::Area::Area0
        };

        let row_in_tile = y % 8;
        let mut lx = 0;

        while lx < SCREEN_WIDTH {
            let x = (lx as u8).wrapping_add(self.sx);

            let col_in_tile = x % 8;

            let tile_index = self
                .tile_map
                .get_tile_index(bg_tile_map_area, x as usize, y as usize);
            let tile_pixels =
                self.tile_data
                    .get_row_pixels(tile_data_area, tile_index, row_in_tile as usize);

            let length = if col_in_tile > 0 {
                8 - (col_in_tile as usize)
            } else if SCREEN_WIDTH - lx < 8 {
                SCREEN_WIDTH - lx
            } else {
                8
            };

            for i in 0..length {
                let color_id = tile_pixels[col_in_tile as usize + i];
                let color = self.bgp.get_color(color_id);
                self.frame_buffer[lx + i][self.ly as usize] = color;
            }

            lx += length;
        }
    }

    fn render_window_line(&mut self) {
        if !self.lcdc.window_display_enable {
            return;
        }

        // Window Y position check - window is only visible when LY >= WY
        if self.ly < self.wy {
            return;
        }

        // Window X position - window starts at WX - 7 (WX=7 means window starts at x=0)
        let window_start_x = if self.wx >= 7 {
            (self.wx as usize) - 7
        } else {
            0
        };

        if window_start_x >= SCREEN_WIDTH {
            return;
        }

        let window_tile_map_area = if self.lcdc.window_tile_map_display_select {
            tile_map::Area::Area1
        } else {
            tile_map::Area::Area0
        };

        let tile_data_area = if self.lcdc.bg_window_tile_data_select {
            tile_data::Area::Area1
        } else {
            tile_data::Area::Area0
        };

        // Window-relative Y coordinate (line within window)
        let window_y = self.ly - self.wy;
        let row_in_tile = window_y % 8;

        let mut lx = window_start_x;
        while lx < SCREEN_WIDTH {
            // Window-relative X coordinate
            let window_x = lx - window_start_x;
            let col_in_tile = window_x % 8;

            let tile_index =
                self.tile_map
                    .get_tile_index(window_tile_map_area, window_x, window_y as usize);
            let tile_pixels =
                self.tile_data
                    .get_row_pixels(tile_data_area, tile_index, row_in_tile as usize);

            let length = if col_in_tile > 0 {
                8 - (col_in_tile as usize)
            } else if SCREEN_WIDTH - lx < 8 {
                SCREEN_WIDTH - lx
            } else {
                8
            };

            for i in 0..length {
                let color_id = tile_pixels[col_in_tile as usize + i];
                let color = self.bgp.get_color(color_id);
                self.frame_buffer[lx + i][self.ly as usize] = color;
            }

            lx += length;
        }
    }

    fn render_obj_line(&mut self) {
        let sprites = self.oam.get_sprites();
        let sprite_height = if self.lcdc.obj_size { 16 } else { 8 };

        // Collect sprites visible on this scanline (max 10)
        // Sprites with y_pos >= 160 or x_pos >= 168 are not visible
        let mut visible_sprites: Vec<(usize, &Sprite)> = Vec::new();
        for (idx, sprite) in sprites.iter().enumerate() {
            // Skip sprites that are off-screen
            if sprite.y_pos >= 160 || sprite.x_pos >= 168 {
                continue;
            }

            let sprite_y = sprite.y_pos as i16;
            let sprite_top = sprite_y;
            let sprite_bottom = sprite_y + sprite_height as i16;

            // Check if this scanline intersects with the sprite
            let ly_i16 = self.ly as i16;
            if ly_i16 >= sprite_top && ly_i16 < sprite_bottom {
                visible_sprites.push((idx, sprite));
            }

            if visible_sprites.len() >= 10 {
                break;
            }
        }

        // Render sprites from right to left (lower OAM index = higher priority)
        for (_, sprite) in visible_sprites.iter().rev() {
            let sprite_y = sprite.y_pos as i16;
            let sprite_x = sprite.x_pos as i16;
            let sprite_top = sprite_y;

            let sprite_row = (self.ly as i16) - sprite_top;
            let tile_row = if sprite.y_flip {
                (sprite_height - 1) as i16 - sprite_row
            } else {
                sprite_row
            };

            let tile_index = if sprite_height == 16 {
                if tile_row < 8 {
                    sprite.tile_index & 0xFE
                } else {
                    (sprite.tile_index & 0xFE) + 1
                }
            } else {
                sprite.tile_index
            };

            let actual_tile_row = if sprite_height == 16 && tile_row >= 8 {
                (tile_row - 8) as usize
            } else {
                tile_row as usize
            };

            let mut tile_pixels = self.tile_data.get_row(tile_index as u16, actual_tile_row);

            if sprite.x_flip {
                tile_pixels.reverse();
            }

            let palette = if sprite.palette { self.op1 } else { self.op0 };

            for (col, color_id) in tile_pixels.iter().enumerate() {
                if matches!(color_id, color_id::ColorId::Id00) {
                    continue;
                }

                let screen_x = sprite_x + col as i16;
                if screen_x < 0 || screen_x >= SCREEN_WIDTH as i16 {
                    continue;
                }

                let screen_x = screen_x as usize;

                if sprite.priority {
                    let bg_color_id = {
                        let bg_y = self.ly.wrapping_add(self.sy);
                        let bg_x = (screen_x as u8).wrapping_add(self.sx);
                        let bg_tile_map_area = if self.lcdc.bg_tile_map_display_select {
                            tile_map::Area::Area1
                        } else {
                            tile_map::Area::Area0
                        };
                        let bg_tile_index = self.tile_map.get_tile_index(
                            bg_tile_map_area,
                            bg_x as usize,
                            bg_y as usize,
                        );
                        let bg_tile_data_area = if self.lcdc.bg_window_tile_data_select {
                            tile_data::Area::Area1
                        } else {
                            tile_data::Area::Area0
                        };
                        let bg_row_in_tile = bg_y % 8;
                        let bg_col_in_tile = bg_x % 8;
                        let bg_tile_pixels = self.tile_data.get_row_pixels(
                            bg_tile_data_area,
                            bg_tile_index,
                            bg_row_in_tile as usize,
                        );
                        bg_tile_pixels[bg_col_in_tile as usize]
                    };

                    if !matches!(bg_color_id, color_id::ColorId::Id00) {
                        continue;
                    }
                }

                let palette_bits = match color_id {
                    color_id::ColorId::Id00 => palette & 0b11,
                    color_id::ColorId::Id01 => (palette >> 2) & 0b11,
                    color_id::ColorId::Id10 => (palette >> 4) & 0b11,
                    color_id::ColorId::Id11 => (palette >> 6) & 0b11,
                };

                let color = palette::Color::from_bits(palette_bits);
                self.frame_buffer[screen_x][self.ly as usize] = color;
            }
        }
    }
}

impl Addressable for Gpu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // VRAM not accessable during pixel transfer
            VRAM_TILE_DATA_BEGIN..=VRAM_TILE_DATA_END => match self.stat.mode {
                Mode::OAM | Mode::HBlank | Mode::VBlank => self.tile_data.read_byte(addr),
                Mode::VRAM => UNDEFINED,
            },
            // tile maps are not accessable during pixel transfer
            0x9800..=0x9FFF => match self.stat.mode {
                Mode::OAM | Mode::HBlank | Mode::VBlank => self.tile_map.read_byte(addr),
                Mode::VRAM => UNDEFINED,
            },
            // VRAM is not accessable during pixel transfer and OAM search
            OAM_BEGIN..=OAM_END => match self.stat.mode {
                Mode::HBlank | Mode::VBlank => self.oam.read_byte(addr),
                Mode::VRAM | Mode::OAM => UNDEFINED,
            },
            0xFF40 => self.lcdc.into(),
            0xFF41 => self.stat.into(),
            0xFF42 => self.sy,
            0xFF43 => self.sx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => UNDEFINED,
            0xFF47 => self.bgp.read_byte(addr),
            0xFF48 => self.op0,
            0xFF49 => self.op1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!("[gpu:read] invalid address {:#06x}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // VRAM not accessable during pixel transfer
            VRAM_TILE_DATA_BEGIN..=VRAM_TILE_DATA_END => match self.stat.mode {
                Mode::OAM | Mode::HBlank | Mode::VBlank => self.tile_data.write_byte(addr, value),
                Mode::VRAM => {}
            },
            0x9800..=0x9FFF => match self.stat.mode {
                Mode::OAM | Mode::HBlank | Mode::VBlank => self.tile_map.write_byte(addr, value),
                Mode::VRAM => {}
            },
            // VRAM is not accessable during pixel transfer and OAM search
            OAM_BEGIN..=OAM_END => match self.stat.mode {
                Mode::HBlank | Mode::VBlank => self.oam.write_byte(addr, value),
                Mode::VRAM | Mode::OAM => {}
            },
            0xFF40 => {
                let was_enabled = self.lcdc.lcd_enable;
                self.lcdc = value.into();
                // When LCD is turned on, reset GPU state
                if !was_enabled && self.lcdc.lcd_enable {
                    self.ly = 0;
                    self.clock = 0;
                    self.stat.mode = Mode::OAM;
                }
            }
            0xFF41 => self.stat = value.into(),
            0xFF42 => self.sy = value,
            0xFF43 => self.sx = value,
            0xFF44 => {} // LY is read-only
            0xFF45 => self.lyc = value,
            0xFF46 => panic!("DMA implemented in MMU"),
            0xFF47 => self.bgp.write_byte(addr, value),
            0xFF48 => self.op0 = value,
            0xFF49 => self.op1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => panic!("[gpu:write] invalid address ${:#06x}", addr),
        }
    }
}
