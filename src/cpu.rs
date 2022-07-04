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

use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridges::CartridgeMode;
use crate::memory::Memory;
use crate::registers::{CpuFlag, Registers};

pub struct CPU {
    pub registers: Registers,
    pub memory: Rc<RefCell<dyn Memory>>,
    pub halted: bool,
    pub stopped: bool,
}

impl CPU {
    pub fn new(mode: CartridgeMode, memory: Rc<RefCell<dyn Memory>>) -> CPU {
        CPU {
            registers: Registers::new(mode),
            memory,
            halted: false,
            stopped: false,
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
}

// CPU instruction functions
impl CPU {
    // Increment register value.
    // value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - Set if carry from bit 3.
    // C - Not affected.
    pub fn inst_alu_inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers
            .set_flag(CpuFlag::H, (value & 0x0F) + 0x01 > 0x0F);
        result
    }

    // Decrement register value.
    // value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if reselt is zero.
    // N - Set (set to true).
    // H - Set if no borrow from bit 4.
    // C - Not affected
    pub fn inst_alu_dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, true);
        self.registers
            .set_flag(CpuFlag::H, value.trailing_zeros() >= 4);
        result
    }

    // Rotate value left. Old bit 7 to Carry flag.
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 7 data.
    pub fn inst_alu_rlc(&mut self, value: u8) -> u8 {
        let has_carry = (value & 0x80) >> 7 == 0x01;
        let result = (value << 1) | (has_carry as u8);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, has_carry);
        result
    }

    // Rotate value right. Old bit 0 to Carry flag.
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 0 data
    pub fn inst_alu_rrc(&mut self, value: u8) -> u8 {
        let has_carry = value & 0x01 == 0x01;
        let result = if has_carry {
            0x80 | (value >> 1)
        } else {
            value >> 1
        };
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, has_carry);
        result
    }

    // Add value to A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - Set if carry from bit 3.
    // C - Set if carry from bit 7.
    pub fn inst_alu_add(&mut self, value: u8) {
        let curr = self.registers.a;
        let result = curr.wrapping_add(value);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers
            .set_flag(CpuFlag::H, (curr & 0x0F) + (value & 0x0F) > 0x0F);
        self.registers
            .set_flag(CpuFlag::C, (curr as u16) + (value as u16) > 0xFF);
        self.registers.a = result;
    }

    // Add value to HL
    // value = BC,DE,HL,SP
    //
    // Flags affected:
    // Z - Not affected.
    // N - unset (set to false).
    // H - Set if carry from bit 11.
    // C - Set if carry from bit 15.
    pub fn inst_alu_add_hl(&mut self, value: u16) {
        let curr = self.registers.hl();
        let result = curr.wrapping_add(value);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers
            .set_flag(CpuFlag::H, (curr & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
        self.registers.set_flag(CpuFlag::C, curr > 0xFFFF - value);
        self.registers.set_hl(result);
    }
}

// CPU OP Code Mapping
impl CPU {
    fn execute(&mut self, op_code: u8) {
        match op_code {
            // NOP
            0x00 => {}
            // LD BC, d16
            0x01 => {
                let value = self.get_word_at_pc();
                self.registers.set_bc(value);
            }
            // LC (BC), A
            0x02 => {
                self.set_byte_in_memory(self.registers.bc(), self.registers.a);
            }
            // INC BC
            0x03 => {
                let value = self.registers.bc().wrapping_add(1);
                self.registers.set_bc(value);
            }
            // INC B
            0x04 => self.registers.b = self.inst_alu_inc(self.registers.b),
            // DEC B
            0x05 => self.registers.b = self.inst_alu_dec(self.registers.b),
            // LD B, d8
            0x06 => self.registers.b = self.get_byte_at_pc(),
            // RLCA
            0x07 => {
                self.registers.a = self.inst_alu_rlc(self.registers.a);
                // Z flag should be unset (set to false)
                self.registers.set_flag(CpuFlag::Z, false);
            }
            // LD (d16), SP
            0x08 => {
                let addr = self.get_word_at_pc();
                self.set_word_in_memory(addr, self.registers.sp);
            }
            // ADD HL, BC
            0x09 => self.inst_alu_add_hl(self.registers.bc()),
            // LD A, (BC)
            0x0A => self.registers.a = self.get_byte_in_memory(self.registers.bc()),
            // DEC BC
            0x0B => {
                let value = self.registers.bc().wrapping_sub(1);
                self.registers.set_bc(value)
            }
            // INC C
            0x0C => self.registers.c = self.inst_alu_inc(self.registers.c),
            // DEC C
            0x0D => self.registers.c = self.inst_alu_dec(self.registers.c),
            // LD C, d8
            0x0E => self.registers.c = self.get_byte_at_pc(),
            // RRCA
            0x0F => {
                self.registers.a = self.inst_alu_rrc(self.registers.a);
                // Z flag should be unset (set to false)
                self.registers.set_flag(CpuFlag::Z, false);
            }
            // STOP
            0x10 => self.stopped = true,
            // LD DE, d16
            0x11 => {
                let value = self.get_word_at_pc();
                self.registers.set_de(value);
            }
            // LD (DE), A
            0x12 => self.set_byte_in_memory(self.registers.de(), self.registers.a),
            // INC DE
            0x13 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC D
            0x14 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC D
            0x15 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, d8
            0x16 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RLA
            0x17 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JR r8
            0x18 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD HL, DE
            0x19 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, (DE)
            0x1A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC DE
            0x1B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC E
            0x1C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC E
            0x1D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, d8
            0x1E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RRA
            0x1F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JR NZ, r8
            0x20 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD HL, d16
            0x21 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL+), A
            0x22 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC HL
            0x23 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC H
            0x24 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC H
            0x25 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, d8
            0x26 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DAA
            0x27 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JR Z, r8
            0x28 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD HL, HL
            0x29 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, (HL+)
            0x2A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC HL
            0x2B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC L
            0x2C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC L
            0x2D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, d8
            0x2E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CPL
            0x2F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JR NC, r8
            0x30 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD SP, d16
            0x31 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL-), A
            0x32 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC SP
            0x33 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC (HL)
            0x34 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC (HL)
            0x35 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), d8
            0x36 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SCF
            0x37 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JR C, r8
            0x38 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD HL, SP
            0x39 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, (HL-)
            0x3A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC SP
            0x3B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // INC A
            0x3C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DEC A
            0x3D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, d8
            0x3E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CCF
            0x3F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, B
            0x40 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LB B, C
            0x41 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, D
            0x42 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, E
            0x43 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, H
            0x44 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, L
            0x45 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, (HL)
            0x46 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD B, A
            0x47 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, B
            0x48 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, C
            0x49 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, D
            0x4A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, E
            0x4B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, H
            0x4C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, L
            0x4D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, (HL)
            0x4E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD C, A
            0x4F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, B
            0x50 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, C
            0x51 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, D
            0x52 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, E
            0x53 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, H
            0x54 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, L
            0x55 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, (HL)
            0x56 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD D, A
            0x57 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, B
            0x58 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, C
            0x59 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, D
            0x5A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, E
            0x5B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, H
            0x5C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, L
            0x5D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, (HL)
            0x5E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD E, A
            0x5F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, B
            0x60 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, C
            0x61 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, D
            0x62 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, E
            0x63 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, H
            0x64 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, L
            0x65 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, (HL)
            0x66 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD H, A
            0x67 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, B
            0x68 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, C
            0x69 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, D
            0x6A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, E
            0x6B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, H
            0x6C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, L
            0x6D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, (HL)
            0x6E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD L, A
            0x6F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), B
            0x70 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), C
            0x71 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), D
            0x72 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), E
            0x73 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), H
            0x74 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), L
            0x75 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // HALT
            0x76 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (HL), A
            0x77 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, B
            0x78 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, C
            0x79 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, D
            0x7A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, E
            0x7B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, H
            0x7C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, L
            0x7D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, (HL)
            0x7E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, A
            0x7F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, B
            0x80 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, C
            0x81 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, D
            0x82 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, E
            0x83 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, H
            0x84 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, L
            0x85 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, (HL)
            0x86 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, A
            0x87 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, B
            0x88 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, C
            0x89 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, D
            0x8A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, E
            0x8B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, H
            0x8C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, L
            0x8D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, (HL)
            0x8E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, A
            0x8F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB B
            0x90 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB C
            0x91 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB D
            0x92 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB E
            0x93 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB H
            0x94 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB L
            0x95 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB (HL)
            0x96 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB A
            0x97 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, B
            0x98 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, C
            0x99 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, D
            0x9A => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, E
            0x9B => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, H
            0x9C => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, L
            0x9D => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, (HL)
            0x9E => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SBC A, A
            0x9F => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND B
            0xA0 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND C
            0xA1 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND D
            0xA2 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND E
            0xA3 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND H
            0xA4 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND L
            0xA5 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND (HL)
            0xA6 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND A
            0xA7 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR B
            0xA8 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR C
            0xA9 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR D
            0xAA => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR E
            0xAB => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR H
            0xAC => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR L
            0xAD => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR (HL)
            0xAE => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // XOR A
            0xAF => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR B
            0xB0 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR C
            0xB1 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR D
            0xB2 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR E
            0xB3 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR H
            0xB4 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR L
            0xB5 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR (HL)
            0xB6 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR A
            0xB7 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP B
            0xB8 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP C
            0xB9 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP D
            0xBA => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP E
            0xBB => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP H
            0xBC => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP L
            0xBD => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP (HL)
            0xBE => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CP A
            0xBF => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RET NZ
            0xC0 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // POP BC
            0xC1 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JP NZ, a16
            0xC2 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JP a16
            0xC3 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CALL NX, a16
            0xC4 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // PUSH BC
            0xC5 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD A, d8
            0xC6 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 00H
            0xC7 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RET Z
            0xC8 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RET
            0xC9 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JP Z, a16
            0xCA => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // PREFIX CB
            0xCB => {
                let cb_code = self.get_byte_at_pc();
                self.execute_cb(cb_code);
            }
            // CALL Z, a16
            0xCC => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // CALL a16
            0xCD => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADC A, d8
            0xCE => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 08H
            0xCF => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RET NC
            0xD0 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // POP DE
            0xD1 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JP NC, a16
            0xD2 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xD3 => panic!("cpu: invalid op code 0xD3"),
            // CALL NC, a16
            0xD4 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // PUSH DE
            0xD5 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // SUB d8
            0xD6 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 10H
            0xD7 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RET C
            0xD8 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RETI
            0xD9 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JP C, a16
            0xDA => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xDB => panic!("cpu: invalid op code 0xDB"),
            // CALL C, a16
            0xDC => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xDD => panic!("cpu: invalid op code 0xDD"),
            // SBC A, d8
            0xDE => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 18H
            0xDF => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LDH (a8), A
            0xE0 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // POP HL
            0xE1 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (C), A
            0xE2 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xE3 => panic!("cpu: invalid op code 0xE3"),
            // Not Valid
            0xE4 => panic!("cpu: invalid op code 0xE4"),
            // PUSH HL
            0xE5 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // AND d8
            0xE6 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 20H
            0xE7 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // ADD SP, r8
            0xE8 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // JP (HL)
            0xE9 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD (a16), A
            0xEA => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xEB => panic!("cpu: invalid op code 0xEB"),
            // Not Valid
            0xEC => panic!("cpu: invalid op code 0xEC"),
            // Not Valid
            0xED => panic!("cpu: invalid op code 0xED"),
            // XOR d8
            0xEE => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 28H
            0xEF => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LDH A, (a8)
            0xF0 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // POP AF
            0xF1 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, (C)
            0xF2 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // DI
            0xF3 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xF4 => panic!("cpu: invalid op code 0xF4"),
            // PUSH AF
            0xF5 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // OR d8
            0xF6 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 30H
            0xF7 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD HL, SP+r8
            0xF8 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD SP, HL
            0xF9 => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // LD A, (a16)
            0xFA => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // EI
            0xFB => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // Not Valid
            0xFC => panic!("cpu: invalid op code 0xFC"),
            // Not Valid
            0xFD => panic!("cpu: invalid op code 0xFD"),
            // CP d8
            0xFE => panic!("cpu: OP code not implemented {:#04X?}", op_code),
            // RST 38H
            0xFF => panic!("cpu: OP code not implemented {:#04X?}", op_code),
        }
    }
}

