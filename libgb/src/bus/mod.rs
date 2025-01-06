use std::cell::RefCell;
use std::rc::Rc;

use crate::utils::addressable::Addressable;
use crate::{gpu::Gpu, timer::Timer};

pub mod interrupts;

use self::interrupts::Interrupts;

pub const UNDEFINED: u8 = 0xFF;

/// The Gameboy memory map
pub const BIOS_LEN: usize = 256;

pub const ROM_BEGIN: usize = 0x0000;
pub const ROM_END: usize = 0x7FFF;
pub const ROM_LEN: usize = ROM_END - ROM_BEGIN + 1;

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_LEN: usize = VRAM_END - VRAM_BEGIN + 1;

pub const ERAM_BEGIN: usize = 0xA000;
pub const ERAM_END: usize = 0xBFFF;
pub const ERAM_LEN: usize = ERAM_END - ERAM_BEGIN + 1;

pub const WRAM_BEGIN: usize = 0xC000;
pub const WRAM_END: usize = 0xDFFF;
pub const WRAM_LEN: usize = WRAM_END - WRAM_BEGIN + 1;

pub const ECHO_BEGIN: usize = 0xE000;
pub const ECHO_END: usize = 0xFDFF;
pub const ECHO_LEN: usize = ECHO_END - ECHO_BEGIN + 1;

pub const OAM_BEGIN: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;
pub const OAM_LEN: usize = OAM_END - OAM_BEGIN + 1;

pub const ZRAM_BEGIN: usize = 0xFF80;
pub const ZRAM_END: usize = 0xFFFE;
pub const ZRAM_LEN: usize = ZRAM_END - ZRAM_BEGIN + 1;

pub const INTERRUPT_FLAGS: usize = 0xFF0F;
pub const INTERRUPT_ENABLE: usize = 0xFFFF;

/// DMG Gameboy boot ROM
const BIOS: [u8; BIOS_LEN] = [
    0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26, 0xFF, 0x0E,
    0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0,
    0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
    0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06, 0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9,
    0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21, 0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20,
    0xF9, 0x2E, 0x0F, 0x18, 0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04,
    0x1E, 0x02, 0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
    0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64, 0x20, 0x06,
    0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15, 0x20, 0xD2, 0x05, 0x20,
    0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB, 0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17,
    0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9, 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
    0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,
    0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE, 0x34, 0x20,
    0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE, 0x3E, 0x01, 0xE0, 0x50,
];

/// Gameboy memory
#[derive(Debug)]
pub struct Bus {
    pub in_bios: bool,
    pub bios: [u8; BIOS_LEN],
    pub rom: [u8; ROM_LEN],
    pub wram: [u8; WRAM_LEN + ECHO_LEN],
    pub eram: [u8; ERAM_LEN],
    pub zram: [u8; ZRAM_LEN],
    /// Serial data
    pub sb: u8,
    /// Serial control
    pub sc: u8,
    pub joypad: u8,
    pub gpu: Gpu,
    pub intf: Rc<RefCell<Interrupts>>,
    pub inte: u8,
    pub timer: Timer,
}

impl Bus {
    pub fn new() -> Self {
        let intf = Rc::new(RefCell::new(Interrupts::new()));
        Self {
            gpu: Gpu::new(intf.clone()),
            timer: Timer::new(intf.clone()),
            intf: intf.clone(),
            inte: 0,
            bios: BIOS,
            in_bios: false,
            rom: [0; ROM_LEN],
            wram: [0; WRAM_LEN + ECHO_LEN],
            eram: [0; ERAM_LEN],
            zram: [0; ZRAM_LEN],
            sb: 0,
            sc: 0,
            joypad: 0,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self.gpu.step(cycles);
        self.timer.step(cycles);
    }
}

impl Addressable for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            ROM_BEGIN..=ROM_END => {
                if self.in_bios && addr < 0x0100 {
                    self.bios[addr]
                } else {
                    self.rom[addr]
                }
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.read_byte(addr as u16),
            ERAM_BEGIN..=ERAM_END => self.eram[addr - ERAM_BEGIN],
            WRAM_BEGIN..=ECHO_END => self.wram[addr - WRAM_BEGIN],
            OAM_BEGIN..=OAM_END => self.gpu.read_byte(addr as u16),
            0xFF00 => self.joypad,
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            0xFF04..=0xFF07 => self.timer.read_byte(addr as u16),
            // TODO: audio
            0xFF10..=0xFF26 => 0,
            0xFF30..=0xFF3F => 0,
            0xFF40..=0xFF4B => self.gpu.read_byte(addr as u16),
            ZRAM_BEGIN..=ZRAM_END => self.zram[addr - ZRAM_BEGIN],
            INTERRUPT_FLAGS => self.intf.borrow().data,
            INTERRUPT_ENABLE => self.inte,
            _ => panic!("[r] attempt to read invalid memory location {:02x}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        let addr = addr as usize;
        match addr {
            ROM_BEGIN..=ROM_END => {
                if self.in_bios && addr < 0x0100 {
                    self.bios[addr] = value;
                } else {
                    self.rom[addr] = value;
                }
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.write_byte(addr as u16, value),
            ERAM_BEGIN..=ERAM_END => self.eram[addr - ERAM_BEGIN] = value,
            WRAM_BEGIN..=ECHO_END => self.wram[addr - WRAM_BEGIN] = value,
            OAM_BEGIN..=OAM_END => self.gpu.write_byte(addr as u16, value),
            0xFF00 => self.joypad = value,
            0xFF01 => self.sb = value,
            0xFF02 => self.sc = value,
            0xFF04..=0xFF07 => self.timer.write_byte(addr as u16, value),
            // TODO: audio
            0xFF10..=0xFF26 => (),
            0xFF30..=0xFF3F => (),
            0xFF40..=0xFF45 => self.gpu.write_byte(addr as u16, value),
            0xFF46 => {
                // TODO: DMA transfer
            }
            0xFF47..=0xFF4B => self.gpu.write_byte(addr as u16, value),
            ZRAM_BEGIN..=ZRAM_END => self.zram[addr - ZRAM_BEGIN] = value,
            // Interrupts
            INTERRUPT_FLAGS => self.intf.borrow_mut().data = value,
            INTERRUPT_ENABLE => self.inte = value,
            _ => panic!("[w] attempt to write to invalid memory location {:02x}", addr),
        }
    }
}
