#!/usr/bin/env python3

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('Vte', '2.91')

from gi.repository import Gtk, Vte, GLib, Gdk

import os
import time

# Get the directory where this script is located
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
# Change to the project root directory (parent of scripts)
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)

# Your command to run
COMMAND = ['bash', '-c', 'echo "Running your CLI tool..."; ./target/debug/rdfless --no-pager ./doap.ttl ; sleep 2']

class TerminalScreenshotApp(Gtk.Window):
    def __init__(self):
        super().__init__(title="CLI Tool Output")
        self.set_default_size(800, 400)

        self.term = Vte.Terminal()
        self.add(self.term)

        # Start the command
        try:
            self.term.spawn_sync(
                Vte.PtyFlags.DEFAULT,
                PROJECT_ROOT,  # Use project root as working directory
                COMMAND,
                [],
                GLib.SpawnFlags.DEFAULT,
                None, None,
                None
            )
        except Exception as e:
            print(f"Failed to spawn command: {e}")
            Gtk.main_quit()
            return

        self.show_all()

        # Schedule screenshot after delay
        GLib.timeout_add_seconds(3, self.take_screenshot)

    def take_screenshot(self):
        win = self.get_window()
        if not win:
            print("No GdkWindow found")
            Gtk.main_quit()
            return False

        # Capture a screenshot of the window
        width, height = self.get_size()
        screenshot = Gdk.pixbuf_get_from_window(win, 0, 0, width, height)
        if screenshot:
            filename = f"screenshot_{int(time.time())}.png"
            screenshot.savev(filename, "png", [], [])
            print(f"Screenshot saved as {filename}")
        else:
            print("Failed to take screenshot")

        Gtk.main_quit()
        return False

if __name__ == "__main__":
    app = TerminalScreenshotApp()
    Gtk.main()
