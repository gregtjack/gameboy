use std::ops::BitOrAssign;

pub enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptVector {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl InterruptVector {
    pub fn new() -> Self {
        Self {
            vblank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }

    pub fn set_vblank(&mut self, vblank: bool) {
        self.vblank = vblank;
    }

    pub fn set_lcd_stat(&mut self, lcd_stat: bool) {
        self.lcd_stat = lcd_stat;
    }

    pub fn set_timer(&mut self, timer: bool) {
        self.timer = timer;
    }

    pub fn set_serial(&mut self, serial: bool) {
        self.serial = serial;
    }

    pub fn set_joypad(&mut self, joypad: bool) {
        self.joypad = joypad;
    }

    pub fn reset(&mut self) {
        self.vblank = false;
        self.lcd_stat = false;
        self.timer = false;
        self.serial = false;
        self.joypad = false;
    }

    pub fn is_zero(&self) -> bool {
        !(self.vblank || self.lcd_stat || self.timer || self.serial || self.joypad)
    }
}

impl From<InterruptVector> for u8 {
    fn from(value: InterruptVector) -> Self {
        (value.vblank as u8) << 0
            | (value.lcd_stat as u8) << 1
            | (value.timer as u8) << 2
            | (value.serial as u8) << 3
            | (value.joypad as u8) << 4
    }
}

impl From<u8> for InterruptVector {
    fn from(byte: u8) -> Self {
        let vblank = ((byte >> 0) & 1) != 0;
        let lcd_stat = ((byte >> 1) & 1) != 0;
        let timer = ((byte >> 2) & 1) != 0;
        let serial = ((byte >> 3) & 1) != 0;
        let joypad = ((byte >> 4) & 1) != 0;

        InterruptVector {
            vblank,
            lcd_stat,
            timer,
            serial,
            joypad,
        }
    }
}

impl BitOrAssign for InterruptVector {
    fn bitor_assign(&mut self, rhs: Self) {
        self.vblank |= rhs.vblank;
        self.lcd_stat |= rhs.lcd_stat;
        self.timer |= rhs.timer;
        self.serial |= rhs.serial;
        self.joypad |= rhs.joypad;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Interrupts {
    /// Interrupt flags
    pub int_r: InterruptVector,
    /// Interrupt enable
    pub int_e: InterruptVector,
}

impl Interrupts {
    pub fn new() -> Self {
        Interrupts {
            int_r: InterruptVector::new(),
            int_e: InterruptVector::new(),
        }
    }

    pub fn set_int_r(&mut self, int_r: InterruptVector) {
        self.int_r = int_r;
    }

    pub fn set_int_e(&mut self, int_e: InterruptVector) {
        self.int_e = int_e;
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.int_r.set_vblank(true),
            Interrupt::LCDStat => self.int_r.set_lcd_stat(true),
            Interrupt::Timer => self.int_r.set_timer(true),
            Interrupt::Serial => self.int_r.set_serial(true),
            Interrupt::Joypad => self.int_r.set_joypad(true),
        }
    }
}
