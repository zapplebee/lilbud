use crate::config::{HEIGHT, WIDTH};
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

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

    pub fn flush(&mut self, buffer: &[u8; WIDTH * HEIGHT * 2]) {
        for (i, chunk) in buffer.chunks_exact(2).enumerate() {
            let raw = ((chunk[0] as u16) << 8) | chunk[1] as u16;
            let color = Rgb565::from(RawU16::new(raw));
            let rgb = Rgb888::from(color);
            let x = (i % WIDTH) as i32;
            let y = (i / WIDTH) as i32;
            self.canvas.set_draw_color(Color::RGB(rgb.r(), rgb.g(), rgb.b()));
            self.canvas.draw_point(sdl2::rect::Point::new(x, y)).unwrap();
        }
        self.canvas.present();
    }
}
