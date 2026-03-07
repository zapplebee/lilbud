# ── Host target detection ─────────────────────────────────────────────────────

OS   := $(shell uname -s)
ARCH := $(shell uname -m)

ifeq ($(OS)-$(ARCH), Darwin-arm64)
  HOST_TRIPLE := aarch64-apple-darwin
else ifeq ($(OS)-$(ARCH), Darwin-x86_64)
  HOST_TRIPLE := x86_64-apple-darwin
else ifeq ($(OS)-$(ARCH), Linux-x86_64)
  HOST_TRIPLE := x86_64-unknown-linux-gnu
else ifeq ($(OS)-$(ARCH), Linux-aarch64)
  HOST_TRIPLE := aarch64-unknown-linux-gnu
endif

# ── Paths ─────────────────────────────────────────────────────────────────────

FACE_FILE     ?= faces.ndjson
ELF           := target/thumbv6m-none-eabi/release/lilbud
UF2           := target/lilbud.uf2
WASM_BIN      := target/wasm32-unknown-unknown/release/lilbud.wasm
MOUNT         ?= /Volumes/RPI-RP2

# ── Phony targets ─────────────────────────────────────────────────────────────

.PHONY: help desktop wasm wasm-bindgen serve embedded uf2 flash faces clean

help:
	@echo ""
	@echo "  make faces          Regenerate src/face_data.rs from \$$FACE_FILE (default: faces.ndjson)"
	@echo "  make desktop        Build and run the SDL2 desktop preview"
	@echo "  make wasm           Build WASM binary + JS glue into pkg/"
	@echo "  make serve          Build WASM then serve on http://localhost:8080"
	@echo "  make embedded       Build release ELF for RP2040"
	@echo "  make uf2            Build ELF and convert to UF2"
	@echo "  make flash          Build UF2 and copy to \$$MOUNT (default: /Volumes/RPI-RP2)"
	@echo "  make clean          Remove build artifacts"
	@echo ""
	@echo "  Override FACE_FILE to use a different ndjson source:"
	@echo "    make faces FACE_FILE=~/lilbudmaker/lilbudz.ndjson"
	@echo ""

# ── Face data codegen ─────────────────────────────────────────────────────────

faces:
	@echo "Using face file: $(FACE_FILE)"
	@test -f $(FACE_FILE) || (echo "Error: $(FACE_FILE) not found." && exit 1)
	python3 gen_faces.py $(FACE_FILE) > src/face_data.rs.tmp
	mv src/face_data.rs.tmp src/face_data.rs
	@echo "Updated src/face_data.rs"

# ── Desktop ───────────────────────────────────────────────────────────────────

desktop:
	cargo run --target $(HOST_TRIPLE) --no-default-features --features desktop

# ── WASM ──────────────────────────────────────────────────────────────────────

$(WASM_BIN):
	cargo build --release --target wasm32-unknown-unknown \
		--no-default-features --features wasm

wasm: $(WASM_BIN)
	wasm-bindgen $(WASM_BIN) --out-dir pkg --target web

serve: wasm
	python3 -m http.server 8080

# ── Embedded ──────────────────────────────────────────────────────────────────

$(ELF):
	cargo build --release --no-default-features --features embedded

embedded: $(ELF)

$(UF2): $(ELF)
	elf2uf2-rs $(ELF) $(UF2)

uf2: $(UF2)

flash: $(UF2)
	@test -d $(MOUNT) || (echo "Error: $(MOUNT) not mounted. Put the board in BOOT mode first." && exit 1)
	cp $(UF2) $(MOUNT)/

# ── Clean ─────────────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -rf pkg
