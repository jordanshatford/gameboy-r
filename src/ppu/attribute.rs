// VRAM Sprite Attribute Table (OAM)
// Byte3 - Attributes/Flags:
//   Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
//          (Used for both BG and Window. BG color 0 is always behind OBJ)
//   Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
//   Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
//   Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
//   Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
//   Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
pub struct Attribute {
    pub priority: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub palette_number: usize,
    pub vram_bank: bool,
    pub cgb_palette_number: usize,
}

impl From<u8> for Attribute {
    fn from(value: u8) -> Attribute {
        Attribute {
            priority: value & (1 << 7) != 0,
            y_flip: value & (1 << 6) != 0,
            x_flip: value & (1 << 5) != 0,
            palette_number: value as usize & (1 << 4),
            vram_bank: value & (1 << 3) != 0,
            cgb_palette_number: value as usize & 0x07,
        }
    }
}
