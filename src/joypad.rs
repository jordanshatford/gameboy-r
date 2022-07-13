// Joypad Input
// FF00 - P1/JOYP - Joypad (R/W)
// The eight gameboy buttons/direction keys are arranged in form of a 2x4 matrix. Select either button or direction keys by
// writing to this register, then read-out bit 0-3.
//   Bit 7 - Not used
//   Bit 6 - Not used
//   Bit 5 - P15 Select Button Keys      (0=Select)
//   Bit 4 - P14 Select Direction Keys   (0=Select)
//   Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
//   Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
//   Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
//   Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
// Note: Most programs are repeatedly reading from this port several times (the first reads used as short delay, allowing the
// inputs to stabilize, and only the value from the last read actually used).
// Usage in SGB software
// Beside for normal joypad input, SGB games mis-use the joypad register to output SGB command packets to the SNES, also, SGB
// programs may read out gamepad states from up to four different joypads which can be connected to the SNES.
// See SGB description for details.
// INT 60 - Joypad Interrupt
// Joypad interrupt is requested when any of the above Input lines changes from High to Low. Generally this should happen when
// a key becomes pressed (provided that the button/direction key is enabled by above Bit4/5), however, because of switch bounce,
// one or more High to Low transitions are usually produced both when pressing or releasing a key.
// Using the Joypad Interrupt
// It's more or less useless for programmers, even when selecting both buttons and direction keys simultaneously it still cannot
// recognize all keystrokes, because in that case a bit might be already held low by a button key, and pressing the corresponding
// direction key would thus cause no difference. The only meaningful purpose of the keystroke interrupt would be to terminate STOP
// (low power) standby state.
// Also, the joypad interrupt does not appear to work with CGB and GBA hardware (the STOP function can be still terminated by
// joypad keystrokes though).

use crate::memory::Memory;
use crate::mmu::InterruptFlag;

#[derive(Debug, Copy, Clone)]
pub enum JoypadKey {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Up = 0b0000_0100,
    Down = 0b0000_1000,
    A = 0b0001_0000,
    B = 0b0010_0000,
    Select = 0b0100_0000,
    Start = 0b1000_0000,
}

#[derive(Debug, Copy, Clone)]
pub struct Joypad {
    matrix: u8,
    select: u8,
    pub interrupt: u8,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            matrix: 0xFF,
            select: 0x00,
            interrupt: InterruptFlag::None as u8,
        }
    }

    pub fn keydown(&mut self, key: JoypadKey) {
        self.matrix &= !(key as u8);
        self.interrupt |= InterruptFlag::Joypad as u8;
    }

    pub fn keyup(&mut self, key: JoypadKey) {
        self.matrix |= key as u8;
    }

    #[cfg(test)]
    fn get_matrix(&mut self) -> u8 {
       self.matrix
    }

    #[cfg(test)]
    fn get_select(&mut self) -> u8 {
        self.select
    }
}

impl Default for Joypad {
    fn default() -> Joypad {
        Joypad::new()
    }
}

impl Memory for Joypad {
    fn get_byte(&self, addr: u16) -> u8 {
        assert_eq!(addr, 0xFF00);
        if (self.select & 0b0001_0000) == 0x00 {
            return self.select | (self.matrix & 0x0F);
        }
        if (self.select & 0b0010_0000) == 0x00 {
            return self.select | (self.matrix >> 4);
        }
        self.select
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        assert_eq!(addr, 0xFF00);
        self.select = value
    }
}

#[cfg(test)]
mod test {
    use super::InterruptFlag;
    use super::{Joypad, JoypadKey};
    use super::Memory;

    #[test]
    fn joypad_functionality() {
        let mut joypad = Joypad::new();
        assert_eq!(joypad.interrupt, InterruptFlag::None as u8);
        assert_eq!(joypad.get_matrix(), 0xFF);
        assert_eq!(joypad.get_select(), 0x00);
        joypad.keydown(JoypadKey::A);
        assert_eq!(joypad.interrupt, InterruptFlag::Joypad as u8);
        assert_eq!(joypad.get_matrix(), 0b1110_1111);
        joypad.keyup(JoypadKey::A);
        assert_eq!(joypad.interrupt, InterruptFlag::Joypad as u8);
        assert_eq!(joypad.get_matrix(), 0xFF);
    }

    #[test]
    #[should_panic]
    fn out_of_range_get_addr() {
        let joypad = Joypad::new();
        joypad.get_byte(0x0000);
    }

    #[test]
    #[should_panic]
    fn out_of_range_set_addr() {
        let mut joypad = Joypad::new();
        joypad.set_byte(0x0000, 0x00);
    }
}
