// MBC3 (max 2MByte ROM and/or 64KByte RAM and Timer)
// Beside for the ability to access up to 2MB ROM (128 banks), and 64KB RAM (8 banks), the MBC3 also includes
// a built-in Real Time Clock (RTC). The RTC requires an external 32.768 kHz Quartz Oscillator, and an external
// battery (if it should continue to tick when the gameboy is turned off).

// 0000-3FFF - ROM Bank 00 (Read Only)
// Same as for MBC1.

// 4000-7FFF - ROM Bank 01-7F (Read Only)
// Same as for MBC1, except that accessing banks 20h, 40h, and 60h is supported now.

// A000-BFFF - RAM Bank 00-03, if any (Read/Write)

// A000-BFFF - RTC Register 08-0C (Read/Write)
// Depending on the current Bank Number/RTC Register selection (see below), this memory space is used to access an
// 8KByte external RAM Bank, or a single RTC Register.

// 0000-1FFF - RAM and Timer Enable (Write Only)
// Mostly the same as for MBC1, a value of 0Ah will enable reading and writing to external RAM - and to the RTC
// Registers! A value of 00h will disable either.

// 2000-3FFF - ROM Bank Number (Write Only)
// Same as for MBC1, except that the whole 7 bits of the RAM Bank Number are written directly to this address.
// As for the MBC1, writing a value of 00h, will select Bank 01h instead. All other values 01-7Fh select the
// corresponding ROM Banks.

// 4000-5FFF - RAM Bank Number - or - RTC Register Select (Write Only)
// As for the MBC1s RAM Banking Mode, writing a value in range for 00h-07h maps the corresponding external RAM Bank
// (if any) into memory at A000-BFFF. When writing a value of 08h-0Ch, this will map the corresponding RTC register
// into memory at A000-BFFF. That register could then be read/written by accessing any address in that area, typically
// that is done by using address A000.

// 6000-7FFF - Latch Clock Data (Write Only)
// When writing 00h, and then 01h to this register, the current time becomes latched into the RTC registers. The
// latched data will not change until it becomes latched again, by repeating the write 00h->01h procedure. This is
// supposed for <reading> from the RTC registers. This can be proven by reading the latched (frozen) time from the
// RTC registers, and then unlatch the registers to show the clock itself continues to tick in background.
// The Clock Counter Registers
//  08h  RTC S   Seconds   0-59 (0-3Bh)
//  09h  RTC M   Minutes   0-59 (0-3Bh)
//  0Ah  RTC H   Hours     0-23 (0-17h)
//  0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
//  0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
//        Bit 0  Most significant bit of Day Counter (Bit 8)
//        Bit 6  Halt (0=Active, 1=Stop Timer)
//        Bit 7  Day Counter Carry Bit (1=Counter Overflow)
// The Halt Flag is supposed to be set before <writing> to the RTC Registers.
// The Day Counter
// The total 9 bits of the Day Counter allow to count days in range from 0-511 (0-1FFh). The Day Counter Carry Bit becomes
// set when this value overflows. In that case the Carry Bit remains set until the program does reset it. Note that you
// can store an offset to the Day Counter in battery RAM. For example, every time you read a non-zero Day Counter, add
// this Counter to the offset in RAM, and reset the Counter to zero. This method allows to count any number of days,
// making your program Year-10000-Proof, provided that the cartridge gets used at least every 511 days.
// Delays
// When accessing the RTC Registers it is recommended to execute a 4ms delay (4 Cycles in Normal Speed Mode) between the
// separate accesses.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::cartridges::{Cartridge, Stable};
use crate::memory::Memory;

struct RealTimeClock {
    s: u8,
    m: u8,
    h: u8,
    dl: u8,
    dh: u8,
    zero: u64,
    save_path: PathBuf,
}

