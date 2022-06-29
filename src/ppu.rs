use super::memory::Memory;

#[derive(Debug, Copy, Clone)]
pub struct PPU {}

impl PPU {
    pub fn new() -> PPU {
        PPU {}
    }
}

impl Memory for PPU {
    fn get_byte(&self, addr: u16) -> u8 {
        0x00
    }

    // ROM cartridges are read only
    fn set_byte(&mut self, addr: u16, value: u8) {}
}
