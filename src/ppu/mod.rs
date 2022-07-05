mod attribute;
pub mod hdma;
mod lcd;

use crate::cartridges::CartridgeMode;
use crate::memory::Memory;
use crate::mmu::InterruptFlag;
use crate::ppu::attribute::Attribute;
use crate::ppu::lcd::{LCDControl, LCDStatus, BGPI};

// Resolution - 160x144 (20x18 tiles)
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

#[derive(Debug, Copy, Clone)]
pub struct PPU {
    // Digital image with mode RGB. Size = 144 * 160 * 3 (RGB).
    data: [[[u8; 3]; SCREEN_WIDTH]; SCREEN_HEIGHT],
    mode: CartridgeMode,
    pub interrupt: u8,
    vblank: bool,
    hblank: bool,
    lcd_control: LCDControl,
    lcd_status: LCDStatus,
    // FF42 - SCY - Scroll Y (R/W)
    // FF43 - SCX - Scroll X (R/W)
    // Specifies the position in the 256x256 pixels BG map (32x32 tiles) which is to be displayed at the upper/left
    // LCD display position. Values in range from 0-255 may be used for X/Y each, the video controller automatically
    // wraps back to the upper (left) position in BG map when drawing exceeds the lower (right) border of
    // the BG map area.
    scroll_y: u8,
    scroll_x: u8,
    // FF44 - LY - LCDC Y-Coordinate (R)
    // The LY indicates the vertical line to which the present data is transferred to the LCD Driver. The LY can take
    // on any value between 0 through 153. The values between 144 and 153 indicate the V-Blank period. Writing will
    // reset the counter.
    lcdc_y: u8,
    // FF45 - LYC - LY Compare (R/W)
    // The gameboy permanently compares the value of the LYC and LY registers. When both values are identical, the
    // coincident bit in the STAT register becomes set, and (if enabled) a STAT interrupt is requested.
    ly_compare: u8,
    // FF4A - WY - Window Y Position (R/W)
    // FF4B - WX - Window X Position minus 7 (R/W)
    // Specifies the upper/left positions of the Window area. (The window is an alternate background area which can
    // be displayed above of the normal background. OBJs (sprites) may be still displayed above or behinf the window,
    // just as for normal BG.)
    // The window becomes visible (if enabled) when positions are set in range WX=0..166, WY=0..143. A postion of
    // WX=7, WY=0 locates the window at upper left, it is then completly covering normal background.
    window_y: u8,
    window_x: u8,
    //  FF47 - BGP - BG Palette Data (R/W) - Non CGB Mode Only
    // This register assigns gray shades to the color numbers of the BG and Window tiles.
    //   Bit 7-6 - Shade for Color Number 3
    //   Bit 5-4 - Shade for Color Number 2
    //   Bit 3-2 - Shade for Color Number 1
    //   Bit 1-0 - Shade for Color Number 0
    // The four possible gray shades are:
    //   0  White
    //   1  Light gray
    //   2  Dark gray
    //   3  Black
    // In CGB Mode the Color Palettes are taken from CGB Palette Memory instead.
    bg_palette: u8,
    // FF48 - OBP0 - Object Palette 0 Data (R/W) - Non CGB Mode Only
    // This register assigns gray shades for sprite palette 0. It works exactly as BGP (FF47), except that the
    // lower two bits aren't used because sprite data 00 is transparent.
    object_pallete_0: u8,
    // FF49 - OBP1 - Object Palette 1 Data (R/W) - Non CGB Mode Only
    // This register assigns gray shades for sprite palette 1. It works exactly as BGP (FF47), except that the
    //lower two bits aren't used because sprite data 00 is transparent.
    object_pallete_1: u8,
    // FF68 - BCPS/BGPI - CGB Mode Only - Background Palette Index
    // This register is used to address a byte in the CGBs Background Palette Memory. Each two byte in that memory
    // define a color value. The first 8 bytes define Color 0-3 of Palette 0 (BGP0), and so on for BGP1-7.
    //   Bit 0-5   Index (00-3F)
    //   Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
    // Data can be read/written to/from the specified index address through Register FF69. When the Auto Increment
    // Bit is set then the index is automatically incremented after each <write> to FF69. Auto Increment has no
    // effect when <reading> from FF69, so the index must be manually incremented in that case.
    bgpi: BGPI,
    // FF69 - BCPD/BGPD - CGB Mode Only - Background Palette Data
    // This register allows to read/write data to the CGBs Background Palette Memory, addressed through Register FF68.
    // Each color is defined by two bytes (Bit 0-7 in first byte).
    //   Bit 0-4   Red Intensity   (00-1F)
    //   Bit 5-9   Green Intensity (00-1F)
    //   Bit 10-14 Blue Intensity  (00-1F)
    // Much like VRAM, Data in Palette Memory cannot be read/written during the time when the LCD Controller is reading
    // from it. (That is when the STAT register indicates Mode 3). Note: Initially all background colors are initialized as white.
    bgp_data: [[[u8; 3]; 4]; 8],
    // FF6A - OCPS/OBPI - CGB Mode Only - Sprite Palette Index
    // FF6B - OCPD/OBPD - CGB Mode Only - Sprite Palette Data
    // These registers are used to initialize the Sprite Palettes OBP0-7, identically as described above for Background Palettes.
    // Note that four colors may be defined for each OBP Palettes - but only Color 1-3 of each Sprite Palette can be displayed,
    // Color 0 is always transparent, and can be initialized to a don't care value.
    // Note: Initially all sprite colors are uninitialized.
    obpi: BGPI,
    obp_data: [[[u8; 3]; 4]; 8],
    vram: [u8; 0x4000],
    // LCD VRAM Bank (CGB only)
    // FF4F - VBK - CGB Mode Only - VRAM Bank
    // This 1bit register selects the current Video Memory (VRAM) Bank.
    //   Bit 0 - VRAM Bank (0-1)
    // Bank 0 contains 192 Tiles, and two background maps, just as for monochrome games. Bank 1 contains another 192 Tiles,
    // and color attribute maps for the background maps in bank 0.
    vram_bank: usize,
    // VRAM Sprite Attribute Table (OAM)
    // GameBoy video controller can display up to 40 sprites either in 8x8 or in 8x16 pixels. Because of a limitation of hardware,
    // only ten sprites can be displayed per scan line. Sprite patterns have the same format as BG tiles, but they are taken from
    // the Sprite Pattern Table located at $8000-8FFF and have unsigned numbering.
    // Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F. Each of the 40 entries
    // consists of four bytes with the following meanings:
    //  Byte0 - Y Position
    //      Specifies the sprites vertical position on the screen (minus 16).
    //      An offscreen value (for example, Y=0 or Y>=160) hides the sprite.
    //  Byte1 - X Position
    //      Specifies the sprites horizontal position on the screen (minus 8).
    //      An offscreen value (X=0 or X>=168) hides the sprite, but the sprite
    //      still affects the priority ordering - a better way to hide a sprite is to set its Y-coordinate offscreen.
    //  Byte2 - Tile/Pattern Number
    //      Specifies the sprites Tile Number (00-FF). This (unsigned) value selects a tile from memory at 8000h-8FFFh. In CGB
    //      Mode this could be either in VRAM Bank 0 or 1, depending on Bit 3 of the following byte.  In 8x16 mode, the lower
    //      bit of the tile number is ignored. Ie. the upper 8x8 tile is "NN AND FEh", and the lower 8x8 tile is "NN OR 01h".
    //  Byte3 - Attributes/Flags:
    //   Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    //          (Used for both BG and Window. BG color 0 is always behind OBJ)
    //   Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
    //   Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
    //   Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    //   Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
    //   Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
    // Sprite Priorities and Conflicts
    // When sprites with different x coordinate values overlap, the one with the smaller x coordinate (closer to the left) will
    // have priority and appear above any others. This applies in Non CGB Mode only. When sprites with the same x coordinate values
    // overlap, they have priority according to table ordering. (i.e. $FE00 - highest, $FE04 - next highest, etc.) In CGB Mode
    // priorities are always assigned like this. Only 10 sprites can be displayed on any one line. When this limit is exceeded,
    // the lower priority sprites (priorities listed above) won't be displayed. To keep unused sprites from affecting onscreen
    // sprites set their Y coordinate to Y=0 or Y=>144+16. Just setting the X coordinate to X=0 or X=>160+8 on a sprite will hide
    // it but it will still affect other sprites sharing the same lines.
    // Writing Data to OAM Memory
    // The recommened method is to write the data to normal RAM first, and to copy that RAM to OAM by using the DMA transfer function,
    // initiated through DMA register (FF46). Beside for that, it is also possible to write data directly to the OAM area by using normal
    // LD commands, this works only during the H-Blank and V-Blank periods. The current state of the LCD controller can be read out
    // from the STAT register (FF41).
    oam: [u8; 0xA0],
    priorities: [(bool, usize); SCREEN_WIDTH],
    // The LCD controller operates on a 222 Hz = 4.194 MHz dot clock. An entire frame is 154 scanlines, 70224 dots, or 16.74 ms.
    // On scanlines 0 through 143, the LCD controller cycles through modes 2, 3, and 0 once every 456 dots. Scanlines 144 through
    // 153 are mode 1.
    dots: u32,
}

