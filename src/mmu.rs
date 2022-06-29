// https://mgba-emu.github.io/gbdoc/#memory-map

use std::path::Path;

use crate::apu::APU;
use crate::cartridges::{self, Cartridge};
use crate::memory::Memory;
use crate::ppu::PPU;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Speed {
    Normal = 0x01,
    Double = 0x02,
}

pub struct MMU {
    cartridge: Box<dyn Cartridge>,
    apu: Option<APU>,
    ppu: PPU,
    speed: Speed,
    interruptes_asserted: u8, // IF
    interruptes_enabled: u8,  // IE
}

impl MMU {
    pub fn new(path: impl AsRef<Path>) -> MMU {
        MMU {
            cartridge: cartridges::new(path),
            apu: None,
            ppu: PPU::new(),
            speed: Speed::Normal,
            interruptes_asserted: 0x00,
            interruptes_enabled: 0x00,
        }
    }
}

impl Memory for MMU {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // External bus (ROM region)
            0x0000..=0x7FFF => self.cartridge.get_byte(addr),
            // VRAM
            0x8000..=0x9FFF => self.ppu.get_byte(addr),
            // External bus (RAM region)
            0xA000..=0xBFFF => self.cartridge.get_byte(addr),
            // WRAM
            0xC000..=0xDFFF => panic!("wram: not implemented"),
            // ECHO (WRAM secondary mapping)
            0xE000..=0xFDFF => panic!("echo: not implemented"),
            // Object Attribute Memory (OAM)
            0xFE00..=0xFE9F => self.ppu.get_byte(addr),
            // Invalid OAM region (behavior varies per revision)
            0xFEA0..=0xFEFF => 0x00,
            // Memory mapped I/O
            0xFF00..=0xFF7F => {
                match addr {
                    // Sound Controller (APU)
                    0xFF10..=0xFF3F => match &self.apu {
                        Some(apu) => apu.get_byte(addr),
                        None => 0x00,
                    },
                    _ => 0x00,
                }
            }
            // High RAM (HRAM)
            0xFF80..=0xFFFE => panic!("hram: not implemented"),
            // IE Register
            0xFFFF => self.interruptes_enabled,
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // External bus (ROM region)
            0x0000..=0x7FFF => self.cartridge.set_byte(addr, value),
            // VRAM
            0x8000..=0x9FFF => self.ppu.set_byte(addr, value),
            // External bus (RAM region)
            0xA000..=0xBFFF => self.cartridge.set_byte(addr, value),
            // WRAM
            0xC000..=0xDFFF => panic!("wram: not implemented"),
            // ECHO (WRAM secondary mapping)
            0xE000..=0xFDFF => panic!("echo: not implemented"),
            // Object Attribute Memory (OAM)
            0xFE00..=0xFE9F => self.ppu.set_byte(addr, value),
            // Invalid OAM region (behavior varies per revision)
            0xFEA0..=0xFEFF => {}
            // Memory mapped I/O
            0xFF00..=0xFF7F => {
                match addr {
                    // Sound Controller (APU)
                    0xFF10..=0xFF3F => self
                        .apu
                        .as_mut()
                        .map_or((), |apu| apu.set_byte(addr, value)),
                    _ => {}
                }
            }
            // High RAM (HRAM)
            0xFF80..=0xFFFE => panic!("hram: not implemented"),
            // IE Register
            0xFFFF => self.interruptes_enabled = value,
        }
    }
}
