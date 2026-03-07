# Feature: IRL Connection

Two lilbud boards physically connected via a simple pin interface can recognize each other and interact — sharing emotional state, reacting to one another, or synchronizing.

---

## Physical Connection

### Pins

The display uses SPI1 (GPIO8–12, GPIO25). The RP2040 has plenty of free GPIO. The natural choice for simple two-board communication is **UART0**:

| Signal | GPIO | Notes |
|--------|------|-------|
| TX     | 0    | Board A transmit → Board B receive |
| RX     | 1    | Board B transmit → Board A receive |
| GND    | GND  | Common ground — required |

Three wires total. Full-duplex. No clock line needed.

Both boards run the same firmware — each listens on RX and talks on TX. The connection is symmetric.

### Connector

Since the boards live in a 3D printed shell, the connector needs to be accessible from the outside. Options:

| Option | Pros | Cons |
|--------|------|-------|
| **Exposed pogo pins on edge** | No plug/unplug wear, satisfying snap-together | Requires precise shell alignment |
| **3-pin JST or similar** | Cheap, reliable, easy to cable | Visible cable between boards |
| **Magnetic pogo connector** | Elegant, self-aligning | More expensive, harder to source |
| **Direct solder bridge** | Permanent, zero connector cost | Not removable |

Pogo pins or a small edge connector on the shell seem most fitting for a "two friends touching" interaction.

### Detection

No separate "connected" pin needed. If RX receives valid framing, a board is present. If RX is silent for a timeout window, the other board is gone.

---

## Protocol

Keep it minimal. Each board broadcasts its current state periodically:

```
[ 0xLB ][ face_idx: u8 ][ checksum: u8 ]
```

- `0xLB` — magic byte (`0x4C 0x42` or a single sentinel) for framing
- `face_idx` — index into `FACE_DATA` currently being displayed
- `checksum` — XOR of the previous bytes, for noise rejection

Broadcast rate: ~10Hz. At 9600 baud, each 3-byte packet takes ~3ms — negligible.

No request/response. Both boards just emit continuously. Each listens and reacts.

### Connection states

```
SOLO        — no signal on RX → normal solo animation loop
CONNECTED   — valid packets received → peer is present
LOST        — was connected, RX went silent for >500ms → return to SOLO
```

---

## Behavior

When two boards connect, something should happen that feels social — not just a technical handshake. A few design directions:

### Option A: Mirror
Both boards show the same face. When one changes, the other follows within a tick or two. They're synchronized.

- Simple to implement: receiver sets its TARGET to the sender's face index
- Feels like two beings in agreement

### Option B: Call and Response
Each board picks its own face independently but is influenced by what it receives. A happy neighbor nudges you toward happy. A sad neighbor nudges you toward sad. Influence is gradual, not instant.

- Richer emergent behavior
- Could blend `TARGET` toward the received face rather than snapping to it

### Option C: Complementary
Faces pair up: one leads, one responds. The receiver picks a face from the same emotional family but a different expression. Happy → happy-variant. Curious → attentive.

- Requires emotional tagging of face data (add `emo` field to `FACE_DATA`)
- More expressive but needs more face variety

### Option D: Recognition greeting
On initial connection, both boards play a short fixed sequence (e.g. a surprised face → happy face) before settling into normal behavior. A "hello" animation.

- Works well combined with any of the above
- Requires a small sequencer in the animation loop

**Recommended starting point**: Option A (mirror) for simplicity, with Option B (influence) as the follow-up.

---

## Firmware Changes

### `Cargo.toml`
No new crates needed — `rp2040-hal` already supports UART.

### `main.rs` (embedded)
- Initialize UART0 at 9600 baud on GPIO0/GPIO1
- Spawn a soft "task" in the main loop: read incoming packets, write outgoing packets
- Pass received `face_idx` into `ui` state

### `ui.rs` (embedded)
- Add `set_peer_face(idx: usize)` — sets TARGET to the peer's current face
- Add connection state: `SOLO` / `CONNECTED` / `LOST`
- On `LOST`: resume normal random face selection

### `get_faces.rs` (embedded)
- Add `get_face_by_index(idx: usize) -> PointMap` alongside `get_random_face()`

### `face_data.rs` / `gen_faces.py`
- If Option C is pursued: add emotion tag per face so complementary pairs can be found

