// Timer and Divider Registers
// FF04 - DIV - Divider Register (R/W)
// This register is incremented at rate of 16384Hz (~16779Hz on SGB). In CGB Double Speed Mode it is incremented twice
// as fast, ie. at 32768Hz. Writing any value to this register resets it to 00h.
// FF05 - TIMA - Timer counter (R/W)
// This timer is incremented by a clock frequency specified by the TAC register ($FF07). When the value overflows
// (gets bigger than FFh) then it will be reset to the value specified in TMA (FF06), and an interrupt will be
// requested, as described below.
// FF06 - TMA - Timer Modulo (R/W)
// When the TIMA overflows, this data will be loaded.
// FF07 - TAC - Timer Control (R/W)
//   Bit 2    - Timer Stop  (0=Stop, 1=Start)
//   Bits 1-0 - Input Clock Select
//      00: CPU Clock / 1024 (DMG, CGB:   4096 Hz, SGB:   ~4194 Hz)
//      01: CPU Clock / 16   (DMG, CGB: 262144 Hz, SGB: ~268400 Hz)
//      10: CPU Clock / 64   (DMG, CGB:  65536 Hz, SGB:  ~67110 Hz)
//      11: CPU Clock / 256  (DMG, CGB:  16384 Hz, SGB:  ~16780 Hz)
// INT 50 - Timer Interrupt
// Each time when the timer overflows (ie. when TIMA gets bigger than FFh), then an interrupt is requested by setting Bit 2
// in the IF Register (FF0F). When that interrupt is enabled, then the CPU will execute it by calling the timer interrupt
// vector at 0050h.
// Note
// The above described Timer is the built-in timer in the gameboy. It has nothing to do with the MBC3s battery buffered
// Real Time Clock - that's a completely different thing, described in the chapter about Memory Banking Controllers.

use crate::clock::Clock;
use crate::memory::Memory;
use crate::mmu::InterruptFlag;

#[derive(Debug, Copy, Clone)]
struct Registers {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            div: 0x00,
            tima: 0x00,
            tma: 0x00,
            tac: 0x00,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Timer {
    registers: Registers,
    div_clock: Clock,
    tma_clock: Clock,
    pub interrupt: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            registers: Registers::new(),
            div_clock: Clock::new(256),
            tma_clock: Clock::new(1024),
            interrupt: InterruptFlag::None as u8,
        }
    }

    pub fn run_cycles(&mut self, cycles: u32) {
        // Increment div
        self.registers.div = self
            .registers
            .div
            .wrapping_add(self.div_clock.run_cycles(cycles) as u8);

        // Increment TIMA if enabled
        //   Bit 2    - Timer Stop  (0=Stop, 1=Start)
        if (self.registers.tac & 0x04) != 0x00 {
            let cycles = self.tma_clock.run_cycles(cycles);
            for _ in 0..cycles {
                self.registers.tima = self.registers.tima.wrapping_add(1);
                // When the value overflows (gets bigger than FFh) then it will be reset to the value
                // specified in TMA (FF06), and an interrupt will be requested, as described below.
                if self.registers.tima == 0x00 {
                    self.registers.tima = self.registers.tma;
                    self.interrupt |= InterruptFlag::Timer as u8;
                }
            }
        }
    }
}

impl Memory for Timer {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // DIV - Divider Register (R/W)
            0xFF04 => self.registers.div,
            // TIMA - Timer counter (R/W)
            0xFF05 => self.registers.tima,
            // TMA - Timer Modulo (R/W)
            0xFF06 => self.registers.tma,
            // TAC - Timer Control (R/W)
            0xFF07 => self.registers.tac,
            _ => panic!("timer: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => {
                // Writing any value to this register resets it to 00h
                self.registers.div = 0x00;
                self.div_clock.num_cycles = 0x00;
            }
            0xFF05 => self.registers.tima = value,
            0xFF06 => self.registers.tma = value,
            0xFF07 => {
                //   Bit 2    - Timer Stop  (0=Stop, 1=Start)
                //   Bits 1-0 - Input Clock Select
                //      00: CPU Clock / 1024 (DMG, CGB:   4096 Hz, SGB:   ~4194 Hz)
                //      01: CPU Clock / 16   (DMG, CGB: 262144 Hz, SGB: ~268400 Hz)
                //      10: CPU Clock / 64   (DMG, CGB:  65536 Hz, SGB:  ~67110 Hz)
                //      11: CPU Clock / 256  (DMG, CGB:  16384 Hz, SGB:  ~16780 Hz)
                if (self.registers.tac & 0x03) != (value & 0x03) {
                    // TMA clock change
                    self.tma_clock.num_cycles = 0x00;
                    self.tma_clock.period = match value & 0x03 {
                        0x00 => 1024,
                        0x01 => 16,
                        0x02 => 64,
                        0x03 => 256,
                        _ => panic!("timer: invalid tma clock period"),
                    };
                    self.registers.tima = self.registers.tma;
                }
                self.registers.tac = value;
            }
            _ => panic!("timer: invalid address {:#06X?}", addr),
        }
    }
}
