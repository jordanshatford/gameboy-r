use super::memory::Memory;

#[derive(Debug, Copy, Clone)]
pub struct APU {}

impl APU {
    pub fn new() -> APU {
        APU {}
    }

    pub fn run_cycles(&mut self, _cycles: u32) {}
}

impl Memory for APU {
    fn get_byte(&self, _addr: u16) -> u8 {
        panic!("apu: not implemented")
    }

    fn set_byte(&mut self, _addr: u16, _value: u8) {
        panic!("apu: not implemented")
    }
}
