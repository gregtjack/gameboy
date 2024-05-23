use std::{
    cell::{Ref, RefCell}, fs, io::{self, Write}, rc::Rc, time::Duration
};

use tracing::debug;

use crate::{
    clock::{Clock, MAX_CYCLES_PER_FRAME},
    cpu::Cpu,
    gpu::Screen,
    mmu::{Memory, Mmu},
};

pub struct Gameboy {
    cpu: Cpu,
    clock: Clock,
    debug: bool,
}

impl Gameboy {
    pub fn new(debug: bool) -> Self {
        let cpu = Cpu::new(debug);

        Gameboy {
            cpu,
            clock: Clock::new(),
            debug,
        }
    }

    pub fn load_rom(&mut self, bin: Vec<u8>) {
        debug!("rom length: {}", bin.len());
        self.cpu.bus.rom[..bin.len()].copy_from_slice(&bin)
    }

    pub fn update(&mut self) {
        while self.clock.t <= MAX_CYCLES_PER_FRAME {
            let cycles = self.cpu.step();
            self.cpu.bus.step(cycles);
            self.clock.step(cycles);

            // Blargg tests serial output
           
            // if self.cpu.bus.read8(0xFF02) == 0x81 {
            //     let c: char = self.cpu.bus.read8(0xFF01).into();
            //     print!("{}", c);
            //     self.cpu.bus.write8(0xFF02, 0);
            // }
            

        }

        self.clock.reset();
    }

    pub fn get_screen(&self) -> Screen {
        self.cpu.bus.gpu.screen
    }
}
