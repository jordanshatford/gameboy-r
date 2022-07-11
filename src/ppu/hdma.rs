use crate::memory::Memory;

#[derive(Debug, Eq, PartialEq)]
pub enum HdmaMode {
    // When using this transfer method, all data is transferred at once. The execution of the program
    // is halted until the transfer has completed. Note that the General Purpose DMA blindly attempts
    // to copy the data, even if the CD controller is currently accessing VRAM. So General Purpose DMA
    // should be used only if the Display is disabled, or during V-Blank, or (for rather short blocks)
    // during H-Blank. The execution of the program continues when the transfer has been completed, and
    // FF55 then contains a value of FFh.
    Gdma,
    // The H-Blank DMA transfers 10h bytes of data during each H-Blank, ie. at LY=0-143, no data is
    // transferred during V-Blank (LY=144-153), but the transfer will then continue at LY=00. The
    // execution of the program is halted during the separate transfers, but the program execution
    // continues during the 'spaces' between each data block. Note that the program should not change
    // the Destination VRAM bank (FF4F), or the Source ROM/RAM bank (in case data is transferred from
    // bankable memory) until the transfer has completed! (The transfer should be paused as described
    // below while the banks are switched) Reading from Register FF55 returns the remaining length (divided
    // by 10h, minus 1), a value of 0FFh indicates that the transfer has completed. It is also
    // possible to terminate an active H-Blank transfer by writing zero to Bit 7 of FF55. In that case
    // reading from FF55 will return how many $10 "blocks" remained (minus 1) in the lower 7 bits, but Bit
    // 7 will be read as "1". Stopping the transfer doesn't set HDMA1-4 to $FF.
    Hdma,
}

pub struct Hdma {
    pub source: u16,
    pub destination: u16,
    pub active: bool,
    pub mode: HdmaMode,
    pub remain: u8,
}

impl Hdma {
    pub fn new() -> Hdma {
        Hdma {
            source: 0x0000,
            destination: 0x0000,
            active: false,
            mode: HdmaMode::Gdma,
            remain: 0x00,
        }
    }
}

impl Memory for Hdma {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // HDMA1 - CGB Mode Only - New DMA Source, High
            0xFF51 => (self.source >> 8) as u8,
            // HDMA2 - CGB Mode Only - New DMA Source, Low
            0xFF52 => self.source as u8,
            // HDMA3 - CGB Mode Only - New DMA Destination, High
            0xFF53 => (self.destination >> 8) as u8,
            // HDMA4 - CGB Mode Only - New DMA Destination, Low
            0xFF54 => self.destination as u8,
            // HDMA5 - CGB Mode Only - New DMA Length/Mode/Start
            // Bit7=0 - General Purpose DMA
            // When using this transfer method, all data is transferred at once. The execution of the program
            // is halted until the transfer has completed. Note that the General Purpose DMA blindly attempts
            // to copy the data, even if the LCD controller is currently accessing VRAM. So General Purpose DMA
            // should be used only if the Display is disabled, or during V-Blank, or (for rather short blocks)
            // during H-Blank.
            // The execution of the program continues when the transfer has been completed, and FF55 then
            // contains a value if FFh.
            // Bit7=1 - H-Blank DMA
            // The H-Blank DMA transfers 10h bytes of data during each H-Blank, ie. at LY=0-143, no data is
            // transferred during V-Blank (LY=144-153), but the transfer will then continue at LY=00. The
            // execution of the program is halted during the separate transfers, but the program execution
            // continues during the 'spaces' between each data block.
            // Note that the program may not change the Destination VRAM bank (FF4F), or the Source ROM/RAM bank
            // (in case data is transferred from bankable memory) until the transfer has completed! Reading from
            // Register FF55 returns the remaining length (divided by 10h, minus 1), a value of 0FFh indicates
            // that the transfer has completed. It is also possible to terminate an active H-Blank transfer by
            // writing zero to Bit 7 of FF55. In that case reading from FF55 may return any value for the lower
            // 7 bits, but Bit 7 will be read as "1".
            0xFF55 => self.remain | if self.active { 0x00 } else { 0x80 },
            _ => panic!("hdma: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // HDMA1 - CGB Mode Only - New DMA Source, High
            0xFF51 => self.source = (u16::from(value) << 8) | (self.source & 0x00FF),
            // HDMA2 - CGB Mode Only - New DMA Source, Low
            0xFF52 => self.source = (self.source & 0xFF00) | u16::from(value & 0xF0),
            // HDMA3 - CGB Mode Only - New DMA Destination, High
            0xFF53 => {
                self.destination =
                    0x8000 | (u16::from(value & 0x1F) << 8) | (self.destination & 0x00FF)
            }
            // HDMA4 - CGB Mode Only - New DMA Destination, Low
            0xFF54 => self.destination = (self.destination & 0xFF00) | u16::from(value & 0xF0),
            // HDMA5 - CGB Mode Only - New DMA Length/Mode/Start
            // Writing to FF55 starts the transfer, the lower 7 bits of FF55 specify the Transfer Length
            // (divided by 10h, minus 1). Ie. lengths of 10h-800h bytes can be defined by the values 00h-7Fh.
            // And the upper bit of FF55 indicates the Transfer Mode
            0xFF55 => {
                if self.active && self.mode == HdmaMode::Hdma {
                    if value & 0x80 == 0x00 {
                        self.active = false;
                    };
                    return;
                }
                self.active = true;
                self.remain = value & 0x7F;
                self.mode = if value & 0x80 != 0x00 {
                    HdmaMode::Hdma
                } else {
                    HdmaMode::Gdma
                };
            }
            _ => panic!("hdma: invalid address {:#06X?}", addr),
        }
    }
}
