const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

#[derive(Debug)]
pub struct FlagsReg {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl FlagsReg {
    pub fn reset(&mut self) {
        self.zero = false;
        self.subtract = false;
        self.half_carry = false;
        self.carry = false;
    }

    /// Set all the flags at once
    pub fn set_all(&mut self, zero: bool, subtract: bool, half_carry: bool, carry: bool) {
        self.zero = zero;
        self.subtract = subtract;
        self.half_carry = half_carry;
        self.carry = carry;
    }
}

impl From<FlagsReg> for u8 {
    fn from(flag: FlagsReg) -> u8 {
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flag.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl From<u8> for FlagsReg {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 1) != 0;

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

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}