impl RealTimeClock {
    fn new(save_path: impl AsRef<Path>) -> RealTimeClock {
        let zero = match std::fs::read(save_path.as_ref()) {
            Ok(ok) => {
                let mut b: [u8; 8] = Default::default();
                b.copy_from_slice(&ok);
                u64::from_be_bytes(b)
            }
            Err(_) => SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        RealTimeClock {
            zero,
            s: 0,
            m: 0,
            h: 0,
            dl: 0,
            dh: 0,
            save_path: PathBuf::from(save_path.as_ref()),
        }
    }

    fn tic(&mut self) {
        let d = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.zero;

        self.s = (d % 60) as u8;
        self.m = (d / 60 % 60) as u8;
        self.h = (d / 3600 % 24) as u8;
        let days = (d / 3600 / 24) as u16;
        self.dl = (days % 256) as u8;
        match days {
            0x0000..=0x00FF => {}
            0x0100..=0x01FF => {
                self.dh |= 0x01;
            }
            _ => {
                self.dh |= 0x01;
                self.dh |= 0x80;
            }
        }
    }
}

impl Memory for RealTimeClock {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // 08h  RTC S   Seconds   0-59 (0-3Bh)
            0x08 => self.s,
            // 09h  RTC M   Minutes   0-59 (0-3Bh)
            0x09 => self.m,
            // 0Ah  RTC H   Hours     0-23 (0-17h)
            0x0A => self.h,
            // 0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
            0x0B => self.dl,
            // 0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
            //    Bit 0  Most significant bit of Day Counter (Bit 8)
            //    Bit 6  Halt (0=Active, 1=Stop Timer)
            //    Bit 7  Day Counter Carry Bit (1=Counter Overflow)
            0x0C => self.dh,
            _ => panic!("MBC3 (rtc): invalid address {:#04X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // 08h  RTC S   Seconds   0-59 (0-3Bh)
            0x08 => self.s = value,
            // 09h  RTC M   Minutes   0-59 (0-3Bh)
            0x09 => self.m = value,
            // 0Ah  RTC H   Hours     0-23 (0-17h)
            0x0A => self.h = value,
            // 0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
            0x0B => self.dl = value,
            // 0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
            //    Bit 0  Most significant bit of Day Counter (Bit 8)
            //    Bit 6  Halt (0=Active, 1=Stop Timer)
            //    Bit 7  Day Counter Carry Bit (1=Counter Overflow)
            0x0C => self.dh = value,
            _ => panic!("MBC3 (rtc): invalid address {:#04X?}", addr),
        }
    }
}

impl Stable for RealTimeClock {
    fn save(&self) {
        self.save_to_file(self.save_path.clone(), &self.zero.to_be_bytes());
    }
}

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rtc: RealTimeClock,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    save_path: PathBuf,
}

impl Mbc3 {
    pub fn new(
        rom: Vec<u8>,
        ram: Vec<u8>,
        save_path: impl AsRef<Path>,
        rtc_save_path: impl AsRef<Path>,
    ) -> Mbc3 {
        Mbc3 {
            rom,
            ram,
            rtc: RealTimeClock::new(rtc_save_path),
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            save_path: PathBuf::from(save_path.as_ref()),
        }
    }
}

impl Memory for Mbc3 {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 00 (Read Only)
            0x0000..=0x3FFF => self.rom[addr as usize],
            // ROM Bank 01-7F (Read Only)
            0x4000..=0x7FFF => {
                let index = self.rom_bank * 0x4000 + addr as usize - 0x4000;
                self.rom[index]
            }
            // RAM Bank 00-03, if any (Read/Write)
            // RTC Register 08-0C (Read/Write)
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 0x03 {
                        let index = self.ram_bank * 0x2000 + addr as usize - 0xa000;
                        self.ram[index]
                    } else {
                        self.rtc.get_byte(self.ram_bank as u16)
                    }
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM Bank 00-03, if any (Read/Write)
            // RTC Register 08-0C (Read/Write)
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 0x03 {
                        let index = self.ram_bank * 0x2000 + addr as usize - 0xA000;
                        self.ram[index] = value;
                    } else {
                        self.rtc.set_byte(self.ram_bank as u16, value)
                    }
                }
            }
            // RAM and Timer Enable (Write Only)
            0x0000..=0x1FFF => {
                // a value of 0Ah will enable reading and writing to external RAM -
                // and to the RTC Registers! A value of 00h will disable either.
                self.ram_enable = value & 0x0F == 0x0A;
            }
            // ROM Bank Number (Write Only)
            0x2000..=0x3FFF => {
                // Writing a value of 00h, will select Bank 01h instead. All other values 01-7Fh
                // select the corresponding ROM Banks.
                let n = (value & 0x7F) as usize;
                let n = match n {
                    0x00 => 0x01,
                    _ => n,
                };
                self.rom_bank = n;
            }
            // RAM Bank Number - or - RTC Register Select (Write Only)
            0x4000..=0x5FFF => {
                let n = (value & 0x0F) as usize;
                self.ram_bank = n;
            }
            // Latch Clock Data (Write Only)
            0x6000..=0x7FFF => {
                if value & 0x01 != 0 {
                    self.rtc.tic();
                }
            }
            _ => {}
        }
    }
}

impl Stable for Mbc3 {
    fn save(&self) {
        self.rtc.save();
        self.save_to_file(self.save_path.clone(), &self.ram);
    }
}

impl Cartridge for Mbc3 {}
