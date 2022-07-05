mod attribute;
pub mod hdma;
mod lcd;

use crate::cartridges::CartridgeMode;
use crate::memory::Memory;
use crate::mmu::InterruptFlag;
use crate::ppu::attribute::Attribute;
use crate::ppu::lcd::{LCDControl, LCDStatus};

// Resolution - 160x144 (20x18 tiles)
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

#[derive(Debug, Copy, Clone)]
pub struct PPU {
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
}

impl PPU {
    pub fn new(mode: CartridgeMode) -> PPU {
        PPU {
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
