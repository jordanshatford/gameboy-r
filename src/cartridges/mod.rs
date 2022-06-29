mod none;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::cartridges::none::RomOnly;
use crate::memory::Memory;

pub trait Stable {
    fn sav(&self);
}

pub trait Cartridge: Memory + Stable + Send {}

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
pub fn new(path: impl AsRef<Path>) -> Box<dyn Cartridge> {
    let mut file = File::open(path.as_ref()).unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();

    // In each cartridge, the required (or preferred) MBC type should
    // be specified in the byte at 0147h of the ROM, as described in
    // the cartridge header.
    let cartridge: Box<dyn Cartridge> = match rom[0x0147] {
        0x00 => Box::new(RomOnly::new(rom)),
        byte => panic!("cartridge: unsupported type {:#04X?}", byte),
    };
    cartridge
}
