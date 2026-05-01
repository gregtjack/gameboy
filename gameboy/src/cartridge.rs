use crate::addressable::Addressable;
use crate::mmu::{ERAM_BEGIN, ERAM_END, ERAM_LEN, ROM_BEGIN, ROM_END, ROM_LEN, UNDEFINED};

const ROM_BANK_LEN: usize = 0x4000;
const RAM_BANK_LEN: usize = ERAM_LEN as usize;
const MBC2_RAM_LEN: usize = 0x200;

const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
const RAM_SIZE_ADDR: usize = 0x0149;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mapper {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BankingMode {
    Rom,
    Ram,
}

#[derive(Debug)]
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    mapper: Mapper,
    ram_enabled: bool,
    rom_bank_low: usize,
    bank_high: usize,
    banking_mode: BankingMode,
}

impl Cartridge {
    pub fn from_rom(rom: Vec<u8>) -> Self {
        assert!(
            rom.len() >= ROM_LEN as usize,
            "cartridge ROM must contain at least two 16KiB banks"
        );

        let cartridge_type = rom[CARTRIDGE_TYPE_ADDR];
        let mapper = mapper(cartridge_type).unwrap_or_else(|| {
            panic!("unsupported cartridge type: 0x{cartridge_type:02X}");
        });
        let ram = vec![0; ram_len(cartridge_type, rom[RAM_SIZE_ADDR])];

        Self {
            rom,
            ram,
            mapper,
            ram_enabled: false,
            rom_bank_low: 1,
            bank_high: 0,
            banking_mode: BankingMode::Rom,
        }
    }

    pub fn empty() -> Self {
        Self::from_rom(vec![0; ROM_LEN as usize])
    }

