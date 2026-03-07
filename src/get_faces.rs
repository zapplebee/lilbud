#[cfg(not(any(feature = "desktop", feature = "embedded")))]
compile_error!("Enable either the 'desktop' or 'embedded' feature.");

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use core::fmt;

// ── PointData ─────────────────────────────────────────────────────────────────
// Custom deserializer written with plain serde visitors so it compiles under
// both serde_json (desktop) and serde_json_core (embedded).

#[derive(Debug, Clone)]
pub struct PointData {
    pub x: i32,
    pub y: i32,
}

/// Deserializes a JSON number (integer or float) into i32.
struct FlexNum(i32);

impl<'de> Deserialize<'de> for FlexNum {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = FlexNum;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number")
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> { Ok(FlexNum(v as i32)) }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> { Ok(FlexNum(v as i32)) }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> { Ok(FlexNum(v as i32)) }
        }
        d.deserialize_any(V)
    }
}

impl<'de> Deserialize<'de> for PointData {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = PointData;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a point {x, y}")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut x: Option<i32> = None;
                let mut y: Option<i32> = None;
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "x" => x = Some(map.next_value::<FlexNum>()?.0),
                        "y" => y = Some(map.next_value::<FlexNum>()?.0),
                        _   => { let _ = map.next_value::<de::IgnoredAny>()?; }
                    }
                }
                Ok(PointData {
                    x: x.ok_or_else(|| de::Error::missing_field("x"))? / 2 + 50,
                    y: y.ok_or_else(|| de::Error::missing_field("y"))? / 2 + 50,
                })
            }
        }
        d.deserialize_map(V)
    }
}

// ── Desktop ───────────────────────────────────────────────────────────────────

#[cfg(feature = "desktop")]
mod inner {
    use super::PointData;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::sync::OnceLock;

    pub type PointMap = HashMap<String, PointData>;

    #[derive(Deserialize, Clone)]
    struct LogLine {
        #[allow(dead_code)]
        level: String,
        message: Message,
    }

    #[derive(Deserialize, Clone)]
    struct Message {
        #[allow(dead_code)]
        emo: String,
        points: PointMap,
    }

    static FACES: OnceLock<Vec<LogLine>> = OnceLock::new();

    fn init() -> &'static Vec<LogLine> {
        FACES.get_or_init(|| {
            include_str!(env!("FACE_FILE_PATH"))
                .lines()
                .filter_map(|line| serde_json::from_str::<LogLine>(line).ok())
                .collect()
        })
    }

    pub fn get_random_face() -> PointMap {
        use rand::Rng;
        let faces = init();
        let mut rng = rand::thread_rng();
        faces[rng.gen_range(0..faces.len())].clone().message.points
    }
}

// ── Embedded ─────────────────────────────────────────────────────────────────

#[cfg(feature = "embedded")]
mod inner {
    use super::PointData;
    use heapless::{FnvIndexMap, String, Vec};
    use portable_atomic::{AtomicU32, Ordering};
    use serde::Deserialize;

    // Point keys are single ASCII chars ("a".."r")
    pub type PointMap = FnvIndexMap<String<2>, PointData, 32>;

    #[derive(Deserialize)]
    struct LogLine<'a> {
        #[serde(borrow)]
        #[allow(dead_code)]
        level: &'a str,
        message: Message<'a>,
    }

    #[derive(Deserialize)]
    struct Message<'a> {
        #[allow(dead_code)]
        #[serde(borrow)]
        emo: &'a str,
        points: PointMap,
    }

    static mut FACES: Vec<PointMap, 128> = Vec::new();
    static FACE_COUNT: AtomicU32 = AtomicU32::new(0);
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

    pub fn init() {
        for line in include_str!(env!("FACE_FILE_PATH")).lines() {
            if let Ok((log, _)) = serde_json_core::from_str::<LogLine>(line) {
                unsafe { let _ = FACES.push(log.message.points); }
            }
        }
        FACE_COUNT.store(unsafe { FACES.len() } as u32, Ordering::Relaxed);
    }

    pub fn get_random_face() -> PointMap {
        let count = FACE_COUNT.load(Ordering::Relaxed);
        if count == 0 {
            return PointMap::new();
        }
        let idx = (lcg_next() % count) as usize;
        unsafe { FACES[idx].clone() }
    }
}

pub use inner::*;
