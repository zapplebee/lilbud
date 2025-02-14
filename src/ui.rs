use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Polyline, PrimitiveStyle, PrimitiveStyleBuilder, Triangle,
};
use rand::Rng;

use crate::config::{HEIGHT, WIDTH};
use crate::get_faces::{get_random_face, PointData};

static TICK_STATE: OnceLock<AtomicU32> = OnceLock::new();
static FACE: OnceLock<Mutex<HashMap<String, PointData>>> = OnceLock::new();
static TARGET_FACE: OnceLock<Mutex<HashMap<String, PointData>>> = OnceLock::new();

pub fn set_face() {
    let mutex_face = TARGET_FACE.get_or_init(|| Mutex::new(get_random_face()));
    let mut face = mutex_face.lock().unwrap();
    *face = get_random_face();
}




/// ✅ Atomically increment tick count and wrap at 56
pub fn tick() {
    let tick_state = TICK_STATE.get_or_init(|| AtomicU32::new(0));

    let current_face_mutex = FACE.get_or_init(|| Mutex::new(get_random_face()));
    let target_face_mutex = TARGET_FACE.get_or_init(|| Mutex::new(get_random_face()));

    let current_face = current_face_mutex.lock().unwrap();
    let target_face = target_face_mutex.lock().unwrap();

    let new_face: HashMap<String, PointData> = current_face
        .iter()
        .filter_map(|(key, val)| {
            target_face.get(key).map(|target_val| {

                let new_x = interpolate(val.x, target_val.x, 2); // Move by 2 pixels per tick
                let new_y = interpolate(val.y, target_val.y, 2);

                (key.clone(), PointData { x: new_x, y: new_y })
            })
        })
        .collect();

        drop(current_face); // ✅ Release lock before modifying

    // ✅ Overwrite the mutex-protected HashMap safely
    {
        let mut current_face = current_face_mutex.lock().unwrap();
        *current_face = new_face; // ✅ Replace the old HashMap with the new one
    }

    tick_state
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
            Some((val + 1) % 128)
        })
        .ok();
}

fn get_tick_state() -> u32 {
    TICK_STATE
        .get_or_init(|| AtomicU32::new(0))
        .load(Ordering::Relaxed)
}

fn interpolate(start: i32, target: i32, step: i32) -> i32 {
    if start < target {
        (start + step).min(target) // Move up, but don’t overshoot
    } else if start > target {
        (start - step).max(target) // Move down, but don’t overshoot
    } else {
        start // Already at target
    }
}

/// Function to render UI into a pixel buffer
pub fn draw_ui() -> [Rgb565; 57600] {
    let tick_state = get_tick_state();
    print!("{:?}\n", tick_state);
    let mut buffer = [Rgb565::CSS_BLACK; WIDTH * HEIGHT];
    let face_1 = FACE.get_or_init(|| Mutex::new(get_random_face())).lock().unwrap();
    let mut rng = rand::thread_rng();
    let mut fb = Framebuffer::new(&mut buffer);

    // Clear screen to black
    fb.clear(Rgb565::BLUE).unwrap();

    // Convert points to embedded-graphics Points
    let points: std::collections::HashMap<&str, Point> = face_1
        .iter()
        .map(|(key, val)| {
            (
                key.as_str(),
                Point::new(val.x + rng.gen_range(1..=5), val.y + rng.gen_range(1..=5)),
            )
        })
        .collect();

    // create secondary background. it ineeds to get painted first so that the face is on top

    // create the points first so that the randomization is consistent

    let secondary_points = [
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
        Point::new(rng.gen_range(-10..=10), rng.gen_range(-10..=10)),
    ];
    Triangle::new(
        points["a"] + secondary_points[0],
        points["b"] + secondary_points[1],
        points["c"] + secondary_points[2],
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
    .draw(&mut fb)
    .unwrap();

    Triangle::new(
        points["c"] + secondary_points[2],
        points["d"] + secondary_points[3],
        points["a"] + secondary_points[0],
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
    .draw(&mut fb)
    .unwrap();

    // create a secondary background

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
    buffer
}

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

impl<'a> OriginDimensions for Framebuffer<'a> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}
