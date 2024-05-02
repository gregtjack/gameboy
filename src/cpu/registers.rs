use crate::addr;

#[derive(Debug, Clone, Copy)]
pub struct FlagsReg {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl FlagsReg {
    /// Reset all the flags
    pub fn reset(&mut self) {
        self.zero = false;
        self.subtract = false;
        self.half_carry = false;
        self.carry = false;
    }

    pub fn set_z(&mut self, zero: bool) {
        self.zero = zero;
    }

    pub fn set_s(&mut self, subtract: bool) {
        self.subtract = subtract;
    }

    pub fn set_h(&mut self, half_carry: bool) {
        self.half_carry = half_carry;
    }

    pub fn set_c(&mut self, carry: bool) {
        self.carry = carry;
    }
}

impl From<FlagsReg> for u8 {
    fn from(flag: FlagsReg) -> u8 {
        (flag.zero as u8) << 7
            | (flag.subtract as u8) << 6
            | (flag.half_carry as u8) << 5
            | (flag.carry as u8) << 4
    }
}

impl From<u8> for FlagsReg {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> 7) & 1) != 0;
        let subtract = ((byte >> 6) & 1) != 0;
        let half_carry = ((byte >> 5) & 1) != 0;
        let carry = ((byte >> 4) & 1) != 0;

        FlagsReg {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: FlagsReg,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: FlagsReg {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            },
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.f.reset();
    }

    pub fn af(&self) -> u16 {
        let f: u8 = self.f.into();
        addr!(self.a, f)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsReg::from((value & 0xFF) as u8);
    }

    pub fn bc(&self) -> u16 {
        addr!(self.b, self.c)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn de(&self) -> u16 {
        addr!(self.d, self.e)
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}