    fn read_rom(&self, addr: u16) -> u8 {
        let offset = match addr {
            0x0000..=0x3FFF => {
                let bank = match self.mapper {
                    Mapper::Mbc1 if self.banking_mode == BankingMode::Ram => self.bank_high << 5,
                    _ => 0,
                };
                self.rom_offset(bank, addr as usize)
            }
            0x4000..=0x7FFF => {
                let bank = self.selected_rom_bank();
                self.rom_offset(bank, (addr as usize) - ROM_BANK_LEN)
            }
            _ => return UNDEFINED,
        };

        self.rom.get(offset).copied().unwrap_or(UNDEFINED)
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match self.mapper {
            Mapper::RomOnly => {}
            Mapper::Mbc1 => self.write_mbc1_register(addr, value),
            Mapper::Mbc2 => self.write_mbc2_register(addr, value),
            Mapper::Mbc3 => self.write_mbc3_register(addr, value),
            Mapper::Mbc5 => self.write_mbc5_register(addr, value),
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        match self.mapper {
            Mapper::Mbc2 => {
                if !self.ram_enabled || self.ram.is_empty() {
                    return UNDEFINED;
                }

                0xF0 | self.ram[(addr as usize - ERAM_BEGIN as usize) % MBC2_RAM_LEN]
            }
            Mapper::RomOnly => self.read_ram_bank(0, addr),
            Mapper::Mbc1 | Mapper::Mbc5 => {
                if !self.ram_enabled {
                    return UNDEFINED;
                }

                self.read_ram_bank(self.selected_ram_bank(), addr)
            }
            Mapper::Mbc3 => {
                if !self.ram_enabled || self.bank_high > 0x03 {
                    return UNDEFINED;
                }

                self.read_ram_bank(self.bank_high, addr)
            }
        }
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        match self.mapper {
            Mapper::Mbc2 => {
                if self.ram_enabled && !self.ram.is_empty() {
                    let offset = (addr as usize - ERAM_BEGIN as usize) % MBC2_RAM_LEN;
                    self.ram[offset] = value & 0x0F;
                }
            }
            Mapper::RomOnly => self.write_ram_bank(0, addr, value),
            Mapper::Mbc1 | Mapper::Mbc5 => {
                if self.ram_enabled {
                    self.write_ram_bank(self.selected_ram_bank(), addr, value);
                }
            }
            Mapper::Mbc3 => {
                if self.ram_enabled && self.bank_high <= 0x03 {
                    self.write_ram_bank(self.bank_high, addr, value);
                }
            }
        }
    }

    fn write_mbc1_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x3FFF => self.rom_bank_low = non_zero_bank((value & 0x1F) as usize),
            0x4000..=0x5FFF => self.bank_high = (value & 0x03) as usize,
            0x6000..=0x7FFF => {
                self.banking_mode = if value & 0x01 == 0 {
                    BankingMode::Rom
                } else {
                    BankingMode::Ram
                };
            }
            _ => {}
        }
    }

    fn write_mbc2_register(&mut self, addr: u16, value: u8) {
        if addr & 0x0100 == 0 {
            self.ram_enabled = value & 0x0F == 0x0A;
        } else {
            self.rom_bank_low = non_zero_bank((value & 0x0F) as usize);
        }
    }

    fn write_mbc3_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x3FFF => self.rom_bank_low = non_zero_bank((value & 0x7F) as usize),
            0x4000..=0x5FFF => self.bank_high = (value & 0x0F) as usize,
            0x6000..=0x7FFF => {}
            _ => {}
        }
    }

    fn write_mbc5_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x2FFF => {
                self.rom_bank_low = (self.rom_bank_low & 0x100) | value as usize;
            }
            0x3000..=0x3FFF => {
                self.rom_bank_low = (self.rom_bank_low & 0x0FF) | (((value & 0x01) as usize) << 8);
            }
            0x4000..=0x5FFF => self.bank_high = (value & 0x0F) as usize,
            0x6000..=0x7FFF => {}
            _ => {}
        }
    }

    fn selected_rom_bank(&self) -> usize {
        match self.mapper {
            Mapper::RomOnly => 1,
            Mapper::Mbc1 => (self.bank_high << 5) | self.rom_bank_low,
            Mapper::Mbc2 | Mapper::Mbc3 | Mapper::Mbc5 => self.rom_bank_low,
        }
    }

    fn selected_ram_bank(&self) -> usize {
        match self.mapper {
            Mapper::Mbc1 if self.banking_mode == BankingMode::Ram => self.bank_high,
            Mapper::Mbc3 if self.bank_high <= 0x03 => self.bank_high,
            Mapper::Mbc5 => self.bank_high,
            _ => 0,
        }
    }

    fn rom_offset(&self, bank: usize, offset: usize) -> usize {
        let bank_count = self.rom.len().div_ceil(ROM_BANK_LEN);
        let bank = if bank_count == 0 {
            0
        } else {
            bank % bank_count
        };

        bank * ROM_BANK_LEN + offset
    }

    fn read_ram_bank(&self, bank: usize, addr: u16) -> u8 {
        if self.ram.is_empty() {
            return UNDEFINED;
        }

        let offset = self.ram_offset(bank, addr);
        self.ram.get(offset).copied().unwrap_or(UNDEFINED)
    }

    fn write_ram_bank(&mut self, bank: usize, addr: u16, value: u8) {
        if self.ram.is_empty() {
            return;
        }

        let offset = self.ram_offset(bank, addr);
        if let Some(byte) = self.ram.get_mut(offset) {
            *byte = value;
        }
    }

    fn ram_offset(&self, bank: usize, addr: u16) -> usize {
        let bank_count = self.ram.len().div_ceil(RAM_BANK_LEN);
        let bank = if bank_count == 0 {
            0
        } else {
            bank % bank_count
        };

        bank * RAM_BANK_LEN + (addr as usize - ERAM_BEGIN as usize)
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self::empty()
    }
}

impl Addressable for Cartridge {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            ROM_BEGIN..=ROM_END => self.read_rom(addr),
            ERAM_BEGIN..=ERAM_END => self.read_ram(addr),
            _ => UNDEFINED,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            ROM_BEGIN..=ROM_END => self.write_rom(addr, value),
            ERAM_BEGIN..=ERAM_END => self.write_ram(addr, value),
            _ => {}
        }
    }
}

