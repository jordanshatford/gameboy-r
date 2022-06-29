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
        panic!("cartridge: get_byte not implemented")
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        panic!("cartridge: set_byte not implemented")
    }
}
