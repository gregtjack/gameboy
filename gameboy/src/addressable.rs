/// A trait for performing read and write operations
pub trait Addressable {
    /// Read a byte at the given address.
    fn read_byte(&self, addr: u16) -> u8;

    /// Write a byte at the given address.
    fn write_byte(&mut self, addr: u16, value: u8);

    /// Read a word at the given address.
    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | (self.read_byte(addr + 1) as u16) << 8
    }

    /// Write a word at the given address.
    fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }
}
