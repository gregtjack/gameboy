use crate::{
    addressable::Addressable,
    clock::{Clock, MAX_CYCLES_PER_FRAME},
    cpu::Cpu,
    gpu::{Color, Screen, Theme},
    joypad::Key,
};
use std::fmt::Write;

pub use crate::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Gameboy {
    cpu: Cpu,
    clock: Clock,
    serial_output: String,
    paused: bool,
}

impl Gameboy {
    pub fn new(debug: bool) -> Self {
        let cpu = Cpu::new(debug);

        Gameboy {
            cpu,
            clock: Clock::new(),
            serial_output: String::new(),
            paused: false,
        }
    }

    pub fn load_rom(&mut self, bin: Vec<u8>) {
        self.cpu.bus.load_rom(bin);
    }

    pub fn update(&mut self) {
        while self.clock.t <= MAX_CYCLES_PER_FRAME && !self.paused {
            let cycles = self.cpu.step();
            self.cpu.bus.step(cycles);
            self.clock.step(cycles);

            // Serial output
            if self.cpu.bus.sc == 0x81 {
                let c: char = self.cpu.bus.sb.into();
                write!(self.serial_output, "{}", c)
                    .expect("should be able to write into serial output");
                self.cpu.bus.write_byte(0xFF02, 0);
            }
        }

        self.clock.reset();
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn press_key(&mut self, key: Key) {
        self.cpu.bus.joypad.press(key);
    }

    pub fn release_key(&mut self, key: Key) {
        self.cpu.bus.joypad.release(key);
    }

    pub fn screen(&self) -> &Screen {
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
        use std::path::PathBuf;

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

        fn run_test_rom(name: &str) {
            let rom = load_rom(PathBuf::new().join("../test").join(name));
            let gameboy = Gameboy::new(false);
            let mut gb = gameboy;
            gb.load_rom(rom);

            // Run for enough cycles to complete the test
            for _ in 0..5000 {
                gb.update();
                if gb.read_serial_output().contains("Passed") {
                    return;
                }
            }

            panic!("Test rom timed out");
        }

        test_roms! {
            test_01_special: "01-special.gb",
            test_02_interrupts: "02-interrupts.gb",
            test_03_op_sp_hl: "03-op sp,hl.gb",
            test_04_op_r_imm: "04-op r,imm.gb",
            test_05_op_rp: "05-op rp.gb",
            test_06_ld_r_r: "06-ld r,r.gb",
            test_07_jr_jp_call_ret_rst: "07-jr,jp,call,ret,rst.gb",
            test_08_misc_instrs: "08-misc instrs.gb",
            test_09_op_r_r: "09-op r,r.gb",
            test_10_bit_ops: "10-bit ops.gb",
            test_11_op_a_hl: "11-op a,(hl).gb",
        }
    }
}
