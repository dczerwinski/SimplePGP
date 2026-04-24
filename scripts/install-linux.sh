#!/usr/bin/env bash
# Installs the SimplePGP icon theme entries and .desktop file on Linux so that
# GNOME / Ubuntu Dock / Alt-Tab show the application icon correctly.
#
# Default target is the current user (no root needed). Pass --system to install
# to /usr for all users (requires sudo).
#
# Usage:
#   scripts/install-linux.sh              # per-user (~/.local/share)
#   sudo scripts/install-linux.sh --system # system-wide (/usr/share)

set -euo pipefail

APP_ID="org.tailsos.simplepgp"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

MODE="user"
if [[ "${1:-}" == "--system" ]]; then
    MODE="system"
fi

if [[ "$MODE" == "system" ]]; then
    PREFIX="/usr"
    ICON_DIR="$PREFIX/share/icons/hicolor"
    APPS_DIR="$PREFIX/share/applications"
else
    ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor"
    APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
fi

echo ">> Installing SimplePGP icons to: $ICON_DIR"
install -Dm644 \
    "$PROJECT_ROOT/data/icons/hicolor/scalable/apps/$APP_ID.svg" \
    "$ICON_DIR/scalable/apps/$APP_ID.svg"

install -Dm644 \
    "$PROJECT_ROOT/data/icons/hicolor/symbolic/apps/$APP_ID-symbolic.svg" \
    "$ICON_DIR/symbolic/apps/$APP_ID-symbolic.svg"

echo ">> Installing .desktop file to: $APPS_DIR"
install -Dm644 \
    "$PROJECT_ROOT/data/$APP_ID.desktop" \
    "$APPS_DIR/$APP_ID.desktop"

# Rewrite Exec= to point at the locally built release binary so launching from
# the GNOME dock / activities actually runs the freshly built executable.
RELEASE_BIN="$PROJECT_ROOT/target/release/simplepgp"
if [[ -x "$RELEASE_BIN" ]]; then
    echo ">> Pointing Exec= at: $RELEASE_BIN"
    sed -i "s|^Exec=.*|Exec=$RELEASE_BIN|" "$APPS_DIR/$APP_ID.desktop"
else
    echo ">> NOTE: $RELEASE_BIN not found yet (run 'cargo build --release' first)."
fi

echo ">> Refreshing icon cache and desktop database"
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -q "$ICON_DIR" || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q "$APPS_DIR" || true
fi

echo
echo "Done. If the dock still shows the old icon, run:"
echo "    killall -HUP gnome-shell     # X11"
echo "    # or log out and back in on Wayland"
