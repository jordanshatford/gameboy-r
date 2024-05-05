mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod rom;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::cartridges::mbc1::Mbc1;
use crate::cartridges::mbc2::Mbc2;
use crate::cartridges::mbc3::Mbc3;
use crate::cartridges::mbc5::Mbc5;
use crate::cartridges::rom::RomOnly;
use crate::memory::Memory;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CartridgeMode {
    Gb,  // Original Gameboy
    Gbc, // Gameboy Color
}

pub trait Stable {
    fn save(&self) {}

    fn save_to_file(&self, path: PathBuf, contents: &[u8]) {
        if path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(path)
            .and_then(|mut f| f.write_all(contents))
            .unwrap()
    }
}

// These bytes define the bitmap of the Nintendo logo that is displayed when the gameboy gets turned on.
// The reason for joining is because if the pirates copy the cartridge, they must also copy Nintendo's LOGO,
// which infringes the trademark law. In the early days, the copyright law is not perfect for the
// determination of electronic data.
// The hexdump of this bitmap is:
const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub trait Cartridge: Memory + Stable + Send {
    // 0134-0143 - Title
    // Title of the game in UPPER CASE ASCII. If it is less than 16 characters then the
    // remaining bytes are filled with 00's. When inventing the CGB, Nintendo has reduced
    // the length of this area to 15 characters, and some months later they had the
    // fantastic idea to reduce it to 11 characters only. The new meaning of the ex-title
    // bytes is described below.

    // 013F-0142 - Manufacturer Code
    // In older cartridges this area has been part of the Title (see above), in newer
    // cartridges this area contains an 4 character uppercase manufacturer code. Purpose and
    // Deeper Meaning unknown.

    // 0143 - CGB Flag
    // In older cartridges this byte has been part of the Title (see above). In CGB cartridges
    // the upper bit is used to enable CGB functions. This is required, otherwise the CGB
    // switches itself into Non-CGB-Mode. Typical values are:
    //  80h - Game supports CGB functions, but works on old gameboys also.
    //  C0h - Game works on CGB only (physically the same as 80h).
    fn get_title(&self) -> String {
        let mut buffer = String::new();
        let has_short_title = self.get_byte(0x0143) == 0x80;
        let end_addr: u16 = if has_short_title { 0x013E } else { 0x0143 };
        for addr in 0x0134..=end_addr {
            match self.get_byte(addr) {
                // If it is less than 16 characters then the remaining bytes are filled with 00's.
                0 => break,
                byte => buffer.push(byte as char),
            }
        }
        buffer
    }

    // 0104-0133 - Nintendo Logo
    // The Gameboy's boot procedure verifies the content of this bitmap (after it has displayed it), and
    // LOCKS ITSELF UP if these bytes are incorrect. A CGB verifies only the first 18h bytes of the bitmap,
    // but others (for example a pocket gameboy) verify all 30h bytes.
    fn verify_nintendo_logo(&self) {
        for addr in 0x00..48 {
            if self.get_byte(0x0104 + addr as u16) != NINTENDO_LOGO[addr as usize] {
                panic!("cartridge: could not validate nintendo logo")
            }
        }
    }

    // 014D - Header Checksum
    // Contains an 8 bit checksum across the cartridge header bytes 0134-014C. The checksum is calculated as follows:
    //  x=0:FOR i=0134h TO 014Ch:x=x-MEM[i]-1:NEXT
    // The lower 8 bits of the result must be the same than the value in this entry. The GAME WON'T WORK if
    // this checksum is incorrect.
    fn verify_header_checksum(&self) {
        let mut checksum: u8 = 0;
        for addr in 0x0134..0x014D {
            checksum = checksum.wrapping_sub(self.get_byte(addr)).wrapping_sub(1);
        }
        if self.get_byte(0x014D) != checksum {
            panic!("cartridge: could not validate header checksum")
        }
    }

    // 0143 - CGB Flag
    // In older cartridges this byte has been part of the Title (see above). In CGB cartridges
    // the upper bit is used to enable CGB functions. This is required, otherwise the CGB
    // switches itself into Non-CGB-Mode. Typical values are:
    //  80h - Game supports CGB functions, but works on old gameboys also.
    //  C0h - Game works on CGB only (physically the same as 80h).
    // Values with Bit 7 set, and either Bit 2 or 3 set, will switch the gameboy into a special
    // non-CGB-mode with uninitialized palettes. Purpose unknown, eventually this has been
    // supposed to be used to colorize monochrome games that include fixed palette data at a special
    // location in ROM.
    fn get_mode(&self) -> CartridgeMode {
        match self.get_byte(0x0143) & 0x80 {
            0x80 => CartridgeMode::Gbc,
            _ => CartridgeMode::Gb,
        }
    }
}

