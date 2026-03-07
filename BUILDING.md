# Building & Flashing lilbud

There are two build targets:

| Target | Use |
|--------|-----|
| **SDL2** (desktop) | Preview and development — runs as a native window on your machine |
| **RP2040** (embedded) | Production — runs on the Waveshare RP2040-LCD-1.28 pin |

---

## SDL2 (Desktop)

### Prerequisites

Install SDL2:

```sh
# macOS
brew install sdl2

# Ubuntu / Debian
sudo apt-get install libsdl2-dev
```

Set the face data path. The face file is `lilbudz.ndjson`, written to whichever directory you run [lil bud maker](https://github.com/zapplebee/lilbudmaker) from:

```sh
export FACE_FILE_PATH=/path/to/lilbudmaker/lilbudz.ndjson
```

### Build & Run

The default Cargo target is now `thumbv6m-none-eabi` (embedded). For desktop you must pass the host target and feature flags explicitly:

```sh
# macOS Apple Silicon
FACE_FILE_PATH=/path/to/lilbudmaker/lilbudz.ndjson \
  cargo run --target aarch64-apple-darwin \
  --no-default-features --features desktop

# macOS Intel
FACE_FILE_PATH=/path/to/lilbudmaker/lilbudz.ndjson \
  cargo run --target x86_64-apple-darwin \
  --no-default-features --features desktop
```

---

## RP2040 (Waveshare RP2040-LCD-1.28)

### Prerequisites

**1. Add the cross-compilation target:**

```sh
rustup target add thumbv6m-none-eabi
```

**2. Install `elf2uf2-rs`** — converts the compiled ELF binary to a UF2 file the board can accept:

```sh
cargo install elf2uf2-rs
```

**3. (Optional) Install `probe-rs`** — for flashing over SWD with a debug probe instead of UF2:

```sh
cargo install probe-rs-tools --locked
```

### Build

`thumbv6m-none-eabi` is the default target, so no `--target` flag needed:

```sh
FACE_FILE_PATH=/path/to/lilbudmaker/lilbudz.ndjson \
  cargo build --release --no-default-features --features embedded
```

The ELF binary will be at:

```
target/thumbv6m-none-eabi/release/lilbud
```

Convert it to UF2:

```sh
elf2uf2-rs target/thumbv6m-none-eabi/release/lilbud lilbud.uf2
```

---

## Flashing

### Method 1: UF2 Bootloader (no extra hardware required)

The RP2040-LCD-1.28 has a built-in USB bootloader. To enter it:

1. **Hold the BOOT button** on the board
2. **Plug in USB** (or press RESET while holding BOOT if already connected)
3. The board mounts as a USB mass storage device named **`RPI-RP2`**
4. **Copy `lilbud.uf2` onto the drive:**

```sh
cp lilbud.uf2 /Volumes/RPI-RP2/
```

The board will reboot automatically and start running lilbud.

> On Linux the drive will appear at `/media/$USER/RPI-RP2` or similar — check `lsblk` after connecting.

### Method 2: probe-rs over SWD (requires a debug probe)

If you have a debug probe (a second Pico running [picoprobe](https://github.com/raspberrypi/picoprobe), or a dedicated probe like a CMSIS-DAP adapter), connect it to the SWD pins on the board and flash directly:

```sh
probe-rs download --chip RP2040 --format elf \
  target/thumbv6m-none-eabi/release/lilbud
```

Or, if you configure `probe-rs` as the Cargo runner in `.cargo/config.toml`:

```sh
cargo run --release --target thumbv6m-none-eabi
```

---

## Verifying the Build

The board has no debug output by default over USB. To see panic messages and `println!` output during development, use a probe-rs runner which will stream RTT output to your terminal:

```sh
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/lilbud
```
