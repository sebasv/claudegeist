#!/usr/bin/env bash
# One-line installer for claudegeist (macOS, universal binary).
#   curl -fsSL https://raw.githubusercontent.com/sebasv/claudegeist/main/install.sh | bash
set -euo pipefail

REPO="sebasv/claudegeist"
BIN="claudegeist"
DEST="/usr/local/bin"

[ "$(uname -s)" = "Darwin" ] || { echo "This installer is macOS-only." >&2; exit 1; }

# Resolve the latest release asset via the API. The /releases/latest/download/
# redirect is cached aggressively by GitHub and lags new releases; the API is
# authoritative and unauthenticated for public repos.
url="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep -o "https://github.com/$REPO/releases/download/[^\"]*/$BIN" | head -1)"
[ -n "$url" ] || { echo "Could not find a $BIN release asset for $REPO." >&2; exit 1; }
tmp="$(mktemp)"
echo "Downloading $BIN..."
curl -fSL "$url" -o "$tmp"
chmod +x "$tmp"
# curl downloads aren't quarantined, but strip the attr defensively in case.
xattr -d com.apple.quarantine "$tmp" 2>/dev/null || true

if [ -w "$DEST" ]; then
  mv "$tmp" "$DEST/$BIN"
else
  echo "Installing to $DEST (needs sudo)..."
  sudo mv "$tmp" "$DEST/$BIN"
fi

echo "Installed $BIN to $DEST/$BIN"
echo
echo "Run it:"
echo "  $BIN        # scans ~/.claude/projects, then opens the word cloud at http://127.0.0.1:8080"