// Specifies which Memory Bank Controller (if any) is used in the cartridge, and
// if further external hardware exists in the cartridge.
//  00h  ROM ONLY                 19h  MBC5
//  01h  MBC1                     1Ah  MBC5+RAM
//  02h  MBC1+RAM                 1Bh  MBC5+RAM+BATTERY
//  03h  MBC1+RAM+BATTERY         1Ch  MBC5+RUMBLE
//  05h  MBC2                     1Dh  MBC5+RUMBLE+RAM
//  06h  MBC2+BATTERY             1Eh  MBC5+RUMBLE+RAM+BATTERY
//  08h  ROM+RAM                  20h  MBC6
//  09h  ROM+RAM+BATTERY          22h  MBC7+SENSOR+RUMBLE+RAM+BATTERY
//  0Bh  MMM01
//  0Ch  MMM01+RAM
//  0Dh  MMM01+RAM+BATTERY
//  0Fh  MBC3+TIMER+BATTERY
//  10h  MBC3+TIMER+RAM+BATTERY   FCh  POCKET CAMERA
//  11h  MBC3                     FDh  BANDAI TAMA5
//  12h  MBC3+RAM                 FEh  HuC3
//  13h  MBC3+RAM+BATTERY         FFh  HuC1+RAM+BATTERY
pub fn new(rom: Vec<u8>, path: impl AsRef<Path>, skip_checks: bool) -> Box<dyn Cartridge> {
    // An internal information area is located at 0100-014F in each cartridge.
    if rom.len() < 0x150 {
        panic!("cartridge: invalid rom size")
    }
    let rom_max_size = get_rom_size(rom.as_ref());
    if rom.len() > rom_max_size {
        panic!("cartridge: rom size more than max (max: {})", rom_max_size);
    }
    // In each cartridge, the required (or preferred) MBC type should
    // be specified in the byte at 0147h of the ROM, as described in
    // the cartridge header.
    let cartridge: Box<dyn Cartridge> = match rom[0x0147] {
        0x00 => Box::new(RomOnly::new(rom)),
        0x01 => Box::new(Mbc1::new(rom, vec![], "")),
        0x02 => {
            let ram_size = get_ram_size(rom.as_ref());
            Box::new(Mbc1::new(rom, vec![0; ram_size], ""))
        }
        0x03 => {
            let (save_path, _) = get_save_paths(path);
            let ram_size = get_ram_size(rom.as_ref());
            let ram = read_ram_from_save(save_path.clone(), ram_size);
            Box::new(Mbc1::new(rom, ram, save_path))
        }
        0x05 => {
            let ram_size = 512;
            Box::new(Mbc2::new(rom, vec![0; ram_size], ""))
        }
        0x06 => {
            let (save_path, _) = get_save_paths(path);
            let ram_size = 512;
            let ram = read_ram_from_save(save_path.clone(), ram_size);
            Box::new(Mbc2::new(rom, ram, save_path))
        }
        0x0F => {
            let (save_path, rtc_save_path) = get_save_paths(path);
            Box::new(Mbc3::new(rom, vec![], save_path, rtc_save_path))
        }
        0x10 => {
            let (save_path, rtc_save_path) = get_save_paths(path);
            let ram_size = get_ram_size(rom.as_ref());
            let ram = read_ram_from_save(save_path.clone(), ram_size);
            Box::new(Mbc3::new(rom, ram, save_path, rtc_save_path))
        }
        0x11 => Box::new(Mbc3::new(rom, vec![], "", "")),
        0x12 => {
            let ram_size = get_ram_size(rom.as_ref());
            Box::new(Mbc3::new(rom, vec![0; ram_size], "", ""))
        }
        0x13 => {
            let (save_path, _) = get_save_paths(path);
            let ram_size = get_ram_size(rom.as_ref());
            let ram = read_ram_from_save(save_path.clone(), ram_size);
            Box::new(Mbc3::new(rom, ram, save_path, ""))
        }
        0x19 => Box::new(Mbc5::new(rom, vec![], "")),
        0x1A => {
            let ram_size = get_ram_size(rom.as_ref());
            Box::new(Mbc5::new(rom, vec![0; ram_size], ""))
        }
        0x1B => {
            let (save_path, _) = get_save_paths(path);
            let ram_size = get_ram_size(rom.as_ref());
            let ram = read_ram_from_save(save_path.clone(), ram_size);
            Box::new(Mbc5::new(rom, ram, save_path))
        }
        byte => panic!("cartridge: unsupported type {:#04X?}", byte),
    };
    if !skip_checks {
        cartridge.verify_nintendo_logo();
        cartridge.verify_header_checksum();
    }
    cartridge
}

