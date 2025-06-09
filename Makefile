# Makefile for rdfless distribution

BINARY_NAME=rdfless
TARGET_DIR=target/release

# Linux
LINUX_DIST_NAME=rdfless-linux-x86_64
LINUX_BINARY_PATH=$(TARGET_DIR)/$(BINARY_NAME)
LINUX_DIST_PATH=$(TARGET_DIR)/$(LINUX_DIST_NAME)

# Windows
WIN_DIST_NAME=rdfless-windows-x86_64.exe
WIN_BINARY_PATH=$(TARGET_DIR)/$(BINARY_NAME).exe
WIN_DIST_PATH=$(TARGET_DIR)/$(WIN_DIST_NAME)

# MacOS
MAC_DIST_NAME=rdfless-macos-x86_64
MAC_BINARY_PATH=$(TARGET_DIR)/$(BINARY_NAME)
MAC_DIST_PATH=$(TARGET_DIR)/$(MAC_DIST_NAME)

.PHONY: all fmt clippy build dist-linux dist-windows dist-macos dist

all: dist

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

build:
	cargo build --release

dist-linux: fmt clippy build
	@upx --best --lzma $(LINUX_BINARY_PATH) || echo "UPX not found or failed, skipping compression."
	cp $(LINUX_BINARY_PATH) $(LINUX_DIST_PATH)
	@echo "Distribution binary: $(LINUX_DIST_PATH)"

dist-windows: fmt clippy
	cargo build --release --target x86_64-pc-windows-gnu
	@upx.exe --best --lzma target/x86_64-pc-windows-gnu/release/$(BINARY_NAME).exe || echo "UPX not found or failed, skipping compression."
	cp target/x86_64-pc-windows-gnu/release/$(BINARY_NAME).exe $(WIN_DIST_PATH)
	@echo "Distribution binary: $(WIN_DIST_PATH)"

dist-macos: fmt clippy
	cargo build --release --target x86_64-apple-darwin
	@upx --best --lzma target/x86_64-apple-darwin/release/$(BINARY_NAME) || echo "UPX not found or failed, skipping compression."
	cp target/x86_64-apple-darwin/release/$(BINARY_NAME) $(MAC_DIST_PATH)
	@echo "Distribution binary: $(MAC_DIST_PATH)"

dist: dist-linux dist-windows dist-macos
