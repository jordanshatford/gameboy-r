// The Clock is used to help normalize the cycles in the Timer

#[derive(Debug, Copy, Clone)]
pub struct Clock {
    pub period: u32,
    pub num_cycles: u32,
}

impl Clock {
    pub fn new(period: u32) -> Clock {
        Clock {
            period,
            num_cycles: 0x00,
        }
    }

    pub fn run_cycles(&mut self, cycles: u32) -> u32 {
        self.num_cycles += cycles;
        let normalized_cycles = self.num_cycles / self.period;
        self.num_cycles %= self.period;
        normalized_cycles
    }
}

#[cfg(test)]
mod test {
    use super::Clock;

    #[test]
    fn clock_functionality() {
        let mut clock = Clock::new(100);
        assert_eq!(clock.run_cycles(200), 2);
        assert_eq!(clock.num_cycles, 0);
        clock = Clock::new(111);
        assert_eq!(clock.run_cycles(221), 1);
        assert_eq!(clock.num_cycles, 110);
    }
}
