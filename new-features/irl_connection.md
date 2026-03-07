# Feature: IRL Connection

Two lilbud boards physically connected via a simple pin interface can recognize each other and interact — sharing emotional state, reacting to one another, or synchronizing.

---

## Physical Connection

### Pins

The display uses SPI1 (GPIO8–12, GPIO25). The RP2040 has plenty of free GPIO. The natural choice for simple two-board communication is **UART0**:

| Signal | GPIO | Notes |
|--------|------|-------|
| TX     | 0    | This board's transmit out |
| RX     | 1    | This board's receive in |
| GND    | GND  | Common ground — required |

Three wires total. Full-duplex. No clock line needed. Both boards run identical firmware.

---

### The crossover problem

This is the central case-design question. UART is not symmetric: **TX must connect to RX, not to TX**. A board's transmitter has to feed the other board's receiver. GND connects to GND. So the correct wiring is:

```
Board A           Board B
  TX  ──────────►  RX
  GND ────────────  GND
  RX  ◄──────────  TX
```

That's **A_TX → B_RX, GND → GND, A_RX ← B_TX** — a crossover. If you wire it straight (TX-TX, GND-GND, RX-RX), both transmitters are shouting at each other's transmitters and neither receiver hears anything. The crossover is non-negotiable for UART.

The question you're really asking: **where does the crossover live — in the cable, or in the case geometry?**

---

### Option 1: Crossover in the cable, identical connectors

Both boards have the same connector wired the same way — say, left-to-right: `[TX | GND | RX]`. The cable itself swaps pin 1 and pin 3:

```
Board A connector        cable          Board B connector
  pin 1 (TX) ──────────────────────►  pin 3 (RX)
  pin 2 (GND) ────────────────────── pin 2 (GND)
  pin 3 (RX) ◄──────────────────────  pin 1 (TX)
```

Both shells are identical and interchangeable. The cable is not. A 3-pin JST-style connector would work here but the cable must be made custom (or clearly labeled). A standard cable won't work; a naive cable swap breaks the connection silently.

---

### Option 2: Crossover in the geometry — face-to-face snap

If two boards **face each other** to connect (screen facing screen, or back facing back), the physical mirror does the work. When you hold Board B up to Board A in the opposite orientation, left and right swap. If both connectors are `[TX | GND | RX]` left-to-right on their own face:

```
Board A face              Board B face (flipped, facing A)
  [TX | GND | RX]    →    [RX | GND | TX]  ← as seen from A's perspective
```

When the connectors meet, TX-A lands on RX-B, GND on GND, RX-A on TX-B. The crossover is automatic. The cable is just a straight 1-1-2-2-3-3 connection, or pogo pins that contact directly with no cable at all.

This is the most elegant solution for a case with pogo pins: **the act of pressing two boards together face-to-face performs the crossover**. The connector order is not arbitrary — it must be `[TX | GND | RX]` with GND in the center so that the mirror lands correctly.

GND in the middle also has a practical safety benefit: even if the boards are only partially engaged (partially pressed together, slightly misaligned), GND connects first and last, so neither TX/RX line is floating across an unmatched pin during contact.

---

### Option 3: Crossover in the geometry — side-by-side, mirrored shells

If boards sit side-by-side (both face forward, like two pins on a lanyard next to each other), there is no physical flip. The connectors on adjacent edges are in the same orientation. To get a crossover with a straight cable, the connector **pin order must differ between the two facing edges**:

```
Board A (right edge connector)    Board B (left edge connector)
  [TX | GND | RX]          →      [RX | GND | TX]
```

But here's the problem: both boards run the same firmware and the same PCB. You can't have one board's right edge wired TX-GND-RX and the other board's left edge wired RX-GND-TX unless you route them differently in the shell — for example, pad extensions inside the shell that cross the wires before they hit the connector pads.

This works but adds complexity to the shell's internal routing. Face-to-face (Option 2) achieves the same result for free from geometry alone.

---

### Option 4: Crossover in software — auto-detect and swap

UART on the RP2040 is implemented through the PIO state machines as much as the hardware UART0 peripheral. In principle, you could assign GPIO 0 and 1 as either TX or RX based on detecting which role each board takes. One simple approach:

