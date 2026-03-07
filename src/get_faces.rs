#[cfg(not(any(feature = "desktop", feature = "wasm", feature = "embedded")))]
compile_error!("Enable the 'desktop', 'wasm', or 'embedded' feature.");

// ── PointData ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PointData {
    pub x: i32,
    pub y: i32,
}

// ── Desktop / WASM ────────────────────────────────────────────────────────────
// Face data is read from the codegen'd face_data::FACE_DATA static array —
// no file I/O, no env vars, no serde at runtime.

#[cfg(any(feature = "desktop", feature = "wasm"))]
mod inner {
    use super::PointData;
    use std::collections::HashMap;

    pub type PointMap = HashMap<String, PointData>;

    pub fn get_random_face() -> PointMap {
        use rand::Rng;
        let data = crate::face_data::FACE_DATA;
        let idx = rand::thread_rng().gen_range(0..data.len());
        let raw = &data[idx];
        b"abcdefghijklmnopqr"
            .iter()
            .enumerate()
            .map(|(i, k)| ((*k as char).to_string(), PointData { x: raw[i].0, y: raw[i].1 }))
            .collect()
    }
}

// ── Embedded ─────────────────────────────────────────────────────────────────
// No heap — face data is pre-baked as static Rust arrays by gen_faces.py.

#[cfg(feature = "embedded")]
mod inner {
    use super::PointData;
    use heapless::{FnvIndexMap, String};
    use portable_atomic::{AtomicU32, Ordering};

    pub type PointMap = FnvIndexMap<String<2>, PointData, 32>;

    static RNG: AtomicU32 = AtomicU32::new(0x1234_5678);

    pub fn seed_rng(seed: u32) {
        RNG.store(seed, Ordering::Relaxed);
    }

    fn lcg_next() -> u32 {
        RNG.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |s| {
            Some(s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
        })
        .unwrap()
    }

    pub fn get_random_face() -> PointMap {
        let data = crate::face_data::FACE_DATA;
        if data.is_empty() {
            return PointMap::new();
        }
        let idx = (lcg_next() % data.len() as u32) as usize;
        let raw = &data[idx];
        let mut map = PointMap::new();
        for (i, key) in b"abcdefghijklmnopqr".iter().enumerate() {
            let mut s = String::<2>::new();
            s.push(*key as char).ok();
            map.insert(s, PointData { x: raw[i].0, y: raw[i].1 }).ok();
        }
        map
    }
}

pub use inner::*;
