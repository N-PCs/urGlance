#!/usr/bin/env python3
"""
urGlance Floating Preview Overlay
A frameless, always-on-top GTK3 window that shows file previews.
Listens on a Unix socket for file paths and displays the preview.
"""

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
gi.require_version('GLib', '2.0')
gi.require_version('GdkPixbuf', '2.0')
gi.require_version('Pango', '1.0')
from gi.repository import Gtk, Gdk, GLib, GdkPixbuf, Pango

import json
import os
import socket
import sys
import threading
import urllib.request
import urllib.parse

OVERLAY_SOCKET = "/tmp/urglance-overlay.sock"
DAEMON_URL = "http://127.0.0.1:8080"
WINDOW_WIDTH = 480
WINDOW_HEIGHT = 360

class PreviewOverlay:
    def __init__(self):
        self.window = Gtk.Window.new(Gtk.WindowType.TOPLEVEL)
        self.window.set_title("urGlance Preview")
        self.window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        self.window.set_position(Gtk.WindowPosition.CENTER)
        self.window.set_keep_above(True)
        self.window.set_decorated(False)
        self.window.set_skip_taskbar_hint(True)
        self.window.set_skip_pager_hint(True)
        self.window.set_accept_focus(False)
        self.window.stick()

        # Make window click-through when not hovering over it
        screen = self.window.get_screen()
        visual = screen.get_rgba_visual()
        if visual and screen.is_composited():
            self.window.set_visual(visual)

        # Main container with styling
        self.box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.box.set_name("main-box")
        self.window.add(self.box)

        # Set up CSS
        css = b"""
        #main-box {
            background: rgba(12, 14, 22, 0.95);
            border: 1px solid rgba(139, 92, 246, 0.3);
            border-radius: 12px;
        }
        #header-box {
            background: rgba(139, 92, 246, 0.08);
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
            padding: 8px 12px;
        }
        #file-path {
            color: #f3f4f6;
            font-weight: bold;
            font-size: 12px;
        }
        #file-type {
            color: #a78bfa;
            font-size: 10px;
            font-weight: 600;
        }
        #metadata {
            color: #9ca3af;
            font-size: 10px;
            padding: 2px 12px;
        }
        #snippet-box {
            background: rgba(0, 0, 0, 0.35);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 6px;
            margin: 8px 12px;
            padding: 10px;
        }
        #snippet-text {
            color: #d1d5db;
            font-family: monospace;
            font-size: 11px;
        }
        #close-btn {
            background: rgba(255, 255, 255, 0.06);
            color: #9ca3af;
            border: none;
            border-radius: 4px;
            padding: 2px 8px;
            font-size: 12px;
        }
        #close-btn:hover {
            background: rgba(255, 255, 255, 0.12);
        }
        #image-box {
            margin: 4px 12px;
        }
        """
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(css)
        Gtk.StyleContext.add_provider_for_screen(
            screen, css_provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        # Header
        self.header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        self.header.set_name("header-box")

        self.path_label = Gtk.Label(label="No file")
        self.path_label.set_name("file-path")
        self.path_label.set_halign(Gtk.Align.START)
        self.path_label.set_ellipsize(Pango.EllipsizeMode.START)

        self.close_btn = Gtk.Button.new_with_label("✕")
        self.close_btn.set_name("close-btn")
        self.close_btn.set_relief(Gtk.ReliefStyle.NONE)
        self.close_btn.set_size_request(24, 24)
        self.close_btn.connect("clicked", self.hide_overlay)

        self.header.pack_start(self.path_label, True, True, 0)
        self.header.pack_end(self.close_btn, False, False, 0)
        self.box.pack_start(self.header, False, False, 0)

        # Type label
        self.type_label = Gtk.Label(label="")
        self.type_label.set_name("file-type")
        self.type_label.set_halign(Gtk.Align.START)
        self.type_label.set_margin_start(12)
        self.type_label.set_margin_end(12)
        self.type_label.set_margin_top(4)
        self.box.pack_start(self.type_label, False, False, 0)

        # Metadata
        self.meta_label = Gtk.Label(label="")
        self.meta_label.set_name("metadata")
        self.meta_label.set_halign(Gtk.Align.START)
        self.meta_label.set_ellipsize(Pango.EllipsizeMode.END)
        self.box.pack_start(self.meta_label, False, False, 0)

        # Image preview area
        self.image_scroll = Gtk.ScrolledWindow()
        self.image_scroll.set_name("image-box")
        self.image_scroll.set_min_content_height(160)
        self.image_scroll.set_max_content_height(240)
        self.image_scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.image_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.image_box.set_halign(Gtk.Align.CENTER)
        self.image_box.set_valign(Gtk.Align.CENTER)
        self.image_scroll.add(self.image_box)

        # Content snippet area
        self.snippet_scroll = Gtk.ScrolledWindow()
        self.snippet_scroll.set_name("snippet-box")
        self.snippet_scroll.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.AUTOMATIC)
        self.snippet_label = Gtk.Label(label="")
        self.snippet_label.set_name("snippet-text")
        self.snippet_label.set_halign(Gtk.Align.START)
        self.snippet_label.set_valign(Gtk.Align.START)
        self.snippet_label.set_line_wrap(True)
        self.snippet_scroll.add(self.snippet_label)

        # Pack everything
        self.box.pack_start(self.image_scroll, True, True, 0)
        self.box.pack_start(self.snippet_scroll, True, True, 0)

        # Hide initially
        self.window.hide()

        # Keyboard shortcut: Escape to dismiss
        self.window.connect("key-press-event", self.on_key_press)

        # Auto-hide timer
        self.hide_timer_id = None

    def on_key_press(self, widget, event):
        if event.keyval == Gdk.KEY_Escape:
            self.hide_overlay()
            return True
        return False

    def hide_overlay(self, *args):
        if self.hide_timer_id:
            GLib.source_remove(self.hide_timer_id)
            self.hide_timer_id = None
        self.window.hide()

    def show_preview(self, file_path, preview_data):
        # Update labels
        display_name = os.path.basename(file_path)
        if len(display_name) > 60:
            display_name = display_name[:57] + "..."
        self.path_label.set_text(display_name)
        self.path_label.set_tooltip_text(file_path)
        self.type_label.set_text(f"  {preview_data.get('file_type', 'Unknown')}")

        meta = preview_data.get('metadata_summary', '')
        self.meta_label.set_text(f"  {meta}")

        # Content
        snippet = preview_data.get('content_snippet', '[No content]')
        self.snippet_label.set_text(snippet)

        # Clear previous image
        for child in self.image_box.get_children():
            self.image_box.remove(child)

        has_image = preview_data.get('has_image', False)
        image_b64 = preview_data.get('preview_image_b64', '')

        if has_image and image_b64:
            self.snippet_scroll.hide()
            self.image_scroll.show()
            try:
                # Decode base64 image data
                import base64
                b64_data = image_b64
                if b64_data.startswith('data:image'):
                    b64_data = b64_data.split(',', 1)[1]
                img_bytes = base64.b64decode(b64_data)

                loader = GdkPixbuf.PixbufLoader.new_with_type('jpeg')
                loader.write(img_bytes)
                loader.close()
                pixbuf = loader.get_pixbuf()

                # Scale to fit window
                max_w = WINDOW_WIDTH - 40
                max_h = min(pixbuf.get_height(), 240)
                if pixbuf.get_width() > max_w or pixbuf.get_height() > max_h:
                    scaled = pixbuf.scale_simple(
                        min(max_w, pixbuf.get_width()),
                        min(max_h, pixbuf.get_height()),
                        GdkPixbuf.InterpType.BILINEAR
                    )
                else:
                    scaled = pixbuf

                img = Gtk.Image.new_from_pixbuf(scaled)
                self.image_box.pack_start(img, True, True, 0)
                self.image_box.show_all()
            except Exception as e:
                self.image_scroll.hide()
                self.snippet_scroll.show()
        else:
            self.image_scroll.hide()
            self.snippet_scroll.show()

        self.image_box.show_all()
        self.window.resize(WINDOW_WIDTH, WINDOW_HEIGHT)
        self.window.show_all()
        self.window.present()

        # Auto-hide after 12 seconds
        if self.hide_timer_id:
            GLib.source_remove(self.hide_timer_id)
        self.hide_timer_id = GLib.timeout_add_seconds(12, self.hide_overlay)

    def fetch_and_show(self, file_path):
        try:
            quoted = urllib.parse.quote(file_path, safe='')
            url = f"{DAEMON_URL}/api/preview?path={quoted}"
            with urllib.request.urlopen(url, timeout=2) as resp:
                data = json.loads(resp.read().decode())
                GLib.idle_add(self.show_preview, file_path, data)
        except Exception as e:
            # Fallback: show basic info even if daemon is down
            fallback = {
                'file_type': 'File',
                'content_snippet': f'[Could not reach urGlance daemon]\n{str(e)}\n\nMake sure the service is running on {DAEMON_URL}',
                'metadata_summary': '',
                'has_image': False,
                'preview_image_b64': None,
            }
            GLib.idle_add(self.show_preview, file_path, fallback)

    def handle_ipc(self):
        # Remove stale socket
        try:
            os.unlink(OVERLAY_SOCKET)
        except FileNotFoundError:
            pass

        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.bind(OVERLAY_SOCKET)
        sock.listen(5)
        os.chmod(OVERLAY_SOCKET, 0o666)
        sock.settimeout(1)

        while True:
            try:
                conn, _ = sock.accept()
                data = conn.recv(4096).decode().strip()
                conn.close()
                if data:
                    # Split on newline, take first line (file path)
                    path = data.split('\n')[0].strip()
                    if path:
                        threading.Thread(target=self.fetch_and_show, args=(path,), daemon=True).start()
            except socket.timeout:
                continue
            except Exception:
                continue

    def run(self):
        # Start IPC listener in background
        thread = threading.Thread(target=self.handle_ipc, daemon=True)
        thread.start()

        # Set up a check for the socket thread
        GLib.timeout_add(100, lambda: True)  # keep event loop alive
        Gtk.main()


def main():
    # Check if urGlance daemon is running
    try:
        with urllib.request.urlopen(f"{DAEMON_URL}/api/status", timeout=1) as resp:
            if resp.status != 200:
                print("Warning: urGlance daemon may not be running properly", file=sys.stderr)
    except Exception:
        print("Warning: urGlance daemon is not running. Start it with: urglance", file=sys.stderr)
        print(f"Overlay will attempt to connect to {DAEMON_URL}", file=sys.stderr)

    print(f"urGlance Overlay: Listening on {OVERLAY_SOCKET}")
    print("Select a file in Nautilus/Dolphin to see its preview")
    app = PreviewOverlay()
    app.run()


if __name__ == "__main__":
    main()
