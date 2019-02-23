use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std;

pub struct KeyboardDriver {
    events: sdl2::EventPump,
}

impl KeyboardDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        KeyboardDriver {
            events: sdl_context.event_pump().unwrap(),
        }
    }

    // return a Result<Array of keys>
    pub fn poll(&mut self) -> Result<[bool; 16], ()> {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Err(()),
                _ => (),
            }
        }

        // Create a set of pressed Keys.
        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut key_state = [false; 16];

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = index {
                key_state[i] = true;
            }
        }

        Ok(key_state)
    }
}
