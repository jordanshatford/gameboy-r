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
        self.registers.sp += 2;
        self.set_word_in_memory(self.registers.sp, value);
    }

    pub fn pop_stack(&mut self) -> u16 {
        let result = self.get_word_in_memory(self.registers.sp);
        self.registers.sp += 2;
        result
    }
}

mod cb_codes;
mod instructions;
mod op_codes;
