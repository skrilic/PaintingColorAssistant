#!/bin/sh
# Installs PaletteCraft for the current user, with a menu entry and icon.
# Run from the unpacked release folder:  ./install.sh      Remove again:  ./install.sh --uninstall
set -e
cd "$(dirname "$0")"
BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS="$DATA/applications"
ICONS="$DATA/icons/hicolor/256x256/apps"
NAME=palettecraft

if [ "$1" = "--uninstall" ]; then
    rm -f "$BIN/palettecraft" "$APPS/$NAME.desktop" "$ICONS/$NAME.png"
    echo "Removed. Your palettes stay in ${XDG_CONFIG_HOME:-$HOME/.config}/PaletteCraft."
    exit 0
fi

mkdir -p "$BIN" "$APPS" "$ICONS"
install -m 755 palettecraft "$BIN/palettecraft"
install -m 644 "$NAME.png" "$ICONS/$NAME.png"
sed "s|^Exec=.*|Exec=$BIN/palettecraft %f|" "$NAME.desktop" > "$APPS/$NAME.desktop"
command -v update-desktop-database >/dev/null && update-desktop-database "$APPS" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q "$DATA/icons/hicolor" 2>/dev/null || true
echo "Installed. Find \"PaletteCraft\" in your applications menu."
