#[derive(Debug, Clone, Copy)]
pub enum ColorBit {
    Hi,
    Lo,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorId {
    Id00,
    Id01,
    Id10,
    Id11,
}

impl ColorId {
    pub fn from_bits(hi: bool, lo: bool) -> Self {
        match (hi, lo) {
            (false, false) => Self::Id00,
            (false, true) => Self::Id01,
            (true, false) => Self::Id10,
            (true, true) => Self::Id11,
        }
    }

    pub fn get_bit(&self, bit: ColorBit) -> bool {
        match self {
            ColorId::Id01 => match bit {
                ColorBit::Hi => false,
                ColorBit::Lo => true,
            },
            ColorId::Id10 => match bit {
                ColorBit::Hi => true,
                ColorBit::Lo => false,
            },
            ColorId::Id00 => false,
            ColorId::Id11 => true,
        }
    }

    pub fn set_bit(&mut self, bit: ColorBit, value: bool) {
        match bit {
            ColorBit::Hi => {
                *self = ColorId::from_bits(value, self.get_bit(ColorBit::Lo));
            }
            ColorBit::Lo => {
                *self = ColorId::from_bits(self.get_bit(ColorBit::Hi), value);
            }
        }
    }
}
