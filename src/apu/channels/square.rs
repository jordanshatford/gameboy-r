use std::cell::RefCell;
use std::rc::Rc;

use crate::apu::channels::{Blip, Channel, LengthCounter, Register, VolumeEnvelope};
use crate::clock::Clock;
use crate::memory::Memory;

// A square channel's frequency timer period is set to (2048-frequency)*4. Four duty cycles are available,
// each waveform taking 8 frequency timer clocks to cycle through:
//
//  Duty   Waveform    Ratio
//  -------------------------
//  0      00000001    12.5%
//  1      10000001    25%
//  2      10000111    50%
//  3      01111110    75%
//
// Square 1
// NR10 FF10 -PPP NSSS Sweep period, negate, shift
// NR11 FF11 DDLL LLLL Duty, Length load (64-L)
// NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
// NR13 FF13 FFFF FFFF Frequency LSB
// NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
//
// Square 2
// FF15 ---- ---- Not used
// NR21 FF16 DDLL LLLL Duty, Length load (64-L)
// NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
// NR23 FF18 FFFF FFFF Frequency LSB
// NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
pub struct SquareChannel {
    pub register: Rc<RefCell<Register>>,
    pub clock: Clock,
    pub length_counter: LengthCounter,
    pub volume_envelope: VolumeEnvelope,
    pub frequency_sweep: FrequencySweep,
    pub blip: Blip,
    index: u8,
}

impl SquareChannel {
    pub fn new(sample_rate: u32, mode: Channel) -> SquareChannel {
        let register = Rc::new(RefCell::new(Register::new(mode)));
        SquareChannel {
            register: register.clone(),
            clock: Clock::new(8192),
            length_counter: LengthCounter::new(register.clone()),
            volume_envelope: VolumeEnvelope::new(register.clone()),
            frequency_sweep: FrequencySweep::new(register),
            blip: Blip::new(sample_rate),
            index: 1,
        }
    }

    // This assumes no volume or sweep adjustments need to be done in the meantime
    pub fn run_cycles(&mut self, cycles: u32) {
        let waveform = self.get_waveform();
        let volume = i32::from(self.volume_envelope.volume);
        for _ in 0..self.clock.run_cycles(cycles) {
            let amplitude =
                if !self.register.borrow().get_trigger() || self.volume_envelope.volume == 0 {
                    0x00
                } else if (waveform >> self.index) & 0x01 != 0x00 {
                    volume
                } else {
                    -volume
                };
            self.blip
                .set(self.blip.from.wrapping_add(self.clock.period), amplitude);
            self.index = (self.index + 1) % 8;
        }
    }

    fn get_waveform(&self) -> u8 {
        // Duty   Waveform    Ratio
        // -------------------------
        // 0      00000001    12.5%
        // 1      10000001    25%
        // 2      10000111    50%
        // 3      01111110    75%
        match self.register.borrow().get_duty() {
            0 => 0b0000_0001,
            1 => 0b1000_0001,
            2 => 0b1000_0111,
            3 => 0b0111_1110,
            byte => panic!("apu: invalid duty {:#04X?}", byte),
        }
    }
}

impl Memory for SquareChannel {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // FF10 - NR10 - Channel 1 Sweep register (R/W)
            0xFF10 | 0xFF15 => self.register.borrow().nrx0,
            // FF11 - NR11 - Channel 1 Sound length/Wave pattern duty (R/W)
            // FF16 - NR21 - Channel 2 Sound Length/Wave Pattern Duty (R/W)
            0xFF11 | 0xFF16 => self.register.borrow().nrx1,
            // FF12 - NR12 - Channel 1 Volume Envelope (R/W)
            // FF17 - NR22 - Channel 2 Volume Envelope (R/W)
            0xFF12 | 0xFF17 => self.register.borrow().nrx2,
            // FF14 - NR14 - Channel 1 Frequency hi (R/W)
            // FF19 - NR24 - Channel 2 Frequency hi data (R/W)
            0xFF14 | 0xFF19 => self.register.borrow().nrx4,
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // FF10 - NR10 - Channel 1 Sweep register (R/W)
            0xFF10 | 0xFF15 => self.register.borrow_mut().nrx0 = value,
            // FF11 - NR11 - Channel 1 Sound length/Wave pattern duty (R/W)
            // FF16 - NR21 - Channel 2 Sound Length/Wave Pattern Duty (R/W)
            0xFF11 | 0xFF16 => {
                self.register.borrow_mut().nrx1 = value;
                self.length_counter.n = self.register.borrow().get_length_load();
            }
            // FF12 - NR12 - Channel 1 Volume Envelope (R/W)
            // FF17 - NR22 - Channel 2 Volume Envelope (R/W)
            0xFF12 | 0xFF17 => self.register.borrow_mut().nrx2 = value,
            // FF13 - NR13 - Channel 1 Frequency lo (Write Only)
            // FF18 - NR23 - Channel 2 Frequency lo data (W)
            0xFF13 | 0xFF18 => {
                self.register.borrow_mut().nrx3 = value;
                self.clock.period = self.register.borrow().get_clock_period();
            }
            // FF14 - NR14 - Channel 1 Frequency hi (R/W)
            // FF19 - NR24 - Channel 2 Frequency hi data (R/W)
            0xFF14 | 0xFF19 => {
                self.register.borrow_mut().nrx4 = value;
                self.clock.period = self.register.borrow().get_clock_period();
                // Writing a value to NRx4 with bit 7 set causes the following things to occur:
                //  - Channel is enabled (see length counter).
                //  - If length counter is zero, it is set to 64 (256 for wave channel).
                //  - Frequency timer is reloaded with period.
                //  - Volume envelope timer is reloaded with period.
                //  - Square 1's sweep does several things (see frequency sweep).
                if self.register.borrow().get_trigger() {
                    self.length_counter.reload();
                    self.volume_envelope.reload();
                    if self.register.borrow().channel == Channel::Square1 {
                        self.frequency_sweep.reload();
                    }
                }
            }
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }
}

