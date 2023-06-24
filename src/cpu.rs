use crate::{
    instruction::ArithmaticTarget, instruction::Instruction, mmu::MemBus, registers::Registers,
};

macro_rules! set_flags {
    ($fr:expr, $z:expr, $n:expr, $h:expr, $c:expr) => {
        $fr.zero = $z;
        $fr.subtract = $n;
        $fr.half_carry = $h;
        $fr.carry = $c;
    };
}

/// Implementation of the Sharp LR35902
#[derive(Debug)]
pub struct CPU {
    /// registers A, B, C, D, E, F, H, L
    reg: Registers,
    /// program counter
    pc: u16,
    /// stack pointer
    sp: u16,
    /// MMU
    bus: MemBus,

    /// The clock runs at 4.194304 MHz. This means that each clock cycle
    /// takes 1/4.194304 MHz = 0.238 μs = 238 ns. The CPU clock is also
    /// referred to as the “T-cycle”. This can be divided by 4 to get the
    /// machine cycle (M-cycle) which is 1.048576 MHz.
    clock: Clock,
}

#[derive(Debug)]
pub struct Clock {
    t: u16,
    // unused for now
    _m: u16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            pc: 0,
            sp: 0,
            bus: MemBus::new(),
            clock: Clock { _m: 0, t: 0 },
        }
    }

    pub fn step(&mut self) -> u16 {
        let opcode = self.bus.read_byte(self.pc);
        let instruction = match Instruction::decode(opcode) {
            Ok(ins) => ins,
            Err(_) => panic!("No instruction found for opcode 0x{}", opcode),
        };
        self.pc = self.execute(instruction);
        self.clock.t
    }

    fn reset(&mut self) {
        self.reg.reset();
        self.sp = 0;
        self.pc = 0;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => match target {
                ArithmaticTarget::A => {
                    let value = self.reg.a;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::B => {
                    let value = self.reg.b;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::C => {
                    let value = self.reg.c;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::D => {
                    let value = self.reg.d;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::E => {
                    let value = self.reg.e;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::H => {
                    let value = self.reg.h;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::L => {
                    let value = self.reg.l;
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 1;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::HL => {
                    let value = self.bus.read_byte(self.reg.hl());
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 2;
                    self.pc.wrapping_add(1)
                }
                ArithmaticTarget::Data => {
                    let value = self.bus.read_byte(self.pc.wrapping_add(1));
                    let new_value = self.add(value);
                    self.reg.a = new_value;
                    self.clock.t = 2;
                    self.pc.wrapping_add(2)
                }
                _ => panic!("ADD not implemented for target {:?}", target),
            },
            Instruction::NOP => self.pc.wrapping_add(0),
            _ => panic!("Instruction {:?} not implemented", instruction),
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.reg.a.overflowing_add(value);
        self.reg.f.set_all(
            new_value == 0,
            false,
            // Half Carry is set if adding the lower nibbles of the value and register A
            // together result in a value bigger than 0xF. If the result is larger than 0xF
            // than the addition caused a carry from the lower nibble to the upper nibble.
            (self.reg.a & 0xF) + (value & 0xF) > 0xF,
            did_overflow,
        );
        new_value
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }
}

mod tests {
    #[test]
    fn cpu_test_add() {}
}
