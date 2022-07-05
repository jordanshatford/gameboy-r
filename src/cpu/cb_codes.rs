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
            0x01 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RLC D
            0x02 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RLC E
            0x03 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RLC H
            0x04 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RLC L
            0x05 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RLC (HL)
            0x06 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RLC A
            0x07 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC B
            0x08 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC C
            0x09 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC D
            0x0A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC E
            0x0B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC H
            0x0C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC L
            0x0D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC (HL)
            0x0E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RRC A
            0x0F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL B
            0x10 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL C
            0x11 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL D
            0x12 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL E
            0x13 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL H
            0x14 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL L
            0x15 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL (HL)
            0x16 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RL A
            0x17 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR B
            0x18 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR C
            0x19 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR D
            0x1A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR E
            0x1B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR H
            0x1C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR L
            0x1D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR (HL)
            0x1E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RR A
            0x1F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA B
            0x20 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA C
            0x21 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA D
            0x22 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA E
            0x23 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA H
            0x24 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA L
            0x25 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA (HL)
            0x26 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SLA A
            0x27 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA B
            0x28 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA C
            0x29 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA D
            0x2A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA E
            0x2B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA H
            0x2C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA L
            0x2D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA (HL)
            0x2E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRA A
            0x2F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP B
            0x30 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP C
            0x31 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP D
            0x32 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP E
            0x33 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP H
            0x34 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP L
            0x35 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP (HL)
            0x36 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SWAP A
            0x37 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL B
            0x38 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL C
            0x39 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL D
            0x3A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL E
            0x3B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL H
            0x3C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL L
            0x3D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL (HL)
            0x3E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SRL A
            0x3F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, B
            0x40 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, C
            0x41 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, D
            0x42 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, E
            0x43 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, H
            0x44 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, L
            0x45 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, (HL)
            0x46 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 0, A
            0x47 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, B
            0x48 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, C
            0x49 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, D
            0x4A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, E
            0x4B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, H
            0x4C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, L
            0x4D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, (HL)
            0x4E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 1, A
            0x4F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, B
            0x50 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, C
            0x51 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, D
            0x52 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, E
            0x53 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, H
            0x54 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, L
            0x55 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, (HL)
            0x56 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 2, A
            0x57 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, B
            0x58 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, C
            0x59 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, D
            0x5A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, E
            0x5B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, H
            0x5C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, L
            0x5D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, (HL)
            0x5E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 3, A
            0x5F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, B
            0x60 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, C
            0x61 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, D
            0x62 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, E
            0x63 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, H
            0x64 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, L
            0x65 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, (HL)
            0x66 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 4, A
            0x67 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, B
            0x68 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, C
            0x69 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, D
            0x6A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, E
            0x6B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, H
            0x6C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, L
            0x6D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, (HL)
            0x6E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 5, A
            0x6F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, B
            0x70 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, C
            0x71 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, D
            0x72 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, E
            0x73 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, H
            0x74 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, L
            0x75 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, (HL)
            0x76 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 6, A
            0x77 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, B
            0x78 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, C
            0x79 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, D
            0x7A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, E
            0x7B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, H
            0x7C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, L
            0x7D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, (HL)
            0x7E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // BIT 7, A
            0x7F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, B
            0x80 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, C
            0x81 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, D
            0x82 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, E
            0x83 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, H
            0x84 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, L
            0x85 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, (HL)
            0x86 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 0, A
            0x87 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, B
            0x88 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, C
            0x89 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, D
            0x8A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, E
            0x8B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, H
            0x8C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, L
            0x8D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, (HL)
            0x8E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 1, A
            0x8F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, B
            0x90 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, C
            0x91 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, D
            0x92 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, E
            0x93 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, H
            0x94 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, L
            0x95 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, (HL)
            0x96 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 2, A
            0x97 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, B
            0x98 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, C
            0x99 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, D
            0x9A => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, E
            0x9B => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, H
            0x9C => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, L
            0x9D => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, (HL)
            0x9E => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 3, A
            0x9F => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, B
            0xA0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, C
            0xA1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, D
            0xA2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, E
            0xA3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, H
            0xA4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, L
            0xA5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, (HL)
            0xA6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 4, A
            0xA7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, B
            0xA8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, C
            0xA9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, D
            0xAA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, E
            0xAB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, H
            0xAC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, L
            0xAD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, (HL)
            0xAE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 5, A
            0xAF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, B
            0xB0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, C
            0xB1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, D
            0xB2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, E
            0xB3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, H
            0xB4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, L
            0xB5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, (HL)
            0xB6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 6, A
            0xB7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, B
            0xB8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, C
            0xB9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, D
            0xBA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, E
            0xBB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, H
            0xBC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, L
            0xBD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, (HL)
            0xBE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // RES 7, A
            0xBF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, B
            0xC0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, C
            0xC1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, D
            0xC2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, E
            0xC3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, H
            0xC4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, L
            0xC5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, (HL)
            0xC6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 0, A
            0xC7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, B
            0xC8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, C
            0xC9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, D
            0xCA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, E
            0xCB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, H
            0xCC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, L
            0xCD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, (HL)
            0xCE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 1, A
            0xCF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, B
            0xD0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, C
            0xD1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, D
            0xD2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, E
            0xD3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, H
            0xD4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, L
            0xD5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, (HL)
            0xD6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 2, A
            0xD7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, B
            0xD8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, C
            0xD9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, D
            0xDA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, E
            0xDB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, H
            0xDC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, L
            0xDD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, (HL)
            0xDE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 3, A
            0xDF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, B
            0xE0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, C
            0xE1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, D
            0xE2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, E
            0xE3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, H
            0xE4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, L
            0xE5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, (HL)
            0xE6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 4, A
            0xE7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, B
            0xE8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, C
            0xE9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, D
            0xEA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, E
            0xEB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, H
            0xEC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, L
            0xED => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, (HL)
            0xEE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 5, A
            0xEF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, B
            0xF0 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, C
            0xF1 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, D
            0xF2 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, E
            0xF3 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, H
            0xF4 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, L
            0xF5 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, (HL)
            0xF6 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 6, A
            0xF7 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, B
            0xF8 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, C
            0xF9 => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, D
            0xFA => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, E
            0xFB => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, H
            0xFC => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, L
            0xFD => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, (HL)
            0xFE => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
            // SET 7, A
            0xFF => panic!("cpu: CB code not implemented {:#04X?}", cb_code),
        };
        CB_CYCLES[cb_code as usize]
    }
}
