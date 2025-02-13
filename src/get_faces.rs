use rand::Rng;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{env, fs, sync::OnceLock};
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct LogLine {
    pub level: String,
    pub message: Message,
}
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Message {
    pub emo: String,
    pub points: HashMap<String, PointData>,
}
#[derive(Debug, Clone)]
pub struct PointData {
    pub x: i32,
    pub y: i32,
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
            // shrink to prevent jiggles from overflowing canvas size
            x: to_i32(&raw.x)? / 2 + 50,
            y: to_i32(&raw.y)? / 2 + 50,
        })
    }
}

static FACES: OnceLock<Vec<LogLine>> = OnceLock::new();

pub fn init() -> &'static Vec<LogLine> {
    FACES.get_or_init(|| {
        let face_file_path = env::var("FACE_FILE_PATH").expect("FACE_FILE_PATH not set");
        let contents = fs::read_to_string(face_file_path).expect("Failed to read the file");

        contents
            .lines()
            .filter_map(|line| serde_json::from_str::<LogLine>(line).ok())
            .collect()
    })
}

pub fn get_random_face() -> HashMap<String, PointData> {
    let face_logs = init();
    let mut rng = rand::thread_rng();
    return face_logs[rng.gen_range(0..face_logs.len())]
        .clone()
        .message
        .points;
}
