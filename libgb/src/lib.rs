mod apu;
mod bus;
mod clock;
mod cpu;
mod gpu;
mod joypad;
mod timer;
mod utils;

pub mod gameboy;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    fn load_rom(path: PathBuf) -> Vec<u8> {
        return fs::read(path).expect("rom path should exist");
    }

    #[cfg(test)]
    pub mod blargg {
        use super::{gameboy, load_rom};

        const MAX_FRAMES: u32 = 5000;

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
            let gameboy = gameboy::Gameboy::new(false);
            let mut gb = gameboy;
            gb.load_rom(rom);

            // Run for enough cycles to complete the test
            let mut total_frames = 0;
            while total_frames < MAX_FRAMES {
                gb.update();
                if gb.read_serial_output().contains("Passed") {
                    return;
                }
                total_frames += 1;
            }

            panic!("timed out");
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
