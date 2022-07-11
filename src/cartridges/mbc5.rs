// MBC5 (max 8MByte ROM and/or 128KByte RAM)

// 0000-3FFF - ROM Bank 00 (Read Only)
// Same as for MBC1.

// 4000-7FFF - ROM Bank 00-1FF (Read Only)
// Same as for MBC1, except that accessing up to bank 1E0h is supported now. Also, bank 0 is actually bank 0.

// A000-BFFF - RAM Bank 00-0F, if any (Read/Write)
// Same as for MBC1, except RAM sizes are 8KiB, 32KiB and 128KiB.

// 0000-1FFF - RAM Enable (Write Only)
// Mostly the same as for MBC1, a value of 0Ah will enable reading and writing to external RAM. A value of
// 00h will disable it.

// 2000-2FFF - Low 8 bits of ROM Bank Number (Write Only)
// The lower 8 bits of the ROM bank number goes here. Writing 0 will indeed give bank 0 on MBC5, unlike
// other MBCs.

// 3000-3FFF - High bit of ROM Bank Number (Write Only)
// The 9th bit of the ROM bank number goes here.

// 4000-5FFF - RAM Bank Number (Write Only)
// As for the MBC1s RAM Banking Mode, writing a value in range for 00h-0Fh maps the corresponding external
// RAM Bank (if any) into memory at A000-BFFF.

use std::path::{Path, PathBuf};

use crate::cartridges::{Cartridge, Stable};
use crate::memory::Memory;

pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    save_path: PathBuf,
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>, save_path: impl AsRef<Path>) -> Mbc5 {
        Mbc5 {
            rom,
            ram,
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            save_path: PathBuf::from(save_path.as_ref()),
        }
    }
}

impl Memory for Mbc5 {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 00 (Read Only)
            0x0000..=0x3FFF => self.rom[addr as usize],
            // ROM Bank 00-1FF (Read Only)
            0x4000..=0x7FFF => {
                let index = self.rom_bank * 0x4000 + addr as usize - 0x4000;
                self.rom[index]
            }
            // RAM Bank 00-0F, if any (Read/Write)
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let index = self.ram_bank * 0x2000 + addr as usize - 0xA000;
                    self.ram[index]
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM Bank 00-0F, if any (Read/Write)
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let index = self.ram_bank * 0x2000 + addr as usize - 0xa000;
                    self.ram[index] = value;
                }
            }
            // RAM Enable (Write Only)
            0x0000..=0x1FFF => {
                // Value of 0Ah will enable reading and writing to external RAM. A value of 00h will disable it.
                self.ram_enable = value & 0x0F == 0x0A;
            }
            // Low 8 bits of ROM Bank Number (Write Only)
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | (value as usize),
            // High bit of ROM Bank Number (Write Only)
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x0FF) | (((value & 0x01) as usize) << 8)
            }
            // RAM Bank Number (Write Only)
            0x4000..=0x5FFF => self.ram_bank = (value & 0x0F) as usize,
            _ => {}
        }
    }
}

impl Stable for Mbc5 {
    fn save(&self) {
        self.save_to_file(self.save_path.clone(), &self.ram);
    }
}

impl Cartridge for Mbc5 {}
