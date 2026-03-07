# Feature: Behavior & Care System

lilbud has needs — things the wearer tends to — and expresses its state through face selection, animation speed, and jitter intensity. The Waveshare RP2040-LCD-1.28 includes a QMI8658C 6-axis IMU on I2C. No new hardware is required.

The board is the source of truth. Needs state is persistent on littlefs alongside the ID card. The dock UI can read and display needs state; the IMU handles physical care gestures.

---

## Sensors Available

| Sensor | Interface | What it gives us |
|--------|-----------|-----------------|
| QMI8658C accelerometer | I2C | tap detection, shake, orientation, stillness |
| QMI8658C gyroscope | I2C | rotation rate, flip gestures |
| RP2040 on-chip temp sensor | ADC ch4 | ambient temperature (coarse) |

The QMI8658 supports hardware tap detection — the chip raises an interrupt on tap, no polling needed.

---

## Needs System

Three needs, each 0–100, stored in `/state.bin` on littlefs:

| Need | Depletes | Restored by |
|------|----------|-------------|
| **Hunger** | ~1 point / 10 min | Shake gesture (feed) |
| **Happiness** | ~1 point / 15 min | Tap gesture (pet), IRL meetings |
| **Energy** | ~1 point / 5 min awake | Sleep (automatic when still) |

Needs deplete using elapsed wall-clock time tracked by the RP2040 timer. State is written to flash periodically (every ~5 min) and on USB disconnect, to limit flash wear.

---

## Sleep / Wake

- **Sleep threshold**: accelerometer magnitude below `~0.15g` for 5 continuous minutes
- **Sleep behavior**: animation tick rate drops from ~30fps to ~0.5fps; face held near-still; subtle slow blink cycle
- **Wake trigger**: any acceleration event above threshold → immediate wake, brief "surprised" face selection
- **Energy recovery**: energy restores at ~1 point / 2 min while asleep

---

## Gesture Map

| Gesture | Detection | Effect |
|---------|-----------|--------|
| **Single tap** | QMI8658 tap interrupt | +10 happiness, "surprised" face flash |
| **Double tap** | QMI8658 double-tap interrupt | +20 happiness, "happy" face, faster animation burst |
| **Shake** | accel magnitude spike > 2g for >200ms | +25 hunger satisfaction, "happy" face transition |
| **Flip / upside down** | accel Z axis inverts | "surprised" face, wobble jitter spike |
| **Sustained tilt** | roll/pitch > 45° for > 2s | "thoughtful" face, slightly off-center jitter |
| **Stillness > 5min** | accel below threshold | enter sleep |

---

## Face Selection Driven by Needs

Face selection is weighted by current mood. The `get_random_face_for_mood(mood)` function (see face data re-encoding below) filters the face pool by emotion tag:

```
hunger < 20    → "sad"
happiness < 20 → "sad"
energy < 20    → "thoughtful"
all > 70       → "happy"
post-gesture   → brief override (see gesture map), then revert
default        → None (any face)
```

---

## Jitter as Emotional Texture

The existing per-frame jitter in `draw_ui()` is currently fixed at ±5px. It carries emotional meaning:

| State | Jitter |
|-------|--------|
| Content / default | ±3px |
| Excited (post-tap) | ±8px for ~2s, then decay |
| Frightened (shaken hard) | ±15px spike, rapid decay |
| Tired / sleeping | ±1px or 0 |
| Flipped | ±10px sustained |

---

## Dock UI Extensions

When docked, the data portal gains a needs panel:

- Hunger / Happiness / Energy shown as simple bars (read from board via `READ /state.bin`)
- "Feed" button → equivalent of shake gesture (board processes as +hunger via WRITE command)
- "Pet" button → equivalent of tap (+happiness)
- Needs update live while docked (periodic READ poll, or board pushes state in FACE frame header)

---

## Face Data Re-encoding (Prerequisite)

Emotion tags exist in the NDJSON but are currently discarded by `gen_faces.py` line 19.

### `gen_faces.py` change

Capture `emo = obj["message"]["emo"]` and emit a second parallel static alongside `FACE_DATA`:

```python
# existing (unchanged)
pub static FACE_DATA: &[[(i32, i32); 18]] = &[ ... ];

# new — index-aligned with FACE_DATA
pub static FACE_EMOS: &[&str] = &["neutral", "happy", "sad", "surprised", "thoughtful", ...];
```

Two parallel statics, same index. No structs, no heap, works in no_std.

Known values in the dataset: `neutral`, `happy`, `sad`, `surprised`, `thoughtful`

### `src/get_faces.rs` change

Add alongside the existing `get_random_face()`:

```rust
pub fn get_random_face_for_mood(mood: Option<&str>) -> PointMap
```

Algorithm (O(n), no heap):
1. Linear scan `FACE_EMOS` — count entries matching `mood` (all if `None`)
2. Pick random index in `[0, count)`
3. Second scan to find Nth matching entry → build `PointMap`
4. If no match: fall back to `get_random_face()` (defensive)

---

## New Files / Changes

| File | Change |
|------|--------|
| `gen_faces.py` | Capture `emo`; emit `pub static FACE_EMOS: &[&str]` parallel to `FACE_DATA` |
| `src/face_data.rs` | Auto-regenerated — gains `FACE_EMOS` |
| `src/get_faces.rs` | Add `get_random_face_for_mood(mood: Option<&str>)` |
| `src/qmi8658.rs` | New — I2C driver for QMI8658C: init, tap config, accel read |
| `src/needs.rs` | New — needs state, depletion tick, gesture handlers, mood query |
| `src/ui.rs` | Parameterize jitter range; expose jitter intensity setter |
| `src/main.rs` (embedded) | Init QMI8658, poll IMU each loop, call needs tick, pass mood to set_face |
| `index.html` | Dock: needs bars + feed/pet buttons |
| `memory.x` | Verify littlefs partition fits with added firmware size |

---

## Open Questions

- What I2C pins does the QMI8658 use on this board? (Likely SDA=GPIO6, SCL=GPIO7 — confirm from schematic.)
- Should needs state be visible on the physical display at all? (e.g. a small indicator in a corner)
- What happens when all needs reach 0? Is there a "sick" state?
- Temperature sensor: cold/hot reactions? (ADC ch4 — coarse but evocative)

---

## Verification

1. Flash firmware; confirm QMI8658 initializes (no I2C NACK)
2. Tap board case → face flashes to "surprised" expression
3. Shake board → hunger increases (verify via dock read of `/state.bin`)
4. Leave board still for 5 minutes → animation slows to sleep rate
5. Move board → immediate wake with surprised face
6. Hunger < 20 → face selection weighted toward "sad" faces
7. Dock: needs bars reflect board state; Feed button increases hunger score
