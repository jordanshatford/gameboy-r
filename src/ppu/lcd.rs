// LCD Control Register
//  Bit 7 - LCD Display Enable             (0=Off, 1=On)
//  Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
//  Bit 5 - Window Display Enable          (0=Off, 1=On)
//  Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
//  Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
//  Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
//  Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
//  Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)

// LCDC.7 - LCD Display Enable
// CAUTION: Stopping LCD operation (Bit 7 from 1 to 0) may be performed during V-Blank ONLY,
// disabeling the display outside of the V-Blank period may damage the hardware. This appears
// to be a serious issue, Nintendo is reported to reject any games that do not follow this rule.
// V-blank can be confirmed when the value of LY is greater than or equal to 144. When the
// display is disabled the screen is blank (white), and VRAM and OAM can be accessed freely.

// --- LCDC.0 has different Meanings depending on Gameboy Type ---
// LCDC.0 - 1) Monochrome Gameboy and SGB: BG Display
// When Bit 0 is cleared, the background becomes blank (white). Window and Sprites may still
// be displayed (if enabled in Bit 1 and/or Bit 5).
// LCDC.0 - 2) CGB in CGB Mode: BG and Window Master Priority
// When Bit 0 is cleared, the background and window lose their priority - the sprites will be always
// displayed on top of background and window, independently of the priority flags in OAM and BG Map attributes.
// LCDC.0 - 3) CGB in Non CGB Mode: BG and Window Display
// When Bit 0 is cleared, both background and window become blank (white), ie. the Window Display
// Bit (Bit 5) is ignored in that case. Only Sprites may still be displayed (if enabled in Bit 1).
// This is a possible compatibility problem - any monochrome games (if any) that disable the background,
// but still want to display the window wouldn't work properly on CGBs.
#[derive(Debug, Copy, Clone)]
pub struct LCDControl {
    data: u8,
}

impl LCDControl {
    pub fn new() -> LCDControl {
        LCDControl { data: 0b0100_1000 }
    }

    fn has_bit7(&self) -> bool {
        self.data & 0b1000_0000 != 0x00
    }

    fn has_bit6(&self) -> bool {
        self.data & 0b0100_0000 != 0x00
    }

    fn has_bit5(&self) -> bool {
        self.data & 0b0010_0000 != 0x00
    }

    fn has_bit4(&self) -> bool {
        self.data & 0b0001_0000 != 0x00
    }

    fn has_bit3(&self) -> bool {
        self.data & 0b0000_1000 != 0x00
    }

    fn has_bit2(&self) -> bool {
        self.data & 0b0000_0100 != 0x00
    }

    fn has_bit1(&self) -> bool {
        self.data & 0b0000_0010 != 0x00
    }

    fn has_bit0(&self) -> bool {
        self.data & 0b0000_0001 != 0x00
    }
}

// LCD Status Register
//  Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
//  Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
//  Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
//  Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
//  Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
//  Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
//           0: During H-Blank
//           1: During V-Blank
//           2: During Searching OAM-RAM
//           3: During Transfering Data to LCD Driver

// The two lower STAT bits show the current status of the LCD controller.
//   Mode 0: The LCD controller is in the H-Blank period and
//           the CPU can access both the display RAM (8000h-9FFFh)
//           and OAM (FE00h-FE9Fh)
//   Mode 1: The LCD contoller is in the V-Blank period (or the
//           display is disabled) and the CPU can access both the
//           display RAM (8000h-9FFFh) and OAM (FE00h-FE9Fh)
//   Mode 2: The LCD controller is reading from OAM memory.
//           The CPU <cannot> access OAM memory (FE00h-FE9Fh)
//           during this period.
//   Mode 3: The LCD controller is reading from both OAM and VRAM,
//           The CPU <cannot> access OAM and VRAM during this period.
//           CGB Mode: Cannot access Palette Data (FF69,FF6B) either.
// The following are typical when the display is enabled:
//   Mode 2  2_____2_____2_____2_____2_____2___________________2____
//   Mode 3  _33____33____33____33____33____33__________________3___
//   Mode 0  ___000___000___000___000___000___000________________000
//   Mode 1  ____________________________________11111111111111_____
// The Mode Flag goes through the values 0, 2, and 3 at a cycle of about 109uS. 0 is present about 48.6uS, 2 about 19uS,
// and 3 about 41uS. This is interrupted every 16.6ms by the VBlank (1). The mode flag stays set at 1 for about 1.08 ms.
// Mode 0 is present between 201-207 clks, 2 about 77-83 clks, and 3 about 169-175 clks. A complete cycle through these
// states takes 456 clks. VBlank lasts 4560 clks. A complete screen refresh occurs every 70224 clks.)
#[derive(Debug, Copy, Clone)]
pub struct LCDStatus {
    //  Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    lyc_interrupt_enabled: bool,
    //  Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    m2_oam_interrupt_enabled: bool,
    //  Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    m1_vblank_interrupt_enabled: bool,
    //  Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    m0_hblank_interrupt_enabled: bool,
    //  Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
    //  Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
    //           0: During H-Blank
    //           1: During V-Blank
    //           2: During Searching OAM-RAM
    //           3: During Transfering Data to LCD Driver
    mode: u8,
}

impl LCDStatus {
    pub fn new() -> LCDStatus {
        LCDStatus {
            lyc_interrupt_enabled: false,
            m2_oam_interrupt_enabled: false,
            m1_vblank_interrupt_enabled: false,
            m0_hblank_interrupt_enabled: false,
            mode: 0x00,
        }
    }
}

// FF68 - BCPS/BGPI - CGB Mode Only - Background Palette Index
// This register is used to address a byte in the CGBs Background Palette Memory. Each two byte in that memory
// define a color value. The first 8 bytes define Color 0-3 of Palette 0 (BGP0), and so on for BGP1-7.
//   Bit 0-5   Index (00-3F)
//   Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
// Data can be read/written to/from the specified index address through Register FF69. When the Auto Increment
// Bit is set then the index is automatically incremented after each <write> to FF69. Auto Increment has no
// effect when <reading> from FF69, so the index must be manually incremented in that case.
pub struct BGPI {
    index: u8,
    auto_increment: bool,
}

impl BGPI {
    pub fn new() -> BGPI {
        BGPI {
            index: 0x00,
            auto_increment: false,
        }
    }

    fn get(&self) -> u8 {
        let auto_inc = if self.auto_increment { 0x80 } else { 0x00 };
        auto_inc | self.index
    }

    fn set(&mut self, value: u8) {
        self.auto_increment = value & 0x80 != 0x00;
        self.index = value & 0x3F;
    }
}