- On boot, both boards listen on GPIO 1 (RX) and transmit nothing.
- Simultaneously, both pulse GPIO 0 briefly.
- If GPIO 1 goes high (received the other board's pulse), this board is in "A" role; if it doesn't, "B" role.
- Roles determine which GPIO is TX and which is RX.

With straight wiring (pin-to-pin, same orientation, no crossover), one board's GPIO 0 connects to the other board's GPIO 0. If board A transmits on GPIO 0 and board B receives on GPIO 0 — that works, even though it's not the conventional UART0 pinout. You're using GPIO 0 as RX on one board.

This is doable on the RP2040 because UART0 can be mapped to alternate pins, and PIO UART has no fixed pin assignment at all. The tradeoff: it requires a role-negotiation phase at boot and slightly more complex firmware. The hardware is a dead-simple straight cable.

---

### Summary: which to choose

| Approach | Cable | Connector | Shell complexity | Firmware complexity |
|----------|-------|-----------|-----------------|---------------------|
| Crossover cable | Custom | Identical | Low | None |
| Face-to-face snap | Straight / none | Identical | Low — geometry does it | None |
| Side-by-side mirrored | Straight | Same PCB, different routing | Medium | None |
| Software role swap | Straight | Identical | Low | Low |

**Face-to-face snapping (Option 2) is the best choice for pogo pins**: identical shells, no custom cable, no firmware logic, and the physical interaction — pressing two boards together — is the most satisfying and legible gesture. Place the connector on one flat face of the shell (e.g. the back), in the center, with pin order `[TX | GND | RX]` so the mirror lands correctly.

For a cabled variant, a 3.5mm TRRS jack falls into Option 1 (crossover in the cable) or could be made to work with a standard cable using Option 4 (software role swap, detecting which end is which via the initial pulse exchange).

---

### Connector

Since the boards live in a 3D printed shell, the connector needs to be accessible from the outside. Given the face-to-face geometry above:

| Option | Pros | Cons |
|--------|------|-------|
| **Pogo pins, back face, center** | No cable, satisfying press-together, crossover from geometry | Shell must align precisely back-to-back |
| **Pogo pins, edge** | Boards sit side by side | Need cable or mirrored shell routing |
| **3-pin JST or similar** | Any cable works (with crossover cable made once) | Visible cable; cable must be custom |
| **3.5mm TRRS jack** | Universal cable availability; could combine with audio section | Standard cable is straight; need crossover cable or software swap |
| **Magnetic pogo connector** | Self-aligning, elegant | Expensive; harder to source in 3-pin form |

The back-face pogo pin approach is best matched to the face-to-face snap interaction. The shell design needs:
- A recessed flat region on the back face with 3 pogo pin positions, spaced far enough apart that partial contact (1 of 3) doesn't cause electrical trouble.
- GND in the center position.
- The two boards held flush and parallel when connected — a clip, slot, or magnetic catch keeps them together.

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

## Short-Range Radio

Adding a small radio module removes the need for any physical connector between boards entirely. Two lilbuds in the same room could find each other automatically and begin interacting — no cable, no alignment, no deliberate connection gesture required. This changes the interaction from "plug in to connect" to "walk near each other to connect."

### Why radio fits better than wired

The wired UART approach requires a physical connector on the shell — a design constraint that limits form factor and introduces wear. Radio sidesteps all of that. It also opens up a range of new interaction possibilities: proximity awareness, multiple boards in a room influencing each other, gradual connection as you walk closer.

The RP2040 has no built-in radio, but it has plenty of free GPIO and an SPI0 peripheral untouched by the display. An add-on module soldered to the back of the board (or integrated into a revised shell) is the natural path.

---

### Candidate radios

#### nRF24L01+ — recommended

The nRF24L01+ is a 2.4 GHz packet radio IC from Nordic Semiconductor. It is the default choice for simple embedded peer-to-peer: cheap (~$1–2 as a bare module), tiny (15×19mm with chip antenna), low power, and SPI-native.

| Property | Value |
|----------|-------|
| Frequency | 2.4 GHz (ISM band) |
| Interface | SPI + 2 GPIO (CE, IRQ) |
| Range | ~10–100m (adjustable output power) |
| Standby current | 900 nA |
| TX current | 7–11 mA (lowest power setting) |
| Data rate | 250 kbps / 1 Mbps / 2 Mbps |
| Packet size | Up to 32 bytes |
| Rust support | `nrf24l01` crate (embedded-hal SPI) |

**Pinout** — using SPI0 (clear of the display's SPI1):

| Signal | GPIO |
|--------|------|
| SCK    | 2    |
| MOSI   | 3    |
| MISO   | 4    |
| CS     | 5    |
| CE     | 6    |
| IRQ    | 7    |

Six pins. GND shared with the board.

**Addressing**: nRF24L01+ uses 5-byte pipe addresses. All lilbud boards could listen on a shared broadcast address (e.g. `0x4C494C4255` — "LILBU") with no pairing needed. Any board in range receives any other board's broadcast. For more targeted pairing, addresses could be derived from a handshake.

**Discovery**: Because the radio is always listening, discovery is passive — a board simply starts receiving packets when another lilbud comes into range. No button press, no pairing flow. Presence is inferred from packet arrival, absence from timeout.

---

#### BLE module (nRF52840 or similar)

Bluetooth Low Energy would give lilbud a standard radio that smartphones and computers can also talk to — opening a path to phone-based interaction later. The nRF52840 is a popular BLE SoC that can be added as a UART or SPI companion chip.

| Property | Value |
|----------|-------|
| Frequency | 2.4 GHz (BLE) |
| Interface | UART or SPI |
| Range | ~10–30m |
| Standby current | ~1–3 µA |
| TX current | ~4–8 mA |
| Discovery | BLE advertising/scanning — standardized |
| Rust support | `nrf-softdevice` (complex); AT-command modules are simpler |

The appeal: BLE is a real standard. A lilbud could eventually pair with a phone, appear in Bluetooth settings, or integrate with the dock feature from `dock.md` without a USB cable. The cost is complexity — BLE stack, pairing ceremony, GATT services. For board-to-board toy communication, it's probably overkill. Worth keeping in mind if phone integration is ever on the roadmap.

---

#### NFC — tap to connect

NFC (Near Field Communication) has a range of roughly 0–10cm, which maps perfectly to a deliberate physical gesture: touch two pins together. The PN532 is the most common NFC controller and has SPI/I2C interfaces and embedded-hal Rust support.

| Property | Value |
|----------|-------|
| Frequency | 13.56 MHz |
| Interface | SPI or I2C |
| Range | 0–10cm |
| Current | ~150 mA active (significant) |
| Rust support | `pn532` crate |

NFC alone isn't suitable for ongoing connection — the range is too short to maintain while boards are worn normally. But as an **initial handshake** — touch to pair, then the nRF24L01+ takes over for ongoing communication — it's a compelling experience. Tap → recognize → walk apart → stay emotionally linked.

---

#### IR (infrared) — line of sight only

Infrared is the most primitive option: a TX LED and RX phototransistor, ~38kHz carrier, range of ~1–5m but strictly line of sight. No interference with WiFi, no RF regulations, works globally.

The charm of IR for lilbud: it would require two boards to be **facing each other** to communicate. The character on the front of the board would have to "look at" the other board. This is a weird and interesting constraint — it makes the interaction feel more intentional, almost like eye contact.

Practically, IR is sensitive to ambient light, has limited range, and is one-directional per LED/sensor pair (you'd need two pairs for full-duplex). It is the quirkiest option and probably not the right default, but worth noting as a distinct interaction mechanic.

---

### Comparison

| Radio | Range | Connector | Power | Complexity | Multi-board | Notes |
|-------|-------|-----------|-------|------------|-------------|-------|
| nRF24L01+ | 10–100m | None | Very low | Low | Yes | Best all-round choice |
| BLE module | 10–30m | None | Low | High | Yes | Future-proof for phone integration |
| NFC (PN532) | 0–10cm | None | Medium | Medium | No | Tap-to-pair gesture only |
| IR | 1–5m | None | Very low | Low | Tricky | Line-of-sight, facing required |
| UART (GPIO) | N/A (wired) | Physical | None | Very low | Chain only | No radio needed |

---

### Physical integration

The nRF24L01+ module in its smallest form (chip antenna, ~15×19mm) would sit on the back of the RP2040-LCD-1.28 board. The revised 3D printed shell would need a thin cavity in the rear face to accommodate it, or the module could be mounted flush and the shell deepened slightly.

The antenna should not be enclosed in metal. A plastic or resin shell is fine. Keep the antenna region free of copper pours if a custom PCB revision is made.

For a future custom PCB: Nordic makes the nRF24L01+ in a QFN-20 package (4×4mm). At that point, the radio is simply part of the board design rather than a module, and the shell constraint disappears.

---

### Multi-board behavior

Radio opens up the possibility of more than two boards interacting. Possible modes:

- **Broadcast mood**: every board emits its current face index; nearby boards are influenced by the aggregate emotional field of whoever's in range
- **Nearest neighbor**: pair with the board sending the strongest signal (RSSI-based); ignore others
- **Room sync**: all boards in range gradually converge on the same face (emergent consensus)
- **Unique pairs**: boards remember past connections and treat returning acquaintances differently

The nRF24L01+ supports up to 6 receive pipes simultaneously, so a board can listen to up to 6 specific peers at once without any protocol complexity.

---

## Audio: Modem-Style Tone Communication

Two boards could talk to each other using sound — encoding data as audio tones the way a dial-up modem does. This is called **AFSK (Audio Frequency-Shift Keying)**: transmit a high tone for a `1` bit, a low tone for a `0` bit, and demodulate on the receiving end by detecting which tone is present.

The appeal for lilbud specifically is that the communication is *audible*. When two boards connect, they make a little chirp or squawk — a handshake you can hear. The sound isn't just a side effect; it's part of the character.

---

### How it would work on the RP2040

The RP2040 has no dedicated audio hardware, but it has everything needed:

#### Transmit — PWM tone generation

Any GPIO pin can produce PWM at audio frequencies. At 125 MHz system clock, generating a 1200 Hz tone means setting a PWM period of ~104,000 ticks — trivial. A simple RC low-pass filter (one resistor, one capacitor) turns the square wave into something close enough to a sine for tone detection. Connect the filtered output to the tip of a 3.5mm jack or to a small piezo/speaker.

#### Receive — ADC sampling

GPIO 26–29 on the RP2040 are ADC-capable (12-bit, up to ~500 ksps). A small analog microphone or the ring of a 3.5mm jack wired to an ADC pin gives audio input. At 8 kHz sample rate the ADC has plenty of bandwidth for the 1–3 kHz tone range used by modems.

#### Tone detection — Goertzel algorithm

FFT is overkill. The **Goertzel algorithm** detects the energy at a single specific frequency with a handful of multiplies per sample. Detecting two target frequencies (e.g. 1200 Hz and 2200 Hz) requires two Goertzel filters running in parallel — trivially cheap on a Cortex-M0+ at 125 MHz. If energy at 2200 Hz exceeds a threshold → `1` bit; energy at 1200 Hz → `0` bit.

```
Goertzel for frequency f at sample rate Fs:
  k   = round(N × f / Fs)
  ω   = 2π × k / N
  coeff = 2 × cos(ω)
  per sample: s = x[n] + coeff × s1 − s2
  power at end of N samples: s1² + s2² − coeff × s1 × s2
```

Run over a window of ~64–128 samples, evaluated ~60–120 times per second. More than fast enough for 300–1200 baud.

---

### Modulation parameters

Two frequency pairs that are well-separated and easy to generate/detect at audio rates:

| Standard | Mark (1) | Space (0) | Baud | Notes |
|----------|----------|-----------|------|-------|
| Bell 103 originate | 1270 Hz | 1070 Hz | 300 | Classic US modem, 1962 |
| Bell 103 answer | 2225 Hz | 2025 Hz | 300 | Paired with originate end |
| AFSK (APRS/packet) | 2200 Hz | 1200 Hz | 1200 | Common in amateur radio |
| Custom (simpler) | 2000 Hz | 1000 Hz | 300–600 | Easy round numbers, 2:1 ratio |

For lilbud's data rate needs — a 3-byte packet at 300 baud takes 80ms — anything in this table works. The custom 2000/1000 Hz pair has the advantage of a clean 2:1 ratio: detecting an octave jump is extremely robust. Even a cheap ADC and sloppy RC filter can reliably distinguish them.

At 300 baud, 3-byte packets (magic + face_idx + checksum) transmitted 10× per second use less than 1% of the available bandwidth. Baud rate is not a constraint here at all.

---

### Connection medium

#### Wired — 3.5mm audio jack

A TRRS jack on the shell edge is the natural fit. Any standard audio cable connects two boards. The wiring:

| TRRS pin | Signal |
|----------|--------|
| Tip | TX audio out (PWM filtered) |
| Ring 1 | RX audio in (to ADC) |
| Ring 2 | (unused, or second RX channel) |
| Sleeve | GND |

This is exactly the same shell cutout as a headphone jack — a completely standard form factor. Cables are universally available. Length up to a meter or two is fine at these frequencies. The connector is self-documenting: everyone knows what a headphone jack is.

A fun detail: the existing note in this document about using a "3.5mm TRRS jack — UART on tip/ring, GND on sleeve" was proposing UART voltage levels over audio wiring. AFSK instead uses the audio path as designed — frequencies within the normal audio band, compatible with passive cables and no level-shifting concerns.

#### Acoustic — open air (acoustic coupler)

The boards could communicate without any cable at all, purely through sound. One board's speaker/piezo broadcasts tones; the other's microphone picks them up. This is how the original acoustic modem couplers worked in the 1960s before direct-connect modems.

At close range (5–30 cm), a small piezo buzzer and the RP2040 ADC on a decent analog mic can reliably exchange 300 baud. The interaction model changes completely: instead of plugging in a cable, you hold two boards up to each other. They chirp at each other, negotiate, and connect.

The limitations are real: ambient noise, distance sensitivity, directionality. But for a device that lives on a person and interacts with another person's device, the range and noise constraints map well to the interaction. You have to be close and intentional. That's not a bug.

---

### The sound itself

Unlike every other connection method in this document, AFSK makes audible noise. That's a design affordance, not a side effect.

A lilbud handshake could sound like:
- A short chirp pair (one tone from each board acknowledging the other)
- A brief modem-style squall (the actual data exchange, rendered at audio speed)
- Silence once connected (data exchange continues but below audible threshold, or at ultrasonic frequencies)

The handshake sound can be a designed piece of the character. A happy connection tone vs. a confused or rejected tone. The board's face and its sound are the same output channel.

---

### Boards with native audio

The current Waveshare RP2040-LCD-1.28 has no audio hardware, but Waveshare makes other round LCD boards in the same form factor with audio built in:

#### ESP32-S3 1.28inch Round LCD (~$16–22)

Same 240×240 round display, same size, but on an ESP32-S3 instead of RP2040.

| Property | Value |
|----------|-------|
| MCU | ESP32-S3 (dual-core Xtensa, 240 MHz) |
| Display | 240×240 GC9A01A, same as current board |
| Connectivity | WiFi 6, Bluetooth 5 |
| Audio | I2S peripheral — DAC/ADC modules attach directly |
| GPIO | More free pins than RP2040 |
| Rust support | `esp-hal`, actively developed |

With I2S, you can attach a small MEMS microphone (e.g. INMP441) and an I2S DAC (e.g. MAX98357A for a tiny speaker) without any analog circuitry. The audio path is digital end-to-end.

WiFi and Bluetooth also open up wireless AFSK-over-UDP (yes, that's absurd, but it's possible) or just switching to BLE for the same "proximity finds peers" behavior the nRF24L01+ would provide.

#### ESP32-S3 1.85inch Round LCD (~$26–37)

Larger display (360×360), ESP32-S3, listed by Waveshare as supporting "AI speech." Same I2S audio capability.

---

### RP2040 audio without extra hardware

If staying on the current board:

- **PWM out**: GPIO → 1 kΩ resistor → 10 nF capacitor → 3.5mm tip. Done. Audible tones at the cost of one GPIO and two passive components.
- **ADC in**: GPIO 26 → to the ring of the 3.5mm jack (with a 100 nF blocking cap). The RP2040 ADC input range is 0–3.3V; audio signal from a cable is typically biased to ~1.65V and well within that range.
- **Piezo**: A piezo element attached directly to a GPIO (with a series resistor) produces tones without any filtering. Rough sound, but sufficient for tone detection at the other end.
- **Acoustic detection**: An electret mic module (sub-$1, includes a preamp) on an ADC pin would work for open-air coupling.

No extra crates needed beyond what the RP2040 HAL already provides for PWM and ADC.

---

### Comparison to other approaches

| Method | Cable? | Audible? | Range | Complexity | Board changes |
|--------|--------|----------|-------|------------|---------------|
| UART (pogo pins) | Yes | No | N/A (wired) | Very low | Shell only |
| AFSK wired (3.5mm) | Yes | Optional | N/A (wired) | Low | Shell + RC filter |
| AFSK acoustic | No | Yes | 5–30 cm | Medium | Mic + speaker |
| nRF24L01+ | No | No | 10–100m | Low | Module + shell cavity |
| BLE | No | No | 10–30m | High | Module |
| NFC tap + nRF24 | Physical tap | No | Tap + 100m | Medium | Two modules |

AFSK wired is the most interesting middle ground: the simplicity of a physical cable (no RF, no pairing, no protocol discovery) with the charm of audible character. A 3.5mm jack is also the most socially legible connector — everyone has cables for it.

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
- For AFSK: what should the handshake sound like? Is the audio purely functional or is it designed as character expression?
- Is the ESP32-S3 1.28inch board a viable successor/alternative to the RP2040 board? Same display, same size, native audio, WiFi+BT.
- For acoustic coupling: what's the minimum mic/speaker quality needed for reliable 300 baud at 10cm? Would a piezo on each side work?
- Should wired AFSK use a standard 3.5mm TRRS jack (universally available cable) or a purpose-built connector?
