pub mod noise;
pub mod square;
pub mod wave;

pub use noise::NoiseChannel;
pub use square::SquareChannel;
pub use wave::WaveChannel;

use std::cell::RefCell;
use std::rc::Rc;

use blip_buf::BlipBuf;

use crate::clock::Clock;

// Name Addr 7654 3210 Function
// -------------------------------------------------------------------
//        Square 1
// NR10 FF10 -PPP NSSS Sweep period, negate, shift
// NR11 FF11 DDLL LLLL Duty, Length load (64-L)
// NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
// NR13 FF13 FFFF FFFF Frequency LSB
// NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
//
//        Square 2
//      FF15 ---- ---- Not used
// NR21 FF16 DDLL LLLL Duty, Length load (64-L)
// NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
// NR23 FF18 FFFF FFFF Frequency LSB
// NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
//
//        Wave
// NR30 FF1A E--- ---- DAC power
// NR31 FF1B LLLL LLLL Length load (256-L)
// NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
// NR33 FF1D FFFF FFFF Frequency LSB
// NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
//
//        Noise
//      FF1F ---- ---- Not used
// NR41 FF20 --LL LLLL Length load (64-L)
// NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
// NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
// NR44 FF23 TL-- ---- Trigger, Length enable
//
//        Control/Status
// NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
// NR51 FF25 NW21 NW21 Left enables, Right enables
// NR52 FF26 P--- NW21 Power control/status, Channel length statuses
// -------------------------------------------------------------------
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Channel {
    Square1,
    Square2,
    Wave,
    Noise,
    Control,
}

pub struct Register {
    pub channel: Channel,
    pub nrx0: u8,
    pub nrx1: u8,
    pub nrx2: u8,
    pub nrx3: u8,
    pub nrx4: u8,
}

impl Register {
    pub fn new(channel: Channel) -> Register {
        let nrx1 = match channel {
            // NR11 FF11 DDLL LLLL Duty, Length load (64-L)
            // NR21 FF16 DDLL LLLL Duty, Length load (64-L)
            Channel::Square1 | Channel::Square2 => 0b0100_0000,
            // NR31 FF1B LLLL LLLL Length load (256-L)
            // NR41 FF20 --LL LLLL Length load (64-L)
            _ => 0b0000_0000,
        };
        Register {
            channel,
            nrx0: 0b0000_0000,
            nrx1,
            nrx2: 0b0000_0000,
            nrx3: 0b0000_0000,
            nrx4: 0b0000_0000,
        }
    }

    pub fn clear(&mut self) {
        self.nrx0 = 0x00;
        self.nrx1 = 0x00;
        self.nrx2 = 0x00;
        self.nrx3 = 0x00;
        self.nrx4 = 0x00;
    }

    pub fn get_sweep_period(&self) -> u8 {
        assert!(
            self.channel == Channel::Square1,
            "apu: get_sweep_period not valid for channel: {:?}",
            self.channel
        );
        // NR10 FF10 -PPP NSSS Sweep period, negate, shift
        (self.nrx0 & 0b0111_0000) >> 4
    }

    pub fn get_negate(&self) -> bool {
        assert!(
            self.channel == Channel::Square1,
            "apu: get_negate not valid for channel: {:?}",
            self.channel
        );
        // NR10 FF10 -PPP NSSS Sweep period, negate, shift
        self.nrx0 & 0b0000_1000 != 0b0000_0000
    }

    pub fn get_shift(&self) -> u8 {
        assert!(
            self.channel == Channel::Square1,
            "apu: get_shift not valid for channel: {:?}",
            self.channel
        );
        // NR10 FF10 -PPP NSSS Sweep period, negate, shift
        self.nrx0 & 0b0000_0111
    }

    pub fn get_duty(&self) -> u8 {
        assert!(
            self.channel == Channel::Square1 || self.channel == Channel::Square2,
            "apu: get_duty not valid for channel: {:?}",
            self.channel
        );
        // NRX1 DDLL LLLL Duty, Length load (64-L)
        self.nrx1 >> 6
    }

    pub fn get_length_load(&self) -> u16 {
        // NR11 FF11 DDLL LLLL Duty, Length load (64-L)
        // NR21 FF16 DDLL LLLL Duty, Length load (64-L)
        // NR31 FF1B LLLL LLLL Length load (256-L)
        // NR41 FF20 --LL LLLL Length load (64-L)
        if self.channel == Channel::Wave {
            (1 << 8) - u16::from(self.nrx1)
        } else {
            (1 << 6) - u16::from(self.nrx1 & 0b0011_1111)
        }
    }

