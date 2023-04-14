<p align="center">
  <br />
  <img width="150" height="150" src="./gameboy.png" alt="Logo">
  <h1 align="center"><b>GameboyR</b></h1>
  <div align="center">
    <a href="https://www.rust-lang.org/">
      <img src="https://img.shields.io/badge/language-Rust-%23000000.svg?style=flat&logo=rust" alt="Language: Rust">
    </a>
    <a href="https://crates.io/crates/gameboyr">
      <img src="https://img.shields.io/crates/v/gameboyr" alt="Crates.io version">
    </a>
    <a href="https://crates.io/crates/gameboyr">
      <img src="https://img.shields.io/crates/d/gameboyr" alt="Crates.io downloads">
    </a>
    <a href="https://github.com/jordanshatford/gameboy-r/blob/main/LICENSE">
      <img src="https://img.shields.io/crates/l/gameboyr" alt="Crates.io license MIT">
    </a>
  </div>
  <p align="center">
    A Gameboy emulator written in Rust.
    <br />
    <a href="https://crates.io/crates/gameboyr"><strong>crates.io/crates/gameboyr »</strong></a>
    <br />
    <br />
  </p>
</p>

A Gameboy emulator written in Rust. This emulator supports both Gameboy and Gameboy Color. A checklist of implemented features can be found below.

## How to Use:
You can run the emulator using the following command and specifying the path to your rom:
```
$ cargo run --release -- "./path/to/rom.gb"
```
The following options can be specified
```
-s, --save          Path to .sav file of the specified ROM (Default: location of ROM)
-x, --scale         Scale the window by a factor of 1, 2, 4 (Default: 1)
    --skip-checks   Skip header checksum and nintendo logo checks for ROM
```

### With Crates.io
You can install the emulator from [Crates.io](https://crates.io/crates/gameboyr) using:
```
$ cargo install gameboyr
```
Then run the emulator using:
```
$ gameboyr "./path/to/rom.gb"
```
The options specified above are also available here.

### Controls:
```
                    __________________________
                  |                          |
                  | .----------------------. |
                  | |  .----------------.  | |
                  | |  |                |  | |
                  | |  |                |  | |
                  | |  |                |  | |
                  | |  |                |  | |
                  | |  |                |  | |
                  | |  |                |  | |
                  | |  |                |  | |
                  | |  '----------------'  | |
                  | |______________________/ |
                  |                          |
                  |    .    GAMEBOY R        |
     Up           |  _| |_              .-.  | ---> Z
Left + Right <--- |-[_   _]-       .-. (   ) |
    Down          |   |_|         (   ) '-'  | ---> X
                  |    '           '-'   A   |
                  |                 B        |
                  |          ___   ___       |
                  |         (___) (___)  ,., | ---> Space / Enter
                  |        select start ;:;: |
                  |                    ,;:;' /
                  |                   ,:;:'.'
                  '-----------------------`
```

## Implemented:
- [x] CPU
  - [x] Registers & Flags
  - [x] Instruction Set
  - [x] Timing
- [x] MMU
  - [x] Work RAM (WRAM)
  - [x] High Ram (HRAM)
  - [x] HDMA & GDMA
  - [x] Speed switch
- [x] I/O
  - [x] Video Display (PPU)
  - [ ] Sound Controller (APU)
  - [x] Joypad Input
  - [x] Serial Data Transfer
  - [x] Timer
- [x] Cartridges
  - [x] None (32KByte ROM only)
  - [x] MBC1 (max 2MByte ROM and/or 32KByte RAM)
  - [x] MBC2 (max 256KByte ROM and 512x4 bits RAM)
  - [x] MBC3 (max 2MByte ROM and/or 64KByte RAM and Timer)
  - [x] MBC5 (max 8MByte ROM and/or 128KByte RAM)

## References:
- [Pandocs](https://bgb.bircd.org/pandocs.htm)
- [GBDev Pandocs](https://gbdev.io/pandocs/)
- [GBDev Wiki](https://gbdev.gg8.se/wiki/articles/Video_Display)
- [GBOps](https://izik1.github.io/gbops/)
- [Pastraiser (OP Codes)](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [Cartridge Header](https://gbdev.gg8.se/wiki/articles/The_Cartridge_Header)
- [Cartridge Types](https://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers)
- [GB ROMS](https://www.romsgames.net/roms/gameboy/)
- [GBC ROMS](https://www.romsgames.net/roms/gameboy-color/)
