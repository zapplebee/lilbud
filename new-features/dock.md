# Feature: Dock

When lilbud's board is plugged into the computer over USB, he "moves" to the computer — the macOS WebView app becomes the primary display AND a data portal for the board's identity and social files. When unplugged, the board resumes as the sole display and the app goes dormant.

The board is the single source of truth for all state — animation, identity, and messages. The Mac Rust host is a dumb proxy: it forwards face frames from the board to the WebView and relays file read/write commands from the WebView to the board. It never stores data locally. All persistent data lives on the board's flash filesystem.

When two buds meet IRL (see `irl_connection.md`), they exchange files directly board-to-board. The dock interface is how owners read what arrived and manage their own identity.

---

## Narrative

- **Board unplugged (undocked)**: Board animates normally on its physical display. macOS app shows an empty room — lilbud is not here.
- **Board plugged in (docked)**: Board redirects its animation output over USB instead of to its physical display; board screen shows the empty room. macOS app renders the face from board-streamed data and exposes the data portal (ID card, messages received IRL).
- The board is always the animation engine. The Mac just switches which display it's rendering to.
- The "empty room" image is identical in concept on both ends: the enriched backdrop, darkened, with no face drawn.

---

## Visual Design

### Backdrop enrichment

The current backdrop is a solid blue. It needs subtle texture:

- Wide, gentle blue-on-blue wave shapes (elliptical arcs or filled ellipses, slightly darker or lighter than the base blue)
- Low contrast — atmospheric, not distracting
- Drawn in code via `embedded-graphics` primitives; no image assets

This enriched backdrop becomes the base layer for all rendering (animated and static).

### Static "empty room" image

- Enriched backdrop, drawn darker (dim the base blue further)
- No face elements drawn
- Used by: board when docked, app when undocked

---

## Scope

**In scope:**
- Embedded firmware (RP2040)
- macOS WebView app (`webview` feature)

**Removed as part of this work:**
- SDL2 / `desktop` feature — removed entirely

---

## Board Filesystem

The RP2040's flash beyond the firmware partition is formatted with **littlefs** (`littlefs2` crate — no_std, wear-leveled). The board stores:

```
/id_card.txt          — name + free-text bio (plain text, owner-editable via dock)
/messages/            — files received from IRL exchanges (format TBD)
```

The ID card is a simple UTF-8 text file. Line 1 is the display name; the rest is the bio. No schema enforced beyond that.

---

## USB Protocol

One bidirectional CDC serial channel carries two logical streams:

**Board → Host (push, continuous when docked):**
- `FACE` frames: 2-byte sync header `0xFA 0xCE` + 72 bytes (18 × i16 x + i16 y = 18 × 4 bytes). Sent each animation tick.

**Host → Board (request/response, on demand):**
- `READ <path>\n` → board responds with `OK <len>\n<bytes>` or `ERR\n`
- `WRITE <path> <len>\n<bytes>` → board responds with `OK\n` or `ERR\n`
- `LIST /messages/\n` → board responds with newline-separated filenames + `END\n`

The host never buffers responses beyond the current transaction. All file data flows through and is immediately forwarded to the WebView.

---

## WebView State Machine

The WebView owns all application logic. The Mac Rust host knows nothing about state — it just forwards frames and relays file commands.

```
UNDOCKED
  - render: empty room
  - data panels: hidden

DOCKED / FACE
  - render: face from board FACE frames via requestAnimationFrame
  - data panels: ID card (read), messages inbox (read)

DOCKED / EDITING
  - render: face continues in background
  - data panels: ID card editor active, save triggers WRITE to board
```

Transitions:
- `UNDOCKED → DOCKED/FACE`: first FACE frame received
- `DOCKED → UNDOCKED`: no FACE frame for 1.5s (disconnect or pause)
- `DOCKED/FACE → DOCKED/EDITING`: user opens editor panel
- `DOCKED/EDITING → DOCKED/FACE`: user saves or cancels

---

## WASM Role

WASM is a rendering library only. All logic lives in JS.

```rust
#[wasm_bindgen]
pub fn render_face(points: &[i32]) { /* draw face from 18×(x,y) coords to canvas */ }

#[wasm_bindgen]
pub fn render_empty_room() { /* draw darkened backdrop to canvas */ }
```

JS in `index.html` owns the state machine, IPC calls, panel DOM, and timeout logic.

---

## Test Strategy

### Layer 1 — Protocol unit tests

`src/usb_protocol.rs` is pure data: frame encode/decode, command serialization. Runs in `cargo test` with no I/O or hardware.

### Layer 2 — Board simulator (`src/bin/board_sim.rs`)

A host binary that pretends to be a board:
- Allocates a PTY pair; prints the slave path on startup
- Streams fake FACE frames at a configurable rate (default ~30fps)
- Responds to `READ`/`WRITE`/`LIST` with in-memory fixture data
- Accepts stdin commands:

```
connect              resume streaming FACE frames
disconnect           stop frames (simulate unplug)
frames 0             pause frame stream (test 1.5s timeout)
frames 30            resume at 30fps
read_id_card         print current fixture ID card
set_id_card <s>      update fixture ID card
```

