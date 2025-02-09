mod sdl2_display;

use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2_display::{GraphicsBackend, SDL2Display};
use std::time::Duration;

fn draw_ui<B: GraphicsBackend>(display: &mut B)
where
    <B as DrawTarget>::Error: std::fmt::Debug,
{
    let square = Rectangle::new(Point::new(70, 70), Size::new(100, 100))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    square.draw(display).unwrap();
}

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
                _ => {}
            }
        }

        GraphicsBackend::clear(&mut display, Rgb565::BLACK);
        draw_ui(&mut display);

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
