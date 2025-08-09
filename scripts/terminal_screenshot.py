#!/usr/bin/env python3

import argparse
import os
import sys
import time

try:
    import gi
    gi.require_version('Gtk', '3.0')
    gi.require_version('Vte', '2.91')
    from gi.repository import Gtk, Vte, GLib, Gdk, Pango
except Exception as e:
    sys.stderr.write("This script requires GTK 3 and VTE GI bindings.\n")
    sys.stderr.write("Install: sudo apt-get install -y python3-gi gir1.2-gtk-3.0 gir1.2-vte-2.91\n")
    raise


# Paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)


def resolve_output_path(path: str) -> str:
    if os.path.isabs(path):
        return path
    return os.path.join(PROJECT_ROOT, path)


def parse_hex_rgba(value: str) -> Gdk.RGBA:
    rgba = Gdk.RGBA()
    if value.startswith('#') and (len(value) in (4, 7)):
        # Expand #RGB to #RRGGBB
        if len(value) == 4:
            value = '#' + ''.join([c*2 for c in value[1:]])
        r = int(value[1:3], 16) / 255.0
        g = int(value[3:5], 16) / 255.0
        b = int(value[5:7], 16) / 255.0
        rgba.red = r
        rgba.green = g
        rgba.blue = b
        rgba.alpha = 1.0
        return rgba
    # Fallback parse (accepts named colors too)
    if not rgba.parse(value):
        # default to black/white
        rgba.parse("#000000")
    return rgba


class TerminalScreenshotApp(Gtk.Window):
    def __init__(self, *, cwd: str, command: list[str], output_path: str, width: int, height: int, delay: float, font: str | None, fg: str | None, bg: str | None):
        super().__init__(title="rdfless screenshot")
        self.set_default_size(width, height)

        self._output_path = output_path

        self.term = Vte.Terminal()
        # Configure font
        if font:
            try:
                desc = Pango.FontDescription(font)
                self.term.set_font(desc)
            except Exception as e:
                print(f"Warning: failed to set font '{font}': {e}")
        # Configure colors
        fg_rgba = parse_hex_rgba(fg) if fg else None
        bg_rgba = parse_hex_rgba(bg) if bg else None
        try:
            if fg_rgba or bg_rgba:
                # set_colors(foreground, background, palette)
                self.term.set_colors(fg_rgba, bg_rgba, [])
        except Exception as e:
            print(f"Warning: failed to set colors: {e}")
        self.add(self.term)

        # Start the command
        try:
            self.term.spawn_sync(
                Vte.PtyFlags.DEFAULT,
                cwd,
                command,
                [],
                GLib.SpawnFlags.DEFAULT,
                None, None,
                None,
            )
        except Exception as e:
            print(f"Failed to spawn command: {e}")
            Gtk.main_quit()
            return

        self.show_all()

        # Schedule screenshot after delay
        ms = int(delay * 1000)
        GLib.timeout_add(ms, self.take_screenshot)

    def take_screenshot(self):
        win = self.get_window()
        if not win:
            print("No GdkWindow found")
            Gtk.main_quit()
            return False

        width, height = self.get_size()
        screenshot = Gdk.pixbuf_get_from_window(win, 0, 0, width, height)
        if screenshot:
            out_dir = os.path.dirname(self._output_path)
            if out_dir:
                os.makedirs(out_dir, exist_ok=True)
            screenshot.savev(self._output_path, "png", [], [])
            print(f"Screenshot saved: {self._output_path}")
        else:
            print("Failed to take screenshot")

        Gtk.main_quit()
        return False


def main():
    parser = argparse.ArgumentParser(description="Capture a VTE terminal screenshot of a command's output")
    parser.add_argument("--output", required=True, help="Output PNG path (relative to project root or absolute)")
    parser.add_argument("--args", required=True, help="Shell command to run inside the terminal")
    parser.add_argument("--cwd", default=PROJECT_ROOT, help="Working directory for the command")
    parser.add_argument("--width", type=int, default=1000, help="Window width")
    parser.add_argument("--height", type=int, default=520, help="Window height")
    parser.add_argument("--delay", type=float, default=2.0, help="Seconds to wait before taking screenshot")
    parser.add_argument("--font", default=None, help="Pango font description, e.g., 'DejaVu Sans Mono 12'")
    parser.add_argument("--fg", default=None, help="Terminal foreground color (e.g., #d0d0d0)")
    parser.add_argument("--bg", default=None, help="Terminal background color (e.g., #111111)")

    args = parser.parse_args()

    out_path = resolve_output_path(args.output)
    cmd = ['bash', '-lc', args.args]

    app = TerminalScreenshotApp(
        cwd=args.cwd,
        command=cmd,
        output_path=out_path,
        width=args.width,
        height=args.height,
        delay=args.delay,
        font=args.font,
        fg=args.fg,
        bg=args.bg,
    )
    Gtk.main()


if __name__ == "__main__":
    main()
