/// LCDC Status Register
// Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
// Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
// Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
// Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
// Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
// Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
#[derive(Debug, Clone, Copy)]
pub struct Stat {
    pub mode: Mode,
    pub coincidence_flag: bool,
    pub hblank_interrupt: bool,
    pub vblank_interrupt: bool,
    pub oam_interrupt: bool,
    pub coincidence_interrupt: bool,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            mode: Mode::HBlank,
            coincidence_flag: false,
            hblank_interrupt: false,
            vblank_interrupt: false,
            oam_interrupt: false,
            coincidence_interrupt: false,
        }
    }
}

impl From<Stat> for u8 {
    fn from(value: Stat) -> Self {
        let mut byte = 0;
        let mode: u8 = value.mode.into();
        byte |= mode;
        byte |= (value.coincidence_flag as u8) << 2;
        byte |= (value.hblank_interrupt as u8) << 3;
        byte |= (value.vblank_interrupt as u8) << 4;
        byte |= (value.oam_interrupt as u8) << 5;
        byte |= (value.coincidence_interrupt as u8) << 6;
        byte
    }
}

impl From<u8> for Stat {
    fn from(byte: u8) -> Self {
        Self {
            mode: Mode::from(byte & 0b11),
            coincidence_flag: ((byte >> 2) & 1) != 0,
            hblank_interrupt: ((byte >> 3) & 1) != 0,
            vblank_interrupt: ((byte >> 4) & 1) != 0,
            oam_interrupt: ((byte >> 5) & 1) != 0,
            coincidence_interrupt: ((byte >> 6) & 1) != 0,
        }
    }
}

// The Mode Flag goes through the values 0, 2, and 3 at a cycle of about 109uS.
// 0 is present about 48.6uS, 2 about 19uS, and 3 about 41uS. This is interrupted
// every 16.6ms by the VBlank (1). The mode flag stays set at 1 for about 1.08 ms.
//
// Mode 0 is present between 201-207 clks, 2 about 77-83 clks, and 3 about 169-175 clks.
// A complete cycle through these states takes 456 clks. VBlank lasts 4560 clks.
// A complete screen refresh occurs every 70224 clks.)
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Mode 0: The LCD controller is in the H-Blank period and
    ///         the CPU can access both the display RAM (8000h-9FFFh)
    ///         and OAM (FE00h-FE9Fh)
    HBlank,
    /// Mode 1: The LCD contoller is in the V-Blank period (or the
    ///         display is disabled) and the CPU can access both the
    ///         display RAM (8000h-9FFFh) and OAM (FE00h-FE9Fh).
    VBlank,
    /// Mode 2: The LCD controller is reading from OAM memory.
    ///         the CPU <cannot> access OAM memory (FE00h-FE9Fh)
    ///         during this period.
    OAM,
    /// Mode 3: The LCD controller is reading from both OAM and VRAM,
    ///         The CPU <cannot> access OAM and VRAM during this period.
    ///         CGB Mode: Cannot access Palette Data (FF69,FF6B) either.
    VRAM,
}

impl Mode {
    pub fn cycles(&self) -> u32 {
        match self {
            Mode::HBlank => 204,
            Mode::VBlank => 4560,
            Mode::OAM => 80,
            Mode::VRAM => 172,
        }
    }
}

impl From<Mode> for u8 {
    #[inline]
    fn from(mode: Mode) -> u8 {
        match mode {
            Mode::HBlank => 0,
            Mode::VBlank => 1,
            Mode::OAM => 2,
            Mode::VRAM => 3,
        }
    }
}

impl From<u8> for Mode {
    #[inline]
    fn from(byte: u8) -> Self {
        match byte {
            0 => Mode::HBlank,
            1 => Mode::VBlank,
            2 => Mode::OAM,
            3 => Mode::VRAM,
            _ => panic!("Invalid mode"),
        }
    }
}
