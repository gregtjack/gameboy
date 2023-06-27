pub mod mmu;

// Helpful constants for GB memory blocks

pub const BIOS_BEGIN: usize = 0x0000;
pub const BIOS_END: usize = 0x00FF;
pub const BIOS_LEN: usize = BIOS_END - BIOS_BEGIN + 1;

pub const ROM_BANK_0_BEGIN: usize = 0x0000;
pub const ROM_BANK_0_END: usize = 0x3FFF;
pub const ROM_BANK_0_LEN: usize = ROM_BANK_0_END - ROM_BANK_0_BEGIN + 1;

pub const ROM_BANK_1_BEGIN: usize = 0x4000;
pub const ROM_BANK_1_END: usize = 0x7FFF;
pub const ROM_BANK_1_LEN: usize = ROM_BANK_1_END - ROM_BANK_1_BEGIN + 1;

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_LEN: usize = VRAM_END - VRAM_BEGIN + 1;

pub const EXT_RAM_BEGIN: usize = 0xA000;
pub const EXT_RAM_END: usize = 0xBFFF;
pub const EXT_RAM_LEN: usize = EXT_RAM_END - EXT_RAM_BEGIN + 1;

pub const WORKING_RAM_BEGIN: usize = 0xC000;
pub const WORKING_RAM_END: usize = 0xDFFF;
pub const WORKING_RAM_LEN: usize = WORKING_RAM_END - WORKING_RAM_BEGIN + 1;

pub const ECHO_RAM_BEGIN: usize = 0xE000;
pub const ECHO_RAM_LEN: usize = 0xFDFF;

pub const OAM_BEGIN: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;
pub const OAM_LEN: usize = OAM_END - OAM_BEGIN + 1;

pub const UNUSED_BEGIN: usize = 0xFEA0;
pub const UNUSED_END: usize = 0xFEFF;

pub const IO_REGISTERS_BEGIN: usize = 0xFF00;
pub const IO_REGISTERS_END: usize = 0xFF7F;

pub const ZERO_PAGE_BEGIN: usize = 0xFF80;
pub const ZERO_PAGE_END: usize = 0xFFFE;
pub const ZERO_PAGE_LEN: usize = ZERO_PAGE_END - ZERO_PAGE_BEGIN + 1;

pub trait Memory {
    fn read_byte(&self, addr: u16) -> u8;

    fn write_byte(&mut self, addr: u16, value: u8);

    fn read_word(&self, addr: u16) -> u16 {
        self.read_byte(addr) as u16 | (self.read_byte(addr + 1) as u16) << 8
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, (value & 0xFF00) as u8);
        self.write_byte(addr + 1, (value & 0x00FF) as u8);
    }
}
