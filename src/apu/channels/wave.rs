use std::cell::RefCell;
use std::rc::Rc;

use crate::apu::channels::{Channel, Register};
use crate::clock::Clock;
use crate::memory::Memory;

use crate::apu::channels::{Blip, LengthCounter};

// The wave channel plays a 32-entry wave table made up of 4-bit samples. Each byte encodes two samples, the first in
// the high bits. The wave channel has a sample buffer and position counter.
// The wave channel's frequency timer period is set to (2048-frequency)*2. When the timer generates a clock, the
// position counter is advanced one sample in the wave table, looping back to the beginning when it goes past the end,
// then a sample is read into the sample buffer from this NEW position.
// The DAC receives the current value from the upper/lower nibble of the sample buffer, shifted right by the volume
// control.
//
// Code   Shift   Volume
// -----------------------
// 0      4         0% (silent)
// 1      0       100%
// 2      1        50%
// 3      2        25%
//
// Wave RAM can only be properly accessed when the channel is disabled (see obscure behavior).

// Wave
// NR30 FF1A E--- ---- DAC power
// NR31 FF1B LLLL LLLL Length load (256-L)
// NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
// NR33 FF1D FFFF FFFF Frequency LSB
// NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB

pub struct WaveChannel {
    pub register: Rc<RefCell<Register>>,
    pub length_counter: LengthCounter,
    pub blip: Blip,
    clock: Clock,
    // This storage area holds 32 4-bit samples that are played back upper 4 bits first.
    wave_ram: [u8; 16],
    wave_index: usize,
}

impl WaveChannel {
    pub fn new(sample_rate: u32) -> WaveChannel {
        let register = Rc::new(RefCell::new(Register::new(Channel::Wave)));
        WaveChannel {
            register: register.clone(),
            clock: Clock::new(8192),
            length_counter: LengthCounter::new(register),
            blip: Blip::new(sample_rate),
            wave_ram: [0x00; 16],
            wave_index: 0x00,
        }
    }

    pub fn run_cycles(&mut self, cycles: u32) {
        let shift = self.get_shift();
        for _ in 0..self.clock.run_cycles(cycles) {
            let sample = if self.wave_index & 0x01 == 0x00 {
                // Take lower 4 bits
                self.wave_ram[self.wave_index / 2] & 0x0F
            } else {
                // Take upper 4 bits
                self.wave_ram[self.wave_index / 2] >> 4
            };
            let amplitude = if !self.register.borrow().get_trigger()
                || !self.register.borrow().get_dac_power()
            {
                0x00
            } else {
                i32::from(sample >> shift)
            };
            self.blip
                .set(self.blip.from.wrapping_add(self.clock.period), amplitude);
            self.wave_index = (self.wave_index + 1) % 32;
        }
    }

    fn get_shift(&self) -> u8 {
        // Code   Shift   Volume
        // -----------------------
        // 0      4         0% (silent)
        // 1      0       100%
        // 2      1        50%
        // 3      2        25%
        match self.register.borrow().get_volume_code() {
            0 => 4,
            1 => 0,
            2 => 1,
            3 => 2,
            byte => panic!("apu: invalid volume code {:#04X?}", byte),
        }
    }
}

impl Memory for WaveChannel {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // FF1A - NR30 - Channel 3 Sound on/off (R/W)
            0xFF1A => self.register.borrow().nrx0,
            // FF1B - NR31 - Channel 3 Sound Length
            0xFF1B => self.register.borrow().nrx1,
            // FF1C - NR32 - Channel 3 Select output level (R/W)
            0xFF1C => self.register.borrow().nrx2,
            // FF1D - NR33 - Channel 3 Frequency's lower data (W)
            0xFF1D => self.register.borrow().nrx3,
            // FF1E - NR34 - Channel 3 Frequency's higher data (R/W)
            0xFF1E => self.register.borrow().nrx4,
            // FF30-FF3F - Wave Pattern RAM
            0xFF30..=0xFF3F => self.wave_ram[addr as usize - 0xFF30],
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // FF1A - NR30 - Channel 3 Sound on/off (R/W)
            0xFF1A => self.register.borrow_mut().nrx0 = value,
            // FF1B - NR31 - Channel 3 Sound Length
            0xFF1B => {
                self.register.borrow_mut().nrx1 = value;
                self.length_counter.n = self.register.borrow().get_length_load();
            }
            // FF1C - NR32 - Channel 3 Select output level (R/W)
            0xFF1C => self.register.borrow_mut().nrx2 = value,
            // FF1D - NR33 - Channel 3 Frequency's lower data (W)
            0xFF1D => {
                self.register.borrow_mut().nrx3 = value;
                self.clock.period = self.register.borrow().get_clock_period();
            }
            // FF1E - NR34 - Channel 3 Frequency's higher data (R/W)
            0xFF1E => {
                self.register.borrow_mut().nrx4 = value;
                self.clock.period = self.register.borrow().get_clock_period();
                // Writing a value to NRx4 with bit 7 set causes the following things to occur:
                //  - Channel is enabled (see length counter).
                //  - If length counter is zero, it is set to 64 (256 for wave channel).
                //  - Wave channel's position is set to 0 but sample buffer is NOT refilled.
                if self.register.borrow().get_trigger() {
                    self.length_counter.reload();
                    self.wave_index = 0x00;
                }
            }
            // FF30-FF3F - Wave Pattern RAM
            0xFF30..=0xFF3F => self.wave_ram[addr as usize - 0xFF30] = value,
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }
}
