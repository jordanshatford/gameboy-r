// https://mgba-emu.github.io/gbdoc/#memory-map

use std::path::Path;

use crate::apu::APU;
use crate::cartridges::{self, Cartridge, CartridgeMode};
use crate::joypad::Joypad;
use crate::memory::Memory;
use crate::ppu::hdma::{HDMAMode, HDMA};
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
// C000-CFFF   4KB Work RAM Bank 0 (WRAM)
// D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
const WRAM_SIZE: usize = 0x8000;
const WRAM_BANK_SIZE: usize = 0x1000;

pub struct MMU {
    cartridge: Box<dyn Cartridge>,
    apu: Option<APU>,
    ppu: PPU,
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    speed: Speed,
    prepare_speed_switch: bool,
    hdma: HDMA,
    hram: [u8; HRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    wram_bank: usize,
    interrupts_asserted: u8, // IF
    // FFFF - IE - Interrupt Enable (R/W)
    // Bit 0: VBlank   Interrupt Enable  (INT $40)  (1=Enable)
    // Bit 1: LCD STAT Interrupt Enable  (INT $48)  (1=Enable)
    // Bit 2: Timer    Interrupt Enable  (INT $50)  (1=Enable)
    // Bit 3: Serial   Interrupt Enable  (INT $58)  (1=Enable)
    // Bit 4: Joypad   Interrupt Enable  (INT $60)  (1=Enable)
    interrupts_enabled: u8, // IE
}

impl MMU {
    pub fn new(cartridge: Box<dyn Cartridge>) -> MMU {
        let cartridge_mode = cartridge.get_cartridge_mode();
        let mut mmu = MMU {
            cartridge: cartridge,
            apu: None,
            ppu: PPU::new(cartridge_mode),
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::new(),
            speed: Speed::Normal,
            prepare_speed_switch: false,
            hdma: HDMA::new(),
            hram: [0x00; HRAM_SIZE],
            wram: [0x00; WRAM_SIZE],
            wram_bank: 0x01,
            interrupts_asserted: InterruptFlag::None as u8,
            interrupts_enabled: 0x00,
        };
        // Set initial values
        mmu.set_byte(0xFF05, 0x00);
        mmu.set_byte(0xFF06, 0x00);
        mmu.set_byte(0xFF07, 0x00);
        mmu.set_byte(0xFF10, 0x80);
        mmu.set_byte(0xFF11, 0xBF);
        mmu.set_byte(0xFF12, 0xF3);
        mmu.set_byte(0xFF14, 0xBF);
        mmu.set_byte(0xFF16, 0x3F);
        mmu.set_byte(0xFF16, 0x3F);
        mmu.set_byte(0xFF17, 0x00);
        mmu.set_byte(0xFF19, 0xBF);
        mmu.set_byte(0xFF1A, 0x7F);
        mmu.set_byte(0xFF1B, 0xFF);
        mmu.set_byte(0xFF1C, 0x9F);
        mmu.set_byte(0xFF1E, 0xFF);
        mmu.set_byte(0xFF20, 0xFF);
        mmu.set_byte(0xFF21, 0x00);
        mmu.set_byte(0xFF22, 0x00);
        mmu.set_byte(0xFF23, 0xbF);
        mmu.set_byte(0xFF24, 0x77);
        mmu.set_byte(0xFF25, 0xF3);
        mmu.set_byte(0xFF26, 0xF1);
        mmu.set_byte(0xFF40, 0x91);
        mmu.set_byte(0xFF42, 0x00);
        mmu.set_byte(0xFF43, 0x00);
        mmu.set_byte(0xFF45, 0x00);
        mmu.set_byte(0xFF47, 0xFC);
        mmu.set_byte(0xFF48, 0xFF);
        mmu.set_byte(0xFF49, 0xFF);
        mmu.set_byte(0xFF4A, 0x00);
        mmu.set_byte(0xFF4B, 0x00);
        mmu
    }

    pub fn perform_speed_switch(&mut self) {
        if self.prepare_speed_switch {
            self.speed = if self.speed == Speed::Double {
                Speed::Normal
            } else {
                Speed::Double
            }
        }
        self.prepare_speed_switch = false;
    }

