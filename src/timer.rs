use crate::memory::Memory;
use crate::mmu::InterruptFlag;

#[derive(Debug, Copy, Clone)]
pub struct Timer {
    pub interrupt: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            interrupt: InterruptFlag::None as u8,
        }
    }

    pub fn run_cycle(&mut self, ticks: u32) {}
}

impl Memory for Timer {
    fn get_byte(&self, addr: u16) -> u8 {
        0x00
    }

    fn set_byte(&mut self, addr: u16, value: u8) {}
}
