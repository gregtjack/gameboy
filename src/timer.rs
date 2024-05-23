use std::{cell::RefCell, rc::Rc};

use crate::mmu::{interrupts::{InterruptFlag, InterruptVector}, Memory};

const DIVIDER_CYCLES: u32 = 256;
const SPEED_0_CYCLES: u32 = 1024;
const SPEED_1_CYCLES: u32 = 16;
const SPEED_2_CYCLES: u32 = 64;
const SPEED_3_CYCLES: u32 = 256;

#[derive(Debug, Clone, Copy)]
pub struct TimerControl {
    pub speed: Speed,
    pub running: bool,
}

impl TimerControl {
    pub fn new() -> Self {
        Self { speed: Speed::T4, running: false }
    }
}

impl From<TimerControl> for u8 {
    fn from(value: TimerControl) -> Self {
        let speed: u8 = match value.speed {
            Speed::T4 => 0x0,
            Speed::T256 => 0x1,
            Speed::T64 => 0x2,
            Speed::T16 => 0x3,
        };
        speed | ((value.running as u8) << 2)
    }
}

impl From<u8> for TimerControl {
    fn from(value: u8) -> Self {
        let speed = match value & 0x3 {
            0x0 => Speed::T4,
            0x1 => Speed::T256,
            0x2 => Speed::T64,
            0x3 => Speed::T16,
            _ => panic!("[tac] invalid speed")
        };
        let running = (value & 0x4) != 0;
        Self { speed, running }
    }
}

/// Possible divider values usable as timer clock source.
#[derive(Clone,Copy,Debug)]
pub enum Speed {
    /// Divide sysclk by 1024. Timer clock is 4.096kHz
    T4,
    /// Divide sysclk by 16. Timer clock is 262.144kHz
    T256,
    /// Divide sysclk by 64. Timer clock is 65.536kHz
    T64,
    /// Divide sysclk by 256. Timer clock is 16.384kHz
    T16,
}

#[derive(Debug)]
struct TimerClock {
    pub main: u32,
    pub sub: u32,
    pub div: u32,
}
#[derive(Debug)]
pub struct Timer {
    _clock: TimerClock,
    // Timer registers
    /// divider
    pub div: u8,
    /// timer counter
    pub tima: u8,
    /// timer modulo
    pub tma: u8,
    /// timer control
    pub tac: TimerControl,
    /// interrupt state
    pub int: Rc<RefCell<InterruptVector>>,
}

impl Timer {
    pub fn new(intf: Rc<RefCell<InterruptVector>>) -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: TimerControl::new(),
            int: intf,
            _clock: TimerClock { main: 0, sub: 0, div: 0 }
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self._clock.sub += cycles / 4;
        
        if self._clock.sub >= 4 {
            self._clock.main += 1;
            self._clock.sub -= 4;

            self._clock.div += 1;
            if self._clock.div == 16 {
                self.div = self.div.wrapping_add(1);
                self._clock.div = 0;
            }
        }

        self.check();
    }

    fn check(&mut self) {
        if self.tac.running {
            let threshold: u32 = match self.tac.speed {
                Speed::T4 => 64,
                Speed::T256 => 1,
                Speed::T64 => 4,
                Speed::T16 => 16,
            };

            if self._clock.main >= threshold {
                self._clock.main = 0;
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0x0 {
                    self.tima = self.tma;
                    self.int.borrow_mut().set_flag(InterruptFlag::Timer)
                }
            }
        }
    }
}

impl Memory for Timer {
    fn read8(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac.into(),
            _ => panic!("[timer] read: Invalid timer address {:02x}", addr)
        }
    }

    fn write8(&mut self, addr: u16, value: u8) {
        let addr = addr as usize;
        match addr {
            0xFF04 => self.div = 0x0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value.into(),
            _ => panic!("[timer] write: Invalid timer address {:02x}", addr)
        }
    }
}