use rand::Rng;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Polyline, PrimitiveStyle, PrimitiveStyleBuilder, Triangle,
};

use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::text::Text;

static WIDTH: i32 = 240;
static HEIGHT: i32 = 240;

use crate::get_faces::{self, PointCollection};

/// Renders the UI into a pixel buffer.
/// This function expects a mutable RNG reference for randomness.
pub fn draw_ui<R: Rng>(rng: &mut R) -> [Rgb565; (WIDTH as usize) * (HEIGHT as usize)] {
    let mut buffer = [Rgb565::CSS_DODGER_BLUE; (WIDTH as usize) * (HEIGHT as usize)];

    let face_2: PointCollection = get_faces::get_random_face(&mut *rng);
    let mut points_2 = [Point { x: 0, y: 0 }; 18];

    for (i, point) in face_2.points.iter().enumerate() {
        points_2[i] = Point::new(
            point.x + rng.gen_range(1..=5),
            point.y + rng.gen_range(1..=5),
        );
    }

    let mut fb = Framebuffer::new(&mut buffer);

    // Generate secondary random offsets.
    let secondary_points = [
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
    ];

    // Draw triangles with a purple fill.
    Triangle::new(
        points_2[0] + secondary_points[0],
        points_2[1] + secondary_points[1],
        points_2[2] + secondary_points[2],
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
    .draw(&mut fb)
    .unwrap();

    Triangle::new(
        points_2[2] + secondary_points[2],
        points_2[3] + secondary_points[3],
        points_2[0] + secondary_points[0],
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
    .draw(&mut fb)
    .unwrap();

    // Draw additional background triangles in green.
    Triangle::new(points_2[0], points_2[1], points_2[2])
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb)
        .unwrap();

    Triangle::new(points_2[2], points_2[3], points_2[0])
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb)
        .unwrap();

    // Set a line style for polyline primitives.
    let line_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::BLACK)
        .stroke_width(2)
        .build();

    // Draw facial feature lines.
    Polyline::new(&[points_2[4], points_2[5]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points_2[6], points_2[7]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points_2[8], points_2[9]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points_2[10], points_2[11]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points_2[12], points_2[13]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points_2[14], points_2[15]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();

    // Draw circles for additional facial features.
    Circle::new(points_2[16], 10)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLACK)
                .build(),
        )
        .draw(&mut fb)
        .unwrap();
    Circle::new(points_2[17], 10)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLACK)
                .build(),
        )
        .draw(&mut fb)
        .unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_DODGER_BLUE);

    // Draw "Hello, world!" at position (10,10)
    Text::new("aaaaa", Point::new(100, 100), text_style)
        .draw(&mut fb)
        .unwrap();

    buffer
}

/// A simple framebuffer that implements DrawTarget.
pub struct Framebuffer<'a> {
    buffer: &'a mut [Rgb565; (WIDTH as usize) * (HEIGHT as usize)],
}

impl<'a> Framebuffer<'a> {
    pub fn new(buffer: &'a mut [Rgb565; (WIDTH as usize) * (HEIGHT as usize)]) -> Self {
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
            if coord.x >= 0 && coord.x < WIDTH && coord.y >= 0 && coord.y < HEIGHT {
                let index = (coord.y as usize * (WIDTH as usize)) + (coord.x as usize);
                self.buffer[index] = color;
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for Framebuffer<'a> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}
