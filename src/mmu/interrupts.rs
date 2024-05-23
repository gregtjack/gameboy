use std::ops::{BitAnd, BitOrAssign};

pub enum InterruptFlag {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptVector {
    pub data: u8,
}

impl InterruptVector {
    pub fn new() -> Self {
        Self { data: 0x0 }
    }

    pub fn reset(&mut self) {
        self.data = 0x0;
    }

    pub fn set_flag(&mut self, flag: InterruptFlag) {
        self.data |= 1 << (flag as u8)
    }
}
