use crate::gpu::GPU;

use super::{Memory, VRAM_BEGIN, VRAM_END};

#[derive(Clone, Copy, Debug)]
pub struct MemBus {
    pub gpu: GPU,
    pub mem: [u8; 0xFFFF],
}

impl MemBus {
    pub fn new() -> Self {
        Self {
            mem: [0; 0xFFFF],
            gpu: GPU::new(),
        }
    }
}

impl Memory for MemBus {
    fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;
        match address {
            VRAM_BEGIN..=VRAM_END => self.gpu.read_byte((address - VRAM_BEGIN) as u16),
            _ => self.mem[address],
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;
        match address {
            VRAM_BEGIN..=VRAM_END => self.gpu.write_byte((address - VRAM_BEGIN) as u16, value),
            _ => self.mem[address] = value,
        }
    }
}
