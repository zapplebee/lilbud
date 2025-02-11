use crate::config::{HEIGHT, WIDTH};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// SDL2 Display wrapper
pub struct SDL2Display {
    canvas: Canvas<Window>,
}

impl SDL2Display {
    pub fn new(video_subsystem: &sdl2::VideoSubsystem) -> Self {
        let window = video_subsystem
            .window("Embedded UI", WIDTH as u32, HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        SDL2Display { canvas }
    }

    pub fn flush(&mut self, buffer: &[Rgb565; WIDTH * HEIGHT]) {
        for (i, pixel) in buffer.iter().enumerate() {
            let x = (i % WIDTH) as i32;
            let y = (i / WIDTH) as i32;

            let rgb = embedded_graphics::pixelcolor::Rgb888::from(*pixel);
            self.canvas
                .set_draw_color(Color::RGB(rgb.r(), rgb.g(), rgb.b()));
            self.canvas
                .draw_point(sdl2::rect::Point::new(x, y))
                .unwrap();
        }
        self.canvas.present();
    }
}
