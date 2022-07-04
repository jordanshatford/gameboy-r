use crate::cpu::registers::CpuFlag;
use crate::cpu::CPU;

// Nintendo documents describe the CPU & instructions speed in machine cycles while
// this document describes them in clock cycles. Here is the translation:
//   1 machine cycle = 4 clock cycles
//                   GB CPU Speed    NOP Instruction
// Machine Cycles    1.05MHz         1 cycle
// Clock Cycles      4.19MHz         4 cycles
//
//  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const OP_CYCLES: [u32; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0
    0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 2
    2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6
    2, 2, 2, 2, 2, 2, 0, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 8
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 9
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // a
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // b
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, // c
    2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, // d
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // e
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4, // f
];

// CPU OP Code Mapping
impl CPU {
    pub fn execute(&mut self, op_code: u8) -> u32 {
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
            0x13 => {
                let value = self.registers.de().wrapping_add(1);
                self.registers.set_de(value);
            }
            // INC D
            0x14 => self.registers.d = self.inst_alu_inc(self.registers.d),
            // DEC D
            0x15 => self.registers.d = self.inst_alu_dec(self.registers.d),
            // LD D, d8
            0x16 => self.registers.d = self.get_byte_at_pc(),
            // RLA
            0x17 => {
                self.registers.a = self.inst_alu_rl(self.registers.a);
                // Z flag should be unset (set to false)
                self.registers.set_flag(CpuFlag::Z, false);
            }
            // JR r8
            0x18 => {
                let n = self.get_byte_at_pc();
                self.inst_alu_jr(n);
            }
            // ADD HL, DE
            0x19 => self.inst_alu_add_hl(self.registers.de()),
            // LD A, (DE)
            0x1A => self.registers.a = self.get_byte_in_memory(self.registers.de()),
            // DEC DE
            0x1B => {
                let value = self.registers.de().wrapping_sub(1);
                self.registers.set_de(value);
            }
            // INC E
            0x1C => self.registers.e = self.inst_alu_inc(self.registers.e),
            // DEC E
            0x1D => self.registers.e = self.inst_alu_dec(self.registers.e),
            // LD E, d8
            0x1E => self.registers.e = self.get_byte_at_pc(),
            // RRA
            0x1F => {
                self.registers.a = self.inst_alu_rr(self.registers.a);
                // Z flag should be unset (set to false)
                self.registers.set_flag(CpuFlag::Z, false);
            }
            // JR NZ, r8
            0x20 => {
                let n = self.get_byte_at_pc();
                // Not Zero
                if !self.registers.has_flag(CpuFlag::Z) {
                    self.inst_alu_jr(n);
                }
            }
            // LD HL, d16
            0x21 => {
                let value = self.get_word_at_pc();
                self.registers.set_hl(value);
            }
            // LD (HL+), A
            0x22 => {
                let addr = self.registers.hl_then_inc();
                self.set_byte_in_memory(addr, self.registers.a);
            }
            // INC HL
            0x23 => {
                let value = self.registers.hl().wrapping_add(1);
                self.registers.set_hl(value);
            }
            // INC H
            0x24 => self.registers.h = self.inst_alu_inc(self.registers.h),
            // DEC H
            0x25 => self.registers.h = self.inst_alu_dec(self.registers.h),
            // LD H, d8
            0x26 => self.registers.h = self.get_byte_at_pc(),
            // DAA
            0x27 => self.inst_alu_daa(),
            // JR Z, r8
            0x28 => {
                let n = self.get_byte_at_pc();
                // Zero
                if self.registers.has_flag(CpuFlag::Z) {
                    self.inst_alu_jr(n);
                }
            }
            // ADD HL, HL
            0x29 => self.inst_alu_add_hl(self.registers.hl()),
            // LD A, (HL+)
            0x2A => {
                let addr = self.registers.hl_then_inc();
                self.registers.a = self.get_byte_in_memory(addr);
            }
            // DEC HL
            0x2B => {
                let value = self.registers.hl().wrapping_sub(1);
                self.registers.set_hl(value);
            }
            // INC L
            0x2C => self.registers.l = self.inst_alu_inc(self.registers.l),
            // DEC L
            0x2D => self.registers.l = self.inst_alu_dec(self.registers.l),
            // LD L, d8
            0x2E => self.registers.l = self.get_byte_at_pc(),
            // CPL
            0x2F => self.inst_alu_cpl(),
            // JR NC, r8
            0x30 => {
                let n = self.get_byte_at_pc();
                // Not Zero
                if !self.registers.has_flag(CpuFlag::C) {
                    self.inst_alu_jr(n);
                }
            }
            // LD SP, d16
            0x31 => self.registers.sp = self.get_word_at_pc(),
            // LD (HL-), A
            0x32 => {
                let addr = self.registers.hl_then_dec();
                self.set_byte_in_memory(addr, self.registers.a);
            }
            // INC SP
            0x33 => {
                let value = self.registers.sp.wrapping_add(1);
                self.registers.sp = value;
            }
            // INC (HL)
            0x34 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_inc(value);
                self.set_byte_in_memory(addr, result);
            }
            // DEC (HL)
            0x35 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_dec(value);
                self.set_byte_in_memory(addr, result);
            }
            // LD (HL), d8
            0x36 => {
                let addr = self.registers.hl();
                let value = self.get_byte_at_pc();
                self.set_byte_in_memory(addr, value);
            }
            // SCF
            0x37 => self.inst_alu_scf(),
            // JR C, r8
            0x38 => {
                let n = self.get_byte_at_pc();
                // Carry
                if self.registers.has_flag(CpuFlag::C) {
                    self.inst_alu_jr(n);
                }
            }
            // ADD HL, SP
            0x39 => self.inst_alu_add_hl(self.registers.sp),
            // LD A, (HL-)
            0x3A => {
                let addr = self.registers.hl_then_dec();
                self.registers.a = self.get_byte_in_memory(addr);
            }
            // DEC SP
            0x3B => {
                let value = self.registers.sp.wrapping_sub(1);
                self.registers.sp = value;
            }
            // INC A
            0x3C => self.registers.a = self.inst_alu_inc(self.registers.a),
            // DEC A
            0x3D => self.registers.a = self.inst_alu_dec(self.registers.a),
            // LD A, d8
            0x3E => self.registers.a = self.get_byte_at_pc(),
            // CCF
            0x3F => self.inst_alu_ccf(),
            // LD B, B
            0x40 => {}
            // LB B, C
            0x41 => self.registers.b = self.registers.c,
            // LD B, D
            0x42 => self.registers.b = self.registers.d,
            // LD B, E
            0x43 => self.registers.b = self.registers.e,
            // LD B, H
            0x44 => self.registers.b = self.registers.h,
            // LD B, L
            0x45 => self.registers.b = self.registers.l,
            // LD B, (HL)
            0x46 => self.registers.b = self.get_byte_in_memory(self.registers.hl()),
            // LD B, A
            0x47 => self.registers.b = self.registers.a,
            // LD C, B
            0x48 => self.registers.c = self.registers.b,
            // LD C, C
            0x49 => {}
            // LD C, D
            0x4A => self.registers.c = self.registers.d,
            // LD C, E
            0x4B => self.registers.c = self.registers.e,
            // LD C, H
            0x4C => self.registers.c = self.registers.h,
            // LD C, L
            0x4D => self.registers.c = self.registers.l,
            // LD C, (HL)
            0x4E => self.registers.c = self.get_byte_in_memory(self.registers.hl()),
            // LD C, A
            0x4F => self.registers.c = self.registers.a,
            // LD D, B
            0x50 => self.registers.d = self.registers.b,
            // LD D, C
            0x51 => self.registers.d = self.registers.c,
            // LD D, D
            0x52 => {}
            // LD D, E
            0x53 => self.registers.d = self.registers.e,
            // LD D, H
            0x54 => self.registers.d = self.registers.h,
            // LD D, L
            0x55 => self.registers.d = self.registers.l,
            // LD D, (HL)
            0x56 => self.registers.d = self.get_byte_in_memory(self.registers.hl()),
            // LD D, A
            0x57 => self.registers.d = self.registers.a,
            // LD E, B
            0x58 => self.registers.e = self.registers.b,
            // LD E, C
            0x59 => self.registers.e = self.registers.c,
            // LD E, D
            0x5A => self.registers.e = self.registers.d,
            // LD E, E
            0x5B => {}
            // LD E, H
            0x5C => self.registers.e = self.registers.h,
            // LD E, L
            0x5D => self.registers.e = self.registers.l,
            // LD E, (HL)
            0x5E => self.registers.e = self.get_byte_in_memory(self.registers.hl()),
            // LD E, A
            0x5F => self.registers.e = self.registers.a,
            // LD H, B
            0x60 => self.registers.h = self.registers.b,
            // LD H, C
            0x61 => self.registers.h = self.registers.c,
            // LD H, D
            0x62 => self.registers.h = self.registers.d,
            // LD H, E
            0x63 => self.registers.h = self.registers.e,
            // LD H, H
            0x64 => {}
            // LD H, L
            0x65 => self.registers.h = self.registers.l,
            // LD H, (HL)
            0x66 => self.registers.h = self.get_byte_in_memory(self.registers.hl()),
            // LD H, A
            0x67 => self.registers.h = self.registers.a,
            // LD L, B
            0x68 => self.registers.l = self.registers.b,
            // LD L, C
            0x69 => self.registers.l = self.registers.c,
            // LD L, D
            0x6A => self.registers.l = self.registers.d,
            // LD L, E
            0x6B => self.registers.l = self.registers.e,
            // LD L, H
            0x6C => self.registers.l = self.registers.h,
            // LD L, L
            0x6D => {}
            // LD L, (HL)
            0x6E => self.registers.l = self.get_byte_in_memory(self.registers.hl()),
            // LD L, A
            0x6F => self.registers.l = self.registers.a,
            // LD (HL), B
            0x70 => self.set_byte_in_memory(self.registers.hl(), self.registers.b),
            // LD (HL), C
            0x71 => self.set_byte_in_memory(self.registers.hl(), self.registers.c),
            // LD (HL), D
            0x72 => self.set_byte_in_memory(self.registers.hl(), self.registers.d),
            // LD (HL), E
            0x73 => self.set_byte_in_memory(self.registers.hl(), self.registers.e),
            // LD (HL), H
            0x74 => self.set_byte_in_memory(self.registers.hl(), self.registers.h),
            // LD (HL), L
            0x75 => self.set_byte_in_memory(self.registers.hl(), self.registers.l),
            // HALT
            0x76 => self.halted = true,
            // LD (HL), A
            0x77 => self.set_byte_in_memory(self.registers.hl(), self.registers.a),
            // LD A, B
            0x78 => self.registers.a = self.registers.b,
            // LD A, C
            0x79 => self.registers.a = self.registers.c,
            // LD A, D
            0x7A => self.registers.a = self.registers.d,
            // LD A, E
            0x7B => self.registers.a = self.registers.e,
            // LD A, H
            0x7C => self.registers.a = self.registers.h,
            // LD A, L
            0x7D => self.registers.a = self.registers.l,
            // LD A, (HL)
            0x7E => self.registers.a = self.get_byte_in_memory(self.registers.hl()),
            // LD A, A
            0x7F => {}
            // ADD A, B
            0x80 => self.inst_alu_add(self.registers.b),
            // ADD A, C
            0x81 => self.inst_alu_add(self.registers.c),
            // ADD A, D
            0x82 => self.inst_alu_add(self.registers.d),
            // ADD A, E
            0x83 => self.inst_alu_add(self.registers.e),
            // ADD A, H
            0x84 => self.inst_alu_add(self.registers.h),
            // ADD A, L
            0x85 => self.inst_alu_add(self.registers.l),
            // ADD A, (HL)
            0x86 => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_add(value);
            }
            // ADD A, A
            0x87 => self.inst_alu_add(self.registers.a),
            // ADC A, B
            0x88 => self.inst_alu_adc(self.registers.b),
            // ADC A, C
            0x89 => self.inst_alu_adc(self.registers.c),
            // ADC A, D
            0x8A => self.inst_alu_adc(self.registers.d),
            // ADC A, E
            0x8B => self.inst_alu_adc(self.registers.e),
            // ADC A, H
            0x8C => self.inst_alu_adc(self.registers.h),
            // ADC A, L
            0x8D => self.inst_alu_adc(self.registers.l),
            // ADC A, (HL)
            0x8E => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_adc(value);
            }
            // ADC A, A
            0x8F => self.inst_alu_adc(self.registers.a),
            // SUB B
            0x90 => self.inst_alu_sub(self.registers.b),
            // SUB C
            0x91 => self.inst_alu_sub(self.registers.c),
            // SUB D
            0x92 => self.inst_alu_sub(self.registers.d),
            // SUB E
            0x93 => self.inst_alu_sub(self.registers.e),
            // SUB H
            0x94 => self.inst_alu_sub(self.registers.h),
            // SUB L
            0x95 => self.inst_alu_sub(self.registers.l),
            // SUB (HL)
            0x96 => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_sub(value);
            }
            // SUB A
            0x97 => self.inst_alu_sub(self.registers.a),
            // SBC A, B
            0x98 => self.inst_alu_sbc(self.registers.b),
            // SBC A, C
            0x99 => self.inst_alu_sbc(self.registers.c),
            // SBC A, D
            0x9A => self.inst_alu_sbc(self.registers.d),
            // SBC A, E
            0x9B => self.inst_alu_sbc(self.registers.e),
            // SBC A, H
            0x9C => self.inst_alu_sbc(self.registers.h),
            // SBC A, L
            0x9D => self.inst_alu_sbc(self.registers.l),
            // SBC A, (HL)
            0x9E => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_sbc(value);
            }
            // SBC A, A
            0x9F => self.inst_alu_sbc(self.registers.a),
            // AND B
            0xA0 => self.inst_alu_and(self.registers.b),
            // AND C
            0xA1 => self.inst_alu_and(self.registers.c),
            // AND D
            0xA2 => self.inst_alu_and(self.registers.d),
            // AND E
            0xA3 => self.inst_alu_and(self.registers.e),
            // AND H
            0xA4 => self.inst_alu_and(self.registers.h),
            // AND L
            0xA5 => self.inst_alu_and(self.registers.l),
            // AND (HL)
            0xA6 => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_and(value);
            }
            // AND A
            0xA7 => self.inst_alu_and(self.registers.a),
            // XOR B
            0xA8 => self.inst_alu_xor(self.registers.b),
            // XOR C
            0xA9 => self.inst_alu_xor(self.registers.c),
            // XOR D
            0xAA => self.inst_alu_xor(self.registers.d),
            // XOR E
            0xAB => self.inst_alu_xor(self.registers.e),
            // XOR H
            0xAC => self.inst_alu_xor(self.registers.h),
            // XOR L
            0xAD => self.inst_alu_xor(self.registers.l),
            // XOR (HL)
            0xAE => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_xor(value);
            }
            // XOR A
            0xAF => self.inst_alu_xor(self.registers.a),
            // OR B
            0xB0 => self.inst_alu_or(self.registers.b),
            // OR C
            0xB1 => self.inst_alu_or(self.registers.c),
            // OR D
            0xB2 => self.inst_alu_or(self.registers.d),
            // OR E
            0xB3 => self.inst_alu_or(self.registers.e),
            // OR H
            0xB4 => self.inst_alu_or(self.registers.h),
            // OR L
            0xB5 => self.inst_alu_or(self.registers.l),
            // OR (HL)
            0xB6 => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_or(value);
            }
            // OR A
            0xB7 => self.inst_alu_or(self.registers.a),
            // CP B
            0xB8 => self.inst_alu_cp(self.registers.b),
            // CP C
            0xB9 => self.inst_alu_cp(self.registers.c),
            // CP D
            0xBA => self.inst_alu_cp(self.registers.d),
            // CP E
            0xBB => self.inst_alu_cp(self.registers.e),
            // CP H
            0xBC => self.inst_alu_cp(self.registers.h),
            // CP L
            0xBD => self.inst_alu_cp(self.registers.l),
            // CP (HL)
            0xBE => {
                let value = self.get_byte_in_memory(self.registers.hl());
                self.inst_alu_cp(value);
            }
            // CP A
            0xBF => self.inst_alu_cp(self.registers.a),
            // RET NZ
            0xC0 => {
                // Not Zero
                if !self.registers.has_flag(CpuFlag::Z) {
                    self.registers.pc = self.pop_stack();
                }
            }
            // POP BC
            0xC1 => {
                let value = self.pop_stack();
                self.registers.set_bc(value);
            }
            // JP NZ, a16
            0xC2 => {
                let pc = self.get_word_at_pc();
                if !self.registers.has_flag(CpuFlag::Z) {
                    self.registers.pc = pc;
                }
            }
            // JP a16
            0xC3 => self.registers.pc = self.get_word_at_pc(),
            // CALL NZ, a16
            0xC4 => {
                let n = self.get_word_at_pc();
                if !self.registers.has_flag(CpuFlag::Z) {
                    self.add_to_stack(self.registers.pc);
                    self.registers.pc = n;
                }
            }
            // PUSH BC
            0xC5 => self.add_to_stack(self.registers.bc()),
            // ADD A, d8
            0xC6 => {
                let value = self.get_byte_at_pc();
                self.inst_alu_add(value);
            }
            // RST 00H
            0xC7 => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x00;
            }
            // RET Z
            0xC8 => {
                // Not Zero
                if self.registers.has_flag(CpuFlag::Z) {
                    self.registers.pc = self.pop_stack();
                }
            }
            // RET
            0xC9 => self.registers.pc = self.pop_stack(),
            // JP Z, a16
            0xCA => {
                let pc = self.get_word_at_pc();
                if self.registers.has_flag(CpuFlag::Z) {
                    self.registers.pc = pc;
                }
            }
            // PREFIX CB
            0xCB => {
                let cb_code = self.get_byte_at_pc();
                return self.execute_cb(cb_code);
            }
            // CALL Z, a16
            0xCC => {
                let n = self.get_word_at_pc();
                if self.registers.has_flag(CpuFlag::Z) {
                    self.add_to_stack(self.registers.pc);
                    self.registers.pc = n;
                }
            }
            // CALL a16
            0xCD => {
                let n = self.get_word_at_pc();
                self.add_to_stack(self.registers.pc);
                self.registers.pc = n;
            }
            // ADC A, d8
            0xCE => {
                let value = self.get_byte_at_pc();
                self.inst_alu_adc(value);
            }
            // RST 08H
            0xCF => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x08;
            }
            // RET NC
            0xD0 => {
                // Not Carry
                if !self.registers.has_flag(CpuFlag::C) {
                    self.registers.pc = self.pop_stack();
                }
            }
            // POP DE
            0xD1 => {
                let value = self.pop_stack();
                self.registers.set_de(value);
            }
            // JP NC, a16
            0xD2 => {
                let pc = self.get_word_at_pc();
                if !self.registers.has_flag(CpuFlag::C) {
                    self.registers.pc = pc;
                }
            }
            // Not Valid
            0xD3 => panic!("cpu: invalid op code 0xD3"),
            // CALL NC, a16
            0xD4 => {
                let n = self.get_word_at_pc();
                if !self.registers.has_flag(CpuFlag::C) {
                    self.add_to_stack(self.registers.pc);
                    self.registers.pc = n;
                }
            }
            // PUSH DE
            0xD5 => self.add_to_stack(self.registers.de()),
            // SUB d8
            0xD6 => {
                let value = self.get_byte_at_pc();
                self.inst_alu_sub(value);
            }
            // RST 10H
            0xD7 => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x10;
            }
            // RET C
            0xD8 => {
                // Carry
                if self.registers.has_flag(CpuFlag::C) {
                    self.registers.pc = self.pop_stack();
                }
            }
            // RETI
            0xD9 => {
                self.registers.pc = self.pop_stack();
                self.ei = true;
            }
            // JP C, a16
            0xDA => {
                let pc = self.get_word_at_pc();
                if self.registers.has_flag(CpuFlag::C) {
                    self.registers.pc = pc;
                }
            }
            // Not Valid
            0xDB => panic!("cpu: invalid op code 0xDB"),
            // CALL C, a16
            0xDC => {
                let n = self.get_word_at_pc();
                if self.registers.has_flag(CpuFlag::C) {
                    self.add_to_stack(self.registers.pc);
                    self.registers.pc = n;
                }
            }
            // Not Valid
            0xDD => panic!("cpu: invalid op code 0xDD"),
            // SBC A, d8
            0xDE => {
                let value = self.get_byte_at_pc();
                self.inst_alu_sbc(value);
            }
            // RST 18H
            0xDF => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x18;
            }
            // LDH (a8), A
            0xE0 => {
                let addr = 0xFF00 | (self.get_byte_at_pc() as u16);
                self.set_byte_in_memory(addr, self.registers.a);
            }
            // POP HL
            0xE1 => {
                let value = self.pop_stack();
                self.registers.set_hl(value);
            }
            // LD (C), A
            0xE2 => {
                let addr = 0xFF00 | (self.registers.c as u16);
                self.set_byte_in_memory(addr, self.registers.a);
            }
            // Not Valid
            0xE3 => panic!("cpu: invalid op code 0xE3"),
            // Not Valid
            0xE4 => panic!("cpu: invalid op code 0xE4"),
            // PUSH HL
            0xE5 => self.add_to_stack(self.registers.hl()),
            // AND d8
            0xE6 => {
                let value = self.get_byte_at_pc();
                self.inst_alu_and(value);
            }
            // RST 20H
            0xE7 => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x20;
            }
            // ADD SP, r8
            0xE8 => {
                let value = self.get_byte_at_pc();
                self.inst_alu_add_sp(value);
            }
            // JP (HL)
            0xE9 => self.registers.pc = self.registers.hl(),
            // LD (a16), A
            0xEA => {
                let addr = self.get_word_at_pc();
                self.set_byte_in_memory(addr, self.registers.a);
            }
            // Not Valid
            0xEB => panic!("cpu: invalid op code 0xEB"),
            // Not Valid
            0xEC => panic!("cpu: invalid op code 0xEC"),
            // Not Valid
            0xED => panic!("cpu: invalid op code 0xED"),
            // XOR d8
            0xEE => {
                let value = self.get_byte_at_pc();
                self.inst_alu_xor(value);
            }
            // RST 28H
            0xEF => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x28;
            }
            // LDH A, (a8)
            0xF0 => {
                let addr = 0xFF00 | (self.get_byte_at_pc() as u16);
                self.registers.a = self.get_byte_in_memory(addr);
            }
            // POP AF
            0xF1 => {
                let value = self.pop_stack();
                self.registers.set_af(value);
            }
            // LD A, (C)
            0xF2 => {
                let addr = 0xFF00 | (self.registers.c as u16);
                self.registers.a = self.get_byte_in_memory(addr);
            }
            // DI
            0xF3 => self.ei = false,
            // Not Valid
            0xF4 => panic!("cpu: invalid op code 0xF4"),
            // PUSH AF
            0xF5 => self.add_to_stack(self.registers.af()),
            // OR d8
            0xF6 => {
                let value = self.get_byte_at_pc();
                self.inst_alu_or(value);
            }
            // RST 30H
            0xF7 => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x30;
            }
            // LD HL, SP+r8
            0xF8 => {
                let sp = self.registers.sp;
                let value = i16::from(self.get_byte_at_pc() as i8) as u16;
                self.registers.set_flag(CpuFlag::Z, false);
                self.registers.set_flag(CpuFlag::N, false);
                self.registers
                    .set_flag(CpuFlag::H, (sp & 0x000F) + (value & 0x000F) > 0x000F);
                self.registers
                    .set_flag(CpuFlag::C, (sp & 0x00FF) + (value & 0x00FF) > 0x00FF);
                self.registers.set_hl(sp.wrapping_add(value));
            }
            // LD SP, HL
            0xF9 => self.registers.sp = self.registers.hl(),
            // LD A, (a16)
            0xFA => {
                let addr = self.get_word_at_pc();
                self.registers.a = self.get_byte_in_memory(addr);
            }
            // EI
            0xFB => self.ei = true,
            // Not Valid
            0xFC => panic!("cpu: invalid op code 0xFC"),
            // Not Valid
            0xFD => panic!("cpu: invalid op code 0xFD"),
            // CP d8
            0xFE => {
                let value = self.get_byte_at_pc();
                self.inst_alu_cp(value);
            }
            // RST 38H
            0xFF => {
                self.add_to_stack(self.registers.pc);
                self.registers.pc = 0x38;
            }
        }
        panic!("cpu: num cycles returned not implemented")
    }
}
