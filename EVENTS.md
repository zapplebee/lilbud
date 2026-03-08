# lilbud Event Protocol

Communication between the device (RP2040 firmware + WebView host) and the dock UI.

## Design Principles

- **Board is the source of truth.** All persistent storage lives on the device (littlefs flash).
- **Web UI stores no state.** It renders whatever the last sync contained. On disconnect, it goes blank.
- **Full sync on every mutation.** When the device processes any event from the web, it responds with a complete `SYNC` payload.
- **Events are typed JSON.** Every message has a `type` field. Payloads are flat where possible.
- **Think Redux reducer.** Web dispatches actions → board reduces them → board emits new state.

---

## Events: Web → Device (Actions)

### `SET_IDENTITY`
Update the device's stored name and bio.

```json
{ "type": "SET_IDENTITY", "name": "string", "bio": "string" }
```

### `SET_STATUS`
Update the device's broadcast status text.

```json
{ "type": "SET_STATUS", "text": "string" }
```

### `DISMISS`
Remove a collected status from another device.

```json
{ "type": "DISMISS", "from": "string" }
```

`from` is the name field of the collected entry to delete.

### `DO_ACTION`
Trigger a behavior on the device.

```json
{ "type": "DO_ACTION", "kind": "feed" | "play" | "sleep" }
```

### `REQUEST_SYNC`
Ask the device for a full state snapshot. Sent on initial connection.

```json
{ "type": "REQUEST_SYNC" }
```

---

## Events: Device → Web (State)

### `SYNC`
Full device state. Sent in response to `REQUEST_SYNC` and after the device processes any action event.

```json
{
  "type": "SYNC",
  "mode": "awake" | "sleeping" | "playing" | "eating",
  "identity": {
    "name": "string",
    "bio": "string"
  },
  "status": "string",
  "collected": [
    { "from": "string", "text": "string" }
  ],
  "face": {
    "current": [36],
    "target": [36]
  }
}
```

`face.current` is the live interpolated position. `face.target` is where the animation is heading. The web UI can use these to resume interpolation without a visible jump.

### `FRAME`
Live face animation data. Sent at ~30fps while the device is awake and docked.

```json
{ "type": "FRAME", "points": [36] }
```

Points are 18 (x, y) pairs in the same order as the face coordinate map. The web UI runs its own interpolation loop; `FRAME` carries the device's authoritative interpolated position each tick.

---

## Sync Protocol

```
Web                          Device
 |                              |
 |-- REQUEST_SYNC ------------> |
 |                              |  read identity, status, collected from flash
 |                              |  read current face animation state
 | <----------- SYNC ---------- |
 |                              |
 |  (render state from SYNC)    |
 |                              |
 | <------- FRAME (30fps) ----- |
 | <------- FRAME (30fps) ----- |
 | <------- FRAME (30fps) ----- |
 |                              |
 |-- SET_STATUS "hello" ------> |
 |                              |  write status to flash
 | <----------- SYNC ---------- |
 |                              |
 |-- DISMISS "alice" ---------> |
 |                              |  delete collected entry from flash
 | <----------- SYNC ---------- |
 |                              |
 |  (disconnect)                |
 |  (web clears to blank state) |
```

---

## TypeScript Types

```typescript
// Web → Device
type DeviceAction =
  | { type: 'REQUEST_SYNC' }
  | { type: 'SET_IDENTITY'; name: string; bio: string }
  | { type: 'SET_STATUS'; text: string }
  | { type: 'DISMISS'; from: string }
  | { type: 'DO_ACTION'; kind: 'feed' | 'play' | 'sleep' };

// Device → Web
type DeviceMode = 'awake' | 'sleeping' | 'playing' | 'eating';

type CollectedStatus = { from: string; text: string };

type DeviceState = {
  mode: DeviceMode;
  identity: { name: string; bio: string };
  status: string;
  collected: CollectedStatus[];
  face: { current: number[]; target: number[] };
};

type DeviceEvent =
  | ({ type: 'SYNC' } & DeviceState)
  | { type: 'FRAME'; points: number[] };
```

---

## Notes

- `FRAME` events do not trigger a re-render of non-face UI — only face canvas updates.
- If no `FRAME` arrives for >1500ms, the web UI should treat the device as disconnected.
- The device never sends partial state; every `SYNC` is the complete picture.
- `collected` entries are ordered by recency (newest first).
- Face `points` are always exactly 36 values (18 x/y pairs). An empty or malformed array should be ignored.
