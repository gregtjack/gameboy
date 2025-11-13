use std::cell::RefCell;
use std::rc::Rc;

use crate::addressable::Addressable;
use crate::joypad::Joypad;
use crate::{gpu::Gpu, timer::Timer};

pub mod interrupts;

use self::interrupts::Interrupts;

pub const UNDEFINED: u8 = 0xFF;

/// The Gameboy memory map
pub const BIOS_LEN: u16 = 256;

pub const ROM_BEGIN: u16 = 0x0000;
pub const ROM_END: u16 = 0x7FFF;
pub const ROM_LEN: u16 = ROM_END - ROM_BEGIN + 1;

pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_LEN: u16 = VRAM_END - VRAM_BEGIN + 1;

pub const ERAM_BEGIN: u16 = 0xA000;
pub const ERAM_END: u16 = 0xBFFF;
pub const ERAM_LEN: u16 = ERAM_END - ERAM_BEGIN + 1;

pub const WRAM_BEGIN: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;
pub const WRAM_LEN: u16 = WRAM_END - WRAM_BEGIN + 1;

pub const ECHO_BEGIN: u16 = 0xE000;
pub const ECHO_END: u16 = 0xFDFF;
pub const ECHO_LEN: u16 = ECHO_END - ECHO_BEGIN + 1;

pub const OAM_BEGIN: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OAM_LEN: u16 = OAM_END - OAM_BEGIN + 1;

pub const ZRAM_BEGIN: u16 = 0xFF80;
pub const ZRAM_END: u16 = 0xFFFE;
pub const ZRAM_LEN: u16 = ZRAM_END - ZRAM_BEGIN + 1;

pub const INTERRUPT_FLAGS: u16 = 0xFF0F;
pub const INTERRUPT_ENABLE: u16 = 0xFFFF;

/// DMG Gameboy boot ROM
const BIOS: [u8; BIOS_LEN as usize] = [
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
pub struct Mmu {
    pub in_bios: bool,
    pub bios: [u8; BIOS_LEN as usize],
    pub rom: [u8; ROM_LEN as usize],
    pub wram: [u8; WRAM_LEN as usize + ECHO_LEN as usize],
    pub eram: [u8; ERAM_LEN as usize],
    pub zram: [u8; ZRAM_LEN as usize],
    /// Serial data
    pub sb: u8,
    /// Serial control
    pub sc: u8,
    pub joypad: Joypad,
    pub gpu: Gpu,
    pub intf: Rc<RefCell<Interrupts>>,
    pub inte: u8,
    pub timer: Timer,
    /// DMA transfer register
    pub dma: u8,
}

impl Mmu {
    pub fn new() -> Self {
        let intf = Rc::new(RefCell::new(Interrupts::new()));
        Self {
            gpu: Gpu::new(intf.clone()),
            timer: Timer::new(intf.clone()),
            joypad: Joypad::new(intf.clone()),
            intf: intf.clone(),
            inte: 0,
            bios: BIOS,
            in_bios: true,
            rom: [0; ROM_LEN as usize],
            wram: [0; (WRAM_LEN + ECHO_LEN) as usize],
            eram: [0; ERAM_LEN as usize],
            zram: [0; ZRAM_LEN as usize],
            sb: 0,
            sc: 0,
            dma: 0,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self.gpu.step(cycles);
        self.timer.step(cycles);
    }

    /// Perform DMA transfer from source memory to OAM
    /// Copies 160 bytes from XX00-XX9F to FE00-FE9F, where XX is the value in the DMA register
    fn dma_transfer(&mut self) {
        let source = (self.dma as u16) << 8;
        for i in 0..OAM_LEN {
            let byte = self.read_byte(source + i);
            self.gpu.write_byte(OAM_BEGIN + i, byte);
        }
    }
}

impl Addressable for Mmu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            ROM_BEGIN..=ROM_END => {
                if self.in_bios && addr < 0x0100 {
                    self.bios[addr as usize]
                } else {
                    self.rom[addr as usize]
                }
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.read_byte(addr as u16),
            ERAM_BEGIN..=ERAM_END => self.eram[(addr - ERAM_BEGIN) as usize],
            WRAM_BEGIN..=WRAM_END => self.wram[(addr - WRAM_BEGIN) as usize],
            ECHO_BEGIN..=ECHO_END => self.wram[(addr - ECHO_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.gpu.read_byte(addr as u16),
            0xFF00 => self.joypad.read_byte(addr),
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            0xFF04..=0xFF07 => self.timer.read_byte(addr as u16),
            // TODO: audio
            0xFF10..=0xFF26 => 0,
            0xFF30..=0xFF3F => 0,
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.gpu.read_byte(addr as u16),
            0xFF46 => self.dma,
            ZRAM_BEGIN..=ZRAM_END => self.zram[(addr - ZRAM_BEGIN) as usize],
            INTERRUPT_FLAGS => self.intf.borrow().data,
            INTERRUPT_ENABLE => self.inte,
            _ => UNDEFINED,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            ROM_BEGIN..=ROM_END => {
                if self.in_bios && addr < 0x0100 {
                    self.bios[addr as usize] = value;
                } else {
                    self.rom[addr as usize] = value;
                }
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.write_byte(addr as u16, value),
            ERAM_BEGIN..=ERAM_END => self.eram[(addr - ERAM_BEGIN) as usize] = value,
            WRAM_BEGIN..=WRAM_END => self.wram[(addr - WRAM_BEGIN) as usize] = value,
            ECHO_BEGIN..=ECHO_END => self.wram[(addr - ECHO_BEGIN) as usize] = value,
            OAM_BEGIN..=OAM_END => self.gpu.write_byte(addr as u16, value),
            0xFEA0..=0xFEFF => (),
            0xFF00 => self.joypad.write_byte(addr, value),
            0xFF01 => self.sb = value,
            0xFF02 => self.sc = value,
            0xFF04..=0xFF07 => self.timer.write_byte(addr as u16, value),
            // TODO: audio
            0xFF10..=0xFF26 => (),
            0xFF30..=0xFF3F => (),
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.gpu.write_byte(addr as u16, value),
            0xFF46 => {
                self.dma = value;
                self.dma_transfer();
            }
            0xFF50 => self.in_bios = false,
            0xFF70..=0xFF7F => {
                // WRAM bank switch
            }
            ZRAM_BEGIN..=ZRAM_END => self.zram[(addr - ZRAM_BEGIN) as usize] = value,
            // Interrupts
            INTERRUPT_FLAGS => self.intf.borrow_mut().data = value,
            INTERRUPT_ENABLE => self.inte = value,
            _ => (),
        }
    }
}
