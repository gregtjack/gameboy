use self::registers::{Flags, Registers};
use crate::{addressable::Addressable, mmu::Mmu};

mod registers;

macro_rules! adc {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let c = $cpu.reg.f.carry as u8;
        let (new_value, overflow1) = $cpu.reg.$reg.overflowing_add($value);
        let (new_value, overflow2) = new_value.overflowing_add(c);
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(false);
        $cpu.reg
            .f
            .set_h(($cpu.reg.$reg & 0xf) + ($value & 0xf) + c > 0xf);
        $cpu.reg.f.set_c(overflow1 || overflow2);
        $cpu.reg.$reg = new_value;
    };
}

macro_rules! sub {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let (new_value, overflow) = $cpu.reg.$reg.overflowing_sub($value);
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(true);
        $cpu.reg.f.set_h(($cpu.reg.$reg & 0xf) < ($value & 0xf));
        $cpu.reg.f.set_c(overflow);
        $cpu.reg.$reg = new_value;
    };
}

macro_rules! sbc {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let c = $cpu.reg.f.carry as u8;
        let (new_value, overflow1) = $cpu.reg.$reg.overflowing_sub($value);
        let (new_value, overflow2) = new_value.overflowing_sub(c);
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(true);
        $cpu.reg.f.set_h(($cpu.reg.$reg & 0xf) < ($value & 0xf) + c);
        $cpu.reg.f.set_c(overflow1 || overflow2);
        $cpu.reg.$reg = new_value;
    };
}

macro_rules! and {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let new_value = $cpu.reg.$reg & $value;
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(false);
        $cpu.reg.f.set_h(true);
        $cpu.reg.f.set_c(false);
        $cpu.reg.$reg = new_value;
    };
}

macro_rules! or {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let new_value = $cpu.reg.$reg | $value;
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(false);
        $cpu.reg.f.set_h(false);
        $cpu.reg.f.set_c(false);
        $cpu.reg.$reg = new_value;
    };
}

macro_rules! xor {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let new_value = $cpu.reg.$reg ^ $value;
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(false);
        $cpu.reg.f.set_h(false);
        $cpu.reg.f.set_c(false);
        $cpu.reg.$reg = new_value;
    };
}

macro_rules! cp {
    ($cpu:expr, $reg:ident, $value:expr) => {
        let (new_value, overflow) = $cpu.reg.$reg.overflowing_sub($value);
        $cpu.reg.f.set_z(new_value == 0x0);
        $cpu.reg.f.set_n(true);
        $cpu.reg.f.set_h(($cpu.reg.$reg & 0xf) < ($value & 0xf));
        $cpu.reg.f.set_c(overflow);
    };
}

macro_rules! inc {
    ($cpu:expr, $reg:ident) => {
        let result = $cpu.reg.$reg.wrapping_add(1);
        $cpu.reg.f.set_z(result == 0x0);
        $cpu.reg.f.set_n(false);
        $cpu.reg.f.set_h(($cpu.reg.$reg & 0xf) + 1 > 0xf);
        $cpu.reg.$reg = result;
    };
}

macro_rules! dec {
    ($cpu:expr, $reg:ident) => {
        let result = $cpu.reg.$reg.wrapping_sub(1);
        $cpu.reg.f.set_z(result == 0x0);
        $cpu.reg.f.set_n(true);
        $cpu.reg.f.set_h(($cpu.reg.$reg & 0xf) < 1);
        $cpu.reg.$reg = result;
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuErr {
    InvalidInstruction(u8),
}

impl std::fmt::Display for CpuErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuErr::InvalidInstruction(instruction) => {
                write!(f, "invalid CPU instruction: {:02x}", instruction)
            }
        }
    }
}

/// Implementation of the Sharp LR35902
#[derive(Debug)]
pub struct Cpu {
    reg: Registers,
    pub bus: Mmu,
    pub halted: bool,
    ime: bool,
    ime_next: bool,
    debug: bool,
}

impl Cpu {
    pub fn new(debug: bool) -> Self {
        Self {
            reg: Registers {
                pc: 0,
                sp: 0,
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
                f: Flags::zero(),
            },
            bus: Mmu::new(),
            halted: false,
            ime: true,
            ime_next: false,
            debug,
        }
    }

    pub fn step(&mut self) -> u32 {
        // execute next instruction
        // Returns the new program counter and t_cycles used
        let opcode = self.bus.read_byte(self.reg.pc);
        let mut cycles = if !self.halted {
            if self.debug {
                self.print_debug()
            }
            match self.execute(opcode) {
                Ok((pc, cycles)) => {
                    self.reg.pc = pc;
                    cycles
                }
                Err(err) => panic!("{}", err),
            }
        } else {
            4
        };

        if let Some((pc, c)) = self.handle_interrupts() {
            self.reg.pc = pc;
            cycles += c;
        }

        cycles
    }

