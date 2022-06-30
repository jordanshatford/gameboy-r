// https://mgba-emu.github.io/gbdoc/#memory-map

use std::path::Path;

use crate::apu::APU;
use crate::cartridges::{self, Cartridge, CartridgeMode};
use crate::joypad::Joypad;
use crate::memory::Memory;
use crate::ppu::PPU;
use crate::serial::Serial;
use crate::timer::Timer;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Speed {
    Normal = 0x01,
    Double = 0x02,
}

// FF0F - IF - Interrupt Flag (R/W)
// Bit 0: VBlank   Interrupt Request (INT $40)  (1=Request)
// Bit 1: LCD STAT Interrupt Request (INT $48)  (1=Request)
// Bit 2: Timer    Interrupt Request (INT $50)  (1=Request)
// Bit 3: Serial   Interrupt Request (INT $58)  (1=Request)
// Bit 4: Joypad   Interrupt Request (INT $60)  (1=Request)
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InterruptFlag {
    VBlank = 0b0000_0001,
    LCDStat = 0b0000_0010,
    Timer = 0b0000_0100,
    Serial = 0b0000_1000,
    Joypad = 0b0001_0000,
    None = 0b0000_0000,
}

// This value (0x7F) is based on the address space available for HRAM (0xFFFE - 0xFF80)
const HRAM_SIZE: usize = 0x7F;

pub struct MMU {
    cartridge: Box<dyn Cartridge>,
    mode: CartridgeMode,
    apu: Option<APU>,
    ppu: PPU,
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    speed: Speed,
    // This value (0x7F) is based on the address space available for HRAM (0xFFFE - 0xFF80)
    hram: [u8; HRAM_SIZE],
    interruptes_asserted: u8, // IF
    // FFFF - IE - Interrupt Enable (R/W)
    // Bit 0: VBlank   Interrupt Enable  (INT $40)  (1=Enable)
    // Bit 1: LCD STAT Interrupt Enable  (INT $48)  (1=Enable)
    // Bit 2: Timer    Interrupt Enable  (INT $50)  (1=Enable)
    // Bit 3: Serial   Interrupt Enable  (INT $58)  (1=Enable)
    // Bit 4: Joypad   Interrupt Enable  (INT $60)  (1=Enable)
    interruptes_enabled: u8, // IE
}

impl MMU {
    pub fn new(path: impl AsRef<Path>) -> MMU {
        let cartridge = cartridges::new(path);
        let cartridge_mode = cartridge.get_cartridge_mode();
        MMU {
            cartridge: cartridge,
            mode: cartridge_mode,
            apu: None,
            ppu: PPU::new(cartridge_mode),
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::new(),
            speed: Speed::Normal,
            hram: [0x00; HRAM_SIZE],
            interruptes_asserted: 0x00,
            interruptes_enabled: 0x00,
        }
    }

    pub fn run_cycles(&mut self, cycles: u32) -> u32 {
        let cpu_divider = self.speed as u32;
        let vram_cycles = 0; // TODO calculate  dma (HDMA, GDMA)
        let ppu_cycles = cycles / cpu_divider + vram_cycles;
        let cpu_cycles = cycles + vram_cycles * cpu_divider;
        self.timer.run_cycles(cpu_cycles);
        self.ppu.run_cycles(ppu_cycles);
        self.apu
            .as_mut()
            .map_or((), |apu| apu.run_cycles(ppu_cycles));
        ppu_cycles
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
                    // P1/JOYP - Joypad (R/W)
                    0xFF00 => self.joypad.get_byte(addr),
                    // SB - Serial transfer data (R/W)
                    // SC - Serial Transfer Control (R/W)
                    0xFF01..=0xFF02 => self.serial.get_byte(addr),
                    // Timer and Divider Registers
                    0xff04..=0xff07 => self.timer.get_byte(addr),
                    // IF - Interrupt Flag (R/W)
                    0xFF0F => self.interruptes_asserted,
                    // Sound Controller (APU)
                    0xFF10..=0xFF3F => match &self.apu {
                        Some(apu) => apu.get_byte(addr),
                        None => 0x00,
                    },
                    // LCD Control Register, LCD Status Register,  LCD Position and Scrolling, LCD Monochrome Palettes
                    0xFF40..=0xFF45 | 0xFF47..=0xFF4b => self.ppu.get_byte(addr),
                    // KEY1 - CGB Mode Only - Prepare Speed Switch
                    0xFF4D => panic!("speed switch: not implemented"),
                    // LCD VRAM Bank (CGB only)
                    0xFF4F => self.ppu.get_byte(addr),
                    // LCD VRAM DMA Transfers (CGB only)
                    0xFF51..=0xFF55 => panic!("hdma: not implemented"),
                    // LCD Color Palettes (CGB only)
                    0xFF68..=0xFF6b => self.ppu.get_byte(addr),
                    // SVBK - CGB Mode Only - WRAM Bank
                    0xFF70 => panic!("wram bank: not implemented"),
                    _ => 0x00,
                }
            }
            // High RAM (HRAM)
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
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
                    // P1/JOYP - Joypad (R/W)
                    0xFF00 => self.joypad.set_byte(addr, value),
                    // SB - Serial transfer data (R/W)
                    // SC - Serial Transfer Control (R/W)
                    0xFF01..=0xFF02 => self.serial.set_byte(addr, value),
                    // Timer and Divider Registers
                    0xff04..=0xff07 => self.timer.set_byte(addr, value),
                    // IF - Interrupt Flag (R/W)
                    0xFF0F => self.interruptes_asserted = value,
                    // Sound Controller (APU)
                    0xFF10..=0xFF3F => self
                        .apu
                        .as_mut()
                        .map_or((), |apu| apu.set_byte(addr, value)),
                    // LCD Control Register, LCD Status Register,  LCD Position and Scrolling, LCD Monochrome Palettes
                    0xFF40..=0xFF45 | 0xFF47..=0xFF4b => self.ppu.set_byte(addr, value),
                    // KEY1 - CGB Mode Only - Prepare Speed Switch
                    0xFF4D => panic!("speed switch: not implemented"),
                    // LCD VRAM Bank (CGB only)
                    0xFF4F => self.ppu.set_byte(addr, value),
                    // LCD VRAM DMA Transfers (CGB only)
                    0xFF51..=0xFF55 => panic!("hdma: not implemented"),
                    // LCD Color Palettes (CGB only)
                    0xFF68..=0xFF6b => self.ppu.set_byte(addr, value),
                    // SVBK - CGB Mode Only - WRAM Bank
                    0xFF70 => panic!("wram bank: not implemented"),
                    _ => {}
                }
            }
            // High RAM (HRAM)
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            // IE Register
            0xFFFF => self.interruptes_enabled = value,
        }
    }
}