impl PPU {
    pub fn new(mode: CartridgeMode) -> PPU {
        PPU {
            data: [[[0xFFu8; 3]; SCREEN_WIDTH]; SCREEN_HEIGHT],
            mode,
            interrupt: InterruptFlag::None as u8,
            vblank: false,
            hblank: false,
            lcd_control: LCDControl::new(),
            lcd_status: LCDStatus::new(),
            scroll_x: 0x00,
            scroll_y: 0x00,
            lcdc_y: 0x00,
            ly_compare: 0x00,
            window_y: 0x00,
            window_x: 0x00,
            bg_palette: 0x00,
            object_pallete_0: 0x00,
            object_pallete_1: 0x01,
            bgpi: BGPI::new(),
            bgp_data: [[[0u8; 3]; 4]; 8],
            obpi: BGPI::new(),
            obp_data: [[[0u8; 3]; 4]; 8],
            vram: [0x00; 0x4000],
            vram_bank: 0x00,
            oam: [0x00; 0xA0],
            priorities: [(true, 0); SCREEN_WIDTH],
            dots: 0,
        }
    }

    pub fn run_cycles(&mut self, cycles: u32) {}
}

impl Memory for PPU {
    fn get_byte(&self, addr: u16) -> u8 {
        panic!("ppu: get_byte not implemented")
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        panic!("ppu: set_byte not implemented")
    }
}
