#!/usr/bin/env python3
"""
urGlance Nautilus Extension
Shows a floating preview popup when you select/hover a file in Nautilus.

Installation:
  mkdir -p ~/.local/share/nautilus-python/extensions/
  cp nautilus_extension.py ~/.local/share/nautilus-python/extensions/urglance.py
  Restart Nautilus: nautilus -q

Requires: nautilus-python (python3-nautilus), urGlance daemon + overlay running
"""

import gi
gi.require_version('Nautilus', '4.1')
gi.require_version('Gtk', '3.0')
from gi.repository import Nautilus, Gtk, GLib
import os
import socket
import threading
import urllib.parse
import json
import time

OVERLAY_SOCKET = "/tmp/urglance-overlay.sock"
LOG_FILE = "/tmp/urglance-nautilus.log"


def log(msg):
    try:
        with open(LOG_FILE, 'a') as f:
            f.write(f"[{time.strftime('%H:%M:%S')}] {msg}\n")
    except:
        pass


class UrGlancePreviewProvider(Nautilus.InfoProvider, Nautilus.MenuProvider):
    def __init__(self):
        super().__init__()
        self._debounce_id = None
        self._current_path = None

    def _send_to_overlay(self, path):
        if not os.path.exists(OVERLAY_SOCKET):
            return
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.settimeout(0.5)
            sock.connect(OVERLAY_SOCKET)
            sock.sendall(path.encode() + b'\n')
            sock.close()
            log(f"Sent: {path}")
        except Exception as e:
            log(f"Socket error: {e}")

    def update_file_info(self, file):
        try:
            if file.get_uri_scheme() != 'file':
                return
            if file.is_directory():
                return

            path = urllib.parse.unquote(file.get_uri()[7:])
            log(f"update_file_info: {path}")

            if path == self._current_path:
                return
            self._current_path = path

            if self._debounce_id is not None:
                GLib.source_remove(self._debounce_id)

            self._debounce_id = GLib.timeout_add(300, self._do_send, path)
        except Exception as e:
            log(f"Error in update_file_info: {e}")

    def _do_send(self, path):
        self._debounce_id = None
        self._send_to_overlay(path)
        return False

    def get_file_items(self, files, window):
        if len(files) != 1:
            return
        file = files[0]
        if file.get_uri_scheme() != 'file' or file.is_directory():
            return

        item = Nautilus.MenuItem(
            name="UrGlance::preview",
            label="Preview with urGlance",
            tip="Show instant file preview popup",
            icon="image-x-generic",
        )
        item.connect('activate', self._on_menu_click, file)
        return [item]

    def _on_menu_click(self, menu, file):
        try:
            path = urllib.parse.unquote(file.get_uri()[7:])
            log(f"Menu: {path}")
            self._send_to_overlay(path)
        except Exception as e:
            log(f"Menu error: {e}")
