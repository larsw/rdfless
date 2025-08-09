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
import tempfile
import textwrap

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
SAMPLES = ROOT / "samples"
SCRIPT = ROOT / "scripts" / "terminal_screenshot.py"
BIN = ROOT / "target" / "release" / "rdfless"

IMAGES = [
    (ASSETS / "sample-ttl-compact.png", f"{BIN} --no-pager --no-auto-theme {SAMPLES / 'sample.ttl'}"),
    (ASSETS / "sample-ttl-expanded.png", f"{BIN} --no-pager --no-auto-theme --expand {SAMPLES / 'sample.ttl'}"),
    (ASSETS / "sample-trig-compact.png", f"{BIN} --no-pager --no-auto-theme {SAMPLES / 'sample.trig'}"),
    (ASSETS / "sample-trig-expanded.png", f"{BIN} --no-pager --no-auto-theme --expand {SAMPLES / 'sample.trig'}"),
    # N-Triples and N-Quads don't have compact/expanded variants; one shot each
    (ASSETS / "sample-nt.png", f"{BIN} --no-pager --no-auto-theme {SAMPLES / 'sample.nt'}"),
    (ASSETS / "sample-nq.png", f"{BIN} --no-pager --no-auto-theme {SAMPLES / 'sample.nq'}"),
]

# Deterministic terminal look
FONT = os.environ.get("RDFLESS_SS_FONT", "DejaVu Sans Mono 12")
FG = os.environ.get("RDFLESS_SS_FG", "#D0D0D0")
BG = os.environ.get("RDFLESS_SS_BG", "#111111")


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


def run_one(output: Path, cmd: str, env: dict[str, str]):
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
        "--font",
        FONT,
        "--fg",
        FG,
        "--bg",
        BG,
    ]
    print(f"Generating {output}...")
    subprocess.check_call(args, cwd=str(ROOT), env=env)


def main():
    ensure_built()
    ensure_gi()
    # Create a temporary HOME with a deterministic rdfless config
    with tempfile.TemporaryDirectory() as tmp_home:
        cfg_dir = Path(tmp_home) / ".local" / "rdfless"
        cfg_dir.mkdir(parents=True, exist_ok=True)
        cfg = textwrap.dedent(
            """
            [theme]
            auto_detect = false

            [colors]
            subject = "#80b3ff"
            predicate = "#9be28d"
            object = "#f0f0f0"
            literal = "#ff8080"
            prefix = "#ffd75f"
            base = "#ffd75f"
            graph = "#ffd75f"
            """
        ).strip()
        (cfg_dir / "config.toml").write_text(cfg)

        env = os.environ.copy()
        env["HOME"] = tmp_home
        for out, cmd in IMAGES:
            run_one(out, cmd, env)
    print("All screenshots generated")


if __name__ == "__main__":
    main()