`board_sim --script <file>` accepts newline-separated commands with `sleep <ms>` for automated test scenarios.

### Layer 3 — Integration

`BOARD_PORT` env var overrides serial port auto-detection:

```sh
# Terminal 1
cargo run --bin board_sim --features webview
# prints: PTY ready at /dev/ttys012

# Terminal 2
BOARD_PORT=/dev/ttys012 make webview
```

---

## Implementation Steps

### Step 0: Bidirectional IPC proof of concept

`WebView` is not `Send`. Prove the wry 0.24 threading pattern works before building on it.

```rust
enum UserEvent { ScriptToRun(String) }
```

- `EventLoop::<UserEvent>::new()`
- Background thread → `proxy.send_event(UserEvent::ScriptToRun(...))`
- Event loop handles `Event::UserEvent` → `webview.evaluate_script()`
- `with_ipc_handler()` for WebView → Host direction

### Step 1: SDL2 removal

- Delete `src/sdl2_display.rs`
- Remove `sdl2` from `Cargo.toml` and `[features]`; set `default = []`
- Remove `desktop` feature gates from `src/main.rs`
- Remove `make desktop` from `Makefile`
- Update `README.md` and `BUILDING.md`

### Step 2: Backdrop enrichment (`src/ui.rs`)

- Add `draw_backdrop(fb: &mut Framebuffer, dark: bool)`
  - Clears to base blue
  - Draws 3–5 wide elliptical wave bands in a slightly shifted blue
  - `dark` dims all colors for the empty-room state
- Replace `fb.clear(Rgb565::BLUE)` in `draw_ui()` with `draw_backdrop(&mut fb, false)`

### Step 3: Empty room renderer (`src/ui.rs`)

- Add `pub fn draw_empty_room(buffer: &mut [u8; WIDTH * HEIGHT * 2])`
  - Calls `draw_backdrop(&mut fb, true)`

### Step 4: Embedded USB CDC + littlefs

New crates: `usb-device = "0.3"`, `usbd-serial = "0.2"`, `littlefs2`

- Initialize littlefs on flash partition beyond firmware
- Initialize USB CDC alongside display
- Main loop two states:
  - **Docked**: draw empty room once; run animation tick; send FACE frame over USB each tick; handle `READ`/`WRITE`/`LIST` against littlefs
  - **Undocked**: normal animation loop to physical display

### Step 5: macOS serial + file relay (`src/webview_app.rs`)

New crate: `serialport = "4"`

- Check `BOARD_PORT` env var before scanning for `cu.usbmodem*`
- Background thread reads FACE frames, calls `evaluate_script("window.__boardFrame([...])")`
- IPC handler receives file commands from JS via `mpsc::Sender` → writes to serial → reads response → `evaluate_script` with result
- No state stored in Rust

### Step 6: WASM renderer + JS state machine

`src/wasm_display.rs`: expose only `render_face(points)` and `render_empty_room()`.

`index.html` JS:
- `window.__boardFrame(points)` → `wasm.render_face(points)`; reset 1.5s timeout; transition to DOCKED_FACE if UNDOCKED
- Frame timeout → UNDOCKED; `wasm.render_empty_room()`
- ID card panel → `window.ipc.postMessage({cmd:"READ", path:"/id_card.txt"})`
- `window.__fileResult(data)` → populate panel
- Save → `window.ipc.postMessage({cmd:"WRITE", ...})`

---

## Files to Modify

| File | Change |
|------|--------|
| `src/webview_app.rs` | Step 0: UserEvent + proxy; Step 5: serial relay + file IPC |
| `src/wasm_display.rs` | Step 6: render_face, render_empty_room |
| `src/ui.rs` | Steps 2–3: draw_backdrop(), draw_empty_room(), update draw_ui() |
| `src/main.rs` | Step 1: remove desktop gates; Step 4: USB CDC + littlefs |
| `index.html` | Step 6: state machine, ID card panel UI |
| `Cargo.toml` | Steps 1, 4, 5: remove sdl2; add usb-device, usbd-serial, littlefs2, serialport |
| `Makefile` | Step 1: remove desktop target |
| `README.md` | Remove SDL2; add dock + data portal section |
| `BUILDING.md` | Remove SDL2 prerequisites |

**New source files:**
- `src/usb_protocol.rs` — FACE frame encoder (embedded) + decoder (host), file command framing
- `src/bin/board_sim.rs` — board simulator binary

**File to delete:** `src/sdl2_display.rs`

---

## Verification

1. **IPC PoC**: `make webview` — Rust logs IPC from JS; JS receives dispatched event
2. **Backdrop**: waves visible behind the face in the WebView
3. **Empty room**: darkened backdrop, no face elements
4. **Face streaming**: WebView renders face driven by board frames, not its own loop
5. **State machine**: unplug → UNDOCKED (empty room); replug → DOCKED_FACE
6. **ID card read**: dock board → ID card content from littlefs appears in panel
7. **ID card write**: edit + save → board's `/id_card.txt` updated; persists across unplug
8. **SDL2 removal**: `cargo build --features desktop` fails cleanly
