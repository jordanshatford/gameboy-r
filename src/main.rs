use std::fs::File;
use std::io::Read;

use argparse::{ArgumentParser, Print, Store, StoreFalse, StoreTrue};
use gameboyr::{Dimensions, Gameboy, GameboyButton};
use minifb::{Key, Scale, Window, WindowOptions};

// Map minifb keys to there respective Gameboy buttons
const KEY_MAPPINGS: [(Key, GameboyButton); 8] = [
    (Key::Right, GameboyButton::Right),
    (Key::Up, GameboyButton::Up),
    (Key::Left, GameboyButton::Left),
    (Key::Down, GameboyButton::Down),
    (Key::Z, GameboyButton::A),
    (Key::X, GameboyButton::B),
    (Key::Space, GameboyButton::Select),
    (Key::Enter, GameboyButton::Start),
];

fn main() {
    let mut rom_path = String::from("");
    let mut save_path = String::from("");
    let mut window_scale = 1;
    let mut use_audio = true;
    let mut skip_checks = false;
    {
        let mut arg_parser = ArgumentParser::new();
        arg_parser.set_description("Gameboy R");
        arg_parser.add_option(
            &["-v", "--version"],
            Print(format!("Gameboy R version: v{}", env!("CARGO_PKG_VERSION"))),
            "Show current version of the program",
        );
        arg_parser.refer(&mut save_path).add_option(
            &["-s", "--save"],
            Store,
            "Path to .sav file of the specified ROM (Default: location of ROM)",
        );
        arg_parser.refer(&mut window_scale).add_option(
            &["-x", "--scale"],
            Store,
            "Scale the window by a factor of 1, 2, 4 (Default: 1)",
        );
        arg_parser.refer(&mut use_audio).add_option(
            &["--no-audio"],
            StoreFalse,
            "Run the emulator without audio support.",
        );
        arg_parser.refer(&mut skip_checks).add_option(
            &["--skip-checks"],
            StoreTrue,
            "Skip header checksum and nintendo logo checks for ROM",
        );
        arg_parser
            .refer(&mut rom_path)
            .add_argument("rom", Store, "Path to the ROM you want to use")
            .required();
        arg_parser.parse_args_or_exit();
    }

    // Default to ROM path if no save path specified
    if save_path.clone().is_empty() {
        save_path.clone_from(&rom_path);
    }

    let window_options = WindowOptions {
        resize: true,
        scale: match window_scale {
            1 => Scale::X2,
            2 => Scale::X4,
            4 => Scale::X8,
            _ => panic!("gameboyr: unsupported scale options (valid options: 1, 2, 4)"),
        },
        ..Default::default()
    };

    let mut file = File::open(rom_path.clone()).unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();

    let mut gameboy = Gameboy::new(rom, save_path, skip_checks);

    if use_audio {
        let success = gameboy.try_enable_audio();
        if !success {
            panic!("gameboyr: failed to enable audio (try using: --no-audio option)")
        }
    }

    let Dimensions { width, height } = gameboy.get_screen_dimensions();

    let mut window = Window::new(
        &format!("GameboyR - {}", gameboy.get_rom_title()),
        width,
        height,
        window_options,
    )
    .unwrap();
    let mut window_buffer = vec![0x00; width * height];
    window
        .update_with_buffer(window_buffer.as_slice(), width, height)
        .unwrap();

    while window.is_open() {
        gameboy.step();
        if gameboy.has_screen_updated() {
            for (i, pixel) in gameboy.get_screen_data().iter().enumerate() {
                let r = u32::from(pixel.r) << 16;
                let g = u32::from(pixel.g) << 8;
                let b = u32::from(pixel.b);
                let a = 0xFF00_0000;
                window_buffer[i] = a | r | g | b;
            }
            window
                .update_with_buffer(window_buffer.as_slice(), width, height)
                .unwrap();
        }
        if gameboy.can_take_input() {
            for (physical_key, gameboy_button) in &KEY_MAPPINGS {
                if window.is_key_down(*physical_key) {
                    gameboy.handle_keydown(*gameboy_button);
                } else {
                    gameboy.handle_keyup(*gameboy_button);
                }
            }
        }
    }
    gameboy.shutdown();
}
