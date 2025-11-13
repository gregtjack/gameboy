use crate::addressable::Addressable;

use super::color_id::ColorId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Grayscale,
    Green,
    PurpleYellow,
}

impl Theme {
    /// Returns RGBA colors for the four Game Boy color IDs (00, 01, 10, 11)
    fn colors(&self) -> [[u8; 4]; 4] {
        match self {
            Theme::Grayscale => [
                [255, 255, 255, 255], // White
                [192, 192, 192, 255], // LightGray
                [96, 96, 96, 255],    // DarkGray
                [0, 0, 0, 255],       // Black
            ],
            Theme::Green => [
                [155, 188, 15, 255], // Light green-yellow (lightest)
                [139, 172, 15, 255], // Medium-light green-yellow
                [48, 98, 48, 255],   // Medium-dark green
                [15, 56, 15, 255],   // Dark green (darkest)
            ],
            Theme::PurpleYellow => [
                [255, 255, 200, 255], // Light yellow (lightest)
                [200, 150, 100, 255], // Light orange/yellow
                [120, 80, 150, 255],  // Purple
                [60, 30, 80, 255],    // Dark purple (darkest)
            ],
        }
    }
}

#[derive(Debug)]
pub struct ThemeManager {
    current_theme: Theme,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: Theme::Grayscale,
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }

    pub fn get_theme(&self) -> Theme {
        self.current_theme
    }

    pub fn get_color_rgba(&self, color: Color) -> [u8; 4] {
        let colors = self.current_theme.colors();
        match color {
            Color::White => colors[0],
            Color::LightGray => colors[1],
            Color::DarkGray => colors[2],
            Color::Black => colors[3],
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Color {
    /// Returns RGBA using the default grayscale theme
    /// For theme-aware rendering, use ThemeManager::get_color_rgba instead
    pub fn rgba(&self) -> [u8; 4] {
        Theme::Grayscale.colors()[match self {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        }]
    }

    pub fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Color::White,
            0b01 => Color::LightGray,
            0b10 => Color::DarkGray,
            0b11 => Color::Black,
            _ => panic!("invalid color bits: {:#04b}", bits),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    id00: Color,
    id01: Color,
    id10: Color,
    id11: Color,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            id00: Color::White,
            id01: Color::LightGray,
            id10: Color::DarkGray,
            id11: Color::Black,
        }
    }

    pub fn get_color(&self, id: ColorId) -> Color {
        match id {
            ColorId::Id00 => self.id00,
            ColorId::Id01 => self.id01,
            ColorId::Id10 => self.id10,
            ColorId::Id11 => self.id11,
        }
    }
}

impl Addressable for Palette {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF47 => {
                let mut value = self.id00 as u8;
                value |= (self.id01 as u8) << 2;
                value |= (self.id10 as u8) << 4;
                value |= (self.id11 as u8) << 6;
                value
            }
            _ => panic!("invalid address for palette: {:#06x}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF47 => {
                self.id00 = Color::from_bits(value & 0b11);
                self.id01 = Color::from_bits((value >> 2) & 0b11);
                self.id10 = Color::from_bits((value >> 4) & 0b11);
                self.id11 = Color::from_bits((value >> 6) & 0b11);
            }
            _ => panic!("invalid address for palette: {:#06x}", addr),
        }
    }
}
