# Feature: Dock

When lilbud's board is plugged into the computer over USB, he "moves" to the computer — the macOS app becomes the active display and the board shows a static "away" image. When unplugged, the app shows a static "closed door" image and lilbud moves back to the board.

---

## Behavior

### Board plugged in

- **Board**: displays a static image (e.g. an empty room, a closed door from the inside, a "be right back" screen). No animation. Board is effectively dormant as a display.
- **macOS app**: lilbud becomes animated and alive — full face morphing loop runs in the app.

### Board unplugged

- **macOS app**: switches to a static "closed door" image — lilbud has left the building. App stays open but is no longer animating.
- **Board**: resumes normal animated face loop.

---

## Implementation sketch

### USB communication

The RP2040 needs a USB stack. Add `usb-device` + `usbd-serial` to the embedded target. The board presents as a USB CDC serial device on connect.

The protocol is minimal — the host just needs to know the board is there. A simple heartbeat is enough:
- Board sends a periodic byte (e.g. `0x01`) every ~500ms
- Host reads it; presence = docked, silence/disconnect = undocked

No face state needs to be transmitted. Each side independently runs its own animation loop — they just hand off responsibility.

### macOS app changes (`webview_app.rs`)

- Spawn a background thread that polls for the serial device (e.g. `/dev/cu.usbmodem*`)
- On connect: send a JS event to the WebView (`webview.evaluate_script("window.dispatchEvent(new Event('docked'))")`)
- On disconnect: send `'undocked'` event
- Use `serialport` crate for serial port detection and reading

### WASM / JS changes (`wasm_display.rs` / `index.html`)

- Listen for `docked` / `undocked` window events
- `docked`: start the animation loop (lilbud is alive in the app)
- `undocked`: stop the animation loop, render the closed door static image

### Embedded firmware changes (`main.rs` embedded)

- Initialize USB CDC alongside the display loop
- While USB is connected and host is active: freeze display on static "docked" image, keep USB heartbeat alive
- On USB disconnect: resume normal animation loop

---

## Static images

Two new images needed:

| Image | Shown on | Description |
|-------|----------|-------------|
| "docked" / away | Board (when plugged in) | lilbud's room — empty, door open, something indicating he stepped out |
| "closed door" | macOS app (when unplugged) | the door to lilbud's world, closed — he's not here right now |

These could be pre-encoded as RGB565 byte arrays (embedded) and as PNG/SVG assets embedded in the WASM bundle.

---

## Open questions

- What do the two static images actually look like?
- Should the app detect the board by USB vendor/product ID, or just by matching the serial port name pattern?
- Should the board's "docked" image be a full static framebuffer baked into the firmware, or drawn with embedded-graphics?
- Timeout: if the host stops sending/receiving but the USB cable is still physically connected, how long before we consider it undocked?
