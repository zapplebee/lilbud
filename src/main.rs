mod config;
mod get_faces;
mod sdl2_display;
mod ui;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2_display::SDL2Display;
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut display = SDL2Display::new(&video_subsystem);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => ui::tick(),
                _ => {}
            }
        }
        ui::tick();
        let buffer = ui::draw_ui();
        display.flush(&buffer);

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
