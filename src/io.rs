extern crate sdl3;

use std::error::Error;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::time::Duration;

pub struct IO {}

impl IO {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("chip8-emulator-rs", 800, 600)
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

        Ok(IO {})
    }
}
