#!/usr/bin/env bash
# urGlance Preview Trigger
# Usage: urglance-preview /path/to/file
# Sends a file to the urGlance overlay for popup preview

set -euo pipefail

FILE="${1:-}"
if [ -z "$FILE" ]; then
    echo "Usage: $0 <file-path>"
    exit 1
fi

FILE="$(realpath "$FILE" 2>/dev/null || readlink -f "$FILE" 2>/dev/null || echo "$FILE")"

if [ ! -e "$FILE" ]; then
    echo "Error: file not found: $FILE"
    exit 1
fi

OVERLAY_SOCKET="/tmp/urglance-overlay.sock"
DAEMON_URL="http://127.0.0.1:8080"

# Try overlay socket first (floating popup)
if [ -S "$OVERLAY_SOCKET" ]; then
    echo "$FILE" | nc -U -w 1 "$OVERLAY_SOCKET" 2>/dev/null && echo "Preview sent to overlay: $FILE" && exit 0
fi

# Fallback: call daemon API directly
ENCODED=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$FILE'))" 2>/dev/null || echo "$FILE")
if curl -sf "http://127.0.0.1:8080/api/preview?path=${ENCODED}" > /dev/null 2>&1; then
    echo "Preview sent to daemon: $FILE"
else
    echo "Error: urGlance daemon not running. Start it with: urglance"
    exit 1
fi
