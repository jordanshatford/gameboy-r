// APU
// The Game Boy has four sound channels: two square waves with adjustable duty, a programmable wave table,
// and a noise generator. Each has some kind of frequency (pitch) control. The first square channel also
// has an automatic frequency sweep unit to help with sound effects. The squares and noise each have a
// volume envelope unit to help with fading notes and sound effects, while the wave channel has only limited
// manual volume control. Each channel has a length counter that can silence the channel after a preset time,
// to handle note durations. Each channel can be individually panned to the far left, center, or far right.
// The master volume of the left and right outputs can also be adjusted.
//
// References:
//  - https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware
//  - https://gbdev.gg8.se/wiki/articles/Sound_Controller
//  - https://problemkaputt.de/pandocs.htm#soundcontroller

mod channels;

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::apu::channels::{
    Channel, FrameSequencer, NoiseChannel, Register, SquareChannel, WaveChannel,
};
use crate::clock::Clock;
use crate::cpu;
use crate::memory::Memory;

const DESIRED_CHANNELS: u16 = 2;
const DESIRED_SAMPLE_FORMAT: cpal::SampleFormat = cpal::SampleFormat::F32;
const DESIRED_SAMPLE_RATE: cpal::SampleRate = cpal::SampleRate(44100);

pub struct Apu {
    pub buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    register: Register,
    clock: Clock,
    frame_sequencer: FrameSequencer,
    channel1: SquareChannel,
    channel2: SquareChannel,
    channel3: WaveChannel,
    channel4: NoiseChannel,
    sample_rate: u32,
    audio_stream: cpal::Stream,
}

impl Apu {
    pub fn new() -> Option<Apu> {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let (stream, sample_rate) = get_audio_stream(buffer.clone())?;
        let rate = sample_rate.0;
        let apu = Apu {
            buffer,
            register: Register::new(Channel::Control),
            clock: Clock::new(cpu::CLOCK_FREQUENCY / 512),
            frame_sequencer: FrameSequencer::new(),
            channel1: SquareChannel::new(rate, Channel::Square1),
            channel2: SquareChannel::new(rate, Channel::Square2),
            channel3: WaveChannel::new(rate),
            channel4: NoiseChannel::new(rate),
            sample_rate: rate,
            audio_stream: stream,
        };
        apu.audio_stream.play().ok()?;
        Some(apu)
    }

    pub fn run_cycles(&mut self, cycles: u32) {
        // If the APU is powered off, do nothing
        if !self.register.get_power_status() {
            return;
        }

        for _ in 0..self.clock.run_cycles(cycles) {
            self.channel1.run_cycles(self.clock.period);
            self.channel2.run_cycles(self.clock.period);
            self.channel3.run_cycles(self.clock.period);
            self.channel4.run_cycles(self.clock.period);
            self.frame_sequencer.next();
            if self.frame_sequencer.should_run_length_counter() {
                self.channel1.length_counter.next();
                self.channel2.length_counter.next();
                self.channel3.length_counter.next();
                self.channel4.length_counter.next();
            }
            if self.frame_sequencer.should_run_volume_envelope() {
                self.channel1.volume_envelope.next();
                self.channel2.volume_envelope.next();
                self.channel4.volume_envelope.next();
            }
            if self.frame_sequencer.should_run_frequency_sweep() {
                self.channel1.frequency_sweep.next();
                self.channel1.clock.period = self.channel1.register.borrow().get_clock_period();
            }
            self.channel1.blip.end_frame(self.clock.period);
            self.channel2.blip.end_frame(self.clock.period);
            self.channel3.blip.end_frame(self.clock.period);
            self.channel4.blip.end_frame(self.clock.period);
            self.channel1.blip.from = self.channel1.blip.from.wrapping_sub(self.clock.period);
            self.channel2.blip.from = self.channel2.blip.from.wrapping_sub(self.clock.period);
            self.channel3.blip.from = self.channel3.blip.from.wrapping_sub(self.clock.period);
            self.channel4.blip.from = self.channel4.blip.from.wrapping_sub(self.clock.period);
            self.mix();
        }
    }

