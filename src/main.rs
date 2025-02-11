mod config;  // ✅ Add this line to include `config.rs`
mod ui;
mod sdl2_display;

use crate::config::{WIDTH, HEIGHT};  use embedded_graphics::prelude::WebColors;
// ✅ Now `config.rs` is available
use sdl2_display::SDL2Display;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use embedded_graphics::pixelcolor::Rgb565;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut display = SDL2Display::new(&video_subsystem);

    let mut buffer = [Rgb565::CSS_BLACK; WIDTH * HEIGHT]; // ✅ No more errors

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        ui::draw_ui(&mut buffer); // ✅ Now works because WIDTH & HEIGHT are found
        display.flush(&buffer);

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
