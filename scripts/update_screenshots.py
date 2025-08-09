#!/usr/bin/env python3

"""
Generate README screenshots by running rdfless in a VTE terminal and capturing the window.
Requires: python3-gi, gir1.2-gtk-3.0, gir1.2-vte-2.91

This script orchestrates four images:
- assets/sample-ttl-compact.png
- assets/sample-ttl-expanded.png
- assets/sample-trig-compact.png
- assets/sample-trig-expanded.png
"""

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
SAMPLES = ROOT / "samples"
SCRIPT = ROOT / "scripts" / "terminal_screenshot.py"
BIN = ROOT / "target" / "release" / "rdfless"

IMAGES = [
    (ASSETS / "sample-ttl-compact.png", f"{BIN} --no-pager {SAMPLES / 'sample.ttl'}"),
    (ASSETS / "sample-ttl-expanded.png", f"{BIN} --no-pager --expand {SAMPLES / 'sample.ttl'}"),
    (ASSETS / "sample-trig-compact.png", f"{BIN} --no-pager {SAMPLES / 'sample.trig'}"),
    (ASSETS / "sample-trig-expanded.png", f"{BIN} --no-pager --expand {SAMPLES / 'sample.trig'}"),
]


def ensure_built():
    if not BIN.exists():
        print("Building rdfless (release)...")
        subprocess.check_call(["cargo", "build", "--release"], cwd=str(ROOT))


def ensure_gi():
    # Best-effort check; we don't auto-install
    try:
        import gi  # noqa: F401
    except Exception:
        print(
            "Missing gi (GTK/VTE). Install: sudo apt-get install -y python3-gi gir1.2-gtk-3.0 gir1.2-vte-2.91",
            file=sys.stderr,
        )
        sys.exit(1)


def run_one(output: Path, cmd: str):
    output.parent.mkdir(parents=True, exist_ok=True)
    py = str(SCRIPT)
    args = [
        py,
        "--output",
        str(output),
        "--args",
        cmd,
        "--width",
        "1000",
        "--height",
        "520",
        "--delay",
        "2",
    ]
    print(f"Generating {output}...")
    subprocess.check_call(args, cwd=str(ROOT))


def main():
    ensure_built()
    ensure_gi()
    for out, cmd in IMAGES:
        run_one(out, cmd)
    print("All screenshots generated")


if __name__ == "__main__":
    main()
