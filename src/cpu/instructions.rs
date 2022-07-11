use crate::cpu::registers::CpuFlag;
use crate::cpu::Cpu;

// CPU instruction functions
impl Cpu {
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

    // Rotate value left through Carry flag.
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 7 data.
    pub fn inst_alu_rl(&mut self, value: u8) -> u8 {
        let has_carry = (value & 0x80) >> 7 == 0x01;
        let result = (value << 1) + (self.registers.has_flag(CpuFlag::C) as u8);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, has_carry);
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

    // Rotate value right through Carry flag.
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 0 data.
    pub fn inst_alu_rr(&mut self, value: u8) -> u8 {
        let has_carry = value & 0x01 == 0x01;
        let result = if self.registers.has_flag(CpuFlag::C) {
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

    // Subtract value from A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - Set.
    // H - Set if no borrow from bit 4.
    // C - Set if no borrow
    pub fn inst_alu_sub(&mut self, value: u8) {
        let curr = self.registers.a;
        let result = curr.wrapping_sub(value);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, true);
        self.registers
            .set_flag(CpuFlag::H, (curr & 0x0F) < (value & 0x0F));
        self.registers
            .set_flag(CpuFlag::C, (curr as u16) < (value as u16));
        self.registers.a = result;
    }

    // Add value + Carry flag to A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - Set if carry from bit 3.
    // C - Set if carry from bit 7.
    pub fn inst_alu_adc(&mut self, value: u8) {
        let curr = self.registers.a;
        let carry = u8::from(self.registers.has_flag(CpuFlag::C));
        let result = curr.wrapping_add(value).wrapping_add(carry);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(
            CpuFlag::H,
            (curr & 0x0F) + (value & 0x0F) + (carry & 0x0F) > 0x0F,
        );
        self.registers.set_flag(
            CpuFlag::C,
            (curr as u16) + (value as u16) + (carry as u16) > 0xFF,
        );
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

    // Add n to current address and jump to it.
    // n = one byte signed immediate value
    pub fn inst_alu_jr(&mut self, n: u8) {
        let n = n as i8;
        self.registers.pc = ((u32::from(self.registers.pc) as i32) + i32::from(n)) as u16;
    }

    // Decimal adjust register A. This instruction adjusts register A so that the correct
    // representation of Binary Coded Decimal (BCD) is obtained.
    //
    // Flags affected:
    // Z - Set if register A is zero.
    // N - Not affected.
    // H - unset (set to false).
    // C - Set or reset according to operation
    pub fn inst_alu_daa(&mut self) {
        let mut a = self.registers.a;
        let mut adjust = if self.registers.has_flag(CpuFlag::C) {
            0x60
        } else {
            0x00
        };
        if self.registers.has_flag(CpuFlag::H) {
            adjust |= 0x06;
        };
        if !self.registers.has_flag(CpuFlag::N) {
            if a & 0x0F > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }
        self.registers.set_flag(CpuFlag::Z, a == 0x00);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, adjust >= 0x60);
        self.registers.a = a;
    }

    // Complement A register. (Flip all bits.)
    //
    // Flags affected:
    // Z - Not affected.
    // N - Set.
    // H - Set.
    // C - Not affected.
    pub fn inst_alu_cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.set_flag(CpuFlag::N, true);
        self.registers.set_flag(CpuFlag::H, true);
    }

