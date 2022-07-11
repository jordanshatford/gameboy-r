use argparse::{ArgumentParser, Print, Store, StoreTrue};
use gameboy_r::gameboy::{Gameboy, GameboyButton};
use minifb::{Key, Scale, Window, WindowOptions};

fn main() {
    let mut rom_path = String::from("");
    let mut window_scale = 1;
    let mut skip_checks = false;
    {
        let mut arg_parser = ArgumentParser::new();
        arg_parser.set_description("Game Boy R");
        arg_parser.add_option(
            &["-v", "--version"],
            Print(format!(
                "Game Boy R version: v{}",
                env!("CARGO_PKG_VERSION")
            )),
            "Show current version of the program",
        );
        arg_parser.refer(&mut window_scale).add_option(
            &["-x", "--scale"],
            Store,
            "Scale the window by a factor of 1, 2, 4 (Default: 1)",
        );
        arg_parser.refer(&mut skip_checks).add_option(
            &["--skip-checks"],
            StoreTrue,
            "Skip header checksum and nintendo logo checks for ROM",
        );
        arg_parser
            .refer(&mut rom_path)
            .add_argument("rom", Store, "Path to the rom you want to use.")
            .required();
        arg_parser.parse_args_or_exit();
    }

    let window_options = WindowOptions {
        resize: true,
        scale: match window_scale {
            1 => Scale::X2,
            2 => Scale::X4,
            4 => Scale::X8,
            _ => panic!("gameboy-r: unsupported scale options (valid options: 1, 2, 4)"),
        },
        ..Default::default()
    };

    let mut gameboy = Gameboy::new(rom_path, skip_checks);

    let (width, height) = gameboy.get_screen_dimensions();

    let mut window = Window::new(&gameboy.get_title(), width, height, window_options).unwrap();
    let mut window_buffer = vec![0x00; width * height];
    window
        .update_with_buffer(window_buffer.as_slice(), width, height)
        .unwrap();

    while window.is_open() {
        gameboy.step();
        if gameboy.has_screen_updated() {
            let mut i: usize = 0;
            for l in gameboy.get_screen_data().iter() {
                for w in l.iter() {
                    let r = u32::from(w[0]) << 16;
                    let g = u32::from(w[1]) << 8;
                    let b = u32::from(w[2]);
                    let a = 0xFF00_0000;
                    window_buffer[i] = a | r | g | b;
                    i += 1;
                }
            }
            window
                .update_with_buffer(window_buffer.as_slice(), width, height)
                .unwrap();
        }

        if gameboy.can_take_input() {
            let key_map = vec![
                (Key::Right, GameboyButton::Right),
                (Key::Up, GameboyButton::Up),
                (Key::Left, GameboyButton::Left),
                (Key::Down, GameboyButton::Down),
                (Key::Z, GameboyButton::A),
                (Key::X, GameboyButton::B),
                (Key::Space, GameboyButton::Select),
                (Key::Enter, GameboyButton::Start),
            ];
            for (physical_key, gameboy_button) in &key_map {
                if window.is_key_down(*physical_key) {
                    gameboy.handle_keydown(*gameboy_button);
                } else {
                    gameboy.handle_keyup(*gameboy_button);
                }
            }
        }
    }
    gameboy.save();
}
