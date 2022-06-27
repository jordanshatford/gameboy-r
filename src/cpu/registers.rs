
// Registers
//   16bit Hi   Lo   Name/Function
//   AF    A    -    Accumulator & Flags
//   BC    B    C    BC
//   DE    D    E    DE
//   HL    H    L    HL
//   SP    -    -    Stack Pointer
//   PC    -    -    Program Counter/Pointer
// As shown above, most registers can be accessed either as one 16bit register,
// or as two separate 8bit registers.

#[derive(Debug, Copy, Clone)]
pub struct Registers {
    // Program counter / pointer
    pub pc: u16,
    // Stack pointer
    pub sp: u16,
    // Accumulator
    pub a: u8,
    // Flags (not directly mutable)
    f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

// The Flag Register (lower 8bit of AF register)
//   Bit  Name  Set Clr  Expl.
//   7    zf    Z   NZ   Zero Flag
//   6    n     -   -    Add/Sub-Flag (BCD)
//   5    h     -   -    Half Carry Flag (BCD)
//   4    cy    C   NC   Carry Flag
//   3-0  -     -   -    Not used (always zero)
// Conatins the result from the recent instruction which has affected flags.

#[derive(Copy, Clone)]
pub enum CpuFlag {
    Z = 0b1000_0000, // Zero
    N = 0b0100_0000, // Subtract
    H = 0b0010_0000, // Half Carry
    C = 0b0001_0000, // Carry
}

// When the GameBoy is powered up, a 256 byte program starting at memory
// location 0 is executed. This program is located in a ROM inside the
// GameBoy. The first thing the program does is read the cartridge locations
// from $104 to $133 and place this graphic of a Nintendo logo on the screen
// at the top. This image is then scrolled until it is in the middle of the
// screen. Two musical notes are then played on the internal speaker. Again,
// the cartridge locations $104 to $133 are read but this time they are
// compared with a table in the internal rom. If any byte fails to compare,
// then the GameBoy stops comparing bytes and simply halts all operations. 
// If all locations compare the same, then the GameBoy starts adding all of 
// the bytes in the cartridge from $134 to $14d. A value of 25 decimal is added
// to this total. If the least significant byte of the result is a not a zero,
// then the GameBoy will stop doing anything. If it is a zero, then the internal
// ROM is disabled and cartridge program execution begins at location $100 with
// the following register values:
//    AF=$01B0
//    BC=$0013
//    DE=$00D8
//    HL=$014D
//    Stack Pointer=$FFFE
impl Registers { 
    pub fn new() -> Registers {
        Registers {
            pc: 0x0100,
            sp: 0xFFFE,
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
        }
    }

    pub fn af(&self) -> u16 {
        // f gets bitwise AND with 0xF0 because the last 4 bits are always 0
        ((self.a as u16) << 8) | ((self.f & 0xF0) as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0x00F0) as u8; 
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, value: u16) { 
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    pub fn set_flag(&mut self, flag: CpuFlag, set: bool) {
        let mask = flag as u8;
        match set {
            // set the flag
            true => self.f |= mask,
            // unset the flag
            false => self.f &= !mask,
        }
        self.f &= 0xF0
    }

    pub fn has_flag(&self, flag: CpuFlag) -> bool {
        let mask = flag as u8;
        self.f & mask > 0        
    }

    #[cfg(test)]
    fn set_f(&mut self, value: u8)
    {
        self.f = value & 0xF0;
    }
}

#[cfg(test)]
mod test {
    use super::Registers;
    use super::CpuFlag;

    #[test]
    fn wide_registers() {
        let mut reg = Registers::new();
        reg.a = 0x12;
        reg.set_f(0x23);
        reg.b = 0x34;
        reg.c = 0x45;
        reg.d = 0x56;
        reg.e = 0x67;
        reg.h = 0x78;
        reg.l = 0x89;

        assert_eq!(reg.af(), 0x1220);
        assert_eq!(reg.bc(), 0x3445);
        assert_eq!(reg.de(), 0x5667);
        assert_eq!(reg.hl(), 0x7889);

        reg.set_af(0x1111);
        reg.set_bc(0x1111);
        reg.set_de(0x1111);
        reg.set_hl(0x1111);

        assert_eq!(reg.af(), 0x1110);
        assert_eq!(reg.bc(), 0x1111);
        assert_eq!(reg.de(), 0x1111);
        assert_eq!(reg.hl(), 0x1111);
    }

    #[test]
    fn flags() {
        let mut reg = Registers::new();
        let flags = [CpuFlag::C, CpuFlag::H, CpuFlag::N, CpuFlag::Z];

        assert_eq!(reg.f & 0x0F, 0);
        reg.set_f(0x00);

        for flag in flags {
            assert_eq!(reg.has_flag(flag), false);
            reg.set_flag(flag, true);
            assert_eq!(reg.has_flag(flag), true);
            reg.set_flag(flag, false);
            assert_eq!(reg.has_flag(flag), false);
        }        
    }
}
