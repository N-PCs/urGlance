#!/usr/bin/env python3
"""
urGlance File Selection Watcher
Monitors file manager windows in real-time and sends selected file paths
to the urGlance preview overlay via Unix socket.

Works with: Nautilus, Dolphin, Thunar, PCManFM, Caja, Nemo
Detection methods: AT-SPI accessibility events, D-Bus, primary selection
"""

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
from gi.repository import Gtk, Gdk, GLib

import os
import socket
import threading
import time
import urllib.request
import urllib.parse
import json

OVERLAY_SOCKET = "/tmp/urglance-overlay.sock"
POLL_INTERVAL_MS = 300  # 0.3 seconds

# Try to import optional backends
try:
    import pyatspi
    import pyatspi.constants
    HAVE_ATSPI = True
except ImportError:
    HAVE_ATSPI = False

try:
    import dbus
    import dbus.mainloop.glib
    HAVE_DBUS = True
except ImportError:
    HAVE_DBUS = False


class FileSelectionWatcher:
    def __init__(self):
        self.last_path = None
        self.last_active_app = None
        self._nautilus_app = None
        self._running = True

        # For AT-SPI event tracking
        self._atspi_listener = None
        self._focused_accessible = None

        # Debounce
        self._pending_path = None
        self._debounce_timer = None

    def send_to_overlay(self, file_path):
        """Send a file path to the floating preview overlay."""
        if not file_path or not os.path.exists(file_path):
            return
        if file_path == self.last_path:
            return
        self.last_path = file_path

        if not os.path.exists(OVERLAY_SOCKET):
            return
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.settimeout(0.3)
            sock.connect(OVERLAY_SOCKET)
            sock.sendall(file_path.encode() + b'\n')
            sock.close()
            print(f"[Watcher] Sent: {os.path.basename(file_path)}")
        except Exception:
            pass

    def on_selection_changed(self, file_path):
        """Called when file selection changes (debounced)."""
        self._pending_path = file_path
        if self._debounce_timer:
            GLib.source_remove(self._debounce_timer)
        self._debounce_timer = GLib.timeout_add(200, self._do_send)

    def _do_send(self):
        self._debounce_timer = None
        if self._pending_path:
            self.send_to_overlay(self._pending_path)
            self._pending_path = None
        return False

    # ── AT-SPI Backend ─────────────────────────────────────

    def _atspi_event_cb(self, event):
        """AT-SPI event callback for focus changes."""
        try:
            source = event.source
            role = source.get_role_name()
            name = source.name or ''
            app_name = source.getApplication().name.lower() if source.getApplication() else ''

            FILE_MANAGERS = ['nautilus', 'dolphin', 'thunar', 'pcmanfm', 'caja', 'nemo']
            is_fm = any(fm in app_name for fm in FILE_MANAGERS)

            if is_fm and role in ['label', 'list item', 'table cell', 'icon', 'table']:
                # Try to get the file path
                path = self._get_path_from_accessible(source)
                if path:
                    self.on_selection_changed(path)
        except Exception:
            pass

    def _get_path_from_accessible(self, acc):
        """Try to extract a file path from an accessible object."""
        try:
            acc_text = acc.queryText()
            text = acc_text.getText(0, -1)
        except Exception:
            text = acc.name or ''

        # The accessible name might be just a display name.
        # We need the full path. Try to get it from the parent hierarchy.
        if text and not text.startswith('/'):
            # This is a display name, not a path.
            # Try to find the URI via D-Bus or parent info
            pass

        if os.path.exists(text):
            return text

        # For Nautilus, accessible objects can expose the URI via
        # object attributes
        try:
            attrs = acc.getAttributes()
            if attrs:
                for key, val in attrs.items():
                    if key == 'uri':
                        uri = val
                        if uri.startswith('file://'):
                            path = urllib.parse.unquote(uri[7:])
                            if os.path.exists(path):
                                return path
        except Exception:
            pass

        return None

    def start_atspi(self):
        """Start AT-SPI event listener for focus changes."""
        if not HAVE_ATSPI:
            return False

        try:
            pyatspi.Registry.registerEventListener(
                self._atspi_event_cb,
                "focus:"
            )
            # Also listen for selection changes
            pyatspi.Registry.registerEventListener(
                self._atspi_event_cb,
                "object:state-changed:selected"
            )
            print("[Watcher] AT-SPI listener started")
            return True
        except Exception as e:
            print(f"[Watcher] AT-SPI start failed: {e}")
            return False

    # ── Polling Backend (works with all file managers) ──────

    def _detect_active_window_xdg(self):
        """Try to detect active file manager via _NET_ACTIVE_WINDOW (X11) or D-Bus."""

        # Method 1: Check via wmctrl (X11 only)
        if os.environ.get('WAYLAND_DISPLAY'):
            return None

        # Try to use xdotool to get active window class
        # (Falls back gracefully if not available)
        try:
            import subprocess
            result = subprocess.run(
                ['xdotool', 'getactivewindow', 'getwindowpid'],
                capture_output=True, text=True, timeout=1
            )
            if result.returncode == 0:
                pid = result.stdout.strip()
                # Get process name from PID
                try:
                    with open(f'/proc/{pid}/comm') as f:
                        comm = f.read().strip()
                    return comm.lower()
                except Exception:
                    pass
        except Exception:
            pass

        # Method 2: Check via xprop
        try:
            import subprocess
            result = subprocess.run(
                ['xprop', '-root', '_NET_ACTIVE_WINDOW'],
                capture_output=True, text=True, timeout=1
            )
            if 'window id' in result.stdout:
                return True  # We're on X11, can use xdotool/get selection
        except Exception:
            pass

        return None

    def _get_selection_via_dbus(self):
        """Try to get selected file via D-Bus from Nautilus/Dolphin."""
        if not HAVE_DBUS:
            return None

        try:
            bus = dbus.SessionBus()
            # Method 1: Try Nautilus via window properties
            # Check if there's an active Nautilus window
            for name in bus.list_names():
                if 'nautilus' in name.lower() and 'Window' in name:
                    # Try to access the window's properties
                    pass

            # Method 2: Try xclip to get PRIMARY selection
            try:
                import subprocess
                result = subprocess.run(
                    ['xclip', '-o', '-selection', 'primary'],
                    capture_output=True, text=True, timeout=1
                )
                path = result.stdout.strip()
                if path and os.path.exists(path):
                    return path
            except Exception:
                pass

        except Exception:
            pass
        return None

    def _get_selection_via_clipboard(self):
        """Try to get file path from PRIMARY selection."""
        TOOLS = ['xclip', 'xsel']
        for tool in TOOLS:
            try:
                import subprocess
                args = [tool, '-o', '-selection', 'primary'] if tool == 'xclip' else [tool, '-o', '-p']
                result = subprocess.run(args, capture_output=True, text=True, timeout=1)
                path = result.stdout.strip()
                if path and os.path.exists(path):
                    return path
            except Exception:
                continue
        return None

    def poll_active_selection(self):
        """Polling callback - checks active file manager selection."""
        if not self._running:
            return False

        path = None

        # Try D-Bus first (most reliable for Nautilus)
        path = self._get_selection_via_dbus()
        if path:
            self.on_selection_changed(path)
            return True

        # Try clipboard PRIMARY selection (works in many file managers)
        path = self._get_selection_via_clipboard()
        if path:
            self.on_selection_changed(path)
            return True

        return True  # Keep polling

    # ── Main ────────────────────────────────────────────────

    def run(self):
        print("[Watcher] urGlance File Selection Watcher")

        # Start AT-SPI listener (best for Nautilus on GNOME)
        atspi_ok = self.start_atspi()
        if atspi_ok:
            print("[Watcher] AT-SPI focus tracking active")
        else:
            print("[Watcher] AT-SPI not available, using polling")

        # Start polling as fallback (catches all file managers)
        GLib.timeout_add(POLL_INTERVAL_MS, self.poll_active_selection)
        print("[Watcher] Polling active every {}ms".format(POLL_INTERVAL_MS))

        print("[Watcher] Ready — select any file in your file manager")
        Gtk.main()


def main():
    # Check overlay socket
    if not os.path.exists(OVERLAY_SOCKET):
        print(f"[Watcher] Warning: overlay socket {OVERLAY_SOCKET} not found")
        print("[Watcher] Start urglance-overlay.py first")

    watcher = FileSelectionWatcher()
    try:
        watcher.run()
    except KeyboardInterrupt:
        print("\n[Watcher] Stopped")


if __name__ == '__main__':
    main()
