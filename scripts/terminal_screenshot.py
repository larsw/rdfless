#!/usr/bin/env python3

import argparse
import os
import sys
import time

try:
    import gi
    gi.require_version('Gtk', '3.0')
    gi.require_version('Vte', '2.91')
    from gi.repository import Gtk, Vte, GLib, Gdk
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


class TerminalScreenshotApp(Gtk.Window):
    def __init__(self, *, cwd: str, command: list[str], output_path: str, width: int, height: int, delay: float):
        super().__init__(title="rdfless screenshot")
        self.set_default_size(width, height)

        self._output_path = output_path

        self.term = Vte.Terminal()
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
    )
    Gtk.main()


if __name__ == "__main__":
    main()
