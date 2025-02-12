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
use serde::{Deserialize, Deserializer};

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
#[derive(Debug)]
struct PointData {
    x: i32,
    y: i32,
}

/// Custom deserialization for `PointData`
impl<'de> Deserialize<'de> for PointData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawPointData {
            x: serde_json::Value,
            y: serde_json::Value,
        }

        let raw = RawPointData::deserialize(deserializer)?;
        
        // Convert the JSON values to i32 safely
        fn to_i32<T: serde::de::Error>(value: &serde_json::Value) -> Result<i32, T> {
            match value {
                serde_json::Value::Number(num) => {
                    if let Some(i) = num.as_i64() {
                        Ok(i as i32) // Safe cast from i64 to i32
                    } else if let Some(f) = num.as_f64() {
                        Ok(f.round() as i32) // Round and cast float to i32
                    } else {
                        Err(T::custom("Invalid number format"))
                    }
                }
                _ => Err(T::custom("Expected a number")),
            }
        }
        

        Ok(PointData {
            x: to_i32(&raw.x)?,
            y: to_i32(&raw.y)?,
        })
    }
}


/// Function to render UI into a pixel buffer
pub fn draw_ui(buffer: &mut [Rgb565; WIDTH * HEIGHT]) {
    let mut rng = rand::thread_rng();
    let mut fb = Framebuffer::new(buffer);

    // Clear screen to black
    fb.clear(Rgb565::BLUE).unwrap();

    let parsed: FaceDataLog = match serde_json::from_str("{\"level\":\"info\",\"message\":{\"emo\":\"silly\",\"points\":{\"a\":{\"x\":75,\"y\":76},\"b\":{\"x\":167,\"y\":87},\"c\":{\"x\":154,\"y\":151},\"d\":{\"x\":71,\"y\":142},\"e\":{\"x\":34,\"y\":84},\"f\":{\"x\":102,\"y\":51},\"g\":{\"x\":39,\"y\":90},\"h\":{\"x\":107,\"y\":60},\"i\":{\"x\":137,\"y\":54},\"j\":{\"x\":199,\"y\":91},\"k\":{\"x\":114,\"y\":68},\"l\":{\"x\":198,\"y\":101},\"m\":{\"x\":96,\"y\":131},\"n\":{\"x\":129,\"y\":128},\"o\":{\"x\":96,\"y\":136},\"p\":{\"x\":130,\"y\":136},\"q\":{\"x\":60,\"y\":103},\"r\":{\"x\":174,\"y\":115}}}}") {
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
