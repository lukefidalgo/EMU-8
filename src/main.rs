use std::env;
use std::thread;
use std::process;
use std::time::Duration;

use minifb::{Key, Window, WindowOptions, Scale};

mod chip8;
mod input;
mod stack;

use chip8::Chip8;
use input::*;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-rom>", args[0]);
        process::exit(1);
    }

    let rom_filename = &args[1];

    let mut window = Window::new(
        "EMU-8",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            topmost: true,
            ..WindowOptions::default() 
        }
    )
    .unwrap();

    let mut chip8 = Chip8::new();
    if let Err(e) = chip8.load_rom(rom_filename) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let keys = poll_input(&window);
        chip8.set_keys(keys);
        
        chip8.decrement_timers();

        // ~700 instructions per second
        for _ in 0..11 {
            chip8.cycle();
        }

        if chip8.update_window {
            window.update_with_buffer(
                chip8.get_display_buffer(),
                DISPLAY_WIDTH, DISPLAY_HEIGHT
            ).unwrap();
            chip8.update_window = false;
        } else {
            window.update();
        }

        // 60hz
        thread::sleep(Duration::from_millis(16));
    }
}
