extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Point as SDLPoint;
use sdl2::render::Canvas;
use sdl2::video::Window;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::geometry::Size;

/// Trait to unify graphics backends (SDL2 and RP2040)
pub trait GraphicsBackend: DrawTarget<Color = Rgb565> + OriginDimensions {
    fn clear(&mut self, color: Rgb565);
}

/// SDL2 Display Backend
pub struct SDL2Display {
    canvas: Canvas<Window>,
}

impl SDL2Display {
    /// Creates a new SDL2 display, **taking `video_subsystem` as a parameter**
    pub fn new(video_subsystem: &sdl2::VideoSubsystem) -> Self {
        let window = video_subsystem.window("Embedded UI", 240, 240)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0)); // Set background color to black
        canvas.clear();
        canvas.present();

        SDL2Display { canvas }
    }
}

/// Implement `DrawTarget` for SDL2 so it can be used with `embedded-graphics`
impl DrawTarget for SDL2Display {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            let x = coord.x as i32;
            let y = coord.y as i32;
            let rgb565 = color.into_storage();

            // Manually extract R, G, B from 16-bit Rgb565
            let r = ((rgb565 >> 11) & 0x1F) as u8 * 8;
            let g = ((rgb565 >> 5) & 0x3F) as u8 * 4;
            let b = (rgb565 & 0x1F) as u8 * 8;

            self.canvas.set_draw_color(Color::RGB(r, g, b));
            self.canvas.draw_point(SDLPoint::new(x, y)).unwrap();
        }
        self.canvas.present();
        Ok(())
    }
}

/// Implement `OriginDimensions` for SDL2Display
impl OriginDimensions for SDL2Display {
    fn size(&self) -> Size {
        Size::new(240, 240) // Match the SDL2 window size
    }
}

/// Implement `GraphicsBackend` for SDL2Display
impl GraphicsBackend for SDL2Display {
    fn clear(&mut self, color: Rgb565) {
        let rgb565 = color.into_storage();
        let r = ((rgb565 >> 11) & 0x1F) as u8 * 8;
        let g = ((rgb565 >> 5) & 0x3F) as u8 * 4;
        let b = (rgb565 & 0x1F) as u8 * 8;

        self.canvas.set_draw_color(Color::RGB(r, g, b));
        self.canvas.clear();
    }
}
