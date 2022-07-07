use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::cartridges;
use crate::cpu::RealTimeCPU;
use crate::joypad::JoypadKey;
use crate::memory::Memory;
use crate::mmu::MMU;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

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
    pub mmu: Rc<RefCell<MMU>>,
    pub cpu: RealTimeCPU,
}

impl Gameboy {
    pub fn new(path: impl AsRef<Path>) -> Gameboy {
        let cartridge = cartridges::new(path);
        let cartridge_mode = cartridge.get_mode();
        let mmu = Rc::new(RefCell::new(MMU::new(cartridge)));
        let cpu = RealTimeCPU::new(cartridge_mode, mmu.clone());
        Gameboy { mmu, cpu }
    }

    pub fn next(&mut self) -> u32 {
        if self.mmu.borrow().get_byte(self.cpu.cpu.registers.pc) == 0x10 {
            self.mmu.borrow_mut().perform_speed_switch();
        }
        let cycles = self.cpu.run();
        self.mmu.borrow_mut().run_cycles(cycles);
        cycles
    }

    pub fn save(&mut self) {
        self.mmu.borrow_mut().cartridge.sav();
    }

    pub fn get_title(&self) -> String {
        let rom_name = self.mmu.borrow().cartridge.get_title();
        format!("Game Boy R - {rom_name}")
    }

    pub fn get_screen_dimensions(&self) -> (usize, usize) {
        (SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    pub fn has_screen_updated(&mut self) -> bool {
        let result = self.mmu.borrow().ppu.vblank;
        self.mmu.borrow_mut().ppu.vblank = false;
        result
    }

    pub fn get_screen_data(&self) -> [[[u8; 3]; SCREEN_WIDTH]; SCREEN_HEIGHT] {
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

    fn get_joypad_key(&self, button: GameboyButton) -> JoypadKey {
        match button {
            GameboyButton::Right => JoypadKey::Right,
            GameboyButton::Left => JoypadKey::Left,
            GameboyButton::Up => JoypadKey::Up,
            GameboyButton::Down => JoypadKey::Down,
            GameboyButton::A => JoypadKey::A,
            GameboyButton::B => JoypadKey::B,
            GameboyButton::Select => JoypadKey::Select,
            GameboyButton::Start => JoypadKey::Start,
        }
    }
}
