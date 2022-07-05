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

    pub fn run_cycles(&mut self, cycles: u32) {
        if !self.lcd_control.has_bit7() {
            return;
        }
        self.hblank = false;
        if cycles == 0 {
            return;
        }
        let c = (cycles - 1) / 80 + 1;
        for i in 0..c {
            if i == (c - 1) {
                self.dots += cycles % 80
            } else {
                self.dots += 80
            }
            let d = self.dots;
            self.dots %= 456;
            if d != self.dots {
                self.lcdc_y = (self.lcdc_y + 1) % 154;
                if self.lcd_status.lyc_interrupt_enabled && self.lcdc_y == self.ly_compare {
                    self.interrupt |= InterruptFlag::LCDStat as u8;
                }
            }
            if self.lcdc_y >= 144 {
                if self.lcd_status.mode == 1 {
                    continue;
                }
                self.lcd_status.mode = 1;
                self.vblank = true;
                self.interrupt |= InterruptFlag::VBlank as u8;
                if self.lcd_status.m1_vblank_interrupt_enabled {
                    self.interrupt |= InterruptFlag::LCDStat as u8;
                }
            } else if self.dots <= 80 {
                if self.lcd_status.mode == 2 {
                    continue;
                }
                self.lcd_status.mode = 2;
                if self.lcd_status.m2_oam_interrupt_enabled {
                    self.interrupt |= InterruptFlag::LCDStat as u8;
                }
            } else if self.dots <= (80 + 172) {
                self.lcd_status.mode = 3;
            } else {
                if self.lcd_status.mode == 0 {
                    continue;
                }
                self.lcd_status.mode = 0;
                self.hblank = true;
                if self.lcd_status.m0_hblank_interrupt_enabled {
                    self.interrupt |= InterruptFlag::LCDStat as u8;
                }
                // Render scanline
                if self.mode == CartridgeMode::GBC || self.lcd_control.has_bit0() {
                    self.draw_background();
                }
                if self.lcd_control.has_bit1() {
                    self.draw_sprites();
                }
            }
        }
    }

    // num can be 0 or 1 for each specific vram
    fn get_vram(&self, num: u8, addr: u16) -> u8 {
        match num {
            0 => self.vram[addr as usize - 0x8000],
            1 => self.vram[addr as usize - 0x6000],
            _ => panic!("ppu: invalid vram number"),
        }
    }

    // This register assigns gray shades to the color numbers of the BG and Window tiles.
    //  Bit 7-6 - Shade for Color Number 3
    //  Bit 5-4 - Shade for Color Number 2
    //  Bit 3-2 - Shade for Color Number 1
    //  Bit 1-0 - Shade for Color Number 0
    // The four possible gray shades are:
    //  0  White
    //  1  Light gray
    //  2  Dark gray
    //  3  Black
    fn get_gray_shade(&self, value: u8, i: usize) -> u8 {
        match value >> (2 * i) & 0x03 {
            0x00 => 0xFF,
            0x01 => 0xC0,
            0x02 => 0x60,
            _ => 0x00,
        }
    }

    // When developing graphics on PCs, note that the RGB values will have different appearance on CGB displays as on
    // VGA/HDMI monitors calibrated to sRGB color. Because the GBC is not lit, the highest intensity will produce
    // Light Gray color rather than White. The intensities are not linear; the values 10h-1Fh will all appear very
    // bright, while medium and darker colors are ranged at 00h-0Fh.
    // The CGB display's pigments aren't perfectly saturated. This means the colors mix quite oddly; increasing
    // intensity of only one R,G,B color will also influence the other two R,G,B colors. For example, a color setting
    // of 03EFh (Blue=0, Green=1Fh, Red=0Fh) will appear as Neon Green on VGA displays, but on the CGB it'll produce
    // a decently washed out Yellow
    fn set_rgb(&mut self, index: usize, r: u8, g: u8, b: u8) {
        assert!(r <= 0x1F);
        assert!(g <= 0x1F);
        assert!(b <= 0x1F);
        let r = u32::from(r);
        let g = u32::from(g);
        let b = u32::from(b);
        let lr = ((r * 13 + g * 2 + b) >> 1) as u8;
        let lg = ((g * 3 + b) << 1) as u8;
        let lb = ((r * 3 + g * 2 + b * 11) >> 1) as u8;
        self.data[self.ly_compare as usize][index] = [lr, lg, lb];
    }

    fn set_greyscale(&mut self, index: usize, g: u8) {
        self.data[self.ly_compare as usize][index] = [g, g, g];
    }

    fn draw_background(&mut self) {
        let show_window = self.lcd_control.has_bit5() && self.window_y <= self.lcdc_y;
        let tile_base = if self.lcd_control.has_bit4() {
            0x8000
        } else {
            0x8800
        };
        let window_x = self.window_x.wrapping_sub(7);
        let picture_y = if show_window {
            self.lcdc_y.wrapping_sub(self.window_y)
        } else {
            self.scroll_y.wrapping_add(self.lcdc_y)
        };
        let tile_y = (u16::from(picture_y) >> 3) & 31;

        for x in 0..SCREEN_WIDTH {
            let picture_x = if show_window && x as u8 >= window_x {
                x as u8 - window_x
            } else {
                self.scroll_x.wrapping_add(x as u8)
            };
            let tile_x = (u16::from(picture_x) >> 3) & 31;
            let background_base_addr = if show_window && x as u8 >= window_x {
                if self.lcd_control.has_bit6() {
                    0x9C00
                } else {
                    0x9800
                }
            } else if self.lcd_control.has_bit3() {
                0x9C00
            } else {
                0x9800
            };
            let tile_addr = background_base_addr + tile_y * 32 + tile_x;
            let tile_number = self.get_vram(0, tile_addr);
            let tile_offset = if self.lcd_control.has_bit4() {
                i16::from(tile_number)
            } else {
                i16::from(tile_number as i8) + 128
            } as u16
                * 16;
            let tile_location = tile_base + tile_offset;
            let tile_attribute = Attribute::from(self.get_vram(1, tile_addr));
            let tile_y = if tile_attribute.y_flip {
                7 - picture_y % 8
            } else {
                picture_y % 8
            };
            let tile_y_data: [u8; 2] =
                if self.mode == CartridgeMode::GBC && tile_attribute.vram_bank {
                    let a = self.get_vram(1, tile_location + u16::from(tile_y * 2));
                    let b = self.get_vram(1, tile_location + u16::from(tile_y * 2) + 1);
                    [a, b]
                } else {
                    let a = self.get_vram(0, tile_location + u16::from(tile_y * 2));
                    let b = self.get_vram(0, tile_location + u16::from(tile_y * 2) + 1);
                    [a, b]
                };
            let tile_x = if tile_attribute.x_flip {
                7 - picture_x % 8
            } else {
                picture_x % 8
            };
            let color_low = if tile_y_data[0] & (0x80 >> tile_x) != 0 {
                1
            } else {
                0
            };
            let color_high = if tile_y_data[1] & (0x80 >> tile_x) != 0 {
                2
            } else {
                0
            };
            let color = color_high | color_low;
            self.priorities[x] = (tile_attribute.priority, color);
            if self.mode == CartridgeMode::GBC {
                let r = self.bgp_data[tile_attribute.cgb_palette_number][color][0];
                let g = self.bgp_data[tile_attribute.cgb_palette_number][color][1];
                let b = self.bgp_data[tile_attribute.cgb_palette_number][color][2];
                self.set_rgb(x as usize, r, g, b);
            } else {
                let color = self.get_gray_shade(self.bg_palette, color) as u8;
                self.set_greyscale(x, color);
            }
        }
    }

    // GameBoy video controller can display up to 40 sprites either in 8x8 or in 8x16 pixels. Because of a limitation of hardware,
    // only ten sprites can be displayed per scan line. Sprite patterns have the same format as BG tiles, but they are taken from
    // the Sprite Pattern Table located at $8000-8FFF and have unsigned numbering.
    // Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F. Each of the 40
    // entries consists of four bytes with the following meanings:
    //  Byte0 - Y Position
    //      Specifies the sprites vertical position on the screen (minus 16).
    //      An offscreen value (for example, Y=0 or Y>=160) hides the sprite.
    //  Byte1 - X Position
    //      Specifies the sprites horizontal position on the screen (minus 8).
    //      An offscreen value (X=0 or X>=168) hides the sprite, but the sprite
    //      still affects the priority ordering - a better way to hide a sprite is to set its Y-coordinate offscreen.
    //  Byte2 - Tile/Pattern Number
    //      Specifies the sprites Tile Number (00-FF). This (unsigned) value selects a tile from memory at 8000h-8FFFh. In CGB Mode this
    //      could be either in VRAM Bank 0 or 1, depending on Bit 3 of the following byte. In 8x16 mode, the lower bit of the tile number
    //      is ignored. Ie. the upper 8x8 tile is "NN AND FEh", and the lower 8x8 tile is "NN OR 01h".
    //  Byte3 - Attributes/Flags:
    //   Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    //          (Used for both BG and Window. BG color 0 is always behind OBJ)
    //   Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
    //   Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
    //   Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    //   Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
    //   Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
    // Sprite Priorities and Conflicts
    // When sprites with different x coordinate values overlap, the one with the smaller x coordinate (closer to the left) will have
    // priority and appear above any others. This applies in Non CGB Mode only. When sprites with the same x coordinate values overlap,
    // they have priority according to table ordering. (i.e. $FE00 - highest, $FE04 - next highest, etc.) In CGB Mode priorities are
    // always assigned like this. Only 10 sprites can be displayed on any one line. When this limit is exceeded, the lower priority
    // sprites (priorities listed above) won't be displayed. To keep unused sprites from affecting onscreen sprites set their Y
    // coordinate to Y=0 or Y=>144+16. Just setting the X coordinate to X=0 or X=>160+8 on a sprite will hide it but it will still
    // affect other sprites sharing the same lines.
    // Writing Data to OAM Memory
    // The recommened method is to write the data to normal RAM first, and to copy that RAM to OAM by using the DMA transfer function,
    // initiated through DMA register (FF46). Beside for that, it is also possible to write data directly to the OAM area by using normal
    // LD commands, this works only during the H-Blank and V-Blank periods. The current state of the LCD controller can be read out
    // from the STAT register (FF41).
    fn draw_sprites(&mut self) {
        // Sprite tile size 8x8 or 8x16(2 stacked vertically).
        let sprite_size = if self.lcd_control.has_bit2() { 16 } else { 8 };
        for i in 0..40 {
            let sprite_addr = 0xFE00 + (i as u16) * 4;
            let picture_y = self.get_byte(sprite_addr).wrapping_sub(16);
            let picture_x = self.get_byte(sprite_addr + 1).wrapping_sub(8);
            let tile_number = self.get_byte(sprite_addr + 2)
                & if self.lcd_control.has_bit2() {
                    0xFE
                } else {
                    0xFF
                };
            let tile_attribute = Attribute::from(self.get_byte(sprite_addr + 3));

            // If this is true the scanline is out of the area we care about
            if picture_y <= 0xFF - sprite_size + 1 {
                if self.lcdc_y < picture_y || self.lcdc_y > picture_y + sprite_size - 1 {
                    continue;
                }
            } else {
                if self.lcdc_y > picture_y.wrapping_add(sprite_size) - 1 {
                    continue;
                }
            }
            if picture_x >= (SCREEN_WIDTH as u8) && picture_x <= (0xFF - 7) {
                continue;
            }

            let tile_y = if tile_attribute.y_flip {
                sprite_size - 1 - self.lcdc_y.wrapping_sub(picture_y)
            } else {
                self.lcdc_y.wrapping_sub(picture_y)
            };
            let tile_y_addr = 0x8000u16 + u16::from(tile_number) * 16 + u16::from(tile_y) * 2;
            let tile_y_data: [u8; 2] =
                if self.mode == CartridgeMode::GBC && tile_attribute.vram_bank {
                    let b1 = self.get_vram(1, tile_y_addr);
                    let b2 = self.get_vram(1, tile_y_addr + 1);
                    [b1, b2]
                } else {
                    let b1 = self.get_vram(0, tile_y_addr);
                    let b2 = self.get_vram(0, tile_y_addr + 1);
                    [b1, b2]
                };

            for x in 0..8 {
                if picture_x.wrapping_add(x) >= (SCREEN_WIDTH as u8) {
                    continue;
                }
                let tile_x = if tile_attribute.x_flip { 7 - x } else { x };
                let color_low = if tile_y_data[0] & (0x80 >> tile_x) != 0 {
                    1
                } else {
                    0
                };
                let color_high = if tile_y_data[1] & (0x80 >> tile_x) != 0 {
                    2
                } else {
                    0
                };
                let color = color_high | color_low;
                if color == 0 {
                    continue;
                }

                // Confirm the priority of background and sprite.
                let priority = self.priorities[picture_x.wrapping_add(x) as usize];
                let skip = if self.mode == CartridgeMode::GBC && !self.lcd_control.has_bit0() {
                    priority.1 == 0
                } else if priority.0 {
                    priority.1 != 0
                } else {
                    tile_attribute.priority && priority.1 != 0
                };
                if skip {
                    continue;
                }

                if self.mode == CartridgeMode::GBC {
                    let r = self.bgp_data[tile_attribute.cgb_palette_number][color][0];
                    let g = self.bgp_data[tile_attribute.cgb_palette_number][color][1];
                    let b = self.bgp_data[tile_attribute.cgb_palette_number][color][2];
                    self.set_rgb(picture_x.wrapping_add(x) as usize, r, g, b);
                } else {
                    let color = if tile_attribute.palette_number == 1 {
                        self.get_gray_shade(self.object_pallete_1, color) as u8
                    } else {
                        self.get_gray_shade(self.object_pallete_0, color) as u8
                    };
                    self.set_greyscale(picture_x.wrapping_add(x) as usize, color);
                }
            }
        }
    }
}

