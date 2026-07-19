# urGlance

Instant file preview for Linux & Windows. **Select any file in your file manager** and a floating preview popup appears instantly — showing text content, image thumbnails, metadata, and more.

## Features

- **Floating Preview Popup**: Select a file in Nautilus/Dolphin/Thunar/Explorer → a frameless GTK overlay shows its content
- **Image Thumbnails**: Generates JPEG thumbnails up to 800px for PNG, JPEG, GIF, BMP, WebP — displayed directly in the popup
- **Text & Code Preview**: Shows up to 30 lines / 4KB of any text or source file
- **Desktop Integration**: Nautilus extension auto-triggers on file selection, Dolphin/Thunar service menus, Windows Explorer script
- **Web Dashboard**: Built-in file browser at `http://127.0.0.1:8080` as a fallback
- **Background Daemon**: Pure Rust backend, systemd user service, low resource usage

## Quick Start

```bash
cd core-logic
cargo build --release
bash scripts/install.sh    # Installs daemon + overlay + file manager plugins

# Or just run manually:
