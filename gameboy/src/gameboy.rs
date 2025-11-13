use crate::{
    addressable::Addressable,
    clock::{Clock, MAX_CYCLES_PER_FRAME},
    cpu::Cpu,
    gpu::{Color, Screen, Theme},
    joypad::Key,
    mmu::ROM_LEN,
};
use std::fmt::Write;

pub use crate::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Gameboy {
    cpu: Cpu,
    clock: Clock,
    serial_output: String,
}

impl Gameboy {
    pub fn new(debug: bool) -> Self {
        let cpu = Cpu::new(debug);

        Gameboy {
            cpu,
            clock: Clock::new(),
            serial_output: String::new(),
        }
    }

    pub fn load_rom(&mut self, bin: Vec<u8>) {
        assert_eq!(bin.len(), ROM_LEN as usize, "size of game ROM is not valid");
        self.cpu.bus.rom[..bin.len()].copy_from_slice(&bin)
    }

    pub fn update(&mut self) {
        while self.clock.t <= MAX_CYCLES_PER_FRAME {
            let cycles = self.cpu.step();
            self.cpu.bus.step(cycles);
            self.clock.step(cycles);

            // Serial output
            if self.cpu.bus.read_byte(0xFF02) == 0x81 {
                let c: char = self.cpu.bus.read_byte(0xFF01).into();
                write!(self.serial_output, "{}", c)
                    .expect("should be able to write into serial output");
                self.cpu.bus.write_byte(0xFF02, 0);
            }
        }

        self.clock.reset();
    }

    pub fn press_key(&mut self, key: Key) {
        self.cpu.bus.joypad.press(key);
    }

    pub fn release_key(&mut self, key: Key) {
        self.cpu.bus.joypad.release(key);
    }

    pub fn frame(&self) -> Screen {
        self.cpu.bus.gpu.get_frame_buffer()
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.cpu.bus.gpu.set_theme(theme);
    }

    pub fn get_theme(&self) -> Theme {
        self.cpu.bus.gpu.get_theme()
    }

    pub fn get_color_rgba(&self, color: Color) -> [u8; 4] {
        self.cpu.bus.gpu.get_color_rgba(color)
    }

    pub fn read_serial_output(&self) -> &str {
        self.serial_output.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    fn load_rom(path: PathBuf) -> Vec<u8> {
        return fs::read(path).expect("rom path should exist");
    }

    #[cfg(test)]
    pub mod blargg {
        use super::{load_rom, Gameboy};

        macro_rules! test_roms {
            ($($label:ident: $name:expr,)*) => {
                $(
                    #[test]
                    fn $label() {
                        run_test_rom($name);
                    }
                )*
            }
        }

        fn run_test_rom(name: String) {
            let rom = load_rom(format!("../test/{}", name).into());
            let gameboy = Gameboy::new(false);
            let mut gb = gameboy;
            gb.load_rom(rom);

            // Run for enough cycles to complete the test. If the test doesn't complete
            // in 5000 frames, it's probably stuck in an infinite loop.
            for _ in 0..5000 {
                gb.update();
                if gb.read_serial_output().contains("Passed") {
                    return;
                }
            }

            panic!("Test rom timed out");
        }

        test_roms! {
            test_01: "01-special.gb".to_string(),
            test_02: "02-interrupts.gb".to_string(),
            test_03: "03-op sp,hl.gb".to_string(),
            test_04: "04-op r,imm.gb".to_string(),
            test_05: "05-op rp.gb".to_string(),
            test_06: "06-ld r,r.gb".to_string(),
            test_07: "07-jr,jp,call,ret,rst.gb".to_string(),
            test_08: "08-misc instrs.gb".to_string(),
            test_09: "09-op r,r.gb".to_string(),
            test_10: "10-bit ops.gb".to_string(),
            test_11: "11-op a,(hl).gb".to_string(),
        }
    }
}
