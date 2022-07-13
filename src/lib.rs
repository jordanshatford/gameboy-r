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

pub struct Gameboy {
    mmu: Rc<RefCell<mmu::Mmu>>,
    cpu: cpu::RealTimeCpu,
}

impl Gameboy {
    pub fn new(rom: Vec<u8>, save_path: impl AsRef<Path>, skip_checks: bool) -> Gameboy {
        let cartridge = cartridges::new(rom, save_path, skip_checks);
        let cartridge_mode = cartridge.get_mode();
        let mmu = Rc::new(RefCell::new(mmu::Mmu::new(cartridge)));
        let cpu = cpu::RealTimeCpu::new(cartridge_mode, mmu.clone());
        Gameboy { mmu, cpu }
    }

    pub fn step(&mut self) -> u32 {
        if self.mmu.borrow().get_byte(self.cpu.cpu.registers.pc) == 0x10 {
            self.mmu.borrow_mut().perform_speed_switch();
        }
        let cycles = self.cpu.run();
        self.mmu.borrow_mut().run_cycles(cycles);
        cycles
    }

    pub fn save(&mut self) {
        self.mmu.borrow_mut().cartridge.save();
    }

    pub fn get_title(&self) -> String {
        let rom_name = self.mmu.borrow().cartridge.get_title();
        format!("Gameboy R - {}", rom_name)
    }

    pub fn get_screen_dimensions(&self) -> (usize, usize) {
        (ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT)
    }

    pub fn has_screen_updated(&mut self) -> bool {
        let result = self.mmu.borrow().ppu.vblank;
        self.mmu.borrow_mut().ppu.vblank = false;
        result
    }

    pub fn get_screen_data(&self) -> [[ppu::Pixel; ppu::SCREEN_WIDTH]; ppu::SCREEN_HEIGHT] {
        self.mmu.borrow().ppu.data
    }

    pub fn can_take_input(&mut self) -> bool {
        self.cpu.flip()
    }

    pub fn handle_keydown(&mut self, button: GameboyButton) {
        let key = self.get_joypad_key(button);
        self.mmu.borrow_mut().joypad.keydown(key);
    }

    pub fn handle_keyup(&mut self, button: GameboyButton) {
        let key = self.get_joypad_key(button);
        self.mmu.borrow_mut().joypad.keyup(key);
    }

    fn get_joypad_key(&self, button: GameboyButton) -> joypad::JoypadKey {
        match button {
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
