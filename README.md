# gameboy-r
A Game Boy emulator written in Rust.

## Implemented
- [~] CPU
  - [x] Registers & Flags
  - [_] Instruction Set
- [~] MMU
  - [_] Work RAM (WRAM)
  - [_] High Ram (HRAM)
- [~] I/O
  - [_] Video Display (PPU)
  - [_] Sound Controller (APU)
  - [_] Joypad Input
  - [_] Serial Data Transfer
  - [_] Timer
- [~] cartridges
  - [x] None (32KByte ROM only)
  - [x] MBC1 (max 2MByte ROM and/or 32KByte RAM)
  - [_] MBC2 (max 256KByte ROM and 512x4 bits RAM)
  - [_] MBC3 (max 2MByte ROM and/or 64KByte RAM and Timer)
  - [_] MBC5 (max 8MByte ROM and/or 128KByte RAM)
  - [_] HuC1 (MBC with Infrared Controller)