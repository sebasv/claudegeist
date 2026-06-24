#!/usr/bin/env bash
# One-line installer for claudegeist (macOS, universal binary).
#   curl -fsSL https://raw.githubusercontent.com/sebasv/claudegeist/main/install.sh | bash
set -euo pipefail

REPO="sebasv/claudegeist"
BIN="claudegeist"
DEST="/usr/local/bin"

[ "$(uname -s)" = "Darwin" ] || { echo "This installer is macOS-only." >&2; exit 1; }

url="https://github.com/$REPO/releases/latest/download/$BIN"
tmp="$(mktemp)"
echo "Downloading $BIN…"
curl -fSL "$url" -o "$tmp"
chmod +x "$tmp"
# curl downloads aren't quarantined, but strip the attr defensively in case.
xattr -d com.apple.quarantine "$tmp" 2>/dev/null || true

if [ -w "$DEST" ]; then
  mv "$tmp" "$DEST/$BIN"
else
  echo "Installing to $DEST (needs sudo)…"
  sudo mv "$tmp" "$DEST/$BIN"
fi

echo "Installed $BIN to $DEST/$BIN"
echo "Run it:  $BIN"