    fn mix(&mut self) {
        let sc1 = self.channel1.blip.samples_available();
        let sc2 = self.channel2.blip.samples_available();
        let sc3 = self.channel3.blip.samples_available();
        let sc4 = self.channel4.blip.samples_available();
        if sc1 != sc2 || sc2 != sc3 || sc3 != sc4 {
            panic!(
                "apu: sample count mismatch (1: {:?}, 2: {:?}, 3: {:?}, 4: {:?})",
                sc1, sc2, sc3, sc4
            );
        }
        let sample_count = sc1 as usize;

        let left_volume = (f32::from(self.register.get_left_volume()) / 7.0) * (1.0 / 15.0) * 0.25;
        let right_volume =
            (f32::from(self.register.get_right_volume()) / 7.0) * (1.0 / 15.0) * 0.25;

        let mut sum = 0;

        while sum < sample_count {
            let left_buffer = &mut [0f32; 2048];
            let right_buffer = &mut [0f32; 2048];
            let buffer = &mut [0i16; 2048];

            let sr1 = self.channel1.blip.read_samples(buffer);
            for (index, value) in buffer[..sr1].iter().enumerate() {
                let (left_enabled, right_enabled) =
                    self.register.get_square_enables(Channel::Square1);
                if left_enabled {
                    left_buffer[index] += f32::from(*value) * left_volume;
                }
                if right_enabled {
                    right_buffer[index] += f32::from(*value) * right_volume;
                }
            }

            let sr2 = self.channel2.blip.read_samples(buffer);
            for (index, value) in buffer[..sr2].iter().enumerate() {
                let (left_enabled, right_enabled) =
                    self.register.get_square_enables(Channel::Square1);
                if left_enabled {
                    left_buffer[index] += f32::from(*value) * left_volume;
                }
                if right_enabled {
                    right_buffer[index] += f32::from(*value) * right_volume;
                }
            }

            let sr3 = self.channel3.blip.read_samples(buffer);
            for (index, value) in buffer[..sr3].iter().enumerate() {
                let (left_enabled, right_enabled) = self.register.get_wave_enables();
                if left_enabled {
                    left_buffer[index] += f32::from(*value) * left_volume;
                }
                if right_enabled {
                    right_buffer[index] += f32::from(*value) * right_volume;
                }
            }

            let sr4 = self.channel4.blip.read_samples(buffer);
            for (index, value) in buffer[..sr4].iter().enumerate() {
                let (left_enabled, right_enabled) = self.register.get_noise_enables();
                if left_enabled {
                    left_buffer[index] += f32::from(*value) * left_volume;
                }
                if right_enabled {
                    right_buffer[index] += f32::from(*value) * right_volume;
                }
            }

            if sr1 != sr2 || sr2 != sr3 || sr3 != sr4 {
                panic!(
                    "apu: sample count mismatch (1: {:?}, 2: {:?}, 3: {:?}, 4: {:?})",
                    sr1, sr2, sr3, sr4
                );
            }

            let samples_read = sr1;
            self.play(&left_buffer[..samples_read], &right_buffer[..samples_read]);
            sum += samples_read;
        }
    }

    fn play(&mut self, left: &[f32], right: &[f32]) {
        assert_eq!(
            left.len(),
            right.len(),
            "apu: left and right buffer length difference (left: {:?}, right: {:?})",
            left.len(),
            right.len()
        );
        let mut buffer = self.buffer.lock().unwrap();
        for (l, r) in left.iter().zip(right) {
            // Ensure there is never more than 1 second of data in the buffer.
            if buffer.len() > self.sample_rate as usize {
                return;
            }
            buffer.push((*l, *r));
        }
    }
}