    // Set Carry flag.
    //
    // Flags affected:
    // Z - Not affected.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Set.
    pub fn inst_alu_scf(&mut self) {
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, true);
    }

    // Complement carry flag. If C flag is set, then reset it. If C flag is
    // reset, then set it.
    //
    // Flags affected:
    // Z - Not affected.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Complemented.
    pub fn inst_alu_ccf(&mut self) {
        let value = !self.registers.has_flag(CpuFlag::C);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, value);
    }

    // Subtract value + Carry flag from A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - Set.
    // H - Set if no borrow from bit 4.
    // C - Set if no borrow.
    pub fn inst_alu_sbc(&mut self, value: u8) {
        let curr = self.registers.a;
        let carry = u8::from(self.registers.has_flag(CpuFlag::C));
        let result = curr.wrapping_sub(value).wrapping_sub(carry);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, true);
        self.registers
            .set_flag(CpuFlag::H, (curr & 0x0F) < (value & 0x0F) + carry);
        self.registers.set_flag(
            CpuFlag::C,
            (curr as u16) < ((value as u16) + (carry as u16)),
        );
        self.registers.a = result;
    }

    // Logically AND value with A, result in A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - Set.
    // C - unset (set to false).
    pub fn inst_alu_and(&mut self, value: u8) {
        let result = self.registers.a & value;
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, true);
        self.registers.set_flag(CpuFlag::C, false);
        self.registers.a = result;
    }

    // Logical exclusive OR value with register A, result in A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - unset (set to false).
    pub fn inst_alu_xor(&mut self, value: u8) {
        let result = self.registers.a ^ value;
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, false);
        self.registers.a = result;
    }

    // Logical OR value with register A, result in A.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - unset (set to false).
    pub fn inst_alu_or(&mut self, value: u8) {
        let result = self.registers.a | value;
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, false);
        self.registers.a = result;
    }

    // Compare A with value.
    // This is basically an A - value subtraction instruction but the results are thrown away.
    // value = A,B,C,D,E,H,L,(HL),#
    //
    // Flags affected:
    // Z - Set if result is zero. (Set if A = value.)
    // N - Set.
    // H - Set if no borrow from bit 4.
    // C - Set for no borrow. (Set if A < n.)
    pub fn inst_alu_cp(&mut self, value: u8) {
        let curr = self.registers.a;
        self.inst_alu_sub(value);
        self.registers.a = curr;
    }

    // Add value to Stack Pointer (SP).
    // value = one byte signed immediate value (#).
    //
    // Flags affected:
    // Z - unset (set to false).
    // N - unset (set to false).
    // H - Set or unset according to operation.
    // C - Set or unset according to operation.
    pub fn inst_alu_add_sp(&mut self, value: u8) {
        let curr = self.registers.sp;
        let val = i16::from(value as i8) as u16;
        self.registers.set_flag(CpuFlag::Z, false);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers
            .set_flag(CpuFlag::H, (curr & 0x000F) + (val & 0x000F) > 0x000F);
        self.registers
            .set_flag(CpuFlag::C, (curr & 0x00FF) + (val & 0x00FF) > 0x00FF);
        self.registers.sp = curr.wrapping_add(val);
    }

    // Shift value left into Carry. LSB of value set to 0.
    // value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 7 data
    pub fn inst_alu_sla(&mut self, value: u8) -> u8 {
        let has_carry = (value & 0x80) >> 7 == 0x01;
        let result = value << 1;
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, has_carry);
        result
    }

    // Shift value right into Carry. MSB doesn't change.
    // value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 0 data.
    pub fn inst_alu_sra(&mut self, value: u8) -> u8 {
        let has_carry = value & 0x01 == 0x01;
        let result = (value >> 1) | (value & 0x80);
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, has_carry);
        result
    }

    // Shift value right into Carry. MSB set to 0.
    // value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - Contains old bit 0 data.
    pub fn inst_alu_srl(&mut self, value: u8) -> u8 {
        let has_carry = value & 0x01 == 0x01;
        let result = value >> 1;
        self.registers.set_flag(CpuFlag::Z, result == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, has_carry);
        result
    }

    // Swap upper & lower nibles of value.
    // value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if result is zero.
    // N - unset (set to false).
    // H - unset (set to false).
    // C - unset (set to false).
    pub fn inst_alu_swap(&mut self, value: u8) -> u8 {
        self.registers.set_flag(CpuFlag::Z, value == 0x00);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, false);
        self.registers.set_flag(CpuFlag::C, false);
        (value >> 4) | (value << 4)
    }

    // Test bit (bit) in register value.
    // bit = 0 - 7, value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:
    // Z - Set if bit (bit) of register (value) is 0.
    // N - unset (set to false).
    // H - Set.
    // C - Not affected
    pub fn inst_alu_bit(&mut self, value: u8, bit: u8) {
        let result = value & (1 << bit) == 0x00;
        self.registers.set_flag(CpuFlag::Z, result);
        self.registers.set_flag(CpuFlag::N, false);
        self.registers.set_flag(CpuFlag::H, true);
    }

    // Reset bit (bit) in register value.
    // bit = 0 - 7, value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:  None
    pub fn inst_alu_res(&mut self, value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }

    // Set bit (bit) in register value.
    // bit = 0 - 7, value = A,B,C,D,E,H,L,(HL)
    //
    // Flags affected:  None.
    pub fn inst_alu_set(&mut self, value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }
}
