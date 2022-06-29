use crate::cartridges::Cartridge;
use crate::cartridges::Stable;
use crate::memory::Memory;

// This is a 32kB (256kb) ROM and occupies 0000-7FFF.
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

impl Stable for RomOnly {
    fn sav(&self) {}
}

impl Cartridge for RomOnly {}