    fn print_debug(&self) {
        println!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.reg.a,
            u8::from(self.reg.f),
            self.reg.b,
            self.reg.c,
            self.reg.d,
            self.reg.e,
            self.reg.h,
            self.reg.l,
            self.reg.sp,
            self.reg.pc,
            self.bus.read_byte(self.reg.pc),
            self.bus.read_byte(self.reg.pc + 1),
            self.bus.read_byte(self.reg.pc + 2),
            self.bus.read_byte(self.reg.pc + 3)
        );
    }

    pub fn handle_interrupts(&mut self) -> Option<(u16, u32)> {
        let intf = self.bus.read_byte(0xFF0F);
        let inte = self.bus.read_byte(0xFFFF);
        let interrupt = intf & inte;
        if interrupt != 0 {
            self.halted = false;

            if self.ime {
                let n = interrupt.trailing_zeros();
                self.bus.write_byte(0xFF0F, intf & !(1 << n));

                self.push(self.reg.pc);

                self.ime = false;
                self.ime_next = false;
                return Some((0x0040 | ((n as u16) << 3), 20));
            }
        }

        if self.ime_next {
            self.ime = true;
            self.ime_next = false;
        }

        None
    }

    fn execute(&mut self, instruction: u8) -> Result<(u16, u32), CpuErr> {
        match instruction {
            /* CPU control */
            0x00 => Ok((self.reg.pc.wrapping_add(1), 4)), // NOP
            0x10 => Ok((self.reg.pc.wrapping_add(1), 4)), // STOP
            0x76 => {
                self.halted = true;
                Ok((self.reg.pc.wrapping_add(1), 4))
            } // HALT

            /* 8-bit load */
            0x40 => Ok((self.reg.pc.wrapping_add(1), 4)), // LD B, B
            0x41 => {
                // LD B, C
                self.reg.b = self.reg.c;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x42 => {
                // LD B, D
                self.reg.b = self.reg.d;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x43 => {
                // LD B, E
                self.reg.b = self.reg.e;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x44 => {
                // LD B, H
                self.reg.b = self.reg.h;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x45 => {
                // LD B, L
                self.reg.b = self.reg.l;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x46 => {
                // LD B, (HL)
                self.reg.b = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x47 => {
                // LD B, A
                self.reg.b = self.reg.a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x06 => {
                // LD B, n
                self.reg.b = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x48 => {
                // LD C, B
                self.reg.c = self.reg.b;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x49 => {
                // LD C, C
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x4A => {
                // LD C, D
                self.reg.c = self.reg.d;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x4B => {
                // LD C, E
                self.reg.c = self.reg.e;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x4C => {
                // LD C, H
                self.reg.c = self.reg.h;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x4D => {
                // LD C, L
                self.reg.c = self.reg.l;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x4E => {
                // LD C, (HL)
                self.reg.c = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x4F => {
                // LD C, A
                self.reg.c = self.reg.a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x50 => {
                // LD D, B
                self.reg.d = self.reg.b;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x51 => {
                // LD D, C
                self.reg.d = self.reg.c;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x52 => {
                // LD D, D
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x53 => {
                // LD D, E
                self.reg.d = self.reg.e;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x54 => {
                // LD D, H
                self.reg.d = self.reg.h;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x55 => {
                // LD D, B
                self.reg.d = self.reg.l;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x56 => {
                // LD D, (HL)
                self.reg.d = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x57 => {
                // LD D, A
                self.reg.d = self.reg.a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x58 => {
                // LD E, B
                self.reg.e = self.reg.b;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x59 => {
                // LD E, C
                self.reg.e = self.reg.c;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x5A => {
                // LD E, D
                self.reg.e = self.reg.d;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x5B => {
                // LD E, E
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x5C => {
                // LD E, H
                self.reg.e = self.reg.h;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x5D => {
                // LD E, L
                self.reg.e = self.reg.l;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x5E => {
                // LD E, (HL)
                self.reg.e = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x5F => {
                // LD E, A
                self.reg.e = self.reg.a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x60 => {
                // LD H, B
                self.reg.h = self.reg.b;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x61 => {
                // LD H, C
                self.reg.h = self.reg.c;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x62 => {
                // LD H, D
                self.reg.h = self.reg.d;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x63 => {
                // LD H, E
                self.reg.h = self.reg.e;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x64 => {
                // LD H, H
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x65 => {
                // LD H, L
                self.reg.h = self.reg.l;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x66 => {
                // LD H, (HL)
                self.reg.h = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x67 => {
                // LD H, A
                self.reg.h = self.reg.a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x68 => {
                // LD L, B
                self.reg.l = self.reg.b;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x69 => {
                // LD L, C
                self.reg.l = self.reg.c;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x6A => {
                // LD L, D
                self.reg.l = self.reg.d;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x6B => {
                // LD L, E
                self.reg.l = self.reg.e;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x6C => {
                // LD L, H
                self.reg.l = self.reg.h;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x6D => {
                // LD L, L
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x6E => {
                // LD L, (HL)
                self.reg.l = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x6F => {
                // LD L, A
                self.reg.l = self.reg.a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x70 => {
                // LD (HL), B
                self.bus.write_byte(self.reg.hl(), self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x71 => {
                // LD (HL), C
                self.bus.write_byte(self.reg.hl(), self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x72 => {
                // LD (HL), D
                self.bus.write_byte(self.reg.hl(), self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x73 => {
                // LD (HL), E
                self.bus.write_byte(self.reg.hl(), self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x74 => {
                // LD (HL), H
                self.bus.write_byte(self.reg.hl(), self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x75 => {
                // LD (HL), L
                self.bus.write_byte(self.reg.hl(), self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x77 => {
                // LD (HL), A
                self.bus.write_byte(self.reg.hl(), self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x78 => {
                // LD A, B
                self.reg.a = self.reg.b;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x79 => {
                // LD A, C
                self.reg.a = self.reg.c;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x7A => {
                // LD A, D
                self.reg.a = self.reg.d;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x7B => {
                // LD A, E
                self.reg.a = self.reg.e;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x7C => {
                // LD A, H
                self.reg.a = self.reg.h;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x7D => {
                // LD A, L
                self.reg.a = self.reg.l;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x7E => {
                // LD A, (HL)
                self.reg.a = self.bus.read_byte(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x7F => {
                // LD A, A
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x02 => {
                // LD (BC), A
                self.bus.write_byte(self.reg.bc(), self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x12 => {
                // LD (DE), A
                self.bus.write_byte(self.reg.de(), self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x22 => {
                // LDi (HL+), A
                self.bus.write_byte(self.reg.hl(), self.reg.a);
                self.reg.set_hl(self.reg.hl().wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x32 => {
                // LDd (HL-), A
                self.bus.write_byte(self.reg.hl(), self.reg.a);
                self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x0A => {
                // LD A, (BC)
                self.reg.a = self.bus.read_byte(self.reg.bc());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x1A => {
                // LD A, (DE)
                self.reg.a = self.bus.read_byte(self.reg.de());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x2A => {
                // LD A, (HL+)
                self.reg.a = self.bus.read_byte(self.reg.hl());
                self.reg.set_hl(self.reg.hl().wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x3A => {
                // LD A, (HL-)
                self.reg.a = self.bus.read_byte(self.reg.hl());
                self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x0E => {
                // LD C, n
                self.reg.c = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x16 => {
                // LD D, n
                self.reg.d = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x1E => {
                // LD E, n
                self.reg.e = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x26 => {
                // LD H, n
                self.reg.h = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x27 => {
                // DAA
                // Borrowed from mvdnes/rboy
                let mut a = self.reg.a;
                let mut adjust = if self.reg.f.carry { 0x60 } else { 0x00 };

                if self.reg.f.half_carry {
                    adjust |= 0x06;
                }

                if !self.reg.f.subtract {
                    if a & 0x0F > 0x09 {
                        adjust |= 0x06;
                    };
                    if a > 0x99 {
                        adjust |= 0x60;
                    };
                    a = a.wrapping_add(adjust);
                } else {
                    a = a.wrapping_sub(adjust);
                }

                self.reg.f.carry = adjust >= 0x60;
                self.reg.f.half_carry = false;
                self.reg.f.zero = a == 0;
                self.reg.a = a;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x2E => {
                // LD L, n
                self.reg.l = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x36 => {
                // LD (HL), n
                let n = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                self.bus.write_byte(self.reg.hl(), n);
                Ok((self.reg.pc.wrapping_add(2), 12))
            }
            0x3E => {
                // LD A, n
                self.reg.a = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE0 => {
                // LDH (n), A
                let n = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as u16;
                self.bus.write_byte(0xFF00 | n, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 12))
            }
            0xF0 => {
                // LDH A, (n)
                let n = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as u16;
                self.reg.a = self.bus.read_byte(0xFF00 | n);
                Ok((self.reg.pc.wrapping_add(2), 12))
            }
            0xE2 => {
                // LD (C), A
                self.bus
                    .write_byte(0xFF00 | (self.reg.c as u16), self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xF2 => {
                // LD A, (C)
                self.reg.a = self.bus.read_byte(0xFF00 | (self.reg.c as u16));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xEA => {
                // LD (nn), A
                let nn = self.bus.read_word(self.reg.pc.wrapping_add(1));
                self.bus.write_byte(nn, self.reg.a);
                Ok((self.reg.pc.wrapping_add(3), 16))
            }
            0xFA => {
                // LD A, (nn)
                let nn = self.bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.a = self.bus.read_byte(nn);
                Ok((self.reg.pc.wrapping_add(3), 16))
            }
            /* 16-bit load */
            0x01 => {
                // LD BC, nn
                self.reg
                    .set_bc(self.bus.read_word(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(3), 12))
            }
            0x11 => {
                // LD DE, nn
                self.reg
                    .set_de(self.bus.read_word(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(3), 12))
            }
            0x21 => {
                // LD HL, nn
                self.reg
                    .set_hl(self.bus.read_word(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(3), 12))
            }
            0x31 => {
                // LD SP, nn
                self.reg.sp = self.bus.read_word(self.reg.pc.wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(3), 12))
            }
            0xF9 => {
                // LD SP, HL
                self.reg.sp = self.reg.hl();
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xF8 => {
                // LD HL, SP+n
                let n = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8 as i16 as u16;
                let result = self.reg.sp.wrapping_add(n);
                self.reg.f.set_z(false);
                self.reg.f.set_n(false);
                self.reg
                    .f
                    .set_h((self.reg.sp & 0x000f) + (n & 0x000f) > 0x000f);
                self.reg
                    .f
                    .set_c((self.reg.sp & 0x00ff) + (n & 0x00ff) > 0x00ff);
                self.reg.set_hl(result);
                Ok((self.reg.pc.wrapping_add(2), 12))
            }
            0x08 => {
                // LD (nn), SP
                let nn = self.bus.read_word(self.reg.pc.wrapping_add(1));
                self.bus.write_word(nn, self.reg.sp);
                Ok((self.reg.pc.wrapping_add(3), 20))
            }
            0xF5 => {
                // PUSH AF
                self.push(self.reg.af());
                Ok((self.reg.pc.wrapping_add(1), 16))
            }
            0xC5 => {
                // PUSH BC
                self.push(self.reg.bc());
                Ok((self.reg.pc.wrapping_add(1), 16))
            }
            0xD5 => {
                // PUSH DE
                self.push(self.reg.de());
                Ok((self.reg.pc.wrapping_add(1), 16))
            }
            0xE5 => {
                // PUSH HL
                self.push(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 16))
            }
            0xF1 => {
                // POP AF
                let n = self.pop();
                self.reg.set_af(n);
                Ok((self.reg.pc.wrapping_add(1), 12))
            }
            0xC1 => {
                // POP BC
                let n = self.pop();
                self.reg.set_bc(n);
                Ok((self.reg.pc.wrapping_add(1), 12))
            }
            0xD1 => {
                // POP DE
                let n = self.pop();
                self.reg.set_de(n);
                Ok((self.reg.pc.wrapping_add(1), 12))
            }
            0xE1 => {
                // POP HL
                let n = self.pop();
                self.reg.set_hl(n);
                Ok((self.reg.pc.wrapping_add(1), 12))
            }

            /* 8-bit arithmetic/logical */
            0x80 => {
                // ADD A, B
                self.add(self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x81 => {
                // ADD A, C
                self.add(self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x82 => {
                // ADD A, D
                self.add(self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x83 => {
                // ADD A, E
                self.add(self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x84 => {
                // ADD A, H
                self.add(self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x85 => {
                // ADD A, L
                self.add(self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x86 => {
                // ADD A, (HL)
                let v = self.bus.read_byte(self.reg.hl());
                self.add(v);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x87 => {
                // ADD A, A
                self.add(self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xC6 => {
                // ADD A, n
                let n = self.bus.read_byte(self.reg.pc.wrapping_add(1));
                self.add(n);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x88 => {
                // ADC A, B
                adc!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x89 => {
                // ADC A, C
                adc!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x8A => {
                // ADC A, D
                adc!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x8B => {
                // ADC A, E
                adc!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x8C => {
                // ADC A, H
                adc!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x8D => {
                // ADC A, L
                adc!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x8E => {
                // ADC A, (HL)
                adc!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x8F => {
                // ADC A, A
                adc!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xCE => {
                // ADC A, n
                adc!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x90 => {
                // SUB A, B
                sub!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x91 => {
                // SUB A, C
                sub!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x92 => {
                // SUB A, D
                sub!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x93 => {
                // SUB A, E
                sub!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x94 => {
                // SUB A, H
                sub!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x95 => {
                // SUB A, L
                sub!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x96 => {
                // SUB A, (HL)
                sub!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x97 => {
                // SUB A, A
                sub!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xD6 => {
                // SUB A, n
                sub!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x98 => {
                // SBC A, B
                sbc!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x99 => {
                // SBC A, C
                sbc!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x9A => {
                // SBC A, D
                sbc!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x9B => {
                // SBC A, E
                sbc!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x9C => {
                // SBC A, H
                sbc!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x9D => {
                // SBC A, L
                sbc!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x9E => {
                // SBC A, (HL)
                sbc!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x9F => {
                // SBC A, A
                sbc!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xDE => {
                // SBC A, n
                sbc!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA0 => {
                // AND A, B
                and!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA1 => {
                // AND A, C
                and!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA2 => {
                // AND A, D
                and!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA3 => {
                // AND A, E
                and!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA4 => {
                // AND A, H
                and!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA5 => {
                // AND A, L
                and!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA6 => {
                // AND A, (HL)
                and!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xA7 => {
                // AND A, A
                and!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xE6 => {
                // AND A, n
                and!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA8 => {
                // XOR A, B
                xor!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xA9 => {
                // XOR A, C
                xor!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xAA => {
                // XOR A, D
                xor!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xAB => {
                // XOR A, E
                xor!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xAC => {
                // XOR A, H
                xor!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xAD => {
                // XOR A, L
                xor!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xAE => {
                // XOR A, (HL)
                xor!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xAF => {
                // XOR A, A
                xor!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xEE => {
                // XOR A, n
                xor!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB0 => {
                // OR A, B
                or!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB1 => {
                // OR A, C
                or!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB2 => {
                // OR A, D
                or!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB3 => {
                // OR A, E
                or!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB4 => {
                // OR A, H
                or!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB5 => {
                // OR A, L
                or!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB6 => {
                // OR A, (HL)
                or!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xB7 => {
                // OR A, A
                or!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xF6 => {
                // OR A, n
                or!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB8 => {
                // CP A, B
                cp!(self, a, self.reg.b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xB9 => {
                // CP A, C
                cp!(self, a, self.reg.c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xBA => {
                // CP A, D
                cp!(self, a, self.reg.d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xBB => {
                // CP A, E
                cp!(self, a, self.reg.e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xBC => {
                // CP A, H
                cp!(self, a, self.reg.h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xBD => {
                // CP A, L
                cp!(self, a, self.reg.l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xBE => {
                // CP A, (HL)
                cp!(self, a, self.bus.read_byte(self.reg.hl()));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xBF => {
                // CP A, A
                cp!(self, a, self.reg.a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xFE => {
                // CP A, n
                cp!(self, a, self.bus.read_byte(self.reg.pc.wrapping_add(1)));
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x04 => {
                // INC B
                inc!(self, b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x0C => {
                // INC C
                inc!(self, c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x14 => {
                // INC D
                inc!(self, d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x1C => {
                // INC E
                inc!(self, e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x24 => {
                // INC H
                inc!(self, h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x2C => {
                // INC L
                inc!(self, l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x34 => {
                // INC (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let result = value.wrapping_add(1);
                self.reg.f.set_z(result == 0);
                self.reg.f.set_n(false);
                self.reg.f.set_h((value & 0xf) + 1 > 0xf);
                self.bus.write_byte(self.reg.hl(), result);
                Ok((self.reg.pc.wrapping_add(1), 12))
            }
            0x3C => {
                // INC A
                inc!(self, a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x05 => {
                // DEC B
                dec!(self, b);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x0D => {
                // DEC C
                dec!(self, c);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x15 => {
                // DEC D
                dec!(self, d);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x1D => {
                // DEC E
                dec!(self, e);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x25 => {
                // DEC H
                dec!(self, h);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x2D => {
                // DEC L0
                dec!(self, l);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x35 => {
                // DEC (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let result = value.wrapping_sub(1);
                self.reg.f.set_z(result == 0);
                self.reg.f.set_n(true);
                self.reg.f.set_h((value & 0xf) < 1);
                self.bus.write_byte(self.reg.hl(), result);
                Ok((self.reg.pc.wrapping_add(1), 12))
            }
            0x3D => {
                // DEC A
                dec!(self, a);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }

            /* 16-bit arithmetic/logical */
            0x09 => {
                // ADD HL, BC
                self.add16(self.reg.bc());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x19 => {
                // ADD HL, DE
                self.add16(self.reg.de());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x29 => {
                // ADD HL, HL
                self.add16(self.reg.hl());
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x39 => {
                // ADD HL, SP
                self.add16(self.reg.sp);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0xE8 => {
                // ADD SP, s8
                let s = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8 as i16 as u16;
                let result = self.reg.sp.wrapping_add(s);
                self.reg.f.set_z(false);
                self.reg.f.set_n(false);
                self.reg
                    .f
                    .set_h((self.reg.sp & 0x000F) + (s & 0x000F) > 0x000F);
                self.reg
                    .f
                    .set_c((self.reg.sp & 0x00FF) + (s & 0x00FF) > 0x00FF);
                self.reg.sp = result;
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x03 => {
                // INC BC
                self.reg.set_bc(self.reg.bc().wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x13 => {
                // INC DE
                self.reg.set_de(self.reg.de().wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x23 => {
                // INC HL
                self.reg.set_hl(self.reg.hl().wrapping_add(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x33 => {
                // INC SP
                self.reg.sp = self.reg.sp.wrapping_add(1);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x0B => {
                // DEC BC
                self.reg.set_bc(self.reg.bc().wrapping_sub(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x1B => {
                // DEC DE
                self.reg.set_de(self.reg.de().wrapping_sub(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x2B => {
                // DEC HL
                self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                Ok((self.reg.pc.wrapping_add(1), 8))
            }
            0x3B => {
                // DEC SP
                self.reg.sp = self.reg.sp.wrapping_sub(1);
                Ok((self.reg.pc.wrapping_add(1), 8))
            }

            /* Rotate/shift */
            0x07 => {
                // RLCA
                let bit7 = self.reg.a & 0x80 != 0;
                self.reg.a = (self.reg.a << 1) | (self.reg.a >> 7);
                self.reg.f.set_z(false);
                self.reg.f.set_n(false);
                self.reg.f.set_h(false);
                self.reg.f.set_c(bit7);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x17 => {
                // RLA
                let bit7 = self.reg.a & 0x80 != 0;
                self.reg.a = (self.reg.a << 1) | (self.reg.f.carry as u8);
                self.reg.f.set_z(false);
                self.reg.f.set_n(false);
                self.reg.f.set_h(false);
                self.reg.f.set_c(bit7);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x0F => {
                // RRCA
                let bit0 = self.reg.a & 0x01 != 0;
                self.reg.a = (self.reg.a >> 1) | (self.reg.a << 7);
                self.reg.f.set_z(false);
                self.reg.f.set_n(false);
                self.reg.f.set_h(false);
                self.reg.f.set_c(bit0);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x1F => {
                // RRA
                let bit0 = self.reg.a & 0x01 != 0;
                self.reg.a = (self.reg.a >> 1) | ((self.reg.f.carry as u8) << 7);
                self.reg.f.set_z(false);
                self.reg.f.set_n(false);
                self.reg.f.set_h(false);
                self.reg.f.set_c(bit0);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }

            /* Jumps */
            0xC3 => {
                // JP nn
                Ok((self.bus.read_word(self.reg.pc.wrapping_add(1)), 16))
            }
            0xC2 => {
                // JP NZ, nn
                if !self.reg.f.zero {
                    Ok((self.bus.read_word(self.reg.pc.wrapping_add(1)), 16))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xCA => {
                // JP Z, nn
                if self.reg.f.zero {
                    Ok((self.bus.read_word(self.reg.pc.wrapping_add(1)), 16))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xD2 => {
                // JP NC, nn
                if !self.reg.f.carry {
                    Ok((self.bus.read_word(self.reg.pc.wrapping_add(1)), 16))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xDA => {
                // JP C, nn
                if self.reg.f.carry {
                    Ok((self.bus.read_word(self.reg.pc.wrapping_add(1)), 16))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xE9 => {
                // JP HL
                Ok((self.reg.hl(), 4))
            }
            0x18 => {
                // JR s8
                let s = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8;
                Ok((((self.reg.pc + 2) as i32 + (s as i32)) as u16, 12))
            }
            0x20 => {
                // JR NZ, s8
                if !self.reg.f.zero {
                    let s = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8;
                    Ok((((self.reg.pc + 2) as i32 + (s as i32)) as u16, 12))
                } else {
                    Ok((self.reg.pc.wrapping_add(2), 8))
                }
            }
            0x28 => {
                // JR Z, s8
                if self.reg.f.zero {
                    let s = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8;
                    Ok((((self.reg.pc + 2) as i32 + (s as i32)) as u16, 12))
                } else {
                    Ok((self.reg.pc.wrapping_add(2), 8))
                }
            }
            0x30 => {
                // JR NC, s8
                if !self.reg.f.carry {
                    let s = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8;
                    Ok((((self.reg.pc + 2) as i32 + (s as i32)) as u16, 12))
                } else {
                    Ok((self.reg.pc.wrapping_add(2), 8))
                }
            }
            0x38 => {
                // JR C, s8
                if self.reg.f.carry {
                    let s = self.bus.read_byte(self.reg.pc.wrapping_add(1)) as i8;
                    Ok((((self.reg.pc + 2) as i32 + (s as i32)) as u16, 12))
                } else {
                    Ok((self.reg.pc.wrapping_add(2), 8))
                }
            }

            /* Calls */
            0xCD => {
                // CALL nn
                let addr = self.bus.read_word(self.reg.pc.wrapping_add(1));
                self.push(self.reg.pc.wrapping_add(3));
                Ok((addr, 24))
            }
            0xC4 => {
                // CALL NZ, nn
                if !self.reg.f.zero {
                    let addr = self.bus.read_word(self.reg.pc.wrapping_add(1));
                    self.push(self.reg.pc.wrapping_add(3));
                    Ok((addr, 24))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xCC => {
                // CALL Z, nn
                if self.reg.f.zero {
                    let addr = self.bus.read_word(self.reg.pc.wrapping_add(1));
                    self.push(self.reg.pc.wrapping_add(3));
                    Ok((addr, 24))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xD4 => {
                // CALL NC, nn
                if !self.reg.f.carry {
                    let addr = self.bus.read_word(self.reg.pc.wrapping_add(1));
                    self.push(self.reg.pc.wrapping_add(3));
                    Ok((addr, 24))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }
            0xDC => {
                // CALL C, nn
                if self.reg.f.carry {
                    let addr = self.bus.read_word(self.reg.pc.wrapping_add(1));
                    self.push(self.reg.pc.wrapping_add(3));
                    Ok((addr, 24))
                } else {
                    Ok((self.reg.pc.wrapping_add(3), 12))
                }
            }

            /* Restarts */
            0xC7 => {
                // RST 00H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x00, 16))
            }
            0xCF => {
                // RST 08H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x08, 16))
            }
            0xD7 => {
                // RST 10H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x10, 16))
            }
            0xDF => {
                // RST 18H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x18, 16))
            }
            0xE7 => {
                // RST 20H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x20, 16))
            }
            0xEF => {
                // RST 28H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x28, 16))
            }
            0xF7 => {
                // RST 30H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x30, 16))
            }
            0xFF => {
                // RST 38H
                self.push(self.reg.pc.wrapping_add(1));
                Ok((0x38, 16))
            }

            /* Returns */
            0xC9 => {
                // RET
                Ok((self.pop(), 16))
            }
            0xC0 => {
                // RET NZ
                if !self.reg.f.zero {
                    Ok((self.pop(), 20))
                } else {
                    Ok((self.reg.pc.wrapping_add(1), 8))
                }
            }
            0xC8 => {
                // RET Z
                if self.reg.f.zero {
                    Ok((self.pop(), 20))
                } else {
                    Ok((self.reg.pc.wrapping_add(1), 8))
                }
            }
            0xD0 => {
                // RET NC
                if !self.reg.f.carry {
                    Ok((self.pop(), 20))
                } else {
                    Ok((self.reg.pc.wrapping_add(1), 8))
                }
            }
            0xD8 => {
                // RET C
                if self.reg.f.carry {
                    Ok((self.pop(), 20))
                } else {
                    Ok((self.reg.pc.wrapping_add(1), 8))
                }
            }
            0xD9 => {
                // RETI
                self.ime = true;
                Ok((self.pop(), 13))
            }

            /* Interrupts */
            0xF3 => {
                // DI
                self.ime = false;
                self.ime_next = false;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0xFB => {
                // EI
                self.ime_next = true;
                Ok((self.reg.pc.wrapping_add(1), 4))
            }

            /* Misc */
            0x2F => {
                // CPL
                self.reg.a = !self.reg.a;
                self.reg.f.set_n(true);
                self.reg.f.set_h(true);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x3F => {
                // CCF
                self.reg.f.set_c(!self.reg.f.carry);
                self.reg.f.set_h(false);
                self.reg.f.set_n(false);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }
            0x37 => {
                // SCF
                self.reg.f.set_c(true);
                self.reg.f.set_h(false);
                self.reg.f.set_n(false);
                Ok((self.reg.pc.wrapping_add(1), 4))
            }

            /* CB prefixed opcodes */
            0xCB => self.execute_prefixed(self.bus.read_byte(self.reg.pc.wrapping_add(1))),

            _ => Err(CpuErr::InvalidInstruction(instruction)),
        }
    }

    fn execute_prefixed(&mut self, instruction: u8) -> Result<(u16, u32), CpuErr> {
        match instruction {
            0x00 => {
                // RLC B
                let new_value = self.rlc(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x01 => {
                // RLC C
                let new_value = self.rlc(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x02 => {
                // RLC D
                let new_value = self.rlc(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x03 => {
                // RLC E
                let new_value = self.rlc(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x04 => {
                // RLC H
                let new_value = self.rlc(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x05 => {
                // RLC L
                let new_value = self.rlc(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x06 => {
                // RLC (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.rlc(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x07 => {
                // RLC A
                let new_value = self.rlc(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x08 => {
                // RRC B
                let new_value = self.rrc(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x09 => {
                // RRC C
                let new_value = self.rrc(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x0A => {
                // RRC D
                let new_value = self.rrc(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x0B => {
                // RRC E
                let new_value = self.rrc(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x0C => {
                // RRC H
                let new_value = self.rrc(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x0D => {
                // RRC L
                let new_value = self.rrc(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x0E => {
                // RRC (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.rrc(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x0F => {
                // RRC A
                let new_value = self.rrc(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x10 => {
                // RL B
                let new_value = self.rl(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x11 => {
                // RL C
                let new_value = self.rl(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x12 => {
                // RL D
                let new_value = self.rl(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x13 => {
                // RL E
                let new_value = self.rl(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x14 => {
                // RL H
                let new_value = self.rl(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x15 => {
                // RL L
                let new_value = self.rl(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x16 => {
                // RL (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.rl(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x17 => {
                // RL A
                let new_value = self.rl(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x18 => {
                // RR B
                let new_value = self.rr(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x19 => {
                // RR C
                let new_value = self.rr(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x1A => {
                // RR D
                let new_value = self.rr(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x1B => {
                // RR E
                let new_value = self.rr(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x1C => {
                // RR H
                let new_value = self.rr(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x1D => {
                // RR L
                let new_value = self.rr(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x1E => {
                // RR (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.rr(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x1F => {
                // RR A
                let new_value = self.rr(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x20 => {
                // SLA B
                let new_value = self.sla(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x21 => {
                // SLA C
                let new_value = self.sla(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x22 => {
                // SLA D
                let new_value = self.sla(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x23 => {
                // SLA E
                let new_value = self.sla(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x24 => {
                // SLA H
                let new_value = self.sla(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x25 => {
                // SLA L
                let new_value = self.sla(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x26 => {
                // SLA (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.sla(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x27 => {
                // SLA A
                let new_value = self.sla(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x28 => {
                // SRA B
                let new_value = self.sra(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x29 => {
                // SRA C
                let new_value = self.sra(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x2A => {
                // SRA D
                let new_value = self.sra(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x2B => {
                // SRA E
                let new_value = self.sra(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x2C => {
                // SRA H
                let new_value = self.sra(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x2D => {
                // SRA L
                let new_value = self.sra(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x2E => {
                // SRA (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.sra(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x2F => {
                // SRA A
                let new_value = self.sra(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x30 => {
                // SWAP B
                let new_value = self.swap(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x31 => {
                // SWAP C
                let new_value = self.swap(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x32 => {
                // SWAP D
                let new_value = self.swap(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x33 => {
                // SWAP E
                let new_value = self.swap(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x34 => {
                // SWAP H
                let new_value = self.swap(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x35 => {
                // SWAP L
                let new_value = self.swap(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x36 => {
                // SWAP (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.swap(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x37 => {
                // SWAP A
                let new_value = self.swap(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x38 => {
                // SRL B
                let new_value = self.srl(self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x39 => {
                // SRL C
                let new_value = self.srl(self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x3A => {
                // SRL D
                let new_value = self.srl(self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x3B => {
                // SRL E
                let new_value = self.srl(self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x3C => {
                // SRL H
                let new_value = self.srl(self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x3D => {
                // SRL L
                let new_value = self.srl(self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x3E => {
                // SRL (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.srl(value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x3F => {
                // SRL A
                let new_value = self.srl(self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x40 => {
                // BIT 0, B
                self.bit(0, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x41 => {
                // BIT 0, C
                self.bit(0, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x42 => {
                // BIT 0, D
                self.bit(0, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x43 => {
                // BIT 0, E
                self.bit(0, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x44 => {
                // BIT 0, H
                self.bit(0, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x45 => {
                // BIT 0, L
                self.bit(0, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x46 => {
                // BIT 0, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(0, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x47 => {
                // BIT 0, A
                self.bit(0, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x48 => {
                // BIT 1, B
                self.bit(1, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x49 => {
                // BIT 1, C
                self.bit(1, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x4A => {
                // BIT 1, D
                self.bit(1, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x4B => {
                // BIT 1, E
                self.bit(1, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x4C => {
                // BIT 1, H
                self.bit(1, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x4D => {
                // BIT 1, L
                self.bit(1, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x4E => {
                // BIT 1, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(1, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x4F => {
                // BIT 1, A
                self.bit(1, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x50 => {
                // BIT 2, B
                self.bit(2, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x51 => {
                // BIT 2, C
                self.bit(2, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x52 => {
                // BIT 2, D
                self.bit(2, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x53 => {
                // BIT 2, E
                self.bit(2, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x54 => {
                // BIT 2, H
                self.bit(2, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x55 => {
                // BIT 2, L
                self.bit(2, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x56 => {
                // BIT 2, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(2, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x57 => {
                // BIT 2, A
                self.bit(2, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x58 => {
                // BIT 3, B
                self.bit(3, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x59 => {
                // BIT 3, C
                self.bit(3, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x5A => {
                // BIT 3, D
                self.bit(3, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x5B => {
                // BIT 3, E
                self.bit(3, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x5C => {
                // BIT 3, H
                self.bit(3, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x5D => {
                // BIT 3, L
                self.bit(3, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x5E => {
                // BIT 3, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(3, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x5F => {
                // BIT 3, A
                self.bit(3, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x60 => {
                // BIT 4, B
                self.bit(4, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x61 => {
                // BIT 4, C
                self.bit(4, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x62 => {
                // BIT 4, D
                self.bit(4, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x63 => {
                // BIT 4, E
                self.bit(4, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x64 => {
                // BIT 4, H
                self.bit(4, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x65 => {
                // BIT 4, L
                self.bit(4, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x66 => {
                // BIT 4, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(4, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x67 => {
                // BIT 4, A
                self.bit(4, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x68 => {
                // BIT 5, B
                self.bit(5, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x69 => {
                // BIT 5, C
                self.bit(5, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x6A => {
                // BIT 5, D
                self.bit(5, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x6B => {
                // BIT 5, E
                self.bit(5, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x6C => {
                // BIT 5, H
                self.bit(5, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x6D => {
                // BIT 5, L
                self.bit(5, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x6E => {
                // BIT 5, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(5, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x6F => {
                // BIT 5, A
                self.bit(5, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x70 => {
                // BIT 6, B
                self.bit(6, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x71 => {
                // BIT 6, C
                self.bit(6, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x72 => {
                // BIT 6, D
                self.bit(6, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x73 => {
                // BIT 6, E
                self.bit(6, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x74 => {
                // BIT 6, H
                self.bit(6, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x75 => {
                // BIT 6, L
                self.bit(6, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x76 => {
                // BIT 6, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(6, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x77 => {
                // BIT 6, A
                self.bit(6, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x78 => {
                // BIT 7, B
                self.bit(7, self.reg.b);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x79 => {
                // BIT 7, C
                self.bit(7, self.reg.c);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x7A => {
                // BIT 7, D
                self.bit(7, self.reg.d);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x7B => {
                // BIT 7, E
                self.bit(7, self.reg.e);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x7C => {
                // BIT 7, H
                self.bit(7, self.reg.h);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x7D => {
                // BIT 7, L
                self.bit(7, self.reg.l);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x7E => {
                // BIT 7, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                self.bit(7, value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x7F => {
                // BIT 7, A
                self.bit(7, self.reg.a);
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x80 => {
                // RES 0, B
                let new_value = self.res(0, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x81 => {
                // RES 0, C
                let new_value = self.res(0, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x82 => {
                // RES 0, D
                let new_value = self.res(0, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x83 => {
                // RES 0, E
                let new_value = self.res(0, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x84 => {
                // RES 0, H
                let new_value = self.res(0, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x85 => {
                // RES 0, L
                let new_value = self.res(0, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x86 => {
                // RES 0, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(0, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x87 => {
                // RES 0, A
                let new_value = self.res(0, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x88 => {
                // RES 1, B
                let new_value = self.res(1, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x89 => {
                // RES 1, C
                let new_value = self.res(1, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x8A => {
                // RES 1, D
                let new_value = self.res(1, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x8B => {
                // RES 1, E
                let new_value = self.res(1, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x8C => {
                // RES 1, H
                let new_value = self.res(1, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x8D => {
                // RES 1, L
                let new_value = self.res(1, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x8E => {
                // RES 1, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(1, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x8F => {
                // RES 1, A
                let new_value = self.res(1, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x90 => {
                // RES 2, B
                let new_value = self.res(2, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x91 => {
                // RES 2, C
                let new_value = self.res(2, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x92 => {
                // RES 2, D
                let new_value = self.res(2, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x93 => {
                // RES 2, E
                let new_value = self.res(2, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x94 => {
                // RES 2, H
                let new_value = self.res(2, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x95 => {
                // RES 2, L
                let new_value = self.res(2, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x96 => {
                // RES 2, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(2, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x97 => {
                // RES 2, A
                let new_value = self.res(2, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x98 => {
                // RES 3, B
                let new_value = self.res(3, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x99 => {
                // RES 3, C
                let new_value = self.res(3, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x9A => {
                // RES 3, D
                let new_value = self.res(3, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x9B => {
                // RES 3, E
                let new_value = self.res(3, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x9C => {
                // RES 3, H
                let new_value = self.res(3, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x9D => {
                // RES 3, L
                let new_value = self.res(3, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0x9E => {
                // RES 3, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(3, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0x9F => {
                // RES 3, A
                let new_value = self.res(3, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA0 => {
                // RES 4, B
                let new_value = self.res(4, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA1 => {
                // RES 4, C
                let new_value = self.res(4, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA2 => {
                // RES 4, D
                let new_value = self.res(4, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA3 => {
                // RES 4, E
                let new_value = self.res(4, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA4 => {
                // RES 4, H
                let new_value = self.res(4, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA5 => {
                // RES 4, L
                let new_value = self.res(4, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA6 => {
                // RES 4, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(4, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xA7 => {
                // RES 4, A
                let new_value = self.res(4, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA8 => {
                // RES 5, B
                let new_value = self.res(5, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xA9 => {
                // RES 5, C
                let new_value = self.res(5, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xAA => {
                // RES 5, D
                let new_value = self.res(5, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xAB => {
                // RES 5, E
                let new_value = self.res(5, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xAC => {
                // RES 5, H
                let new_value = self.res(5, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xAD => {
                // RES 5, L
                let new_value = self.res(5, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xAE => {
                // RES 5, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(5, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xAF => {
                // RES 5, A
                let new_value = self.res(5, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB0 => {
                // RES 6, B
                let new_value = self.res(6, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB1 => {
                // RES 6, C
                let new_value = self.res(6, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB2 => {
                // RES 6, D
                let new_value = self.res(6, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB3 => {
                // RES 6, E
                let new_value = self.res(6, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB4 => {
                // RES 6, H
                let new_value = self.res(6, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB5 => {
                // RES 6, L
                let new_value = self.res(6, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB6 => {
                // RES 6, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(6, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xB7 => {
                // RES 6, A
                let new_value = self.res(6, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB8 => {
                // RES 7, B
                let new_value = self.res(7, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xB9 => {
                // RES 7, C
                let new_value = self.res(7, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xBA => {
                // RES 7, D
                let new_value = self.res(7, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xBB => {
                // RES 7, E
                let new_value = self.res(7, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xBC => {
                // RES 7, H
                let new_value = self.res(7, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xBD => {
                // RES 7, L
                let new_value = self.res(7, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xBE => {
                // RES 7, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.res(7, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xBF => {
                // RES 7, A
                let new_value = self.res(7, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC0 => {
                // SET 0, B
                let new_value = self.set(0, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC1 => {
                // SET 0, C
                let new_value = self.set(0, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC2 => {
                // SET 0, D
                let new_value = self.set(0, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC3 => {
                // SET 0, E
                let new_value = self.set(0, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC4 => {
                // SET 0, H
                let new_value = self.set(0, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC5 => {
                // SET 0, L
                let new_value = self.set(0, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC6 => {
                // SET 0, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(0, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xC7 => {
                // SET 0, A
                let new_value = self.set(0, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC8 => {
                // SET 1, B
                let new_value = self.set(1, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xC9 => {
                // SET 1, C
                let new_value = self.set(1, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xCA => {
                // SET 1, D
                let new_value = self.set(1, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xCB => {
                // SET 1, E
                let new_value = self.set(1, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xCC => {
                // SET 1, H
                let new_value = self.set(1, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xCD => {
                // SET 1, L
                let new_value = self.set(1, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xCE => {
                // SET 1, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(1, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xCF => {
                // SET 1, A
                let new_value = self.set(1, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD0 => {
                // SET 2, B
                let new_value = self.set(2, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD1 => {
                // SET 2, C
                let new_value = self.set(2, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD2 => {
                // SET 2, D
                let new_value = self.set(2, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD3 => {
                // SET 2, E
                let new_value = self.set(2, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD4 => {
                // SET 2, H
                let new_value = self.set(2, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD5 => {
                // SET 2, L
                let new_value = self.set(2, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD6 => {
                // SET 2, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(2, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xD7 => {
                // SET 2, A
                let new_value = self.set(2, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD8 => {
                // SET 3, B
                let new_value = self.set(3, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xD9 => {
                // SET 3, C
                let new_value = self.set(3, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xDA => {
                // SET 3, D
                let new_value = self.set(3, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xDB => {
                // SET 3, E
                let new_value = self.set(3, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xDC => {
                // SET 3, H
                let new_value = self.set(3, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xDD => {
                // SET 3, L
                let new_value = self.set(3, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xDE => {
                // SET 3, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(3, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xDF => {
                // SET 3, A
                let new_value = self.set(3, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE0 => {
                // SET 4, B
                let new_value = self.set(4, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE1 => {
                // SET 4, C
                let new_value = self.set(4, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE2 => {
                // SET 4, D
                let new_value = self.set(4, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE3 => {
                // SET 4, E
                let new_value = self.set(4, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE4 => {
                // SET 4, H
                let new_value = self.set(4, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE5 => {
                // SET 4, L
                let new_value = self.set(4, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE6 => {
                // SET 4, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(4, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xE7 => {
                // SET 4, A
                let new_value = self.set(4, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE8 => {
                // SET 5, B
                let new_value = self.set(5, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xE9 => {
                // SET 5, C
                let new_value = self.set(5, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xEA => {
                // SET 5, D
                let new_value = self.set(5, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xEB => {
                // SET 5, E
                let new_value = self.set(5, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xEC => {
                // SET 5, H
                let new_value = self.set(5, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xED => {
                // SET 5, L
                let new_value = self.set(5, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xEE => {
                // SET 5, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(5, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xEF => {
                // SET 5, A
                let new_value = self.set(5, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF0 => {
                // SET 6, B
                let new_value = self.set(6, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF1 => {
                // SET 6, C
                let new_value = self.set(6, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF2 => {
                // SET 6, D
                let new_value = self.set(6, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF3 => {
                // SET 6, E
                let new_value = self.set(6, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF4 => {
                // SET 6, H
                let new_value = self.set(6, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF5 => {
                // SET 6, L
                let new_value = self.set(6, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF6 => {
                // SET 6, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(6, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xF7 => {
                // SET 6, A
                let new_value = self.set(6, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF8 => {
                // SET 7, B
                let new_value = self.set(7, self.reg.b);
                self.reg.b = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xF9 => {
                // SET 7, C
                let new_value = self.set(7, self.reg.c);
                self.reg.c = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xFA => {
                // SET 7, D
                let new_value = self.set(7, self.reg.d);
                self.reg.d = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xFB => {
                // SET 7, E
                let new_value = self.set(7, self.reg.e);
                self.reg.e = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xFC => {
                // SET 7, H
                let new_value = self.set(7, self.reg.h);
                self.reg.h = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xFD => {
                // SET 7, L
                let new_value = self.set(7, self.reg.l);
                self.reg.l = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
            0xFE => {
                // SET 7, (HL)
                let value = self.bus.read_byte(self.reg.hl());
                let new_value = self.set(7, value);
                self.bus.write_byte(self.reg.hl(), new_value);
                Ok((self.reg.pc.wrapping_add(2), 16))
            }
            0xFF => {
                // SET 7, A
                let new_value = self.set(7, self.reg.a);
                self.reg.a = new_value;
                Ok((self.reg.pc.wrapping_add(2), 8))
            }
        }
    }

    #[inline]
    fn add(&mut self, value: u8) {
        let (new_value, overflow) = self.reg.a.overflowing_add(value);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h((self.reg.a & 0xf) + (value & 0xf) > 0xf);
        self.reg.f.set_c(overflow);
        self.reg.a = new_value;
    }

    #[inline]
    fn add16(&mut self, value: u16) {
        let hl = self.reg.hl();
        let res = hl.wrapping_add(value);
        self.reg.f.set_n(false);
        self.reg.f.set_h((hl & 0xfff) + (value & 0xfff) > 0xfff);
        self.reg.f.set_c(hl > 0xffff - value);
        self.reg.set_hl(res);
    }

    #[inline]
    fn push(&mut self, value: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.bus.write_word(self.reg.sp, value);
    }

    #[inline]
    fn pop(&mut self) -> u16 {
        let value = self.bus.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        value
    }

    #[inline]
    fn rlc(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) != 0;
        let new_value = (value << 1) | (value >> 7);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn rl(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) != 0;
        let new_value = (value << 1) | (self.reg.f.carry as u8);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn rrc(&mut self, value: u8) -> u8 {
        let carry = (value & 1) != 0;
        let new_value = (value >> 1) | (value << 7);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn rr(&mut self, value: u8) -> u8 {
        let carry = (value & 1) != 0;
        let new_value = (value >> 1) | ((self.reg.f.carry as u8) << 7);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn sla(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) != 0;
        let new_value = value << 1;
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn sra(&mut self, value: u8) -> u8 {
        let carry = (value & 1) != 0;
        let new_value = (value & 0x80) | (value >> 1);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn swap(&mut self, value: u8) -> u8 {
        let new_value = (value << 4) | (value >> 4);
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(false);
        new_value
    }

    #[inline]
    fn srl(&mut self, value: u8) -> u8 {
        let carry = (value & 1) != 0;
        let new_value = value >> 1;
        self.reg.f.set_z(new_value == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(false);
        self.reg.f.set_c(carry);
        new_value
    }

    #[inline]
    fn bit(&mut self, bit: u8, value: u8) {
        self.reg.f.set_z((value & (1 << bit)) == 0x0);
        self.reg.f.set_n(false);
        self.reg.f.set_h(true);
    }

    #[inline]
    fn res(&mut self, bit: u8, value: u8) -> u8 {
        value & !(1 << bit)
    }

    #[inline]
    fn set(&mut self, bit: u8, value: u8) -> u8 {
        value | (1 << bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reti_leaves_ime_enabled() {
        let mut cpu = Cpu::new(false);
        cpu.ime = false;
        cpu.reg.sp = 0xC000;
        cpu.bus.write_word(cpu.reg.sp, 0x1234);

        let (pc, _) = cpu.execute(0xD9).unwrap();
        cpu.reg.pc = pc;
        cpu.handle_interrupts();

        assert!(cpu.ime);
        assert!(!cpu.ime_next);
    }

    #[test]
    fn ei_enables_ime_once_after_current_instruction() {
        let mut cpu = Cpu::new(false);
        cpu.ime = false;

        cpu.execute(0xFB).unwrap();
        assert!(!cpu.ime);
        assert!(cpu.ime_next);

        cpu.handle_interrupts();
        assert!(cpu.ime);
        assert!(!cpu.ime_next);

        cpu.handle_interrupts();
        assert!(cpu.ime);
    }

    #[test]
    fn di_cancels_pending_ei() {
        let mut cpu = Cpu::new(false);

        cpu.execute(0xFB).unwrap();
        cpu.execute(0xF3).unwrap();
        cpu.handle_interrupts();

        assert!(!cpu.ime);
        assert!(!cpu.ime_next);
    }
}
