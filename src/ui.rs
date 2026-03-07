use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Polyline, PrimitiveStyle, PrimitiveStyleBuilder, Triangle,
};

use crate::config::{HEIGHT, WIDTH};
use crate::get_faces::{get_random_face, PointMap};

// ── Framebuffer (shared) ──────────────────────────────────────────────────────

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
                self.buffer[(coord.y as usize * WIDTH) + coord.x as usize] = color;
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

// ── Desktop UI state ──────────────────────────────────────────────────────────

#[cfg(feature = "desktop")]
mod state {
    use super::{get_random_face, PointMap};
    use crate::get_faces::PointData;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Mutex, OnceLock};

    static TICK: OnceLock<AtomicU32> = OnceLock::new();
    static FACE: OnceLock<Mutex<PointMap>> = OnceLock::new();
    static TARGET: OnceLock<Mutex<PointMap>> = OnceLock::new();

    pub fn set_face() {
        let mut t = TARGET.get_or_init(|| Mutex::new(get_random_face())).lock().unwrap();
        *t = get_random_face();
    }

    pub fn tick() {
        let tick = TICK.get_or_init(|| AtomicU32::new(0));
        let face_mx = FACE.get_or_init(|| Mutex::new(get_random_face()));
        let target_mx = TARGET.get_or_init(|| Mutex::new(get_random_face()));

        let new_face: PointMap = {
            let face = face_mx.lock().unwrap();
            let target = target_mx.lock().unwrap();
            face.iter()
                .filter_map(|(k, v)| {
                    target.get(k).map(|tv| {
                        (k.clone(), PointData {
                            x: interp(v.x, tv.x, 2),
                            y: interp(v.y, tv.y, 2),
                        })
                    })
                })
                .collect()
        };

        *face_mx.lock().unwrap() = new_face;
        tick.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some((v + 1) % 128)).ok();
    }

    pub fn with_face<F: FnOnce(&PointMap)>(f: F) {
        let face = FACE.get_or_init(|| Mutex::new(get_random_face())).lock().unwrap();
        f(&face);
    }

    pub fn jitter() -> (i32, i32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (rng.gen_range(1..=5), rng.gen_range(1..=5))
    }

    pub fn jitter4() -> [(i32, i32); 4] {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        core::array::from_fn(|_| (rng.gen_range(-10..=10), rng.gen_range(-10..=10)))
    }

    fn interp(start: i32, target: i32, step: i32) -> i32 {
        if start < target { (start + step).min(target) }
        else if start > target { (start - step).max(target) }
        else { start }
    }
}

// ── Embedded UI state ─────────────────────────────────────────────────────────

#[cfg(feature = "embedded")]
mod state {
    use super::{get_random_face, PointMap};
    use portable_atomic::{AtomicU32, Ordering};

    static TICK: AtomicU32 = AtomicU32::new(0);
    static mut FACE: Option<PointMap> = None;
    static mut TARGET: Option<PointMap> = None;

    // Simple LCG — shared with get_faces via the same atomic (jitter only needs weak randomness)
    static JITTER_RNG: AtomicU32 = AtomicU32::new(0xDEAD_BEEF);

    fn jitter_next() -> i32 {
        (JITTER_RNG
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |s| {
                Some(s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
            })
            .unwrap()
            >> 16) as i16 as i32
    }

    pub fn set_face() {
        unsafe { TARGET = Some(get_random_face()); }
    }

    pub fn tick() {
        unsafe {
            let face = FACE.get_or_insert_with(get_random_face);
            let target = TARGET.get_or_insert_with(get_random_face);
            for (k, v) in face.iter_mut() {
                if let Some(tv) = target.get(k) {
                    v.x = interp(v.x, tv.x, 2);
                    v.y = interp(v.y, tv.y, 2);
                }
            }
        }
        TICK.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some((v + 1) % 128)).ok();
    }

    pub fn with_face<F: FnOnce(&PointMap)>(f: F) {
        unsafe {
            let face = FACE.get_or_insert_with(get_random_face);
            f(face);
        }
    }

    pub fn jitter() -> (i32, i32) {
        let a = (jitter_next().abs() % 5) + 1;
        let b = (jitter_next().abs() % 5) + 1;
        (a, b)
    }

    pub fn jitter4() -> [(i32, i32); 4] {
        core::array::from_fn(|_| {
            let a = jitter_next() % 10;
            let b = jitter_next() % 10;
            (a, b)
        })
    }

    fn interp(start: i32, target: i32, step: i32) -> i32 {
        if start < target { (start + step).min(target) }
        else if start > target { (start - step).max(target) }
        else { start }
    }
}

// ── Public API (both targets) ─────────────────────────────────────────────────

pub fn set_face() {
    state::set_face();
}

pub fn tick() {
    state::tick();
}

pub fn draw_ui(buffer: &mut [Rgb565; WIDTH * HEIGHT]) {
    let mut fb = Framebuffer::new(buffer);
    fb.clear(Rgb565::BLUE).unwrap();

    state::with_face(|face| {
        let jxy = state::jitter();
        let points: heapless::FnvIndexMap<&str, Point, 32> = face
            .iter()
            .map(|(k, v)| (k.as_str(), Point::new(v.x + jxy.0, v.y + jxy.1)))
            .collect();

        let sp = state::jitter4();
        let shadow = sp.map(|(dx, dy)| Point::new(dx, dy));

        let line_style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::BLACK)
            .stroke_width(2)
            .build();

        // Shadow head
        Triangle::new(points["a"] + shadow[0], points["b"] + shadow[1], points["c"] + shadow[2])
            .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
            .draw(&mut fb).unwrap();
        Triangle::new(points["c"] + shadow[2], points["d"] + shadow[3], points["a"] + shadow[0])
            .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
            .draw(&mut fb).unwrap();

        // Head
        Triangle::new(points["a"], points["b"], points["c"])
            .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
            .draw(&mut fb).unwrap();
        Triangle::new(points["c"], points["d"], points["a"])
            .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
            .draw(&mut fb).unwrap();

        // Face lines
        for [p, q] in [["e","f"],["g","h"],["i","j"],["k","l"],["m","n"],["o","p"]] {
            Polyline::new(&[points[p], points[q]])
                .into_styled(line_style)
                .draw(&mut fb).unwrap();
        }

        // Eyes
        for eye in ["q", "r"] {
            Circle::new(points[eye], 10)
                .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::BLACK).build())
                .draw(&mut fb).unwrap();
        }
    });
}
