pub struct Interrupt {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            vblank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }

    pub fn set_vblank(&mut self, vblank: bool) {
        self.vblank = vblank;
    }

    pub fn set_lcd_stat(&mut self, lcd_stat: bool) {
        self.lcd_stat = lcd_stat;
    }

    pub fn set_timer(&mut self, timer: bool) {
        self.timer = timer;
    }

    pub fn set_serial(&mut self, serial: bool) {
        self.serial = serial;
    }

    pub fn set_joypad(&mut self, joypad: bool) {
        self.joypad = joypad;
    }
}
