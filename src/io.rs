extern crate sdl3;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::time::Duration;

use crate::error::Chip8Error;

const SCALING: u32 = 8;

#[derive(Clone)]
pub struct IO {
    context: sdl3::Sdl,

    keys_pressed: [u8; 16],
    key_pressed: i32,
    key_released: i32,

    width: u32,
    height: u32,
    debug_width: u32,
    debug_height: u32,
}

impl IO {
    pub fn new(width: usize, height: usize) -> Result<Self, Chip8Error> {
        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("chip8-emulator-rs", width as u32 * SCALING, height as u32 * SCALING)
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump()?;
        let mut i = 0;
        'running: loop {
            i = (i + 1) % 255;
            canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(IO {
            context: sdl_context,
            keys_pressed: [0; 16],
            key_pressed: -1,
            key_released: -1,
            width: width as u32,
            height: height as u32,
            debug_width: 0,
            debug_height: 0,
        })
    }
}
