use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Polyline, PrimitiveStyle, PrimitiveStyleBuilder, Triangle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::geometry::Point;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Size;
use embedded_graphics::geometry::OriginDimensions; // ✅ Import this trait

use crate::config::{WIDTH, HEIGHT};

/// Function to render UI into a pixel buffer
pub fn draw_ui(buffer: &mut [Rgb565; WIDTH * HEIGHT]) {
    let mut fb = Framebuffer::new(buffer);

    // Clear screen to black
    fb.clear(Rgb565::BLUE).unwrap();


    let margin = 20;

    let ppts = [
        Point::new(20 + margin, 20 + margin),
        Point::new(WIDTH as i32 - 20 - margin, 20 + margin),
        Point::new(WIDTH as i32 - 40 - margin, HEIGHT as i32 - 20 - margin),
        Point::new(40 + margin, HEIGHT as i32 - 20 - margin),
        Point::new(20 + margin, 20 + margin),
    ];



    Triangle::new(ppts[0], ppts[1], ppts[2])
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb)
        .unwrap();

        Triangle::new(ppts[2], ppts[3], ppts[4])
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb)
        .unwrap();

    // Polyline::new(&ppts)
    //     .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 5))
    //     .draw(&mut fb)
    //     .unwrap();

    // Polyline::new(&left_eye)
    //     .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 2))
    //     .draw(&mut fb)
    //     .unwrap();


    // left eye
    Polyline::new(&[Point::new(10, 50), Point::new(100, 85)]).into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb565::BLACK).stroke_width(2).build()).draw(&mut fb).unwrap();
    Polyline::new(&[Point::new(10, 80), Point::new(100, 85)]).into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb565::BLACK).stroke_width(2).build()).draw(&mut fb).unwrap();

    Circle::new(Point::new(80, 95), 10)
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::BLACK).build())
        .draw(&mut fb)
        .unwrap();


    // right eye

    Polyline::new(&[Point::new(140, 50), Point::new(230, 85)]).into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb565::BLACK).stroke_width(2).build()).draw(&mut fb).unwrap();
    Polyline::new(&[Point::new(140, 80), Point::new(230, 85)]).into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb565::BLACK).stroke_width(2).build()).draw(&mut fb).unwrap();
    Circle::new(Point::new(160, 95), 10)
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::BLACK).build())
        .draw(&mut fb)
        .unwrap();

    // mouth

    Polyline::new(&[Point::new(80, 150), Point::new(160, 150)]).into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb565::BLACK).stroke_width(2).build()).draw(&mut fb).unwrap();
    Polyline::new(&[Point::new(80, 152), Point::new(160, 160)]).into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb565::BLACK).stroke_width(2).build()).draw(&mut fb).unwrap();
}

/// Framebuffer struct to store pixels
pub struct Framebuffer<'a> {
    buffer: &'a mut [Rgb565; WIDTH * HEIGHT],
}

impl<'a> Framebuffer<'a> {
    pub fn new(buffer: &'a mut [Rgb565; WIDTH * HEIGHT]) -> Self {
        Framebuffer { buffer }
    }
}

impl<'a> DrawTarget for Framebuffer<'a> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.x < WIDTH as i32 && coord.y >= 0 && coord.y < HEIGHT as i32 {
                let index = (coord.y as usize * WIDTH) + coord.x as usize;
                self.buffer[index] = color;
            }
        }
        Ok(())
    }
}

/// ✅ **Fix: Implement `OriginDimensions`**
impl<'a> OriginDimensions for Framebuffer<'a> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}