    pub fn get_dac_power(&self) -> bool {
        assert!(
            self.channel == Channel::Wave,
            "apu: get_dac_power not valid for channel: {:?}",
            self.channel
        );
        // NR30 FF1A E--- ---- DAC power
        self.nrx0 & 0b10000000 != 0b0000_0000
    }

    pub fn get_starting_volume(&self) -> u8 {
        assert!(
            self.channel != Channel::Wave,
            "apu: get_starting_volume not valid for channel: {:?}",
            self.channel
        );
        // NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
        // NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
        // NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
        self.nrx2 >> 4
    }

    pub fn get_volume_code(&self) -> u8 {
        assert!(
            self.channel == Channel::Wave,
            "apu: get_volume_code not valid for channel: {:?}",
            self.channel
        );
        // NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
        (self.nrx2 & 0b0110_0000) >> 5
    }

    pub fn get_envelope_add_mode(&self) -> bool {
        assert!(
            self.channel != Channel::Wave,
            "apu: get_envelope_add_mode not valid for channel: {:?}",
            self.channel
        );
        // NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
        // NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
        // NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
        self.nrx2 & 0b0000_1000 != 0b0000_0000
    }

    pub fn get_period(&self) -> u8 {
        assert!(
            self.channel != Channel::Wave,
            "apu: get_period not valid for channel: {:?}",
            self.channel
        );
        // NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
        // NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
        // NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
        self.nrx2 & 0b0000_0111
    }

    pub fn get_frequency(&self) -> u16 {
        assert!(
            self.channel != Channel::Noise,
            "apu: get_frequency not valid for channel: {:?}",
            self.channel
        );
        // NR13 FF13 FFFF FFFF Frequency LSB
        // NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR23 FF18 FFFF FFFF Frequency LSB
        // NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR33 FF1D FFFF FFFF Frequency LSB
        // NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
        u16::from(self.nrx4 & 0b0000_0111) << 8 | u16::from(self.nrx3)
    }

    pub fn set_frequency(&mut self, frequency: u16) {
        assert!(
            self.channel != Channel::Noise,
            "apu: set_frequency not valid for channel: {:?}",
            self.channel
        );
        // NR13 FF13 FFFF FFFF Frequency LSB
        // NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR23 FF18 FFFF FFFF Frequency LSB
        // NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR33 FF1D FFFF FFFF Frequency LSB
        // NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
        let msb = ((frequency >> 8) & 0b0000_0111) as u8;
        let lsb = frequency as u8;
        self.nrx4 = (self.nrx4 & 0b1111_1000) | msb;
        self.nrx3 = lsb;
    }

    pub fn get_clock_shift(&self) -> u8 {
        assert!(
            self.channel == Channel::Noise,
            "apu: get_clock_shift not valid for channel: {:?}",
            self.channel
        );
        // NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
        self.nrx3 >> 4
    }

    pub fn get_width_mode_of_lfsr(&self) -> bool {
        assert!(
            self.channel == Channel::Noise,
            "apu: get_width_mode_of_lfsr not valid for channel: {:?}",
            self.channel
        );
        // NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
        self.nrx3 & 0b0000_1000 != 0b0000_0000
    }

    pub fn get_dividor_code(&self) -> u8 {
        assert!(
            self.channel == Channel::Noise,
            "apu: get_dividor_code not valid for channel: {:?}",
            self.channel
        );
        // NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
        self.nrx3 & 0b0000_0111
    }

    pub fn get_trigger(&self) -> bool {
        // NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR44 FF23 TL-- ---- Trigger, Length enable
        self.nrx4 & 0b1000_0000 != 0b0000_0000
    }

    pub fn set_trigger(&mut self, value: bool) {
        // NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR44 FF23 TL-- ---- Trigger, Length enable
        if value {
            self.nrx4 |= 0b10000000;
        } else {
            self.nrx4 &= 0b0111_1111;
        };
    }

    pub fn get_length_enable(&self) -> bool {
        // NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
        // NR44 FF23 TL-- ---- Trigger, Length enable
        self.nrx4 & 0b0100_0000 != 0b0000_0000
    }

    pub fn get_left_volume(&self) -> u8 {
        assert!(
            self.channel == Channel::Control,
            "apu: get_left_volume not valid for channel: {:?}",
            self.channel
        );
        // NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
        (self.nrx0 & 0b0111_0000) >> 4
    }