impl Memory for PPU {
    fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            // VRAM (memory at 8000h-9FFFh) is accessable during Mode 0-2
            0x8000..=0x9FFF => self.vram[self.vram_bank * 0x2000 + addr as usize - 0x8000],
            // OAM (memory at FE00h-FE9Fh) is accessable during Mode 0-1
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            // FF40 - LCDC - LCD Control (R/W)
            0xFF40 => self.lcd_control.data,
            // FF41 - STAT - LCDC Status (R/W)
            0xFF41 => {
                let bit6 = if self.lcd_status.lyc_interrupt_enabled {
                    0x40
                } else {
                    0x00
                };
                let bit5 = if self.lcd_status.m2_oam_interrupt_enabled {
                    0x20
                } else {
                    0x00
                };
                let bit4 = if self.lcd_status.m1_vblank_interrupt_enabled {
                    0x10
                } else {
                    0x00
                };
                let bit3 = if self.lcd_status.m0_hblank_interrupt_enabled {
                    0x08
                } else {
                    0x00
                };
                let bit2 = if self.lcdc_y == self.ly_compare {
                    0x04
                } else {
                    0x00
                };
                bit6 | bit5 | bit4 | bit3 | bit2 | self.lcd_status.mode
            }
            // FF42 - SCY - Scroll Y (R/W)
            0xFF42 => self.scroll_y,
            // FF43 - SCX - Scroll X (R/W)
            0xFF43 => self.scroll_x,
            // FF44 - LY - LCDC Y-Coordinate (R)
            0xFF44 => self.lcdc_y,
            // FF45 - LYC - LY Compare (R/W)
            0xFF45 => self.ly_compare,
            // FF47 - BGP - BG Palette Data (R/W) - Non CGB Mode Only
            0xFF47 => self.bg_palette,
            // FF48 - OBP0 - Object Palette 0 Data (R/W) - Non CGB Mode Only
            0xFF48 => self.object_pallete_0,
            // FF49 - OBP1 - Object Palette 1 Data (R/W) - Non CGB Mode Only
            0xFF49 => self.object_pallete_1,
            // FF4A - WY - Window Y Position (R/W)
            0xFF4A => self.window_y,
            // FF4B - WX - Window X Position minus 7 (R/W)
            0xFF4B => self.window_x,
            // FF4F - VBK - CGB Mode Only - VRAM Bank
            0xFF4F => 0xFE | self.vram_bank as u8,
            // FF68 - BCPS/BGPI - CGB Mode Only - Background Palette Index
            0xFF68 => self.bgpi.get(),
            // FF69 - BCPD/BGPD - CGB Mode Only - Background Palette Data
            0xFF69 => {
                let r = self.bgpi.index as usize >> 3;
                let c = self.bgpi.index as usize >> 1 & 0x3;
                if self.bgpi.index & 0x01 == 0x00 {
                    let a = self.bgp_data[r][c][0];
                    let b = self.bgp_data[r][c][1] << 5;
                    a | b
                } else {
                    let a = self.bgp_data[r][c][1] >> 3;
                    let b = self.bgp_data[r][c][2] << 2;
                    a | b
                }
            }
            // FF6A - OCPS/OBPI - CGB Mode Only - Sprite Palette Index
            0xFF6A => self.obpi.get(),
            // FF6B - OCPD/OBPD - CGB Mode Only - Sprite Palette Data
            0xFF6B => {
                let r = self.obpi.index as usize >> 3;
                let c = self.obpi.index as usize >> 1 & 0x3;
                if self.obpi.index & 0x01 == 0x00 {
                    let a = self.obp_data[r][c][0];
                    let b = self.obp_data[r][c][1] << 5;
                    a | b
                } else {
                    let a = self.obp_data[r][c][1] >> 3;
                    let b = self.obp_data[r][c][2] << 2;
                    a | b
                }
            }
            _ => panic!("ppu: invalid address {:#06X?}", addr),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // VRAM (memory at 8000h-9FFFh) is accessable during Mode 0-2
            0x8000..=0x9FFF => self.vram[self.vram_bank * 0x2000 + addr as usize - 0x8000] = value,
            // OAM (memory at FE00h-FE9Fh) is accessable during Mode 0-1
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xfe00] = value,
            // FF40 - LCDC - LCD Control (R/W)
            0xFF40 => {
                self.lcd_control.data = value;
                if !self.lcd_control.has_bit7() {
                    self.dots = 0;
                    self.lcdc_y = 0;
                    self.lcd_status.mode = 0;
                    // Clean screen.
                    self.data = [[[0xffu8; 3]; SCREEN_WIDTH]; SCREEN_HEIGHT];
                    self.vblank = true;
                }
            }
            // FF41 - STAT - LCDC Status (R/W)
            0xFF41 => {
                self.lcd_status.lyc_interrupt_enabled = value & 0x40 != 0x00;
                self.lcd_status.m2_oam_interrupt_enabled = value & 0x20 != 0x00;
                self.lcd_status.m1_vblank_interrupt_enabled = value & 0x10 != 0x00;
                self.lcd_status.m0_hblank_interrupt_enabled = value & 0x08 != 0x00;
            }
            // FF42 - SCY - Scroll Y (R/W)
            0xFF42 => self.scroll_y = value,
            // FF43 - SCX - Scroll X (R/W)
            0xFF43 => self.scroll_x = value,
            // FF44 - LY - LCDC Y-Coordinate (R)
            0xFF44 => {}
            // FF45 - LYC - LY Compare (R/W)
            0xFF45 => self.ly_compare = value,
            // FF47 - BGP - BG Palette Data (R/W) - Non CGB Mode Only
            0xFF47 => self.bg_palette = value,
            // FF48 - OBP0 - Object Palette 0 Data (R/W) - Non CGB Mode Only
            0xFF48 => self.object_pallete_0 = value,
            // FF49 - OBP1 - Object Palette 1 Data (R/W) - Non CGB Mode Only
            0xFF49 => self.object_pallete_1 = value,
            // FF4A - WY - Window Y Position (R/W)
            0xFF4A => self.window_y = value,
            // FF4B - WX - Window X Position minus 7 (R/W)
            0xFF4B => self.window_x = value,
            // FF4F - VBK - CGB Mode Only - VRAM Bank
            0xFF4F => self.vram_bank = (value & 0x01) as usize,
            // FF68 - BCPS/BGPI - CGB Mode Only - Background Palette Index
            0xFF68 => self.bgpi.set(value),
            // FF69 - BCPD/BGPD - CGB Mode Only - Background Palette Data
            0xFF69 => {
                let r = self.bgpi.index as usize >> 3;
                let c = self.bgpi.index as usize >> 1 & 0x03;
                if self.bgpi.index & 0x01 == 0x00 {
                    self.bgp_data[r][c][0] = value & 0x1F;
                    self.bgp_data[r][c][1] = (self.bgp_data[r][c][1] & 0x18) | (value >> 5);
                } else {
                    self.bgp_data[r][c][1] =
                        (self.bgp_data[r][c][1] & 0x07) | ((value & 0x03) << 3);
                    self.bgp_data[r][c][2] = (value >> 2) & 0x1F;
                }
                if self.bgpi.auto_increment {
                    self.bgpi.index += 0x01;
                    self.bgpi.index &= 0x3F;
                }
            }
            // FF6A - OCPS/OBPI - CGB Mode Only - Sprite Palette Index
            0xFF6A => self.obpi.set(value),
            // FF6B - OCPD/OBPD - CGB Mode Only - Sprite Palette Data
            0xFF6B => {
                let r = self.obpi.index as usize >> 3;
                let c = self.obpi.index as usize >> 1 & 0x03;
                if self.obpi.index & 0x01 == 0x00 {
                    self.obp_data[r][c][0] = value & 0x1F;
                    self.obp_data[r][c][1] = (self.obp_data[r][c][1] & 0x18) | (value >> 5);
                } else {
                    self.obp_data[r][c][1] =
                        (self.obp_data[r][c][1] & 0x07) | ((value & 0x03) << 3);
                    self.obp_data[r][c][2] = (value >> 2) & 0x1F;
                }
                if self.obpi.auto_increment {
                    self.obpi.index += 0x01;
                    self.obpi.index &= 0x3F;
                }
            }
            _ => panic!("ppu: invalid address {:#06X?}", addr),
        }
    }
}
