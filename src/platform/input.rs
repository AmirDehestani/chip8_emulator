use crate::constants::{INPUTS_COUNT};
use sdl2::keyboard::Keycode;

pub struct Input {
    pub keys: [bool; INPUTS_COUNT],
}

impl Input {
    pub fn new() -> Self {
        Self { keys: [false; INPUTS_COUNT] }
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        if key < INPUTS_COUNT {
            self.keys[key] = pressed;
        }
    }

    /// Map SDL2 keycodes to CHIP-8 hex keypad values
    pub fn map_sdl_keycode(keycode: sdl2::keyboard::Keycode) -> Option<usize> {
        match keycode {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        }
    }
}

