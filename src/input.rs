const CHIP8_KEYS: [minifb::Key; 16] = [
    minifb::Key::Key1, minifb::Key::Key2, minifb::Key::Key3, minifb::Key::Key4,
    minifb::Key::Q, minifb::Key::W, minifb::Key::E, minifb::Key::R,
    minifb::Key::A, minifb::Key::S, minifb::Key::D, minifb::Key::F,
    minifb::Key::Z, minifb::Key::X, minifb::Key::C, minifb::Key::V,
];

pub fn poll_input(window: &minifb::Window) -> [bool; 16] {
    let mut keys = [false; 16];
    for (i, key) in CHIP8_KEYS.iter().enumerate() {
        keys[i] = window.is_key_down(*key);
    }
    keys
}
