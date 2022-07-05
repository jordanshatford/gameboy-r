use crate::cpu::registers::CpuFlag;
use crate::cpu::CPU;

// Nintendo documents describe the CPU & instructions speed in machine cycles while this document describes them in
// clock cycles. Here is the translation:
//   1 machine cycle = 4 clock cycles
//                   GB CPU Speed    NOP Instruction
// Machine Cycles    1.05MHz         1 cycle
// Clock Cycles      4.19MHz         4 cycles
//
//  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
const CB_CYCLES: [u32; 256] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 1
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 2
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 3
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 4
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 5
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 6
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 7
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 8
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 9
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // a
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // b
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // c
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // d
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // e
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // f
];

// CPU CB Code Mapping (Prefixed by 0xCB)
impl CPU {
    pub fn execute_cb(&mut self, cb_code: u8) -> u32 {
        match cb_code {
            // RLC B
            0x00 => self.registers.b = self.inst_alu_rlc(self.registers.b),
            // RLC C
            0x01 => self.registers.c = self.inst_alu_rlc(self.registers.c),
            // RLC D
            0x02 => self.registers.d = self.inst_alu_rlc(self.registers.d),
            // RLC E
            0x03 => self.registers.e = self.inst_alu_rlc(self.registers.e),
            // RLC H
            0x04 => self.registers.h = self.inst_alu_rlc(self.registers.h),
            // RLC L
            0x05 => self.registers.l = self.inst_alu_rlc(self.registers.l),
            // RLC (HL)
            0x06 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_rlc(value);
                self.set_byte_in_memory(addr, result);
            }
            // RLC A
            0x07 => self.registers.a = self.inst_alu_rlc(self.registers.a),
            // RRC B
            0x08 => self.registers.b = self.inst_alu_rrc(self.registers.b),
            // RRC C
            0x09 => self.registers.c = self.inst_alu_rrc(self.registers.c),
            // RRC D
            0x0A => self.registers.d = self.inst_alu_rrc(self.registers.d),
            // RRC E
            0x0B => self.registers.e = self.inst_alu_rrc(self.registers.e),
            // RRC H
            0x0C => self.registers.h = self.inst_alu_rrc(self.registers.h),
            // RRC L
            0x0D => self.registers.l = self.inst_alu_rrc(self.registers.l),
            // RRC (HL)
            0x0E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_rrc(value);
                self.set_byte_in_memory(addr, result);
            }
            // RRC A
            0x0F => self.registers.a = self.inst_alu_rrc(self.registers.a),
            // RL B
            0x10 => self.registers.b = self.inst_alu_rl(self.registers.b),
            // RL C
            0x11 => self.registers.c = self.inst_alu_rl(self.registers.c),
            // RL D
            0x12 => self.registers.d = self.inst_alu_rl(self.registers.d),
            // RL E
            0x13 => self.registers.e = self.inst_alu_rl(self.registers.e),
            // RL H
            0x14 => self.registers.h = self.inst_alu_rl(self.registers.h),
            // RL L
            0x15 => self.registers.l = self.inst_alu_rl(self.registers.l),
            // RL (HL)
            0x16 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_rl(value);
                self.set_byte_in_memory(addr, result);
            }
            // RL A
            0x17 => self.registers.a = self.inst_alu_rl(self.registers.a),
            // RR B
            0x18 => self.registers.b = self.inst_alu_rr(self.registers.b),
            // RR C
            0x19 => self.registers.c = self.inst_alu_rr(self.registers.c),
            // RR D
            0x1A => self.registers.d = self.inst_alu_rr(self.registers.d),
            // RR E
            0x1B => self.registers.e = self.inst_alu_rr(self.registers.e),
            // RR H
            0x1C => self.registers.h = self.inst_alu_rr(self.registers.h),
            // RR L
            0x1D => self.registers.l = self.inst_alu_rr(self.registers.l),
            // RR (HL)
            0x1E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_rr(value);
                self.set_byte_in_memory(addr, result);
            }
            // RR A
            0x1F => self.registers.a = self.inst_alu_rr(self.registers.a),
            // SLA B
            0x20 => self.registers.b = self.inst_alu_sla(self.registers.b),
            // SLA C
            0x21 => self.registers.c = self.inst_alu_sla(self.registers.c),
            // SLA D
            0x22 => self.registers.d = self.inst_alu_sla(self.registers.d),
            // SLA E
            0x23 => self.registers.e = self.inst_alu_sla(self.registers.e),
            // SLA H
            0x24 => self.registers.h = self.inst_alu_sla(self.registers.h),
            // SLA L
            0x25 => self.registers.l = self.inst_alu_sla(self.registers.l),
            // SLA (HL)
            0x26 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_sla(value);
                self.set_byte_in_memory(addr, result);
            }
            // SLA A
            0x27 => self.registers.a = self.inst_alu_sla(self.registers.a),
            // SRA B
            0x28 => self.registers.b = self.inst_alu_sra(self.registers.b),
            // SRA C
            0x29 => self.registers.c = self.inst_alu_sra(self.registers.c),
            // SRA D
            0x2A => self.registers.d = self.inst_alu_sra(self.registers.d),
            // SRA E
            0x2B => self.registers.e = self.inst_alu_sra(self.registers.e),
            // SRA H
            0x2C => self.registers.h = self.inst_alu_sra(self.registers.h),
            // SRA L
            0x2D => self.registers.l = self.inst_alu_sra(self.registers.l),
            // SRA (HL)
            0x2E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_sra(value);
                self.set_byte_in_memory(addr, result);
            }
            // SRA A
            0x2F => self.registers.a = self.inst_alu_sra(self.registers.a),
            // SWAP B
            0x30 => self.registers.b = self.inst_alu_swap(self.registers.b),
            // SWAP C
            0x31 => self.registers.c = self.inst_alu_swap(self.registers.c),
            // SWAP D
            0x32 => self.registers.d = self.inst_alu_swap(self.registers.d),
            // SWAP E
            0x33 => self.registers.e = self.inst_alu_swap(self.registers.e),
            // SWAP H
            0x34 => self.registers.h = self.inst_alu_swap(self.registers.h),
            // SWAP L
            0x35 => self.registers.l = self.inst_alu_swap(self.registers.l),
            // SWAP (HL)
            0x36 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_swap(value);
                self.set_byte_in_memory(addr, result);
            }
            // SWAP A
            0x37 => self.registers.a = self.inst_alu_swap(self.registers.a),
            // SRL B
            0x38 => self.registers.b = self.inst_alu_srl(self.registers.b),
            // SRL C
            0x39 => self.registers.c = self.inst_alu_srl(self.registers.c),
            // SRL D
            0x3A => self.registers.d = self.inst_alu_srl(self.registers.d),
            // SRL E
            0x3B => self.registers.e = self.inst_alu_srl(self.registers.e),
            // SRL H
            0x3C => self.registers.h = self.inst_alu_srl(self.registers.h),
            // SRL L
            0x3D => self.registers.l = self.inst_alu_srl(self.registers.l),
            // SRL (HL)
            0x3E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_srl(value);
                self.set_byte_in_memory(addr, result);
            }
            // SRL A
            0x3F => self.registers.a = self.inst_alu_srl(self.registers.a),
            // BIT 0, B
            0x40 => self.inst_alu_bit(self.registers.b, 0),
            // BIT 0, C
            0x41 => self.inst_alu_bit(self.registers.c, 0),
            // BIT 0, D
            0x42 => self.inst_alu_bit(self.registers.d, 0),
            // BIT 0, E
            0x43 => self.inst_alu_bit(self.registers.e, 0),
            // BIT 0, H
            0x44 => self.inst_alu_bit(self.registers.h, 0),
            // BIT 0, L
            0x45 => self.inst_alu_bit(self.registers.l, 0),
            // BIT 0, (HL)
            0x46 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 0);
            }
            // BIT 0, A
            0x47 => self.inst_alu_bit(self.registers.a, 0),
            // BIT 1, B
            0x48 => self.inst_alu_bit(self.registers.b, 1),
            // BIT 1, C
            0x49 => self.inst_alu_bit(self.registers.c, 1),
            // BIT 1, D
            0x4A => self.inst_alu_bit(self.registers.d, 1),
            // BIT 1, E
            0x4B => self.inst_alu_bit(self.registers.e, 1),
            // BIT 1, H
            0x4C => self.inst_alu_bit(self.registers.h, 1),
            // BIT 1, L
            0x4D => self.inst_alu_bit(self.registers.l, 1),
            // BIT 1, (HL)
            0x4E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 1);
            }
            // BIT 1, A
            0x4F => self.inst_alu_bit(self.registers.a, 1),
            // BIT 2, B
            0x50 => self.inst_alu_bit(self.registers.b, 2),
            // BIT 2, C
            0x51 => self.inst_alu_bit(self.registers.c, 2),
            // BIT 2, D
            0x52 => self.inst_alu_bit(self.registers.d, 2),
            // BIT 2, E
            0x53 => self.inst_alu_bit(self.registers.e, 2),
            // BIT 2, H
            0x54 => self.inst_alu_bit(self.registers.h, 2),
            // BIT 2, L
            0x55 => self.inst_alu_bit(self.registers.l, 2),
            // BIT 2, (HL)
            0x56 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 2);
            }
            // BIT 2, A
            0x57 => self.inst_alu_bit(self.registers.a, 2),
            // BIT 3, B
            0x58 => self.inst_alu_bit(self.registers.b, 3),
            // BIT 3, C
            0x59 => self.inst_alu_bit(self.registers.c, 3),
            // BIT 3, D
            0x5A => self.inst_alu_bit(self.registers.d, 3),
            // BIT 3, E
            0x5B => self.inst_alu_bit(self.registers.e, 3),
            // BIT 3, H
            0x5C => self.inst_alu_bit(self.registers.h, 3),
            // BIT 3, L
            0x5D => self.inst_alu_bit(self.registers.l, 3),
            // BIT 3, (HL)
            0x5E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 3);
            }
            // BIT 3, A
            0x5F => self.inst_alu_bit(self.registers.a, 3),
            // BIT 4, B
            0x60 => self.inst_alu_bit(self.registers.b, 4),
            // BIT 4, C
            0x61 => self.inst_alu_bit(self.registers.c, 4),
            // BIT 4, D
            0x62 => self.inst_alu_bit(self.registers.d, 4),
            // BIT 4, E
            0x63 => self.inst_alu_bit(self.registers.e, 4),
            // BIT 4, H
            0x64 => self.inst_alu_bit(self.registers.h, 4),
            // BIT 4, L
            0x65 => self.inst_alu_bit(self.registers.l, 4),
            // BIT 4, (HL)
            0x66 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 4);
            }
            // BIT 4, A
            0x67 => self.inst_alu_bit(self.registers.a, 4),
            // BIT 5, B
            0x68 => self.inst_alu_bit(self.registers.b, 5),
            // BIT 5, C
            0x69 => self.inst_alu_bit(self.registers.c, 5),
            // BIT 5, D
            0x6A => self.inst_alu_bit(self.registers.d, 5),
            // BIT 5, E
            0x6B => self.inst_alu_bit(self.registers.e, 5),
            // BIT 5, H
            0x6C => self.inst_alu_bit(self.registers.h, 5),
            // BIT 5, L
            0x6D => self.inst_alu_bit(self.registers.l, 5),
            // BIT 5, (HL)
            0x6E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 5);
            }
            // BIT 5, A
            0x6F => self.inst_alu_bit(self.registers.a, 5),
            // BIT 6, B
            0x70 => self.inst_alu_bit(self.registers.b, 6),
            // BIT 6, C
            0x71 => self.inst_alu_bit(self.registers.c, 6),
            // BIT 6, D
            0x72 => self.inst_alu_bit(self.registers.d, 6),
            // BIT 6, E
            0x73 => self.inst_alu_bit(self.registers.e, 6),
            // BIT 6, H
            0x74 => self.inst_alu_bit(self.registers.h, 6),
            // BIT 6, L
            0x75 => self.inst_alu_bit(self.registers.l, 6),
            // BIT 6, (HL)
            0x76 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 6);
            }
            // BIT 6, A
            0x77 => self.inst_alu_bit(self.registers.a, 6),
            // BIT 7, B
            0x78 => self.inst_alu_bit(self.registers.b, 7),
            // BIT 7, C
            0x79 => self.inst_alu_bit(self.registers.c, 7),
            // BIT 7, D
            0x7A => self.inst_alu_bit(self.registers.d, 7),
            // BIT 7, E
            0x7B => self.inst_alu_bit(self.registers.e, 7),
            // BIT 7, H
            0x7C => self.inst_alu_bit(self.registers.h, 7),
            // BIT 7, L
            0x7D => self.inst_alu_bit(self.registers.l, 7),
            // BIT 7, (HL)
            0x7E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                self.inst_alu_bit(value, 7);
            }
            // BIT 7, A
            0x7F => self.inst_alu_bit(self.registers.a, 7),
            // RES 0, B
            0x80 => self.registers.b = self.inst_alu_res(self.registers.b, 0),
            // RES 0, C
            0x81 => self.registers.c = self.inst_alu_res(self.registers.c, 0),
            // RES 0, D
            0x82 => self.registers.d = self.inst_alu_res(self.registers.d, 0),
            // RES 0, E
            0x83 => self.registers.e = self.inst_alu_res(self.registers.e, 0),
            // RES 0, H
            0x84 => self.registers.h = self.inst_alu_res(self.registers.h, 0),
            // RES 0, L
            0x85 => self.registers.l = self.inst_alu_res(self.registers.l, 0),
            // RES 0, (HL)
            0x86 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 0);
                self.set_byte_in_memory(addr, result);
            }
            // RES 0, A
            0x87 => self.registers.a = self.inst_alu_res(self.registers.a, 0),
            // RES 1, B
            0x88 => self.registers.b = self.inst_alu_res(self.registers.b, 1),
            // RES 1, C
            0x89 => self.registers.c = self.inst_alu_res(self.registers.c, 1),
            // RES 1, D
            0x8A => self.registers.d = self.inst_alu_res(self.registers.d, 1),
            // RES 1, E
            0x8B => self.registers.e = self.inst_alu_res(self.registers.e, 1),
            // RES 1, H
            0x8C => self.registers.h = self.inst_alu_res(self.registers.h, 1),
            // RES 1, L
            0x8D => self.registers.l = self.inst_alu_res(self.registers.l, 1),
            // RES 1, (HL)
            0x8E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 1);
                self.set_byte_in_memory(addr, result);
            }
            // RES 1, A
            0x8F => self.registers.a = self.inst_alu_res(self.registers.a, 1),
            // RES 2, B
            0x90 => self.registers.b = self.inst_alu_res(self.registers.b, 2),
            // RES 2, C
            0x91 => self.registers.c = self.inst_alu_res(self.registers.c, 2),
            // RES 2, D
            0x92 => self.registers.d = self.inst_alu_res(self.registers.d, 2),
            // RES 2, E
            0x93 => self.registers.e = self.inst_alu_res(self.registers.e, 2),
            // RES 2, H
            0x94 => self.registers.h = self.inst_alu_res(self.registers.h, 2),
            // RES 2, L
            0x95 => self.registers.l = self.inst_alu_res(self.registers.l, 2),
            // RES 2, (HL)
            0x96 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 2);
                self.set_byte_in_memory(addr, result);
            }
            // RES 2, A
            0x97 => self.registers.a = self.inst_alu_res(self.registers.a, 2),
            // RES 3, B
            0x98 => self.registers.b = self.inst_alu_res(self.registers.b, 3),
            // RES 3, C
            0x99 => self.registers.c = self.inst_alu_res(self.registers.c, 3),
            // RES 3, D
            0x9A => self.registers.d = self.inst_alu_res(self.registers.d, 3),
            // RES 3, E
            0x9B => self.registers.e = self.inst_alu_res(self.registers.e, 3),
            // RES 3, H
            0x9C => self.registers.h = self.inst_alu_res(self.registers.h, 3),
            // RES 3, L
            0x9D => self.registers.l = self.inst_alu_res(self.registers.l, 3),
            // RES 3, (HL)
            0x9E => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 3);
                self.set_byte_in_memory(addr, result);
            }
            // RES 3, A
            0x9F => self.registers.a = self.inst_alu_res(self.registers.a, 3),
            // RES 4, B
            0xA0 => self.registers.b = self.inst_alu_res(self.registers.b, 4),
            // RES 4, C
            0xA1 => self.registers.c = self.inst_alu_res(self.registers.c, 4),
            // RES 4, D
            0xA2 => self.registers.d = self.inst_alu_res(self.registers.d, 4),
            // RES 4, E
            0xA3 => self.registers.e = self.inst_alu_res(self.registers.e, 4),
            // RES 4, H
            0xA4 => self.registers.h = self.inst_alu_res(self.registers.h, 4),
            // RES 4, L
            0xA5 => self.registers.l = self.inst_alu_res(self.registers.l, 4),
            // RES 4, (HL)
            0xA6 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 4);
                self.set_byte_in_memory(addr, result);
            }
            // RES 4, A
            0xA7 => self.registers.a = self.inst_alu_res(self.registers.a, 4),
            // RES 5, B
            0xA8 => self.registers.b = self.inst_alu_res(self.registers.b, 5),
            // RES 5, C
            0xA9 => self.registers.c = self.inst_alu_res(self.registers.c, 5),
            // RES 5, D
            0xAA => self.registers.d = self.inst_alu_res(self.registers.d, 5),
            // RES 5, E
            0xAB => self.registers.e = self.inst_alu_res(self.registers.e, 5),
            // RES 5, H
            0xAC => self.registers.h = self.inst_alu_res(self.registers.h, 5),
            // RES 5, L
            0xAD => self.registers.l = self.inst_alu_res(self.registers.l, 5),
            // RES 5, (HL)
            0xAE => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 5);
                self.set_byte_in_memory(addr, result);
            }
            // RES 5, A
            0xAF => self.registers.a = self.inst_alu_res(self.registers.a, 5),
            // RES 6, B
            0xB0 => self.registers.b = self.inst_alu_res(self.registers.b, 6),
            // RES 6, C
            0xB1 => self.registers.c = self.inst_alu_res(self.registers.c, 6),
            // RES 6, D
            0xB2 => self.registers.d = self.inst_alu_res(self.registers.d, 6),
            // RES 6, E
            0xB3 => self.registers.e = self.inst_alu_res(self.registers.e, 6),
            // RES 6, H
            0xB4 => self.registers.h = self.inst_alu_res(self.registers.h, 6),
            // RES 6, L
            0xB5 => self.registers.l = self.inst_alu_res(self.registers.l, 6),
            // RES 6, (HL)
            0xB6 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 6);
                self.set_byte_in_memory(addr, result);
            }
            // RES 6, A
            0xB7 => self.registers.a = self.inst_alu_res(self.registers.a, 6),
            // RES 7, B
            0xB8 => self.registers.b = self.inst_alu_res(self.registers.b, 7),
            // RES 7, C
            0xB9 => self.registers.c = self.inst_alu_res(self.registers.c, 7),
            // RES 7, D
            0xBA => self.registers.d = self.inst_alu_res(self.registers.d, 7),
            // RES 7, E
            0xBB => self.registers.e = self.inst_alu_res(self.registers.e, 7),
            // RES 7, H
            0xBC => self.registers.h = self.inst_alu_res(self.registers.h, 7),
            // RES 7, L
            0xBD => self.registers.l = self.inst_alu_res(self.registers.l, 7),
            // RES 7, (HL)
            0xBE => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_res(value, 7);
                self.set_byte_in_memory(addr, result);
            }
            // RES 7, A
            0xBF => self.registers.a = self.inst_alu_res(self.registers.a, 7),
            // SET 0, B
            0xC0 => self.registers.b = self.inst_alu_set(self.registers.b, 0),
            // SET 0, C
            0xC1 => self.registers.c = self.inst_alu_set(self.registers.c, 0),
            // SET 0, D
            0xC2 => self.registers.d = self.inst_alu_set(self.registers.d, 0),
            // SET 0, E
            0xC3 => self.registers.e = self.inst_alu_set(self.registers.e, 0),
            // SET 0, H
            0xC4 => self.registers.h = self.inst_alu_set(self.registers.h, 0),
            // SET 0, L
            0xC5 => self.registers.l = self.inst_alu_set(self.registers.l, 0),
            // SET 0, (HL)
            0xC6 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 0);
                self.set_byte_in_memory(addr, result);
            }
            // SET 0, A
            0xC7 => self.registers.a = self.inst_alu_set(self.registers.a, 0),
            // SET 1, B
            0xC8 => self.registers.b = self.inst_alu_set(self.registers.b, 1),
            // SET 1, C
            0xC9 => self.registers.c = self.inst_alu_set(self.registers.c, 1),
            // SET 1, D
            0xCA => self.registers.d = self.inst_alu_set(self.registers.d, 1),
            // SET 1, E
            0xCB => self.registers.e = self.inst_alu_set(self.registers.e, 1),
            // SET 1, H
            0xCC => self.registers.h = self.inst_alu_set(self.registers.h, 1),
            // SET 1, L
            0xCD => self.registers.l = self.inst_alu_set(self.registers.l, 1),
            // SET 1, (HL)
            0xCE => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 1);
                self.set_byte_in_memory(addr, result);
            }
            // SET 1, A
            0xCF => self.registers.a = self.inst_alu_set(self.registers.a, 1),
            // SET 2, B
            0xD0 => self.registers.b = self.inst_alu_set(self.registers.b, 2),
            // SET 2, C
            0xD1 => self.registers.c = self.inst_alu_set(self.registers.c, 2),
            // SET 2, D
            0xD2 => self.registers.d = self.inst_alu_set(self.registers.d, 2),
            // SET 2, E
            0xD3 => self.registers.e = self.inst_alu_set(self.registers.e, 2),
            // SET 2, H
            0xD4 => self.registers.h = self.inst_alu_set(self.registers.h, 2),
            // SET 2, L
            0xD5 => self.registers.l = self.inst_alu_set(self.registers.l, 2),
            // SET 2, (HL)
            0xD6 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 2);
                self.set_byte_in_memory(addr, result);
            }
            // SET 2, A
            0xD7 => self.registers.a = self.inst_alu_set(self.registers.a, 2),
            // SET 3, B
            0xD8 => self.registers.b = self.inst_alu_set(self.registers.b, 3),
            // SET 3, C
            0xD9 => self.registers.c = self.inst_alu_set(self.registers.c, 3),
            // SET 3, D
            0xDA => self.registers.d = self.inst_alu_set(self.registers.d, 3),
            // SET 3, E
            0xDB => self.registers.e = self.inst_alu_set(self.registers.e, 3),
            // SET 3, H
            0xDC => self.registers.h = self.inst_alu_set(self.registers.h, 3),
            // SET 3, L
            0xDD => self.registers.l = self.inst_alu_set(self.registers.l, 3),
            // SET 3, (HL)
            0xDE => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 3);
                self.set_byte_in_memory(addr, result);
            }
            // SET 3, A
            0xDF => self.registers.a = self.inst_alu_set(self.registers.a, 3),
            // SET 4, B
            0xE0 => self.registers.b = self.inst_alu_set(self.registers.b, 4),
            // SET 4, C
            0xE1 => self.registers.c = self.inst_alu_set(self.registers.c, 4),
            // SET 4, D
            0xE2 => self.registers.d = self.inst_alu_set(self.registers.d, 4),
            // SET 4, E
            0xE3 => self.registers.e = self.inst_alu_set(self.registers.e, 4),
            // SET 4, H
            0xE4 => self.registers.h = self.inst_alu_set(self.registers.h, 4),
            // SET 4, L
            0xE5 => self.registers.l = self.inst_alu_set(self.registers.l, 4),
            // SET 4, (HL)
            0xE6 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 4);
                self.set_byte_in_memory(addr, result);
            }
            // SET 4, A
            0xE7 => self.registers.a = self.inst_alu_set(self.registers.a, 4),
            // SET 5, B
            0xE8 => self.registers.b = self.inst_alu_set(self.registers.b, 5),
            // SET 5, C
            0xE9 => self.registers.c = self.inst_alu_set(self.registers.c, 5),
            // SET 5, D
            0xEA => self.registers.d = self.inst_alu_set(self.registers.d, 5),
            // SET 5, E
            0xEB => self.registers.e = self.inst_alu_set(self.registers.e, 5),
            // SET 5, H
            0xEC => self.registers.h = self.inst_alu_set(self.registers.h, 5),
            // SET 5, L
            0xED => self.registers.l = self.inst_alu_set(self.registers.l, 5),
            // SET 5, (HL)
            0xEE => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 5);
                self.set_byte_in_memory(addr, result);
            }
            // SET 5, A
            0xEF => self.registers.a = self.inst_alu_set(self.registers.a, 5),
            // SET 6, B
            0xF0 => self.registers.b = self.inst_alu_set(self.registers.b, 6),
            // SET 6, C
            0xF1 => self.registers.c = self.inst_alu_set(self.registers.c, 6),
            // SET 6, D
            0xF2 => self.registers.d = self.inst_alu_set(self.registers.d, 6),
            // SET 6, E
            0xF3 => self.registers.e = self.inst_alu_set(self.registers.e, 6),
            // SET 6, H
            0xF4 => self.registers.h = self.inst_alu_set(self.registers.h, 6),
            // SET 6, L
            0xF5 => self.registers.l = self.inst_alu_set(self.registers.l, 6),
            // SET 6, (HL)
            0xF6 => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 6);
                self.set_byte_in_memory(addr, result);
            }
            // SET 6, A
            0xF7 => self.registers.a = self.inst_alu_set(self.registers.a, 6),
            // SET 7, B
            0xF8 => self.registers.b = self.inst_alu_set(self.registers.b, 7),
            // SET 7, C
            0xF9 => self.registers.c = self.inst_alu_set(self.registers.c, 7),
            // SET 7, D
            0xFA => self.registers.d = self.inst_alu_set(self.registers.d, 7),
            // SET 7, E
            0xFB => self.registers.e = self.inst_alu_set(self.registers.e, 7),
            // SET 7, H
            0xFC => self.registers.h = self.inst_alu_set(self.registers.h, 7),
            // SET 7, L
            0xFD => self.registers.l = self.inst_alu_set(self.registers.l, 7),
            // SET 7, (HL)
            0xFE => {
                let addr = self.registers.hl();
                let value = self.get_byte_in_memory(addr);
                let result = self.inst_alu_set(value, 7);
                self.set_byte_in_memory(addr, result);
            }
            // SET 7, A
            0xFF => self.registers.a = self.inst_alu_set(self.registers.a, 7),
        };
        CB_CYCLES[cb_code as usize]
    }
}