fn mapper(cartridge_type: u8) -> Option<Mapper> {
    match cartridge_type {
        0x00 | 0x08 | 0x09 => Some(Mapper::RomOnly),
        0x01..=0x03 => Some(Mapper::Mbc1),
        0x05 | 0x06 => Some(Mapper::Mbc2),
        0x0F..=0x13 => Some(Mapper::Mbc3),
        0x19..=0x1E => Some(Mapper::Mbc5),
        _ => None,
    }
}

fn ram_len(cartridge_type: u8, ram_size: u8) -> usize {
    if matches!(cartridge_type, 0x05 | 0x06) {
        return MBC2_RAM_LEN;
    }

    match ram_size {
        0x00 => 0,
        0x01 => 0x800,
        0x02 => RAM_BANK_LEN,
        0x03 => RAM_BANK_LEN * 4,
        0x04 => RAM_BANK_LEN * 16,
        0x05 => RAM_BANK_LEN * 8,
        _ => 0,
    }
}

fn non_zero_bank(bank: usize) -> usize {
    if bank == 0 {
        1
    } else {
        bank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom(cartridge_type: u8, rom_banks: usize, ram_size: u8) -> Vec<u8> {
        let mut rom = vec![0; ROM_BANK_LEN * rom_banks];
        rom[CARTRIDGE_TYPE_ADDR] = cartridge_type;
        rom[RAM_SIZE_ADDR] = ram_size;

        for bank in 0..rom_banks {
            rom[bank * ROM_BANK_LEN] = bank as u8;
            rom[bank * ROM_BANK_LEN + 1] = (bank >> 8) as u8;
        }

        rom
    }

    #[test]
    fn rom_only_reads_fixed_banks() {
        let cartridge = Cartridge::from_rom(rom(0x00, 2, 0x00));

        assert_eq!(cartridge.read_byte(0x0000), 0);
        assert_eq!(cartridge.read_byte(0x4000), 1);
    }

    #[test]
    fn mbc1_switches_rom_banks() {
        let mut cartridge = Cartridge::from_rom(rom(0x01, 4, 0x00));

        cartridge.write_byte(0x2000, 0x02);

        assert_eq!(cartridge.read_byte(0x4000), 2);
    }

    #[test]
    fn mbc1_switches_ram_banks_in_ram_mode() {
        let mut cartridge = Cartridge::from_rom(rom(0x03, 4, 0x03));

        cartridge.write_byte(0x0000, 0x0A);
        cartridge.write_byte(0x6000, 0x01);
        cartridge.write_byte(0x4000, 0x01);
        cartridge.write_byte(0xA000, 0x42);
        cartridge.write_byte(0x4000, 0x00);
        cartridge.write_byte(0xA000, 0x99);
        cartridge.write_byte(0x4000, 0x01);

        assert_eq!(cartridge.read_byte(0xA000), 0x42);
    }

    #[test]
    fn mbc5_uses_nine_bit_rom_bank() {
        let mut cartridge = Cartridge::from_rom(rom(0x19, 0x102, 0x00));

        cartridge.write_byte(0x2000, 0x01);
        cartridge.write_byte(0x3000, 0x01);

        assert_eq!(cartridge.read_byte(0x4000), 0x01);
        assert_eq!(cartridge.read_byte(0x4001), 0x01);
    }

    #[test]
    fn mbc3_rtc_registers_do_not_alias_ram() {
        let mut cartridge = Cartridge::from_rom(rom(0x13, 4, 0x03));

        cartridge.write_byte(0x0000, 0x0A);
        cartridge.write_byte(0x4000, 0x08);
        cartridge.write_byte(0xA000, 0x42);
        cartridge.write_byte(0x4000, 0x00);

        assert_eq!(cartridge.read_byte(0xA000), 0);
    }
}
