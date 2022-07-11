// MBC2 (max 256KByte ROM and 512x4 bits RAM)

// 0000-3FFF - ROM Bank 00 (Read Only)
// Same as for MBC1.

// 4000-7FFF - ROM Bank 01-0F (Read Only)
// Same as for MBC1, but only a total of 16 ROM banks is supported.

// A000-A1FF - 512x4bits RAM, built-in into the MBC2 chip (Read/Write)
// The MBC2 doesn't support external RAM, instead it includes 512x4 bits of built-in RAM (in the MBC2 chip itself).
// It still requires an external battery to save data during power-off though. As the data consists of 4bit values,
// only the lower 4 bits of the "bytes" in this memory area are used.

// 0000-1FFF - RAM Enable (Write Only)
// The least significant bit of the upper address byte must be zero to enable/disable cart RAM. For example the following
// addresses can be used to enable/disable cart RAM: 0000-00FF, 0200-02FF, 0400-04FF, ..., 1E00-1EFF. The suggested address
// range to use for MBC2 ram enable/disable is 0000-00FF.

// 2000-3FFF - ROM Bank Number (Write Only)
// Writing a value (XXXXBBBB - X = Don't cares, B = bank select bits) into 2000-3FFF area will select an appropriate ROM
// bank at 4000-7FFF.
// The least significant bit of the upper address byte must be one to select a ROM bank. For example the following
// addresses can be used to select a ROM bank: 2100-21FF, 2300-23FF, 2500-25FF, ..., 3F00-3FFF. The suggested address
// range to use for MBC2 rom bank selection is 2100-21FF.

use std::path::{Path, PathBuf};

use crate::cartridges::{Cartridge, Stable};
use crate::memory::Memory;

pub struct Mbc2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_enabled: bool,
    save_path: PathBuf,
}

impl Mbc2 {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>, save_path: impl AsRef<Path>) -> Mbc2 {
        Mbc2 {
            rom,
            ram,
            rom_bank: 1,
            ram_enabled: false,
            save_path: PathBuf::from(save_path.as_ref()),
        }
    }
}

impl Memory for Mbc2 {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 00 (Read Only)
            0x0000..=0x3FFF => self.rom[addr as usize],
            // ROM Bank 01-7F (Read Only)
            0x4000..=0x7FFF => {
                let index = self.rom_bank * 0x4000 + addr as usize - 0x4000;
                self.rom[index]
            }
            // 512x4bits RAM, built-in into the MBC2 chip (Read/Write)
            0xA000..=0xA1FF => {
                if self.ram_enabled {
                    self.ram[(addr - 0xA000) as usize]
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        // Only the lower 4 bits of the "bytes" in this memory area are used.
        let value = value & 0x0F;
        match addr {
            // 512x4bits RAM, built-in into the MBC2 chip (Read/Write)
            0xA000..=0xA1FF => {
                if self.ram_enabled {
                    self.ram[(addr - 0xA000) as usize] = value;
                }
            }
            // RAM Enable (Write Only)
            0x0000..=0x1FFF => {
                if addr & 0x0100 == 0 {
                    self.ram_enabled = value == 0x0A;
                }
            }
            // ROM Bank Number (Write Only)
            0x2000..=0x3FFF => {
                if addr & 0x0100 != 0 {
                    self.rom_bank = value as usize;
                }
            }
            _ => {}
        }
    }
}

impl Stable for Mbc2 {
    fn save(&self) {
        self.save_to_file(self.save_path.clone(), &self.ram);
    }
}

impl Cartridge for Mbc2 {}
