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

# Manual page installation paths
MANDIR ?= /usr/local/share/man
MAN1DIR = $(MANDIR)/man1
MAN5DIR = $(MANDIR)/man5

.PHONY: all fmt clippy build dist-linux dist-windows dist-macos dist install-man uninstall-man

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

# Manual page installation
install-man:
	@echo "Installing manual pages..."
	@mkdir -p $(MAN1DIR) $(MAN5DIR)
	@cp man/rdfless.1 $(MAN1DIR)/
	@cp man/rdfless-config.5 $(MAN5DIR)/
	@chmod 644 $(MAN1DIR)/rdfless.1 $(MAN5DIR)/rdfless-config.5
	@echo "Manual pages installed to $(MANDIR)"
	@echo "Use 'man rdfless' or 'man 5 rdfless-config' to view them"

# Manual page uninstallation
uninstall-man:
	@echo "Removing manual pages..."
	@rm -f $(MAN1DIR)/rdfless.1
	@rm -f $(MAN5DIR)/rdfless-config.5
	@echo "Manual pages removed from $(MANDIR)"

# Install manual pages (requires sudo on most systems)
install: build install-man
	@echo "Installing rdfless binary..."
	@mkdir -p /usr/local/bin
	@cp $(TARGET_DIR)/$(BINARY_NAME) /usr/local/bin/
	@chmod 755 /usr/local/bin/$(BINARY_NAME)
	@echo "rdfless installed to /usr/local/bin/"
	@echo "Installation complete!"

# Uninstall rdfless and manual pages
uninstall: uninstall-man
	@echo "Removing rdfless binary..."
	@rm -f /usr/local/bin/$(BINARY_NAME)
	@echo "rdfless uninstalled"

dist: dist-linux dist-windows dist-macos