    pub fn run_cycles(&mut self, cycles: u32) -> u32 {
        let cpu_divider = self.speed as u32;
        let vram_cycles = 0; // TODO calculate  dma (HDMA, GDMA)
        let ppu_cycles = cycles / cpu_divider + vram_cycles;
        let cpu_cycles = cycles + vram_cycles * cpu_divider;

        self.timer.run_cycles(cpu_cycles);
        self.interrupts_asserted |= self.timer.interrupt;
        self.timer.interrupt = InterruptFlag::None as u8;

        self.interrupts_asserted |= self.joypad.interrupt;
        self.joypad.interrupt = InterruptFlag::None as u8;

        self.ppu.run_cycles(ppu_cycles);
        self.interrupts_asserted |= self.ppu.interrupt;
        self.ppu.interrupt = InterruptFlag::None as u8;

        self.apu
            .as_mut()
            .map_or((), |apu| apu.run_cycles(ppu_cycles));

        self.interrupts_asserted |= self.serial.interrupt;
        self.serial.interrupt = InterruptFlag::None as u8;

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
            0xC000..=0xDFFF => match addr {
                0xC000..=0xCFFF => self.wram[addr as usize - 0xC000],
                0xD000..=0xDFFF => {
                    self.wram[addr as usize - 0xD000 + WRAM_BANK_SIZE * self.wram_bank]
                }
                _ => 0x00,
            },
            // ECHO (WRAM secondary mapping)
            0xE000..=0xFDFF => match addr {
                0xE000..=0xEFFF => self.wram[addr as usize - 0xE000],
                0xF000..=0xFDFF => {
                    self.wram[addr as usize - 0xF000 + WRAM_BANK_SIZE * self.wram_bank]
                }
                _ => 0x00,
            },
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
                    0xFF04..=0xFF07 => self.timer.get_byte(addr),
                    // IF - Interrupt Flag (R/W)
                    0xFF0F => self.interrupts_asserted,
                    // Sound Controller (APU)
                    0xFF10..=0xFF3F => match &self.apu {
                        Some(apu) => apu.get_byte(addr),
                        None => 0x00,
                    },
                    // LCD Control Register, LCD Status Register,  LCD Position and Scrolling, LCD Monochrome Palettes
                    0xFF40..=0xFF45 | 0xFF47..=0xFF4b => self.ppu.get_byte(addr),
                    // KEY1 - CGB Mode Only - Prepare Speed Switch
                    0xFF4D => {
                        // Bit 7: Current Speed (0=Normal, 1=Double) (Read Only)
                        // Bit 0: Prepare Speed Switch (0=No, 1=Prepare) (Read/Write)
                        let current_speed_bit: u8 = match self.speed {
                            Speed::Double => 0b1000_0000,
                            Speed::Normal => 0b0000_0000,
                        };
                        let prepare_switch_bit: u8 = match self.prepare_speed_switch {
                            true => 0b0000_0001,
                            false => 0b0000_0000,
                        };
                        current_speed_bit | prepare_switch_bit
                    }
                    // LCD VRAM Bank (CGB only)
                    0xFF4F => self.ppu.get_byte(addr),
                    // LCD VRAM DMA Transfers (CGB only)
                    0xFF51..=0xFF55 => self.hdma.get_byte(addr),
                    // LCD Color Palettes (CGB only)
                    0xFF68..=0xFF6b => self.ppu.get_byte(addr),
                    // SVBK - CGB Mode Only - WRAM Bank
                    0xFF70 => self.wram_bank as u8,
                    _ => 0x00,
                }
            }
            // High RAM (HRAM)
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            // IE Register
            0xFFFF => self.interrupts_enabled,
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
            0xC000..=0xDFFF => match addr {
                0xC000..=0xCFFF => self.wram[addr as usize - 0xC000] = value,
                0xD000..=0xDFFF => {
                    self.wram[addr as usize - 0xD000 + WRAM_BANK_SIZE * self.wram_bank] = value
                }
                _ => {}
            },
            // ECHO (WRAM secondary mapping)
            0xE000..=0xFDFF => match addr {
                0xE000..=0xEFFF => self.wram[addr as usize - 0xE000] = value,
                0xF000..=0xFDFF => {
                    self.wram[addr as usize - 0xF000 + WRAM_BANK_SIZE * self.wram_bank] = value
                }
                _ => {}
            },
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
                    0xFF04..=0xFF07 => self.timer.set_byte(addr, value),
                    // IF - Interrupt Flag (R/W)
                    0xFF0F => self.interrupts_asserted = value,
                    // Sound Controller (APU)
                    0xFF10..=0xFF3F => self
                        .apu
                        .as_mut()
                        .map_or((), |apu| apu.set_byte(addr, value)),
                    // LCD Control Register, LCD Status Register,  LCD Position and Scrolling, LCD Monochrome Palettes
                    0xFF40..=0xFF45 | 0xFF47..=0xFF4b => self.ppu.set_byte(addr, value),
                    // KEY1 - CGB Mode Only - Prepare Speed Switch
                    0xFF4D => {
                        // This register is used to prepare the gameboy to switch between CGB Double Speed Mode and Normal Speed Mode.
                        // The actual speed switch is performed by executing a STOP command after Bit 0 has been set. After that Bit 0
                        // will be cleared automatically, and the gameboy will operate at the 'other' speed.
                        self.prepare_speed_switch = (value & 0b0000_0001) == 0b0000_0001;
                    }
                    // LCD VRAM Bank (CGB only)
                    0xFF4F => self.ppu.set_byte(addr, value),
                    // LCD VRAM DMA Transfers (CGB only)
                    0xFF51..=0xFF55 => self.hdma.set_byte(addr, value),
                    // LCD Color Palettes (CGB only)
                    0xFF68..=0xFF6B => self.ppu.set_byte(addr, value),
                    // SVBK - CGB Mode Only - WRAM Bank
                    0xFF70 => {
                        // Writing a value of 01h-07h will select Bank 1-7, writing a value of 00h
                        // will select Bank 1 either.
                        // Bit 0-2  Select WRAM Bank (Read/Write)
                        self.wram_bank = match value & 0x07 {
                            0x00 => 1,
                            _ => value as usize,
                        };
                    }
                    _ => {}
                }
            }
            // High RAM (HRAM)
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            // IE Register
            0xFFFF => self.interrupts_enabled = value,
        }
    }
}
