//! A Gameboy emulator written in Rust
//!
//! Gameboy R provides a simple to use implmentation of a Gameboy / Gameboy Color.

mod apu;
mod cartridges;
mod clock;
mod cpu;
mod joypad;
mod memory;
mod mmu;
mod ppu;
mod serial;
mod timer;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::memory::Memory;

/// Dimensions represent length and width of a screen.
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

/// GameboyButton represents each possibly button available.
///
/// This enum is used to provide users with a way to easily map any
/// key to a specific Gameboy button.
#[derive(Clone, Copy)]
pub enum GameboyButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl From<GameboyButton> for joypad::JoypadKey {
    fn from(value: GameboyButton) -> joypad::JoypadKey {
        match value {
            GameboyButton::A => joypad::JoypadKey::A,
            GameboyButton::B => joypad::JoypadKey::B,
            GameboyButton::Right => joypad::JoypadKey::Right,
            GameboyButton::Left => joypad::JoypadKey::Left,
            GameboyButton::Up => joypad::JoypadKey::Up,
            GameboyButton::Down => joypad::JoypadKey::Down,
            GameboyButton::Select => joypad::JoypadKey::Select,
            GameboyButton::Start => joypad::JoypadKey::Start,
        }
    }
}

/// Gameboy represents the physical device itself.
///
/// The Gameboy functionality is provided to the user through a set of
/// consise and useful helper functions. The user can not directly interact
/// with the Gameboy itself.
pub struct Gameboy {
    mmu: Rc<RefCell<mmu::Mmu>>,
    cpu: cpu::RealTimeCpu,
}

impl Gameboy {
    /// Create a new Gameboy by providing ROM data, a save path, and whether to skip checks.
    /// When the save path contains an existing save, that data will be loaded.
    pub fn new(rom: Vec<u8>, save_path: impl AsRef<Path>, skip_checks: bool) -> Gameboy {
        let cartridge = cartridges::new(rom, save_path, skip_checks);
        let cartridge_mode = cartridge.get_mode();
        let mmu = Rc::new(RefCell::new(mmu::Mmu::new(cartridge)));
        let cpu = cpu::RealTimeCpu::new(cartridge_mode, mmu.clone());
        Gameboy { mmu, cpu }
    }

    pub fn shutdown(&mut self) {
        self.save();
    }

    pub fn try_enable_audio(&mut self) -> bool {
        false
    }

    /// Perform one step (including CPU, MMU, PPU), returning the number of CPU cycles run.
    pub fn step(&mut self) -> u32 {
        if self.mmu.borrow().get_byte(self.cpu.cpu.registers.pc) == 0x10 {
            self.mmu.borrow_mut().perform_speed_switch();
        }
        let cycles = self.cpu.run();
        self.mmu.borrow_mut().run_cycles(cycles);
        cycles
    }

    /// Save the current state of the Gameboy.
    pub fn save(&mut self) {
        self.mmu.borrow_mut().cartridge.save();
    }

    /// Get the title of the currently loaded ROM.
    pub fn get_rom_title(&self) -> String {
        self.mmu.borrow().cartridge.get_title()
    }

    /// Get the dimensions of the Gameboys screen.
    pub fn get_screen_dimensions(&self) -> Dimensions {
        Dimensions {
            width: ppu::SCREEN_WIDTH,
            height: ppu::SCREEN_HEIGHT,
        }
    }

    /// Check whether the Gameboy screen has updated and should rerender. This
    /// will also reset the value to false once checked.
    pub fn has_screen_updated(&mut self) -> bool {
        let result = self.mmu.borrow().ppu.vblank;
        self.mmu.borrow_mut().ppu.vblank = false;
        result
    }

    /// Get the current data on the screen. This is returned as a 2D array of Pixel's.
    /// Where each Pixel represents the colors of a single pixel.
    ///
    /// ```
    /// struct Pixel {
    ///     r: u8,
    ///     g: u8,
    ///     b: u8
    /// }
    /// ```
    ///
    /// NOTE: when using a Gameboy without color support, all fields of the Pixel will be
    ///       the same.
    pub fn get_screen_data(&self) -> [ppu::Pixel; ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT] {
        self.mmu.borrow().ppu.data
    }

    /// Check whether the Gameboy is able to take input.
    pub fn can_take_input(&mut self) -> bool {
        self.cpu.flip()
    }

    /// Handle keydown on a GameboyButton.
    pub fn handle_keydown(&mut self, button: GameboyButton) {
        self.mmu.borrow_mut().joypad.keydown(button.into());
    }

    /// Handle keyup on a GameboyButton.
    pub fn handle_keyup(&mut self, button: GameboyButton) {
        self.mmu.borrow_mut().joypad.keyup(button.into());
    }
}
