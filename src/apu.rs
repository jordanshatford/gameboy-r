use super::memory::Memory;

#[derive(Debug, Copy, Clone)]
pub struct Apu {}

impl Apu {
    pub fn _new() -> Apu {
        Apu {}
    }

    pub fn run_cycles(&mut self, _cycles: u32) {}
}

impl Memory for Apu {
    fn get_byte(&self, _addr: u16) -> u8 {
        panic!("apu: not implemented")
    }

    fn set_byte(&mut self, _addr: u16, _value: u8) {
        panic!("apu: not implemented")
    }
}
