use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::geometry::Point;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Polyline, PrimitiveStyle, PrimitiveStyleBuilder, Triangle,
};
use rand::Rng;

use crate::config::{HEIGHT, WIDTH};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FaceDataLog {
    level: String,
    message: FaceData,
}

/// Struct to match the incoming JSON format
#[derive(Deserialize, Debug)]
struct FaceData {
    emo: String,
    points: std::collections::HashMap<String, PointData>,
}

/// Struct for point data
#[derive(Deserialize, Debug)]
struct PointData {
    x: i32,
    y: i32,
}

/// Function to render UI into a pixel buffer
pub fn draw_ui(buffer: &mut [Rgb565; WIDTH * HEIGHT]) {
    let mut rng = rand::thread_rng();
    let mut fb = Framebuffer::new(buffer);

    // Clear screen to black
    fb.clear(Rgb565::BLUE).unwrap();

    let parsed: FaceDataLog = match serde_json::from_str("{\"level\":\"info\",\"message\":{\"emo\":\"sleepy\",\"points\":{\"a\":{\"x\":27,\"y\":16},\"b\":{\"x\":213,\"y\":16},\"c\":{\"x\":224,\"y\":208},\"d\":{\"x\":20,\"y\":206},\"e\":{\"x\":17,\"y\":55},\"f\":{\"x\":88,\"y\":33},\"g\":{\"x\":17,\"y\":65},\"h\":{\"x\":97,\"y\":43},\"i\":{\"x\":136,\"y\":34},\"j\":{\"x\":209,\"y\":50},\"k\":{\"x\":132,\"y\":40},\"l\":{\"x\":209,\"y\":60},\"m\":{\"x\":23,\"y\":123},\"n\":{\"x\":172,\"y\":115},\"o\":{\"x\":117,\"y\":122},\"p\":{\"x\":144,\"y\":121},\"q\":{\"x\":80,\"y\":100},\"r\":{\"x\":140,\"y\":100}}}}") {
        Ok(data) => data,
        Err(e) => {
            println!("Error parsing JSON: {}", e);
            return; // Handle error gracefully
        }
    };

    // Convert points to embedded-graphics Points
    let points: std::collections::HashMap<&str, Point> = parsed
        .message
        .points
        .iter()
        .map(|(key, val)| {
            (
                key.as_str(),
                Point::new(val.x + rng.gen_range(1..=5), val.y + rng.gen_range(1..=5)),
            )
        })
        .collect();

    let margin = 20;

    Triangle::new(
        points["a"] + Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        points["b"] + Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        points["c"] + Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
    .draw(&mut fb)
    .unwrap();

    Triangle::new(
        points["c"] + Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        points["d"] + Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        points["a"] + Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
    .draw(&mut fb)
    .unwrap();

    Triangle::new(points["a"], points["b"], points["c"])
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb)
        .unwrap();

    Triangle::new(points["c"], points["d"], points["a"])
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb)
        .unwrap();

    let line_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::BLACK)
        .stroke_width(2)
        .build();
    // left eye
    Polyline::new(&[points["e"], points["f"]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points["g"], points["h"]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points["i"], points["j"]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points["k"], points["l"]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points["m"], points["n"]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Polyline::new(&[points["o"], points["p"]])
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Circle::new(points["q"], 10)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLACK)
                .build(),
        )
        .draw(&mut fb)
        .unwrap();
    Circle::new(points["r"], 10)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLACK)
                .build(),
        )
        .draw(&mut fb)
        .unwrap();
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
