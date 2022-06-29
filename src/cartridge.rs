use crate::memory::Memory;

#[derive(Debug, Copy, Clone)]
pub struct Cartridge {}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {}
    }
}

impl Memory for Cartridge {
    fn get_byte(&self, addr: u16) -> u8 {
        0x00
    }

    fn set_byte(&mut self, addr: u16, value: u8) {}
}
