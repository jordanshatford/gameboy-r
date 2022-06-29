pub trait Memory {
    fn get_byte(&self, addr: u16) -> u8;

    fn set_byte(&mut self, addr: u16, value: u8);

    fn get_word(&self, addr: u16) -> u16 {
        u16::from(self.get_byte(addr)) | (u16::from(self.get_byte(addr + 1)) << 8)
    }

    fn set_word(&mut self, addr: u16, value: u16) {
        self.set_byte(addr, (value & 0xFF) as u8);
        self.set_byte(addr + 1, (value >> 8) as u8)
    }
}