    pub fn get_right_volume(&self) -> u8 {
        assert!(
            self.channel == Channel::Control,
            "apu: get_right_volume not valid for channel: {:?}",
            self.channel
        );
        // NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
        self.nrx0 & 0b0000_0111
    }

    pub fn get_power_status(&self) -> bool {
        assert!(
            self.channel == Channel::Control,
            "apu: get_power_status not valid for channel: {:?}",
            self.channel
        );
        // NR52 FF26 P--- NW21 Power control/status, Channel length statuses
        self.nrx2 & 0b1000_0000 != 0b0000_0000
    }

    pub fn get_square_enables(&self, mode: Channel) -> (bool, bool) {
        assert!(
            self.channel == Channel::Control,
            "apu: get_square_channel_enabled not valid for channel: {:?}",
            self.channel
        );
        assert!(
            mode == Channel::Square1 || mode == Channel::Square2,
            "apu: get_square_channel_enabled passed invalid parameter: {:?}",
            mode
        );
        // NR51 FF25 NW21 NW21 Left enables, Right enables
        let left_enabled = if mode == Channel::Square1 {
            self.nrx1 & 0b0001_0000 == 0b0001_0000
        } else {
            self.nrx1 & 0b0010_0000 == 0b0010_0000
        };
        let right_enabled = if mode == Channel::Square1 {
            self.nrx1 & 0b0000_0001 == 0b0000_0001
        } else {
            self.nrx1 & 0b0000_0010 == 0b0000_0010
        };
        (left_enabled, right_enabled)
    }

    pub fn get_wave_enables(&self) -> (bool, bool) {
        assert!(
            self.channel == Channel::Control,
            "apu: get_wave_channel_enabled not valid for channel: {:?}",
            self.channel
        );
        // NR51 FF25 NW21 NW21 Left enables, Right enables
        let left_enabled = self.nrx1 & 0b0100_0000 == 0b0100_0000;
        let right_enabled = self.nrx1 & 0b0000_0100 == 0b0000_0100;
        (left_enabled, right_enabled)
    }

    pub fn get_noise_enables(&self) -> (bool, bool) {
        assert!(
            self.channel == Channel::Control,
            "apu: get_noise_channel_enable not valid for channel: {:?}",
            self.channel
        );
        // NR51 FF25 NW21 NW21 Left enables, Right enables
        let left_enabled = self.nrx1 & 0b1000_0000 == 0b1000_0000;
        let right_enabled = self.nrx1 & 0b0000_1000 == 0b0000_1000;
        (left_enabled, right_enabled)
    }

    pub fn get_clock_period(&self) -> u32 {
        match self.channel {
            // A square channel's frequency timer period is set to (2048 - frequency) * 4
            Channel::Square1 | Channel::Square2 => (2048 - u32::from(self.get_frequency())) * 4,
            // The wave channel's frequency timer period is set to (2048 - frequency) * 2.
            Channel::Wave => (2048 - u32::from(self.get_frequency())) * 2,
            // The noise channel's frequency timer period is set by a base divisor shifted left some number of bits.
            //
            //  Divisor code   Divisor
            //  -----------------------
            //      0             8
            //      1            16
            //      2            32
            //      3            48
            //      4            64
            //      5            80
            //      6            96
            //      7           112
            //
            Channel::Noise => {
                let divisor = match self.get_dividor_code() {
                    0 => 8,
                    code => (u32::from(code)) * 16,
                };
                divisor << self.get_clock_shift()
            }
            Channel::Control => crate::cpu::CLOCK_FREQUENCY / 512,
        }
    }
}

pub struct Blip {
    pub from: u32,
    data: BlipBuf,
    amplitude: i32,
}

impl Blip {
    pub fn new(sample_rate: u32) -> Blip {
        let mut blipbuf = BlipBuf::new(sample_rate);
        blipbuf.set_rates(
            f64::from(crate::cpu::CLOCK_FREQUENCY),
            f64::from(sample_rate),
        );
        Blip {
            data: blipbuf,
            from: 0x0000_0000,
            amplitude: 0x0000_0000,
        }
    }

    pub fn set(&mut self, time: u32, ampl: i32) {
        self.from = time;
        let delta = ampl - self.amplitude;
        self.amplitude = ampl;
        self.data.add_delta(time, delta);
    }

    pub fn end_frame(&mut self, clock_duration: u32) {
        self.data.end_frame(clock_duration)
    }

    pub fn samples_available(&self) -> u32 {
        self.data.samples_avail()
    }

    pub fn read_samples(&mut self, buffer: &mut [i16]) -> usize {
        self.data.read_samples(buffer, false)
    }
}

