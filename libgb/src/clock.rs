pub const CLOCK_HZ: u32 = 4_194_304;
pub const MAX_FPS: u32 = 60;
pub const MAX_CYCLES_PER_FRAME: u32 = CLOCK_HZ / MAX_FPS;

/// The clock runs at 4.194304 MHz. This means that each clock cycle
/// takes 1/4.194304 MHz = 0.238 μs = 238 ns. The CPU clock is also
/// referred to as the “T-cycle”. This can be divided by 4 to get the
/// machine cycle (M-cycle) which is 1.048576 MHz.
#[derive(Debug)]
pub struct Clock {
    pub t: u32,
    pub _m: u32,
}

impl Clock {
    pub fn new() -> Self {
        Self { t: 0, _m: 0 }
    }

    pub fn reset(&mut self) {
        self.t = 0;
        self._m = 0;
    }

    pub fn step(&mut self, t_cycles: u32) {
        self.t += t_cycles;
        self._m += t_cycles / 4;
    }
}
