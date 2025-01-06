/// LCD Control Register
#[derive(Debug, Clone, Copy)]
pub struct Lcdc {
    /// Bit 7 - LCD Display Enable             (0=Off, 1=On)
    pub lcd_enable: bool,
    /// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    pub window_tile_map_display_select: bool,
    /// Bit 5 - Window Display Enable          (0=Off, 1=On)
    pub window_display_enable: bool,
    /// Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    pub bg_window_tile_data_select: bool,
    /// Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    pub bg_tile_map_display_select: bool,
    /// Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    pub obj_size: bool,
    /// Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    pub obj_display_enable: bool,
    /// Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
    pub bg_display: bool,
}

impl Lcdc {
    pub fn new() -> Self {
        Self {
            lcd_enable: false,
            window_tile_map_display_select: true,
            window_display_enable: false,
            bg_window_tile_data_select: false,
            bg_tile_map_display_select: true,
            obj_size: false,
            obj_display_enable: false,
            bg_display: false,
        }
    }

    pub fn lcd_enabled(&self) -> bool {
        self.lcd_enable
    }
}

impl From<Lcdc> for u8 {
    fn from(value: Lcdc) -> Self {
        (value.lcd_enable as u8) << 7
            | (value.window_tile_map_display_select as u8) << 6
            | (value.window_display_enable as u8) << 5
            | (value.bg_window_tile_data_select as u8) << 4
            | (value.bg_tile_map_display_select as u8) << 3
            | (value.obj_size as u8) << 2
            | (value.obj_display_enable as u8) << 1
            | (value.bg_display as u8)
    }
}

impl From<u8> for Lcdc {
    fn from(byte: u8) -> Self {
        Lcdc {
            lcd_enable: ((byte >> 7) & 1) != 0,
            window_tile_map_display_select: ((byte >> 6) & 1) != 0,
            window_display_enable: ((byte >> 5) & 1) != 0,
            bg_window_tile_data_select: ((byte >> 4) & 1) != 0,
            bg_tile_map_display_select: ((byte >> 3) & 1) != 0,
            obj_size: ((byte >> 2) & 1) != 0,
            obj_display_enable: ((byte >> 1) & 1) != 0,
            bg_display: (byte & 1) != 0,
        }
    }
}
