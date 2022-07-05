# gameboy-r
A Game Boy emulator written in Rust.

## Implemented
- [x] CPU
  - [x] Registers & Flags
  - [x] Instruction Set
  - [x] Timing
- [ ] MMU
  - [x] Work RAM (WRAM)
  - [x] High Ram (HRAM)
  - [x] HDMA & GDMA
  - [x] Speed switch
- [ ] I/O
  - [ ] Video Display (PPU)
  - [ ] Sound Controller (APU)
  - [x] Joypad Input
  - [x] Serial Data Transfer
  - [x] Timer
- [ ] Cartridges
  - [x] None (32KByte ROM only)
  - [x] MBC1 (max 2MByte ROM and/or 32KByte RAM)
  - [ ] MBC2 (max 256KByte ROM and 512x4 bits RAM)
  - [ ] MBC3 (max 2MByte ROM and/or 64KByte RAM and Timer)
  - [ ] MBC5 (max 8MByte ROM and/or 128KByte RAM)
  - [ ] HuC1 (MBC with Infrared Controller)