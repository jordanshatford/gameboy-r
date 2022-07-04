use crate::cpu::registers::CpuFlag;
use crate::cpu::CPU;

// CPU OP Code Mapping
impl CPU {
    pub fn execute(&mut self, op_code: u8) {
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
