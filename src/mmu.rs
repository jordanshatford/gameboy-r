// https://mgba-emu.github.io/gbdoc/#memory-map

use crate::cartridge::Cartridge;
use crate::memory::Memory;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Speed {
    Normal = 0x01,
    Double = 0x02,
}

pub struct MMU {
    cartridge: Cartridge,
    speed: Speed,
    interruptes_asserted: u8, // IF
    interruptes_enabled: u8, // IE
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            cartridge: Cartridge::new(),
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
            0x8000..=0x9FFF => panic!("vram: not implemented"),
            // External bus (RAM region)
            0xA000..=0xBFFF => self.cartridge.get_byte(addr),
            // WRAM
            0xC000..=0xDFFF => panic!("wram: not implemented"),
            // ECHO (WRAM secondary mapping)
            0xE000..=0xFDFF => panic!("echo: not implemented"),
            // Object Attribute Memory (OAM)
            0xFE00..=0xFE9F => panic!("oam: not implemented"),
            // Invalid OAM region (behavior varies per revision)
            0xFEA0..=0xFEFF => 0x00,
            // Memory mapped I/O
            0xFF00..=0xFF7F => panic!("memory i/o: not implemented"),
            // High RAM (HRAM)
            0xFF80..=0xFFFE => panic!("hram: not implemented"),
            // IE Register
            0xFFFF => self.interruptes_enabled,
            _ => panic!("invalid mmu address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // External bus (ROM region)
            0x0000..=0x7FFF => self.cartridge.set_byte(addr, value),
            // VRAM
            0x8000..=0x9FFF => panic!("vram: not implemented"),
            // External bus (RAM region)
            0xA000..=0xBFFF => self.cartridge.set_byte(addr, value),
            // WRAM
            0xC000..=0xDFFF => panic!("wram: not implemented"),
            // ECHO (WRAM secondary mapping)
            0xE000..=0xFDFF => panic!("echo: not implemented"),
            // Object Attribute Memory (OAM)
            0xFE00..=0xFE9F => panic!("oam: not implemented"),
            // Invalid OAM region (behavior varies per revision)
            0xFEA0..=0xFEFF => {},
            // Memory mapped I/O
            0xFF00..=0xFF7F => panic!("memory i/o: not implemented"),
            // High RAM (HRAM)
            0xFF80..=0xFFFE => panic!("hram: not implemented"),
            // IE Register
            0xFFFF => self.interruptes_enabled = value,
            _ => panic!("invalid mmu set address {:#06X?} {}", addr, value),
        }
    }
}
