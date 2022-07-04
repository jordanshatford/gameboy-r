use crate::cpu::registers::CpuFlag;
use crate::cpu::CPU;

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
