extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Scancode;
use sdl3::pixels::Color;

use crate::error::Chip8Error;

const SCALING: u32 = 8;

const KEYCODES: [Scancode; 16] = [
    Scancode::_1,
    Scancode::_2,
    Scancode::_3,
    Scancode::_4,
    Scancode::Q,
    Scancode::W,
    Scancode::E,
    Scancode::R,
    Scancode::A,
    Scancode::S,
    Scancode::D,
    Scancode::F,
    Scancode::Z,
    Scancode::X,
    Scancode::C,
    Scancode::V,
];

const POSITION_TO_KEY: [u8; 16] = [
    0x1, 0x2, 0x3, 0xC, 0x4, 0x5, 0x6, 0xD, 0x7, 0x8, 0x9, 0xE, 0xA, 0x0, 0xB, 0xF,
];

const KEY_TO_POSITION: [u8; 16] = [
    0xD, 0x0, 0x1, 0x2, 0x4, 0x5, 0x6, 0x8, 0x9, 0xA, 0xC, 0xE, 0x3, 0x7, 0xB, 0xF,
];

pub struct IO {
    context: sdl3::Sdl,
    canvas: sdl3::render::Canvas<sdl3::video::Window>,

    keys_pressed: [bool; 16],
    key_pressed: i32,
    key_released: i32,

    width: u32,
    height: u32,
}

impl IO {
    pub fn new(width: usize, height: usize) -> Result<Self, Chip8Error> {
        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "chip8-emulator-rs",
                width as u32 * SCALING,
                height as u32 * SCALING,
            )
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(IO {
            context: sdl_context,
            canvas,
            keys_pressed: [false; 16],
            key_pressed: -1,
            key_released: -1,
            width: width as u32,
            height: height as u32,
        })
    }

    pub fn poll(&mut self) -> Result<u32, Chip8Error> {
        let mut event_pump = self.context.event_pump()?;
        let mut status = 0;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => {
                    return Ok(crate::emulator::QUIT);
                }
                Event::KeyDown {
                    scancode: Some(Scancode::P),
                    ..
                } => status |= crate::emulator::PAUSE,
                Event::KeyDown {
                    scancode: Some(Scancode::M),
                    ..
                } => status |= crate::emulator::STEP_MODE,
                Event::KeyDown {
                    scancode: Some(Scancode::N),
                    ..
                } => status |= crate::emulator::SHOULD_STEP,
                Event::KeyDown {
                    scancode: Some(Scancode::_0),
                    ..
                } => status |= crate::emulator::RESET,
                Event::KeyDown {
                    scancode: Some(code),
                    ..
                } => self.set_key(code),
                Event::KeyUp {
                    scancode: Some(code),
                    ..
                } => self.reset_key(code),
                _ => {}
            }
        }
        Ok(status)
    }

    fn set_key(&mut self, code: Scancode) {
        if let Some(pos) = KEYCODES.iter().position(|&k| k == code) {
            self.keys_pressed[pos] = true;
        }
    }

    fn reset_key(&mut self, code: Scancode) {
        if let Some(pos) = KEYCODES.iter().position(|&k| k == code) {
            self.keys_pressed[pos] = false;
            self.key_released = POSITION_TO_KEY[pos] as i32;
        }
    }

    pub fn check_key_pressed(&self, key: u8) -> bool {
        if key < 16 {
            self.keys_pressed[KEY_TO_POSITION[key as usize] as usize]
        } else {
            false
        }
    }

    pub fn get_key_pressed(&mut self) -> i32 {
        if self.key_pressed == self.key_released {
            let val = self.key_pressed;
            self.key_pressed = -1;
            return val;
        }

        for (i, _) in POSITION_TO_KEY.iter().enumerate() {
            if self.keys_pressed[i] {
                self.key_pressed = POSITION_TO_KEY[i] as i32;
            }
        }

        crate::emulator::NO_KEY_PRESSED
    }

    pub fn draw(&mut self, pixels: &[u8]) -> Result<(), Chip8Error> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        for x in 0..self.width {
            for y in 0..self.height {
                let pixel_index = (y * self.width + x) as usize;

                if pixels[pixel_index] != 0 {
                    let rect = sdl3::rect::Rect::new(
                        (x * SCALING) as i32,
                        (y * SCALING) as i32,
                        SCALING,
                        SCALING,
                    );
                    self.canvas.fill_rect(rect)?;
                }
            }
        }

        self.canvas.present();
        Ok(())
    }
}