// CPU CB Code Mapping (Prefixed by 0xCB)
impl CPU {
    fn execute_cb(&mut self, cb_code: u8) {
        match cb_code {
            0x00 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x01 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x02 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x03 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x04 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x05 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x06 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x07 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x08 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x09 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x0A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x0B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x0C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x0D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x0E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x0F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x10 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x11 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x12 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x13 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x14 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x15 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x16 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x17 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x18 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x19 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x1A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x1B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x1C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x1D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x1E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x1F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x20 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x21 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x22 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x23 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x24 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x25 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x26 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x27 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x28 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x29 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x2A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x2B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x2C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x2D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x2E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x2F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x30 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x31 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x32 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x33 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x34 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x35 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x36 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x37 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x38 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x39 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x3A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x3B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x3C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x3D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x3E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x3F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x40 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x41 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x42 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x43 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x44 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x45 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x46 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x47 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x48 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x49 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x4A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x4B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x4C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x4D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x4E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x4F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x50 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x51 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x52 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x53 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x54 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x55 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x56 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x57 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x58 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x59 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x5A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x5B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x5C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x5D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x5E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x5F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x60 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x61 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x62 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x63 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x64 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x65 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x66 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x67 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x68 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x69 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x6A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x6B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x6C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x6D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x6E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x6F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x70 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x71 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x72 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x73 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x74 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x75 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x76 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x77 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x78 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x79 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x7A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x7B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x7C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x7D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x7E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x7F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x80 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x81 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x82 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x83 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x84 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x85 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x86 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x87 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x88 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x89 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x8A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x8B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x8C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x8D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x8E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x8F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x90 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x91 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x92 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x93 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x94 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x95 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x96 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x97 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x98 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x99 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x9A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x9B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x9C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x9D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x9E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0x9F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xA9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xAA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xAB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xAC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xAD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xAE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xAF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xB9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xBA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xBB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xBC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xBD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xBE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xBF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xC9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xCA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xCB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xCC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xCD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xCE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xCF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xD9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xDA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xDB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xDC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xDD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xDE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xDF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xE9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xEA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xEB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xEC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xED => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xEE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xEF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xF9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xFA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xFB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xFC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xFD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xFE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            0xFF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
        }
    }
}
