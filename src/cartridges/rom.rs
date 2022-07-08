// None (32KByte ROM only)
// Small games of not more than 32KBytes ROM do not require a MBC chip
// for ROM banking. The ROM is directly mapped to memory at 0000-7FFFh.
// Optionally up to 8KByte of RAM could be connected at A000-BFFF, even
// though that could require a tiny MBC-like circuit, but no real MBC chip.

use crate::cartridges::{Cartridge, Stable};
use crate::memory::Memory;

pub struct RomOnly {
    rom: Vec<u8>,
}

impl RomOnly {
    pub fn new(rom: Vec<u8>) -> RomOnly {
        RomOnly { rom }
    }
}

impl Memory for RomOnly {
    fn get_byte(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    // You cannot set byte in rom only
    fn set_byte(&mut self, _: u16, _: u8) {}
}

impl Stable for RomOnly {}

impl Cartridge for RomOnly {}
