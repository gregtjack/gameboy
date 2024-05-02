use crate::memory::interrupts::InterruptVector;

const DIVIDER_CYCLES: u32 = 256;
const SPEED_0_CYCLES: u32 = 1024;
const SPEED_1_CYCLES: u32 = 16;
const SPEED_2_CYCLES: u32 = 64;
const SPEED_3_CYCLES: u32 = 256;

#[derive(Debug)]
pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub int: InterruptVector,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            int: InterruptVector::new(),
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self.div = self.div.wrapping_add(1);
        if self.div == 0 {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima = self.tma;
                self.int.set_timer(true);
            }
        }
    }
}
