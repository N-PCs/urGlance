#!/usr/bin/env bash
# urGlance Complete Installer
# Installs the daemon, preview overlay, file manager plugins, and systemd service

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY="$REPO_ROOT/target/release/urglance"

echo "╔══════════════════════════════════════╗"
echo "║      urGlance Full Installer        ║"
echo "╚══════════════════════════════════════╝"
echo

# --- 0. Build if needed ---
if [ ! -f "$BINARY" ]; then
    echo "[0/6] Building urGlance..."
    (cd "$REPO_ROOT" && cargo build --release --offline)
fi

# --- 1. Install daemon binary ---
echo "[1/6] Installing daemon binary..."
sudo cp "$BINARY" /usr/local/bin/urglance
sudo chmod +x /usr/local/bin/urglance
echo "  -> /usr/local/bin/urglance"

# --- 2. Install CLI trigger ---
echo "[2/6] Installing preview trigger..."
sudo cp "$SCRIPT_DIR/urglance-preview.sh" /usr/local/bin/urglance-preview
sudo chmod +x /usr/local/bin/urglance-preview
echo "  -> /usr/local/bin/urglance-preview"

# --- 3. Install GTK overlay ---
echo "[3/6] Installing preview overlay..."
sudo cp "$SCRIPT_DIR/urglance-overlay.py" /usr/local/bin/urglance-overlay
sudo chmod +x /usr/local/bin/urglance-overlay
echo "  -> /usr/local/bin/urglance-overlay"

# --- 4. Install file manager plugins ---
echo "[4/6] Installing file manager plugins..."

# Nautilus
NAUTILUS_EXT_DIR="$HOME/.local/share/nautilus-python/extensions"
mkdir -p "$NAUTILUS_EXT_DIR"
cp "$SCRIPT_DIR/nautilus_extension.py" "$NAUTILUS_EXT_DIR/urglance.py"
echo "  -> Nautilus extension: $NAUTILUS_EXT_DIR/urglance.py"

# Dolphin
DOLPHIN_DIR="$HOME/.local/share/kio/servicemenus"
mkdir -p "$DOLPHIN_DIR"
cp "$SCRIPT_DIR/dolphin_service_menu.desktop" "$DOLPHIN_DIR/urglance-preview.desktop"
echo "  -> Dolphin menu: $DOLPHIN_DIR/urglance-preview.desktop"

# --- 5. Install systemd user service ---
echo "[5/6] Installing systemd user service..."
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"

# Daemon service
cat > "$SERVICE_DIR/urglance.service" << 'SERVICE'
[Unit]
Description=urGlance File Preview Daemon
Documentation=https://github.com/N-PCs/urGlance
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/urglance
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
SERVICE
echo "  -> $SERVICE_DIR/urglance.service"

# Overlay service (auto-starts with the daemon)
cat > "$SERVICE_DIR/urglance-overlay.service" << 'SERVICE'
[Unit]
Description=urGlance Preview Overlay
Documentation=https://github.com/N-PCs/urGlance
PartOf=urglance.service
After=urglance.service

[Service]
Type=simple
ExecStart=/usr/local/bin/urglance-overlay
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
SERVICE
echo "  -> $SERVICE_DIR/urglance-overlay.service"

systemctl --user daemon-reload
echo "  -> systemd reloaded"

# --- 6. Enable and start ---
echo "[6/6] Enabling and starting services..."
systemctl --user enable urglance.service urglance-overlay.service 2>/dev/null || true
systemctl --user restart urglance.service 2>/dev/null || true
echo "  -> Services enabled"

echo
echo "╔══════════════════════════════════════╗"
echo "║      Installation Complete!         ║"
echo "╚══════════════════════════════════════╝"
echo
echo "What's next:"
echo "  1. Restart Nautilus: nautilus -q"
echo "  2. Browse your files — select any file to see its preview popup"
echo "  3. Right-click a file → 'Preview with urGlance'"
echo "  4. Open http://127.0.0.1:8080 for the full dashboard"
echo
echo "Manual start:  urglance"
echo "Service:       systemctl --user start urglance"
echo "Overlay:       systemctl --user start urglance-overlay"
echo
echo "To uninstall:"
echo "  systemctl --user stop urglance.service urglance-overlay.service"
echo "  systemctl --user disable urglance.service urglance-overlay.service"
echo "  sudo rm -f /usr/local/bin/urglance /usr/local/bin/urglance-preview /usr/local/bin/urglance-overlay"
echo "  rm -f ~/.local/share/nautilus-python/extensions/urglance.py"
echo "  rm -f ~/.local/share/kio/servicemenus/urglance-preview.desktop"
echo "  rm -f ~/.config/systemd/user/urglance.service ~/.config/systemd/user/urglance-overlay.service"
