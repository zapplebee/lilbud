# Feature: On-Device Generative Face Model

Instead of a fixed static array of hand-authored faces, the board generates face coordinates at runtime using a small neural network decoder with weights baked into the firmware. This gives infinite variety, natural emotion conditioning, and faces that interpolate smoothly between authored examples — all without a heap or floating-point hardware.

The build pipeline mirrors the existing `gen_faces.py → face_data.rs` pattern: a Python script trains the model and emits a Rust static file that is compiled into the binary.

---

## Hardware Constraints

| Resource | Budget | Model cost |
|---|---|---|
| Flash (~2MB total, shared with firmware) | ~500KB available | ~3–6KB for weights |
| SRAM (264KB, 115KB used by framebuffer) | ~149KB free | ~1KB for activations |
| CPU | Cortex-M0+ 133MHz, no FPU | ~1,500 MACs per face → microseconds |

The Cortex-M0+ has no FPU but has hardware multiply. The entire model runs in fixed-point integer arithmetic.

---

## Model Architecture

**Conditional VAE decoder** — emotion label + random latent → face coordinates.

Only the decoder is needed at runtime. The encoder is used only during offline training.

```
input:  [z0, z1, z2, z3,  e0, e1, e2, e3, e4]
         └── 4D latent ──┘  └── 5D emotion one-hot ──┘
         (sampled from LCG RNG at runtime)

layer 1: linear 9→32 + tanh
layer 2: linear 32→36

output: [x0,y0, x1,y1, ..., x17,y17]   (18 face points)
```

**Weight count:**
- Layer 1: 9×32 weights + 32 biases = 320
- Layer 2: 32×36 weights + 36 biases = 1,188
- Total: **1,508 values × 2 bytes (i16) = ~3KB**

**Tanh without FPU:** 256-entry i16 lookup table (~512 bytes in flash). Input clamped to [-4, 4] and mapped to table index.

**Fixed-point scheme:** Q8.8 (8 integer bits, 8 fractional bits). `i16 × i16 → i32 accumulator → shift right 8`. No division needed.

---

## Build Pipeline

```
lilbudz.ndjson
  │
  ▼
gen_model.py                     (new — Python, PyTorch or numpy)
  ├── load + normalize face data to [-1, 1]
  ├── train conditional VAE
  ├── extract decoder weights
  ├── quantize weights to i16 Q8.8
  └── emit src/model_weights.rs

src/model_weights.rs             (auto-generated, do not edit)
  ├── pub static DECODER_W1: [[i16; 9]; 32]
  ├── pub static DECODER_B1: [i16; 32]
  ├── pub static DECODER_W2: [[i16; 32]; 36]
  ├── pub static DECODER_B2: [i16; 36]
  ├── pub static TANH_TABLE: [i16; 256]
  └── pub const COORD_SCALE: i32   (dequantize output → pixel coords)

src/model.rs                     (new — no_std inference)
  └── pub fn generate_face(emotion: Emotion, rng: &mut LcgRng) -> [(i32, i32); 18]
```

Makefile: `make model` runs `gen_model.py` and regenerates `src/model_weights.rs`.

Note: `gen_faces.py` applies `x // 2 + 50` to map source coords into display space. `gen_model.py` normalizes differently — to `[-1, 1]` for training — and stores the inverse transform in `COORD_SCALE`. These must be consistent or generated faces will be out of bounds.

---

## Runtime Inference (`src/model.rs`)

```rust
pub fn generate_face(emotion: Emotion, rng: &mut LcgRng) -> [(i32, i32); 18] {
    // 1. sample 4D latent from LCG
    // 2. build 9D input: [z0..z3, emotion_onehot]
    // 3. layer 1: i16×i16 → i32 accumulator, add bias, tanh via TANH_TABLE
    // 4. layer 2: same, no activation
    // 5. dequantize using COORD_SCALE, clamp to [0, 240]
    // 6. return [(i32, i32); 18]
}
```

No heap, no floats, no `std`. The LCG RNG already in `get_faces.rs` serves double duty.

---

## Relationship to `face_data.rs`

`FACE_DATA` / `FACE_EMOS` are kept as:
- **Validation set** — compare generated faces against hand-authored ones during testing
- **Fallback** — if `model_weights.rs` hasn't been generated (e.g. fresh clone), fall back to static data
- **Training targets** — the data `gen_model.py` trains on

At runtime, `get_random_face_for_mood()` delegates to `model::generate_face(emotion, rng)`. `face_data.rs` is no longer the primary face source.

`board_sim` compiles `model.rs` for the host target to generate realistic test frames without hardware, keeping the test harness consistent with firmware behavior.

---

## Training Data Requirements

| Training set size | Expected quality |
|---|---|
| ~50 total (current test data) | Toy model; faces plausible but low variety |
| ~250 (50 per emotion) | Good baseline; variety noticeably better than static set |
| ~500+ (100 per emotion) | Smooth interpolation; generated faces feel authored |

Before committing the mood → emotion mapping in `needs.rs`, verify which emotion values actually exist in the full `lilbudz.ndjson` dataset — the test `faces.ndjson` may not represent all labels.

---

## Stepping Stone: PCA first

If the full VAE pipeline is too much to validate at once, PCA is a viable stepping stone:

- Compute mean face vector (36 floats) and top-k eigenvectors offline
- Store as ~1KB static (mean + k×36 eigenvectors)
- At runtime: `face = mean + Σ(rng_sample_i × eigenvector_i)`
- No activation function, no tanh table needed
- Works with as few as 5 faces to prove the pipeline end-to-end

Upgrade to VAE once the build pipeline (`gen_model.py → model_weights.rs → model.rs`) is solid.

---

## New Files / Changes

| File | Change |
|---|---|
| `gen_model.py` | New — train VAE (or PCA), quantize, emit `src/model_weights.rs` |
| `src/model_weights.rs` | Auto-generated — decoder weights, biases, tanh table, scale |
| `src/model.rs` | New — no_std fixed-point inference, `generate_face()` |
| `src/get_faces.rs` | `get_random_face_for_mood()` delegates to `model::generate_face()` |
| `src/main.rs` (embedded) | Pass `Emotion` from needs state to `generate_face()` |
| `Makefile` | Add `make model` target |

---

## Verification

1. `make model` completes; `src/model_weights.rs` is generated
2. `cargo build --features embedded` succeeds with model weights compiled in
3. Flash board; faces render correctly (no degenerate/out-of-bounds points)
4. Faces visually vary across reboots (RNG seed from timer ticks)
5. Emotion conditioning: force hunger low → generated faces match "sad" style
6. Compare generated faces visually against hand-authored validation set
