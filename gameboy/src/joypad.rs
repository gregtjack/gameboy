use crate::{
    addressable::Addressable,
    mmu::interrupts::{InterruptFlag, Interrupts},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    None,
    Direction,
    Action,
}

#[derive(Debug, Clone, Copy)]
enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub struct Joypad {
    interrupts: Rc<RefCell<Interrupts>>,
    mode: Mode,
    up: KeyState,
    down: KeyState,
    left: KeyState,
    right: KeyState,
    a: KeyState,
    b: KeyState,
    start: KeyState,
    select: KeyState,
}

impl Joypad {
    pub fn new(interrupts: Rc<RefCell<Interrupts>>) -> Self {
        Self {
            interrupts,
            mode: Mode::Direction,
            up: KeyState::Released,
            down: KeyState::Released,
            left: KeyState::Released,
            right: KeyState::Released,
            a: KeyState::Released,
            b: KeyState::Released,
            start: KeyState::Released,
            select: KeyState::Released,
        }
    }

    pub fn press(&mut self, key: Key) {
        self.interrupts.borrow_mut().set_flag(InterruptFlag::Joypad);
        let key_state = match key {
            Key::Up => &mut self.up,
            Key::Down => &mut self.down,
            Key::Left => &mut self.left,
            Key::Right => &mut self.right,
            Key::A => &mut self.a,
            Key::B => &mut self.b,
            Key::Start => &mut self.start,
            Key::Select => &mut self.select,
        };

        *key_state = KeyState::Pressed;
    }

    pub fn release(&mut self, key: Key) {
        let key_state = match key {
            Key::Up => &mut self.up,
            Key::Down => &mut self.down,
            Key::Left => &mut self.left,
            Key::Right => &mut self.right,
            Key::A => &mut self.a,
            Key::B => &mut self.b,
            Key::Start => &mut self.start,
            Key::Select => &mut self.select,
        };

        *key_state = KeyState::Released;
    }
}

impl Addressable for Joypad {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF00 => match self.mode {
                Mode::Direction => {
                    let mut byte = 0b1110_0000;
                    byte |= (self.down as u8) << 3;
                    byte |= (self.up as u8) << 2;
                    byte |= (self.left as u8) << 1;
                    byte |= self.right as u8;
                    byte
                }
                Mode::Action => {
                    let mut byte = 0b1101_0000;
                    byte |= (self.start as u8) << 3;
                    byte |= (self.select as u8) << 2;
                    byte |= (self.b as u8) << 1;
                    byte |= self.a as u8;
                    byte
                }
                Mode::None => 0xF as u8,
            },
            _ => panic!("Invalid joypad address"),
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        match addr {
            0xFF00 => {
                self.mode = match byte & 0b0011_0000 {
                    0b0001_0000 => Mode::Action,
                    0b0010_0000 => Mode::Direction,
                    0b0011_0000 | 0b0 => Mode::None,
                    _ => panic!("Invalid joypad mode"),
                };
            }
            _ => panic!("Invalid joypad address"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joypad() -> Joypad {
        Joypad::new(Rc::new(RefCell::new(Interrupts::new())))
    }

    #[test]
    fn action_row_is_selected_by_clearing_bit_5() {
        let mut joypad = joypad();
        joypad.press(Key::Start);

        joypad.write_byte(0xFF00, 0b0001_0000);

        assert_eq!(joypad.read_byte(0xFF00), 0b1101_0111);
    }

    #[test]
    fn direction_row_is_selected_by_clearing_bit_4() {
        let mut joypad = joypad();
        joypad.press(Key::Right);

        joypad.write_byte(0xFF00, 0b0010_0000);

        assert_eq!(joypad.read_byte(0xFF00), 0b1110_1110);
    }
}
