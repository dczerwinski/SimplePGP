#!/usr/bin/env bash
# Removes SimplePGP icons and .desktop entry installed by install-linux.sh.
#
# Usage:
#   scripts/uninstall-linux.sh              # per-user
#   sudo scripts/uninstall-linux.sh --system # system-wide

set -euo pipefail

APP_ID="org.tailsos.simplepgp"

MODE="user"
if [[ "${1:-}" == "--system" ]]; then
    MODE="system"
fi

if [[ "$MODE" == "system" ]]; then
    ICON_DIR="/usr/share/icons/hicolor"
    APPS_DIR="/usr/share/applications"
else
    ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor"
    APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
fi

rm -f "$ICON_DIR/scalable/apps/$APP_ID.svg"
rm -f "$ICON_DIR/symbolic/apps/$APP_ID-symbolic.svg"
rm -f "$APPS_DIR/$APP_ID.desktop"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -q "$ICON_DIR" || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q "$APPS_DIR" || true
fi

echo "SimplePGP desktop integration removed ($MODE)."
