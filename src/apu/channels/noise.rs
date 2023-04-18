use std::cell::RefCell;
use std::rc::Rc;

use crate::apu::channels::{Blip, Channel, LengthCounter, Register, VolumeEnvelope};
use crate::clock::Clock;
use crate::memory::Memory;

// The noise channel's frequency timer period is set by a base divisor shifted left some number of bits.
//
//   Divisor code   Divisor
//   -----------------------
//      0             8
//      1            16
//      2            32
//      3            48
//      4            64
//      5            80
//      6            96
//      7           112
//
// Noise
// FF1F ---- ---- Not used
// NR41 FF20 --LL LLLL Length load (64-L)
// NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
// NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
// NR44 FF23 TL-- ---- Trigger, Length enable
pub struct NoiseChannel {
    pub register: Rc<RefCell<Register>>,
    pub length_counter: LengthCounter,
    pub volume_envelope: VolumeEnvelope,
    pub blip: Blip,
    clock: Clock,
    lfsr: Lfsr,
}

impl NoiseChannel {
    pub fn new(sample_rate: u32) -> NoiseChannel {
        let register = Rc::new(RefCell::new(Register::new(Channel::Noise)));
        NoiseChannel {
            register: register.clone(),
            length_counter: LengthCounter::new(register.clone()),
            volume_envelope: VolumeEnvelope::new(register.clone()),
            blip: Blip::new(sample_rate),
            clock: Clock::new(4096),
            lfsr: Lfsr::new(register),
        }
    }

    pub fn run_cycles(&mut self, cycles: u32) {
        for _ in 0..self.clock.run_cycles(cycles) {
            let amplitude =
                if !self.register.borrow().get_trigger() || self.volume_envelope.volume == 0 {
                    0x00
                } else if self.lfsr.next() {
                    i32::from(self.volume_envelope.volume)
                } else {
                    -i32::from(self.volume_envelope.volume)
                };
            self.blip
                .set(self.blip.from.wrapping_add(self.clock.period), amplitude);
        }
    }
}

impl Memory for NoiseChannel {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF1F => self.register.borrow().nrx0,
            // FF20 - NR41 - Channel 4 Sound Length (R/W)
            0xFF20 => self.register.borrow().nrx1,
            // FF21 - NR42 - Channel 4 Volume Envelope (R/W)
            0xFF21 => self.register.borrow().nrx2,
            // FF22 - NR43 - Channel 4 Polynomial Counter (R/W)
            0xFF22 => self.register.borrow().nrx3,
            // FF23 - NR44 - Channel 4 Counter/consecutive; Inital (R/W)
            0xFF23 => self.register.borrow().nrx4,
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1F => self.register.borrow_mut().nrx0 = value,
            // FF20 - NR41 - Channel 4 Sound Length (R/W)
            0xFF20 => {
                self.register.borrow_mut().nrx1 = value;
                self.length_counter.n = self.register.borrow().get_length_load();
            }
            // FF21 - NR42 - Channel 4 Volume Envelope (R/W)
            0xFF21 => self.register.borrow_mut().nrx2 = value,
            // FF22 - NR43 - Channel 4 Polynomial Counter (R/W)
            0xFF22 => {
                self.register.borrow_mut().nrx3 = value;
                self.clock.period = self.register.borrow().get_clock_period();
            }
            // FF23 - NR44 - Channel 4 Counter/consecutive; Inital (R/W)
            0xFF23 => {
                self.register.borrow_mut().nrx4 = value;
                // Writing a value to NRx4 with bit 7 set causes the following things to occur:
                //  - Channel is enabled (see length counter).
                //  - If length counter is zero, it is set to 64 (256 for wave channel).
                //  - Volume envelope timer is reloaded with period.
                //  - Noise channel's LFSR bits are all set to 1.
                if self.register.borrow().get_trigger() {
                    self.length_counter.reload();
                    self.volume_envelope.reload();
                    self.lfsr.reload();
                }
            }
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }
}

// The linear feedback shift register (LFSR) generates a pseudo-random bit sequence. It has a 15-bit shift
// register with feedback. When clocked by the frequency timer, the low two bits (0 and 1) are XORed, all
// bits are shifted right by one, and the result of the XOR is put into the now-empty high bit. If width mode
// is 1 (NR43), the XOR result is ALSO put into bit 6 AFTER the shift, resulting in a 7-bit LFSR. The
// waveform output is bit 0 of the LFSR, INVERTED.
struct Lfsr {
    register: Rc<RefCell<Register>>,
    n: u16,
}

impl Lfsr {
    fn new(register: Rc<RefCell<Register>>) -> Lfsr {
        Lfsr {
            register,
            n: 0x0001,
        }
    }

    fn next(&mut self) -> bool {
        let shift = if self.register.borrow().get_width_mode_of_lfsr() {
            0x06
        } else {
            0x0E
        };
        let src = self.n;
        self.n <<= 1;
        let bit = ((src >> shift) ^ (self.n >> shift)) & 0x0001;
        self.n |= bit;
        (src >> shift) & 0x0001 != 0x0000
    }

    fn reload(&mut self) {
        self.n = 0x0001
    }
}
