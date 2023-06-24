pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

#[derive(Debug, Clone, Copy)]
pub struct GPU {
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub wy: u8,
    pub wx: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub dma: u8,
    pub bgp_palette: [u8; 4],
    pub obp0_palette: [u8; 4],
    pub obp1_palette: [u8; 4],
    pub mode: u8,
    pub modeclock: u8,
    pub line: u8,
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
    pub vblank_interrupt: bool,
    pub lcd_interrupt: bool,
    pub timer_interrupt: bool,
    pub serial_interrupt: bool,
    pub joypad_interrupt: bool,
}
