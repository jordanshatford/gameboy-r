use super::memory::Memory;

#[derive(Debug, Copy, Clone)]
pub struct APU {}

impl APU {
    pub fn new() -> APU {
        APU {}
    }
}

impl Memory for APU {
    fn get_byte(&self, addr: u16) -> u8 {
        0x00
    }

    fn set_byte(&mut self, addr: u16, value: u8) {}
}
