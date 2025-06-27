# Justfile for rdfless distribution

# Variables
binary_name := "rdfless"
target_dir := "target/release"

# Linux
linux_dist_name := "rdfless-linux-x86_64"
linux_binary_path := target_dir / binary_name
linux_dist_path := target_dir / linux_dist_name

# Windows
win_dist_name := "rdfless-windows-x86_64.exe"
win_binary_path := target_dir / (binary_name + ".exe")
win_dist_path := target_dir / win_dist_name

# MacOS
mac_dist_name := "rdfless-macos-x86_64"
mac_binary_path := target_dir / binary_name
mac_dist_path := target_dir / mac_dist_name

# Manual page installation paths
mandir := env_var_or_default("MANDIR", "/usr/local/share/man")
man1dir := mandir / "man1"
man5dir := mandir / "man5"

# Default recipe - equivalent to "make all"
default: dist

# Format code
fmt *args="":
    cargo fmt --all {{args}}

# Run clippy linter
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Build release binary
build:
    cargo build --release

# Build Linux distribution
dist-linux: fmt clippy build
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v upx >/dev/null 2>&1; then
        upx --best --lzma {{linux_binary_path}} || echo "UPX failed, skipping compression."
    else
        echo "UPX not found, skipping compression."
    fi
    cp {{linux_binary_path}} {{linux_dist_path}}
    echo "Distribution binary: {{linux_dist_path}}"

# Build Windows distribution
dist-windows: fmt clippy
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release --target x86_64-pc-windows-gnu
    win_target_path="target/x86_64-pc-windows-gnu/release/{{binary_name}}.exe"
    if command -v upx >/dev/null 2>&1; then
        upx --best --lzma "$win_target_path" || echo "UPX failed, skipping compression."
    else
        echo "UPX not found, skipping compression."
    fi
    cp "$win_target_path" {{win_dist_path}}
    echo "Distribution binary: {{win_dist_path}}"

# Build MacOS distribution
dist-macos: fmt clippy
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release --target x86_64-apple-darwin
    mac_target_path="target/x86_64-apple-darwin/release/{{binary_name}}"
    if command -v upx >/dev/null 2>&1; then
        upx --best --lzma "$mac_target_path" || echo "UPX failed, skipping compression."
    else
        echo "UPX not found, skipping compression."
    fi
    cp "$mac_target_path" {{mac_dist_path}}
    echo "Distribution binary: {{mac_dist_path}}"

# Install manual pages
install-man:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Installing manual pages..."
    mkdir -p {{man1dir}} {{man5dir}}
    cp man/rdfless.1 {{man1dir}}/
    cp man/rdfless-config.5 {{man5dir}}/
    chmod 644 {{man1dir}}/rdfless.1 {{man5dir}}/rdfless-config.5
    echo "Manual pages installed to {{mandir}}"
    echo "Use 'man rdfless' or 'man 5 rdfless-config' to view them"

# Uninstall manual pages
uninstall-man:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Removing manual pages..."
    rm -f {{man1dir}}/rdfless.1
    rm -f {{man5dir}}/rdfless-config.5
    echo "Manual pages removed from {{mandir}}"

# Install rdfless binary and manual pages (requires sudo on most systems)
install: build install-man
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Installing rdfless binary..."
    mkdir -p /usr/local/bin
    cp {{target_dir}}/{{binary_name}} /usr/local/bin/
    chmod 755 /usr/local/bin/{{binary_name}}
    echo "rdfless installed to /usr/local/bin/"
    echo "Installation complete!"

# Uninstall rdfless and manual pages
uninstall: uninstall-man
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Removing rdfless binary..."
    rm -f /usr/local/bin/{{binary_name}}
    echo "rdfless uninstalled"

# Build all platform distributions
dist: dist-linux dist-windows dist-macos

# Run tests
test:
    cargo test --all-features

# Clean build artifacts
clean:
    cargo clean

# Show available recipes
list:
    @just --list

# Development workflow: format, lint, test, and build
dev: fmt clippy test build

# Check if all tools are available
check-tools:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Checking for required tools..."
    
    if command -v cargo >/dev/null 2>&1; then
        echo "✓ cargo found"
    else
        echo "✗ cargo not found - install Rust"
        exit 1
    fi
    
    if command -v upx >/dev/null 2>&1; then
        echo "✓ upx found (optional - for binary compression)"
    else
        echo "○ upx not found (optional - binary compression will be skipped)"
    fi
    
    echo "Tool check complete!"
