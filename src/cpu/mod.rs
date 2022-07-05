// CPU
// The timings assume a CPU clock frequency of 4.194304 MHz (or 8.4
// MHz for CGB in double speed mode), as all gameboy timings are divideable
// by 4, many people specify timings and clock frequency divided by 4.

// Basically, the gameboy CPU works more like an older 8080 CPU rather than
// like a more powerful Z80 CPU. It is, however, supporting CB-prefixed
// instructions. Also, all known gameboy assemblers using the more obvious
// Z80-style syntax, rather than the chaotic 8080-style syntax.

// References:
//  - https://izik1.github.io/gbops/
//  - https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html

mod registers;

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time;

use crate::cartridges::CartridgeMode;
use crate::cpu::registers::Registers;
use crate::memory::Memory;

pub struct CPU {
    pub registers: Registers,
    pub memory: Rc<RefCell<dyn Memory>>,
    pub halted: bool,
    pub stopped: bool,
    pub ei: bool,
}

impl CPU {
    pub fn new(mode: CartridgeMode, memory: Rc<RefCell<dyn Memory>>) -> CPU {
        CPU {
            registers: Registers::new(mode),
            memory,
            halted: false,
            stopped: false,
            ei: false,
        }
    }

    // Get the next byte (based on program counter) and increment pc
    pub fn get_byte_at_pc(&mut self) -> u8 {
        let value = self.memory.borrow().get_byte(self.registers.pc);
        self.registers.pc += 1;
        value
    }

    // Get the next word (based on program counter) and increment
    pub fn get_word_at_pc(&mut self) -> u16 {
        let value = self.memory.borrow().get_word(self.registers.pc);
        self.registers.pc += 2;
        value
    }

    pub fn get_byte_in_memory(&self, addr: u16) -> u8 {
        self.memory.borrow().get_byte(addr)
    }

    pub fn get_word_in_memory(&self, addr: u16) -> u16 {
        self.memory.borrow().get_word(addr)
    }

    pub fn set_byte_in_memory(&mut self, addr: u16, value: u8) {
        self.memory.borrow_mut().set_byte(addr, value);
    }

    pub fn set_word_in_memory(&mut self, addr: u16, value: u16) {
        self.memory.borrow_mut().set_word(addr, value);
    }

    pub fn add_to_stack(&mut self, value: u16) {
        self.registers.sp -= 2;
        self.set_word_in_memory(self.registers.sp, value);
    }

    pub fn pop_stack(&mut self) -> u16 {
        let result = self.get_word_in_memory(self.registers.sp);
        self.registers.sp += 2;
        result
    }

    // The IME (interrupt master enable) flag is reset by DI and prohibits all interrupts.
    // It is set by EI and acknowledges the interrupt setting by the IE register.
    // 1. When an interrupt is generated, the IF flag will be set.
    // 2. If the IME flag is set & the corresponding IE flag is set, the following 3 steps are performed.
    // 3. Reset the IME flag and prevent all interrupts.
    // 4. The PC (program counter) is pushed onto the stack.
    // 5. Jump to the starting address of the interrupt.
    pub fn handle_interrupts(&mut self) -> u32 {
        if !self.halted && !self.ei {
            return 0;
        }
        let interrupts_asserted = self.get_byte_in_memory(0xFF0F);
        let interrupts_enabled = self.get_byte_in_memory(0xFFFF);
        let interrupts = interrupts_asserted & interrupts_enabled;
        if interrupts == 0x00 {
            return 0;
        }
        self.halted = false;
        if !self.ei {
            return 0;
        }
        self.ei = false;
        // Consume an interrupt and write the rest back to memory
        let n = interrupts.trailing_zeros();
        let interrupts_asserted = interrupts_asserted & !(1 << n);
        self.set_byte_in_memory(0xFF0F, interrupts_asserted);
        self.add_to_stack(self.registers.pc);
        // Set the PC to correspond interrupt process program:
        // V-Blank: 0x40
        // LCD: 0x48
        // TIMER: 0x50
        // JOYPAD: 0x60
        // Serial: 0x58
        self.registers.pc = 0x0040 | ((n as u16) << 3);
        4
    }

    pub fn run(&mut self) -> u32 {
        let cycles = {
            match self.handle_interrupts() {
                0 => {}
                n => return n,
            }
            if self.halted {
                // Emulate a noop instruction
                1
            } else {
                let op_code = self.get_byte_at_pc();
                self.execute(op_code)
            }
        };
        cycles * 4
    }
}

mod cb_codes;
mod instructions;
mod op_codes;

pub const CLOCK_FREQUENCY: u32 = 4_194_304;
pub const STEP_TIME: u32 = 16;
pub const STEP_CYCLES: u32 = (STEP_TIME as f64 / (1000_f64 / CLOCK_FREQUENCY as f64)) as u32;

pub struct RealTimeCPU {
    pub cpu: CPU,
    step_cycles: u32,
    step_zero: time::Instant,
    step_flip: bool,
}

impl RealTimeCPU {
    pub fn new(mode: CartridgeMode, memory: Rc<RefCell<dyn Memory>>) -> RealTimeCPU {
        RealTimeCPU {
            cpu: CPU::new(mode, memory),
            step_cycles: 0,
            step_zero: time::Instant::now(),
            step_flip: false,
        }
    }

    // Simulate real hardware execution speed by limiting function call of cpu.run()
    pub fn run(&mut self) -> u32 {
        if self.step_cycles > STEP_CYCLES {
            self.step_flip = true;
            self.step_cycles -= STEP_CYCLES;
            let now = time::Instant::now();
            let duration = now.duration_since(self.step_zero);
            let s = u64::from(STEP_TIME.saturating_sub(duration.as_millis() as u32));
            thread::sleep(time::Duration::from_millis(s));
            self.step_zero = self
                .step_zero
                .checked_add(time::Duration::from_millis(u64::from(STEP_TIME)))
                .unwrap();

            // If now is after the just updated target frame time, reset to avoid drifting
            if now.checked_duration_since(self.step_zero).is_some() {
                self.step_zero = now;
            }
        }
        let cycles = self.cpu.run();
        self.step_cycles += cycles;
        cycles
    }

    pub fn flip(&mut self) -> bool {
        let step_flip = self.step_flip;
        if step_flip {
            self.step_flip = false;
        }
        step_flip
    }
}
