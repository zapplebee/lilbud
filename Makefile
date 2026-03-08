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

FACE_FILE   ?= faces.ndjson
ELF         := target/thumbv6m-none-eabi/release/lilbud
UF2         := target/lilbud.uf2
WEBVIEW_BIN := target/$(HOST_TRIPLE)/release/lilbud
APP_BUNDLE  := lilbud.app
DIST_ZIP    := lilbud-mac.zip
MOUNT       ?= /Volumes/RPI-RP2
DOCK_PORT   ?= 3000

# ── Phony targets ─────────────────────────────────────────────────────────────

.PHONY: help webview app dist embedded uf2 flash faces dock dock-mock clean

help:
	@echo ""
	@echo "  make faces          Regenerate src/face_data.rs from \$$FACE_FILE (default: faces.ndjson)"
	@echo "  make webview        Build and run the WebView host (dev)"
	@echo "  make app            Build distributable lilbud.app bundle (macOS)"
	@echo "  make dist           Build lilbud.app and zip it for distribution"
	@echo "  make embedded       Build release ELF for RP2040"
	@echo "  make uf2            Build ELF and convert to UF2"
	@echo "  make flash          Build UF2 and copy to \$$MOUNT (default: /Volumes/RPI-RP2)"
	@echo "  make dock           Start the dock UI dev server (real board mode)"
	@echo "  make dock-mock      Start the dock UI dev server with mock data (no board needed)"
	@echo "  make clean          Remove build artifacts"
	@echo ""
	@echo "  Override FACE_FILE to use a different ndjson source:"
	@echo "    make faces FACE_FILE=~/lilbudmaker/lilbudz.ndjson"
	@echo ""
	@echo "  Override DOCK_PORT to change the dock UI port (default: 3000):"
	@echo "    make dock-mock DOCK_PORT=4000"
	@echo ""

# ── Face data codegen ─────────────────────────────────────────────────────────

faces:
	@echo "Using face file: $(FACE_FILE)"
	@test -f $(FACE_FILE) || (echo "Error: $(FACE_FILE) not found." && exit 1)
	bun run gen_faces.ts $(FACE_FILE)

# ── WebView host ──────────────────────────────────────────────────────────────
# Native OS window (wry) that bridges the board over USB to the dock UI.

webview:
	cargo run --target $(HOST_TRIPLE) --no-default-features --features webview

# ── macOS app bundle ──────────────────────────────────────────────────────────
# Produces a self-contained lilbud.app that anyone on macOS can double-click.
#
# Note: the app is unsigned. First-time openers must right-click → Open,
# or run: xattr -d com.apple.quarantine lilbud.app

$(WEBVIEW_BIN):
	cargo build --release --target $(HOST_TRIPLE) \
		--no-default-features --features webview

app: $(WEBVIEW_BIN)
	rm -rf $(APP_BUNDLE)
	mkdir -p $(APP_BUNDLE)/Contents/MacOS
	cp $(WEBVIEW_BIN) $(APP_BUNDLE)/Contents/MacOS/lilbud
	cp Info.plist $(APP_BUNDLE)/Contents/Info.plist
	@echo "Built $(APP_BUNDLE)"

dist: app
	rm -f $(DIST_ZIP)
	zip -r $(DIST_ZIP) $(APP_BUNDLE)
	@echo "Built $(DIST_ZIP)"

# ── Dock UI ───────────────────────────────────────────────────────────────────
# Companion web app served at http://localhost:$(DOCK_PORT).
# dock      — real mode: expects a live board connection via the webview host
# dock-mock — mock mode: auto-cycles faces and fixture data (no board needed)

dock:
	cd dock-ui && PORT=$(DOCK_PORT) bun run server.ts

dock-mock:
	cd dock-ui && PORT=$(DOCK_PORT) DOCK_MOCK=1 bun run server.ts

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
	rm -rf $(APP_BUNDLE) $(DIST_ZIP) dock-ui/dist
