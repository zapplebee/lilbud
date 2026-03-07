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

## Open Questions

- Which behavior mode (mirror, influence, complementary, greeting)?
- Should the connection be purely peer-to-peer, or could 3+ boards chain together on a shared bus (would need I2C or RS-485 instead of UART)?
- What does the shell cutout look like — edge slot, bottom port, magnetic face?
- Should connection state affect the display visually beyond face selection (e.g. a subtle color shift or border effect when connected)?
- Does the `emo` tag need to be added to `FACE_DATA` for complementary mode, or is face index grouping enough?
