use crate::cartridges::CartridgeMode;
use crate::memory::Memory;

#[derive(Debug, Copy, Clone)]
pub struct PPU {
    mode: CartridgeMode,
}

impl PPU {
    pub fn new(mode: CartridgeMode) -> PPU {
        PPU { mode }
    }

    pub fn run_cycles(&mut self, cycles: u32) {}
}

impl Memory for PPU {
    fn get_byte(&self, addr: u16) -> u8 {
        panic!("ppu: get_byte not implemented")
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        panic!("ppu: set_byte not implemented")
    }
}