// A volume envelope has a volume counter and an internal timer clocked at 64 Hz by the frame sequencer.
// When the timer generates a clock and the envelope period is not zero, a new volume is calculated by adding
// or subtracting (as set by NRx2) one from the current volume. If this new volume within the 0 to 15 range,
// the volume is updated, otherwise it is left unchanged and no further automatic increments/decrements are
// made to the volume until the channel is triggered again.
// When the waveform input is zero the envelope outputs zero, otherwise it outputs the current volume.
// Writing to NRx2 causes obscure effects on the volume that differ on different Game Boy models (see obscure
// behavior).
//
// Obscure behaviour:
//  - The volume envelope and sweep timers treat a period of 0 as 8.
pub struct VolumeEnvelope {
    pub volume: u8,
    register: Rc<RefCell<Register>>,
    clock: Clock,
}

impl VolumeEnvelope {
    pub fn new(register: Rc<RefCell<Register>>) -> VolumeEnvelope {
        VolumeEnvelope {
            register,
            clock: Clock::new(8),
            volume: 0x00,
        }
    }

    pub fn reload(&mut self) {
        let period = self.register.borrow().get_period();
        // The volume envelope and sweep timers treat a period of 0 as 8.
        self.clock.period = if period == 0 { 8 } else { u32::from(period) };
        self.volume = self.register.borrow().get_starting_volume();
    }

    pub fn next(&mut self) {
        if self.register.borrow().get_period() == 0 {
            return;
        }
        if self.clock.run_cycles(1) == 0x00 {
            return;
        };
        // If this new volume within the 0 to 15 range, the volume is updated, otherwise it is left unchanged
        // and no further automatic increments/decrements are made to the volume until the channel is triggered again.
        let volume = if self.register.borrow().get_envelope_add_mode() {
            self.volume.wrapping_add(1)
        } else {
            self.volume.wrapping_sub(1)
        };
        if volume <= 15 {
            self.volume = volume;
        }
    }
}

// A length counter disables a channel when it decrements to zero. It contains an internal counter and enabled
// flag. Writing a byte to NRx1 loads the counter with 64-data (256-data for wave channel). The counter can be
// reloaded at any time. A channel is said to be disabled when the internal enabled flag is clear. When a channel
// is disabled, its volume unit receives 0, otherwise its volume unit receives the output of the waveform
// generator. Other units besides the length counter can enable/disable the channel as well. Each length counter
// is clocked at 256 Hz by the frame sequencer. When clocked while enabled by NRx4 and the counter is not zero,
// it is decremented. If it becomes zero, the channel is disabled.
pub struct LengthCounter {
    pub n: u16,
    register: Rc<RefCell<Register>>,
}

impl LengthCounter {
    pub fn new(register: Rc<RefCell<Register>>) -> LengthCounter {
        LengthCounter {
            register,
            n: 0x0000,
        }
    }

    pub fn next(&mut self) {
        if self.register.borrow().get_length_enable() && self.n != 0x0000 {
            self.n -= 1;
            if self.n == 0x0000 {
                self.register.borrow_mut().set_trigger(false);
            }
        }
    }

    pub fn reload(&mut self) {
        if self.n == 0x0000 {
            self.n = if self.register.borrow().channel == Channel::Wave {
                1 << 8
            } else {
                1 << 6
            };
        }
    }
}

// Frame Sequencer
// The frame sequencer generates low frequency clocks for the modulation units. It is clocked by a 512 Hz timer.
//
// Step   Length Ctr  Vol Env     Sweep
// ---------------------------------------
// 0      Clock       -           -
// 1      -           -           -
// 2      Clock       -           Clock
// 3      -           -           -
// 4      Clock       -           -
// 5      -           -           -
// 6      Clock       -           Clock
// 7      -           Clock       -
// ---------------------------------------
// Rate   256 Hz      64 Hz       128 Hz
#[derive(Debug, Copy, Clone)]
pub struct FrameSequencer {
    step: u8,
}

impl FrameSequencer {
    pub fn new() -> FrameSequencer {
        FrameSequencer { step: 0x00 }
    }

    pub fn next(&mut self) -> u8 {
        self.step += 1;
        self.step %= 8;
        self.step
    }

    pub fn should_run_length_counter(&self) -> bool {
        self.step == 0 || self.step == 2 || self.step == 4 || self.step == 6
    }

    pub fn should_run_volume_envelope(&self) -> bool {
        self.step == 7
    }

    pub fn should_run_frequency_sweep(&self) -> bool {
        self.step == 2 || self.step == 6
    }
}