// The first square channel has a frequency sweep unit, controlled by NR10. This has a timer, internal enabled flag,
// and frequency shadow register. It can periodically adjust square 1's frequency up or down.
// During a trigger event, several things occur:
//
//   - Square 1's frequency is copied to the shadow register.
//   - The sweep timer is reloaded.
//   - The internal enabled flag is set if either the sweep period or shift are non-zero, cleared otherwise.
//   - If the sweep shift is non-zero, frequency calculation and the overflow check are performed immediately.
//
// Frequency calculation consists of taking the value in the frequency shadow register, shifting it right by sweep
// shift, optionally negating the value, and summing this with the frequency shadow register to produce a new
// frequency. What is done with this new frequency depends on the context.
//
// The overflow check simply calculates the new frequency and if this is greater than 2047, square 1 is disabled.
// The sweep timer is clocked at 128 Hz by the frame sequencer. When it generates a clock and the sweep's internal
// enabled flag is set and the sweep period is not zero, a new frequency is calculated and the overflow check is
// performed. If the new frequency is 2047 or less and the sweep shift is not zero, this new frequency is written
// back to the shadow frequency and square 1's frequency in NR13 and NR14, then frequency calculation and
// overflow check are run AGAIN immediately using this new value, but this second new frequency is not written back.
// Square 1's frequency can be modified via NR13 and NR14 while sweep is active, but the shadow frequency won't be
// affected so the next time the sweep updates the channel's frequency this modification will be lost.
pub struct FrequencySweep {
    register: Rc<RefCell<Register>>,
    clock: Clock,
    enable: bool,
    shadow: u16,
    new_frequency: u16,
}

impl FrequencySweep {
    fn new(register: Rc<RefCell<Register>>) -> FrequencySweep {
        FrequencySweep {
            register,
            clock: Clock::new(8),
            enable: false,
            shadow: 0x0000,
            new_frequency: 0x0000,
        }
    }

    fn reload(&mut self) {
        self.shadow = self.register.borrow().get_frequency();
        let period = self.register.borrow().get_sweep_period();
        // The volume envelope and sweep timers treat a period of 0 as 8.
        self.clock.period = if period == 0 { 8 } else { u32::from(period) };
        self.enable = period != 0x00 || self.register.borrow().get_shift() != 0x00;
        if self.register.borrow().get_shift() != 0x00 {
            self.frequency_calculation();
            self.overflow_check();
        }
    }

    fn frequency_calculation(&mut self) {
        let offset = self.shadow >> self.register.borrow().get_shift();
        if self.register.borrow().get_negate() {
            self.new_frequency = self.shadow.wrapping_sub(offset);
        } else {
            self.new_frequency = self.shadow.wrapping_add(offset);
        }
    }

    fn overflow_check(&mut self) {
        // If this is greater than 2047, square 1 is disabled.
        if self.new_frequency > 2047 {
            self.register.borrow_mut().set_trigger(false);
        }
    }

    pub fn next(&mut self) {
        if !self.enable || self.register.borrow().get_sweep_period() == 0 {
            return;
        }
        if self.clock.run_cycles(1) == 0x00 {
            return;
        }
        self.frequency_calculation();
        self.overflow_check();
        // If the new frequency is 2047 or less and the sweep shift is not zero, this new frequency is written
        // back to the shadow frequency and square 1's frequency in NR13 and NR14, then frequency calculation and
        // overflow check are run AGAIN immediately using this new value, but this second new frequency is not
        // written back.
        if self.new_frequency <= 2047 && self.register.borrow().get_shift() != 0 {
            self.register.borrow_mut().set_frequency(self.new_frequency);
            self.shadow = self.new_frequency;
            self.frequency_calculation();
            self.overflow_check();
        }
    }
}
