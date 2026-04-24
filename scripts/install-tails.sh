#!/usr/bin/env bash
# Installs SimplePGP permanently on Tails OS using the Dotfiles feature of
# Persistent Storage.
#
# What it does:
#   1. Verifies that Tails Persistent Storage is unlocked and that the Dotfiles
#      feature is enabled.
#   2. Copies the .desktop file and hicolor icons into
#      /live/persistence/TailsData_unlocked/dotfiles/ so that Tails re-creates
#      them under ~/.local/share/ on every boot.
#   3. Rewrites Exec= in the installed .desktop to point at the release binary
#      inside Persistent Storage (default: ~/Persistent/SimplePGP).
#   4. Refreshes the current session's icon cache / desktop database so the
#      entry shows up immediately without a reboot.
#
# Requirements:
#   - You must be on Tails, running as the "amnesia" user.
#   - Persistent Storage must be unlocked at boot.
#   - The "Dotfiles" feature must be enabled in Persistent Storage settings
#     (Applications -> Tails -> Persistent Storage -> Dotfiles).
#   - SimplePGP must already be built (cargo build --release).
#
# Usage:
#   scripts/install-tails.sh
#   scripts/install-tails.sh --binary /home/amnesia/Persistent/SimplePGP/target/release/simplepgp
#   scripts/install-tails.sh --uninstall

set -euo pipefail

APP_ID="org.tailsos.simplepgp"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

DOTFILES_ROOT="/live/persistence/TailsData_unlocked/dotfiles"
APPS_REL=".local/share/applications"
ICONS_REL=".local/share/icons/hicolor"

BINARY_OVERRIDE=""
ACTION="install"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --binary)
            BINARY_OVERRIDE="${2:-}"
            shift 2
            ;;
        --uninstall)
            ACTION="uninstall"
            shift
            ;;
        -h|--help)
            sed -n '2,30p' "$0"
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

log()  { echo ">> $*"; }
die()  { echo "ERROR: $*" >&2; exit 1; }

# --- sanity checks ---------------------------------------------------------

if [[ ! -f /etc/amnesia/version ]] && [[ ! -d /live/persistence ]]; then
    die "This script is for Tails OS only. /etc/amnesia/version and /live/persistence not found."
fi

if [[ ! -d "$DOTFILES_ROOT" ]]; then
    die "Dotfiles feature of Persistent Storage is not enabled.
Enable it via: Applications -> Tails -> Persistent Storage -> Dotfiles,
then reboot and re-run this script."
fi

if [[ "$(id -un)" != "amnesia" ]]; then
    die "Run this as the 'amnesia' user (not as root). The script will sudo where needed."
fi

# --- uninstall -------------------------------------------------------------

if [[ "$ACTION" == "uninstall" ]]; then
    log "Removing SimplePGP entries from Tails Dotfiles"
    sudo rm -f \
        "$DOTFILES_ROOT/$APPS_REL/$APP_ID.desktop" \
        "$DOTFILES_ROOT/$ICONS_REL/scalable/apps/$APP_ID.svg" \
        "$DOTFILES_ROOT/$ICONS_REL/symbolic/apps/$APP_ID-symbolic.svg"

    log "Removing live session entries under \$HOME"
    rm -f \
        "$HOME/$APPS_REL/$APP_ID.desktop" \
        "$HOME/$ICONS_REL/scalable/apps/$APP_ID.svg" \
        "$HOME/$ICONS_REL/symbolic/apps/$APP_ID-symbolic.svg"

    command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database -q "$HOME/$APPS_REL" || true
    command -v gtk-update-icon-cache >/dev/null 2>&1 \
        && gtk-update-icon-cache -f -q "$HOME/$ICONS_REL" || true

    log "Done. SimplePGP will no longer be restored on boot."
    exit 0
fi

# --- resolve binary path ---------------------------------------------------

if [[ -n "$BINARY_OVERRIDE" ]]; then
    RELEASE_BIN="$BINARY_OVERRIDE"
else
    RELEASE_BIN="$PROJECT_ROOT/target/release/simplepgp"
fi

if [[ ! -x "$RELEASE_BIN" ]]; then
    die "Release binary not found or not executable:
    $RELEASE_BIN
Build it first with: cargo build --release  (use --offline on the vendor branch)"
fi

case "$RELEASE_BIN" in
    /home/amnesia/Persistent/*|/live/persistence/TailsData_unlocked/Persistent/*)
        : ;;
    *)
        echo "WARNING: binary is NOT inside Persistent Storage:"
        echo "    $RELEASE_BIN"
        echo "It will disappear on the next reboot and the desktop entry will be broken."
        echo "Press Ctrl-C within 5s to abort, or wait to continue anyway..."
        sleep 5
        ;;
esac

# --- install to dotfiles (persistent) --------------------------------------

log "Installing persistent .desktop entry into: $DOTFILES_ROOT/$APPS_REL"
sudo install -Dm644 \
    "$PROJECT_ROOT/data/$APP_ID.desktop" \
    "$DOTFILES_ROOT/$APPS_REL/$APP_ID.desktop"

log "Rewriting Exec= to: $RELEASE_BIN"
sudo sed -i "s|^Exec=.*|Exec=$RELEASE_BIN|" \
    "$DOTFILES_ROOT/$APPS_REL/$APP_ID.desktop"

log "Installing persistent icons into: $DOTFILES_ROOT/$ICONS_REL"
sudo install -Dm644 \
    "$PROJECT_ROOT/data/icons/hicolor/scalable/apps/$APP_ID.svg" \
    "$DOTFILES_ROOT/$ICONS_REL/scalable/apps/$APP_ID.svg"

sudo install -Dm644 \
    "$PROJECT_ROOT/data/icons/hicolor/symbolic/apps/$APP_ID-symbolic.svg" \
    "$DOTFILES_ROOT/$ICONS_REL/symbolic/apps/$APP_ID-symbolic.svg"

# --- mirror into the current live session so it works without reboot -------

log "Activating entries in the current session"
install -Dm644 \
    "$DOTFILES_ROOT/$APPS_REL/$APP_ID.desktop" \
    "$HOME/$APPS_REL/$APP_ID.desktop"

install -Dm644 \
    "$DOTFILES_ROOT/$ICONS_REL/scalable/apps/$APP_ID.svg" \
    "$HOME/$ICONS_REL/scalable/apps/$APP_ID.svg"

install -Dm644 \
    "$DOTFILES_ROOT/$ICONS_REL/symbolic/apps/$APP_ID-symbolic.svg" \
    "$HOME/$ICONS_REL/symbolic/apps/$APP_ID-symbolic.svg"

command -v update-desktop-database >/dev/null 2>&1 \
    && update-desktop-database -q "$HOME/$APPS_REL" || true
command -v gtk-update-icon-cache >/dev/null 2>&1 \
    && gtk-update-icon-cache -f -q "$HOME/$ICONS_REL" || true

log "Validating .desktop file"
if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate "$HOME/$APPS_REL/$APP_ID.desktop" || true
fi

cat <<EOF

Done. SimplePGP is installed persistently on Tails.

  Desktop entry (persistent) : $DOTFILES_ROOT/$APPS_REL/$APP_ID.desktop
  Desktop entry (live)       : $HOME/$APPS_REL/$APP_ID.desktop
  Binary (must stay put)     : $RELEASE_BIN

On every boot, Tails will symlink these dotfiles into \$HOME and the app will
appear in Activities / the dock. To undo:

  scripts/install-tails.sh --uninstall
EOF
