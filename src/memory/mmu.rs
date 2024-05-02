use super::{
    interrupts::Interrupts, Memory, INTERRUPT_ENABLE_ADDRESS, INTERRUPT_FLAGS_ADDRESS, OAM_BEGIN,
    OAM_END, VRAM_BEGIN, VRAM_END,
};
use crate::{gpu::Gpu, timer::Timer};

#[derive(Debug)]
pub struct Mmu {
    pub mem: [u8; 0xFFFF],
    pub gpu: Gpu,
    pub int: Interrupts,
    pub timer: Timer,
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            mem: [0; 0xFFFF],
            gpu: Gpu::new(),
            timer: Timer::new(),
            int: Interrupts::new(),
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self.gpu.step(cycles);
        self.timer.step(cycles);
        self.int.int_r |= self.timer.int;
        self.int.int_r |= self.gpu.int;
        self.gpu.int.reset();
        self.timer.int.reset();
    }
}

impl Memory for Mmu {
    fn read8(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            VRAM_BEGIN..=VRAM_END => self.gpu.read8(addr as u16),
            OAM_BEGIN..=OAM_END => self.gpu.read8(addr as u16),
            0xFF40..=0xFF4B => self.gpu.read8(addr as u16),
            0xFF04 => self.timer.div,
            0xFF05 => self.timer.tima,
            0xFF06 => self.timer.tma,
            0xFF07 => self.timer.tac,
            // Interrupts
            INTERRUPT_FLAGS_ADDRESS => self.int.int_r.into(),
            INTERRUPT_ENABLE_ADDRESS => self.int.int_e.into(),
            _ => self.mem[addr],
        }
    }

    fn write8(&mut self, addr: u16, value: u8) {
        let addr = addr as usize;
        match addr {
            VRAM_BEGIN..=VRAM_END => self.gpu.write8(addr as u16, value),
            OAM_BEGIN..=OAM_END => self.gpu.write8(addr as u16, value),
            0xFF40..=0xFF45 => self.gpu.write8(addr as u16, value),
            0xFF47..=0xFF4B => self.gpu.write8(addr as u16, value),
            0xFF46 => {
                // DMA transfer
                let start_addr = (value as u16) << 8;
                for i in 0..0xA0 {
                    let byte = self.read8(start_addr + i);
                    self.write8(0xFE00 + i, byte);
                }
            }
            0xFF04 => self.timer.div = 0,
            0xFF05 => self.timer.tima = value,
            0xFF06 => self.timer.tma = value,
            0xFF07 => self.timer.tac = value,
            // Interrupts
            INTERRUPT_FLAGS_ADDRESS => self.int.set_int_r(value.into()),
            INTERRUPT_ENABLE_ADDRESS => self.int.set_int_e(value.into()),
            _ => self.mem[addr] = value,
        }
    }
}