// 0148 - ROM Size
// Specifies the ROM Size of the cartridge. Typically calculated as "32KB shl N".
//  00h -  32KByte (no ROM banking)
//  01h -  64KByte (4 banks)
//  02h - 128KByte (8 banks)
//  03h - 256KByte (16 banks)
//  04h - 512KByte (32 banks)
//  05h -   1MByte (64 banks)  - only 63 banks used by MBC1
//  06h -   2MByte (128 banks) - only 125 banks used by MBC1
//  07h -   4MByte (256 banks)
//  08h -   8MByte (512 banks)
//  52h - 1.1MByte (72 banks)
//  53h - 1.2MByte (80 banks)
//  54h - 1.5MByte (96 banks)
pub fn get_rom_size(rom: &[u8]) -> usize {
    let kb_in_bytes = 16384;
    let rom_size_addr = 0x148;
    match rom[rom_size_addr] {
        0x00 => kb_in_bytes * 2,
        0x01 => kb_in_bytes * 4,
        0x02 => kb_in_bytes * 8,
        0x03 => kb_in_bytes * 16,
        0x04 => kb_in_bytes * 32,
        0x05 => kb_in_bytes * 64,
        0x06 => kb_in_bytes * 128,
        0x07 => kb_in_bytes * 256,
        0x08 => kb_in_bytes * 512,
        0x52 => kb_in_bytes * 72,
        0x53 => kb_in_bytes * 80,
        0x54 => kb_in_bytes * 96,
        byte => panic!("cartridge: unsupported rom size {:#04X?}", byte),
    }
}

// 0149 - RAM Size
// Specifies the size of the external RAM in the cartridge (if any).
//  00h - None
//  01h - 2 KBytes
//  02h - 8 Kbytes
//  03h - 32 KBytes (4 banks of 8KBytes each)
//  04h - 128 KBytes (16 banks of 8KBytes each)
//  05h - 64 KBytes (8 banks of 8KBytes each)
// When using a MBC2 chip 00h must be specified in this entry, even though the
// MBC2 includes a built-in RAM of 512 x 4 bits.
pub fn get_ram_size(rom: &[u8]) -> usize {
    let ram_size_addr = 0x149;
    match rom[ram_size_addr] {
        0x00 => 0,
        0x01 => 1024 * 2,
        0x02 => 1024 * 8,
        0x03 => 1024 * 32,
        0x04 => 1024 * 128,
        0x05 => 1024 * 64,
        byte => panic!("cartridge: unsupported ram size {:#04X?}", byte),
    }
}

// Read RAM data from external sav file when available
pub fn read_ram_from_save(path: impl AsRef<Path>, size: usize) -> Vec<u8> {
    match File::open(path) {
        Ok(mut f) => {
            let mut ram = Vec::new();
            f.read_to_end(&mut ram).unwrap();
            ram
        }
        Err(_) => vec![0; size],
    }
}

// Get path for sav and rtc save files
fn get_save_paths(rom_path: impl AsRef<Path>) -> (PathBuf, PathBuf) {
    if rom_path.as_ref().to_str().unwrap().is_empty() {
        return (PathBuf::new(), PathBuf::new());
    }
    let sav_path = rom_path.as_ref().to_path_buf().with_extension("sav");
    let rtc_path = rom_path.as_ref().to_path_buf().with_extension("rtc");
    (sav_path, rtc_path)
}