---

## Shell Design Considerations

- The 3D printed shell needs a cutout or channel for the connector on the edge
- If pogo pins: rails on the shell exterior to guide alignment between two boards
- Ground connection must be reliable — consider a wider/dedicated GND pad
- Cable length for a wired version: 5–10cm is enough to hold two boards side by side or stack them

---

## Could They Use Their USB-C Ports?

The short answer is: not directly, and not symmetrically. Here's why, and what the workarounds look like.

### Why direct USB-C peer-to-peer doesn't work

The RP2040's USB controller is **device-only** — it has no host capability in hardware. USB requires one host and one device; you can't connect two USB devices to each other and have them talk. Additionally, the USB-C pins on the board (D+ and D−) connect to the RP2040's **dedicated USB hardware pins**, which are not GPIO and cannot be repurposed for UART or any other protocol without hardware modification.

So a USB-C to USB-C cable between two boards would leave both sitting there as devices with nothing to host the connection.

### Option 1: PIO USB host (asymmetric)

The RP2040 has PIO (Programmable I/O) state machines that can bit-bang almost any protocol. The [`pio-usb`](https://github.com/sekigon-gonnoc/pio-usb) project implements a USB 1.1 full-speed host entirely in PIO, using two regular GPIO pins for D+ and D−.

This means one board could designate two free GPIO pins as a PIO USB host port, and the other board presents as a USB CDC serial device on its hardware USB port. Connected via USB-C, they could exchange data.

**Tradeoffs:**
- The connection is asymmetric — one board is "host", the other is "device". Same firmware can't run on both.
- PIO USB host consumes two PIO state machines and two GPIO, and requires careful timing.
- Adds the `pio-usb` dependency and significant complexity.
- The USB-C cable needs to correctly orient (a standard cable should work, but OTG detection via CC pins may need a pull resistor on the host side).
- Doable, but it's a meaningful engineering lift.

### Option 2: USB-C as a dumb connector (not recommended)

USB-C is just a physical connector. In theory, you could ignore the USB protocol entirely and route arbitrary signals through the cable's pins. The SBU1 and SBU2 pins (Sideband Use) exist in full-featured USB-C cables and are designed for alternate-mode signaling — they're single-ended 3.3V-compatible lines that could carry UART.

The problem: the Waveshare RP2040-LCD-1.28 almost certainly doesn't route SBU pins to any accessible pad, and the D+/D− lines go to dedicated RP2040 hardware pins that aren't usable as GPIO. You'd need to modify the board or design a custom PCB.

Not worth it when GPIO0/GPIO1 are free and much simpler.

### Option 3: Through a computer (relay)

If both boards are plugged into the same computer (the dock scenario from `dock.md`), the host machine can relay messages between them over their individual USB CDC serial connections. Each board talks to the computer, the computer forwards state between them.

This doesn't require any board-to-board wiring and works with the existing USB ports. The limitation is obvious: a computer has to be in the loop.

### Verdict

| Approach | Symmetric | Hardware changes | Complexity |
|----------|-----------|-----------------|------------|
| GPIO UART (GPIO0/1 + pogo pins) | Yes | Shell only | Low |
| PIO USB host | No | None | High |
| USB-C as dumb connector | Yes | Board mod required | Very high |
| Computer relay | Yes | None | Medium (software) |

**GPIO UART with a physical connector on the shell remains the best path for true board-to-board communication.** The USB-C ports are more useful as the dock interface to a computer (see `dock.md`) than as a peer-to-peer link.

If the shell design ever has room for a second small port (even a 3.5mm TRRS jack — UART on tip/ring, GND on sleeve), that could be a cleaner user experience than exposed pogo pins while still being a simple GPIO UART underneath.

---

## Open Questions

- Which behavior mode (mirror, influence, complementary, greeting)?
- Should the connection be purely peer-to-peer, or could 3+ boards chain together on a shared bus (would need I2C or RS-485 instead of UART)?
- What does the shell cutout look like — edge slot, bottom port, magnetic face?
- Should connection state affect the display visually beyond face selection (e.g. a subtle color shift or border effect when connected)?
- Does the `emo` tag need to be added to `FACE_DATA` for complementary mode, or is face index grouping enough?
