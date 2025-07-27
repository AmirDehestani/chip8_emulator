use crate::constants::{INPUTS_COUNT};

pub struct Input {
    pub keys: [bool; INPUTS_COUNT],
}

impl Input {
    pub fn new() -> Self {
        Self { keys: [false; INPUTS_COUNT] }
    }

    pub fn update(&mut self, event: &sdl2::event::Event) {
        let (down, keycode) = match event {
            sdl2::event::Event::KeyDown { keycode: Some(k), .. } => (true, k),
            sdl2::event::Event::KeyUp { keycode: Some(k), .. } => (false, k),
            _ => return,
        };

        if let Some(mapped) = match keycode {
            Num1 => Some(0x1),
            Num2 => Some(0x2),
            Num3 => Some(0x3),
            Num4 => Some(0xC),
            Q    => Some(0x4),
            W    => Some(0x5),
            E    => Some(0x6),
            R    => Some(0xD),
            A    => Some(0x7),
            S    => Some(0x8),
            D    => Some(0x9),
            F    => Some(0xE),
            Z    => Some(0xA),
            X    => Some(0x0),
            C    => Some(0xB),
            V    => Some(0xF),
            _    => None,
        } {
            self.keys[mapped] = down;
        }
    }
}