impl Memory for Apu {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // Sound Channel 1 - Tone & Sweep
            0xFF10..=0xFF14 => self.channel1.get_byte(addr),
            // Sound Channel 2 - Tone
            0xFF15..=0xFF19 => self.channel2.get_byte(addr),
            // Sound Channel 3 - Wave Output
            0xFF1A..=0xFF1E => self.channel3.get_byte(addr),
            // Sound Channel 4 - Noise
            0xFF1F..=0xFF23 => self.channel4.get_byte(addr),
            // FF24 - NR50 - Channel control / ON-OFF / Volume (R/W)
            0xFF24 => self.register.nrx0,
            // FF25 - NR51 - Selection of Sound output terminal (R/W)
            0xFF25 => self.register.nrx1,
            // FF26 - NR52 - Sound on/off
            0xFF26 => {
                // Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
                // Bit 3 - Sound 4 ON flag (Read Only)
                // Bit 2 - Sound 3 ON flag (Read Only)
                // Bit 1 - Sound 2 ON flag (Read Only)
                // Bit 0 - Sound 1 ON flag (Read Only)
                //
                // Bits 0-3 of this register are read only status bits, writing to these bits does NOT
                // enable/disable sound.
                //
                // A channel is turned on by triggering it (i.e. setting bit 7 of NRx4). A channel
                // is turned off when any of if the channels DAC is turned off.
                let a = self.register.nrx2 & 0b1111_0000;
                let b = u8::from(self.channel1.register.borrow().get_trigger()); // 0b0000_0001
                let c = if self.channel2.register.borrow().get_trigger() {
                    0b0000_0010
                } else {
                    0b0000_0000
                };
                let d = if self.channel3.register.borrow().get_trigger()
                    && self.channel3.register.borrow().get_dac_power()
                {
                    0b0000_0100
                } else {
                    0b0000_0000
                };
                let e = if self.channel4.register.borrow().get_trigger() {
                    0b0000_1000
                } else {
                    0b0000_0000
                };
                a | b | c | d | e
            }
            // FF30-FF3F - Wave Pattern RAM
            0xFF30..=0xFF3F => self.channel3.get_byte(addr),
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        // If APU is off and channel is not the Sound On/Off channel
        if addr != 0xFF26 && !self.register.get_power_status() {
            return;
        }
        match addr {
            // Sound Channel 1 - Tone & Sweep
            0xFF10..=0xFF14 => self.channel1.set_byte(addr, value),
            // Sound Channel 2 - Tone
            0xFF15..=0xFF19 => self.channel2.set_byte(addr, value),
            // Sound Channel 3 - Wave Output
            0xFF1A..=0xFF1E => self.channel3.set_byte(addr, value),
            // Sound Channel 4 - Noise
            0xFF1F..=0xFF23 => self.channel4.set_byte(addr, value),
            // FF24 - NR50 - Channel control / ON-OFF / Volume (R/W)
            0xFF24 => self.register.nrx0 = value,
            // FF25 - NR51 - Selection of Sound output terminal (R/W)
            0xFF25 => self.register.nrx1 = value,
            // FF26 - NR52 - Sound on/off
            0xFF26 => {
                self.register.nrx2 = value;
                // Powering APU off should write 0 to all regs
                // Powering APU off shouldn't affect wave, that wave RAM is unchanged
                if !self.register.get_power_status() {
                    self.channel1.register.borrow_mut().clear();
                    self.channel2.register.borrow_mut().clear();
                    self.channel3.register.borrow_mut().clear();
                    self.channel4.register.borrow_mut().clear();
                    self.register.clear();
                }
            }
            // FF30-FF3F - Wave Pattern RAM
            0xFF30..=0xFF3F => self.channel3.set_byte(addr, value),
            _ => panic!("apu: invalid address {:#06X?}", addr),
        }
    }
}

// Get audio stream and sample rate to use when processing audio. We pass the shared
// buffer which will be used by the APU.
fn get_audio_stream(
    buffer: Arc<Mutex<Vec<(f32, f32)>>>,
) -> Option<(cpal::Stream, cpal::SampleRate)> {
    let device = cpal::default_host().default_output_device()?;
    let supported_configs = device.supported_output_configs().ok()?;
    let mut supported_config = None;
    for c in supported_configs {
        if c.channels() == DESIRED_CHANNELS && c.sample_format() == DESIRED_SAMPLE_FORMAT {
            if c.min_sample_rate() <= DESIRED_SAMPLE_RATE
                && DESIRED_SAMPLE_RATE <= c.max_sample_rate()
            {
                supported_config = Some(c.with_sample_rate(DESIRED_SAMPLE_RATE));
            } else {
                supported_config = Some(c.with_max_sample_rate());
            }
            break;
        }
    }
    let selected_config = supported_config?;
    let sample_rate = selected_config.sample_rate();
    let sample_format = selected_config.sample_format();
    let config: cpal::StreamConfig = selected_config.into();
    let error_function = |err| eprintln!("apu: error playing audio: {}", err);
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _cb: &cpal::OutputCallbackInfo| {
                write_audio_data_to_buffer(&buffer, data)
            },
            error_function,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config,
            move |data: &mut [u16], _cb: &cpal::OutputCallbackInfo| {
                write_audio_data_to_buffer(&buffer, data)
            },
            error_function,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config,
            move |data: &mut [i16], _cb: &cpal::OutputCallbackInfo| {
                write_audio_data_to_buffer(&buffer, data)
            },
            error_function,
            None,
        ),
        _ => panic!("apu: unsupported audio sample format (supported options: F32, U16, I16)"),
    }
    .ok()?;
    Some((stream, sample_rate))
}

// Write audio buffer data to output.
fn write_audio_data_to_buffer<T: cpal::Sample + cpal::FromSample<f32>>(
    buffer: &Arc<Mutex<Vec<(f32, f32)>>>,
    output: &mut [T],
) {
    let mut buffer = buffer.lock().unwrap();
    // The buffer contains pairs of left and right audio samples, while the output is one long
    // array. Calculate length of buffer to write to output based on this.
    let length = std::cmp::min(output.len() / 2, buffer.len());
    for (i, (left, right)) in buffer.drain(..length).enumerate() {
        let left_index = i * 2;
        output[left_index] = T::from_sample(left);
        output[left_index + 1] = T::from_sample(right);
    }
}
