pub mod interrupts;
pub mod mmu;

// Helpful constants for GB memory blocks

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_LEN: usize = VRAM_END - VRAM_BEGIN + 1;

pub const OAM_BEGIN: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;

pub const INTERRUPT_FLAGS_ADDRESS: usize = 0xFF0F;
pub const INTERRUPT_ENABLE_ADDRESS: usize = 0xFFFF;

pub const UNDEFINED: u8 = 0xFF;

pub trait Memory {
    fn read8(&self, addr: u16) -> u8;

    fn write8(&mut self, addr: u16, value: u8);

    fn read16(&self, addr: u16) -> u16 {
        self.read8(addr) as u16 | (self.read8(addr + 1) as u16) << 8
    }

    fn write16(&mut self, addr: u16, value: u16) {
        self.write8(addr, (value & 0xFF00) as u8);
        self.write8(addr + 1, (value & 0x00FF) as u8);
    }
}
