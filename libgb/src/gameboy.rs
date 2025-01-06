use crate::{
    bus::ROM_LEN,
    clock::{Clock, MAX_CYCLES_PER_FRAME},
    cpu::Cpu,
    gpu::Screen,
    utils::addressable::Addressable,
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
        assert_eq!(bin.len(), ROM_LEN, "size of game ROM is not valid");
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

    pub fn frame(&self) -> Screen {
        self.cpu.bus.gpu.get_frame_buffer()
    }

    pub fn read_serial_output(&self) -> String {
        self.serial_output.clone()
    }
}
