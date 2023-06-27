/// LCD Control Register
// Bit 7 - LCD Display Enable             (0=Off, 1=On)
// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
// Bit 5 - Window Display Enable          (0=Off, 1=On)
// Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
// Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
// Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
// Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
// Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
#[derive(Debug, Clone, Copy)]
pub struct LCDC {
    // CAUTION: Stopping LCD operation (Bit 7 from 1 to 0) may be performed during V-Blank ONLY,
    // disabeling the display outside of the V-Blank period may damage the hardware. This appears
    // to be a serious issue, Nintendo is reported to reject any games that do not follow this rule.
    // V-blank can be confirmed when the value of LY is greater than or equal to 144. When the display
    // is disabled the screen is blank (white), and VRAM and OAM can be accessed freely.
    pub lcd_enable: bool,
    pub window_tile_map_display_select: bool,
    pub window_display_enable: bool,
    pub bg_window_tile_data_select: bool,
    pub bg_tile_map_display_select: bool,
    pub sprite_size: bool,
    pub sprite_display_enable: bool,
    pub bg_display: bool,
}

impl LCDC {
    pub fn new() -> Self {
        Self {
            lcd_enable: false,
            window_tile_map_display_select: false,
            window_display_enable: false,
            bg_window_tile_data_select: false,
            bg_tile_map_display_select: false,
            sprite_size: false,
            sprite_display_enable: false,
            bg_display: false,
        }
    }
}

impl From<LCDC> for u8 {
    fn from(value: LCDC) -> Self {
        (value.lcd_enable as u8) << 7
            | (value.window_tile_map_display_select as u8) << 6
            | (value.window_display_enable as u8) << 5
            | (value.bg_window_tile_data_select as u8) << 4
            | (value.bg_tile_map_display_select as u8) << 3
            | (value.sprite_size as u8) << 2
            | (value.sprite_display_enable as u8) << 1
            | (value.bg_display as u8)
    }
}

impl From<u8> for LCDC {
    fn from(byte: u8) -> Self {
        let lcd_enable = ((byte >> 7) & 1) != 0;
        let window_tile_map_display_select = ((byte >> 6) & 1) != 0;
        let window_display_enable = ((byte >> 5) & 1) != 0;
        let bg_window_tile_data_select = ((byte >> 4) & 1) != 0;
        let bg_tile_map_display_select = ((byte >> 3) & 1) != 0;
        let sprite_size = ((byte >> 2) & 1) != 0;
        let sprite_display_enable = ((byte >> 1) & 1) != 0;
        let bg_display = (byte & 1) != 0;

        LCDC {
            lcd_enable,
            window_tile_map_display_select,
            window_display_enable,
            bg_window_tile_data_select,
            bg_tile_map_display_select,
            sprite_size,
            sprite_display_enable,
            bg_display,
        }
    }
}
