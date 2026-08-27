#!/usr/bin/env bash
# Every icon the shipped apps need, from the masters `tools/gen-icons.py`
# draws. Run this, commit the result; nothing here happens at build time.
#
# `tauri icon` does most of the expansion, but it works from ONE source and
# two of its outputs are then wrong for the App Store, in opposite ways:
#
#   icon.icns    it derives this from the full-bleed master, so the Dock
#                gets a hard-edged square. macOS applies no mask of its
#                own; the rounded shape has to BE the artwork. Rebuilt
#                below from the squircle master.
#   ios/*.png    Apple rejects an alpha channel in an iOS app icon, and
#                the 1024 marketing icon most visibly of all. Flattened
#                below.
#
# Needs: rsvg-convert (brew install librsvg), Pillow, and the Tauri CLI
# from web/app's devDependencies.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ICONS="$ROOT/web/app/src-tauri/icons"

echo "== masters and the PWA payload"
python3 "$ROOT/tools/gen-icons.py"

echo
echo "== platform expansion (tauri icon)"
SRC="$(mktemp -d)/app-icon.png"
cp "$ICONS/icon.png" "$SRC"
(cd "$ROOT/web/app" && npx tauri icon "$SRC" -o src-tauri/icons >/dev/null)
rm -rf "$(dirname "$SRC")"

# `tauri icon` writes its own `icon.png` over ours: the same picture, a
# different encoding. Harmless in itself, but it means the committed master
# no longer matches what gen-icons.py draws, and `--check` rightly fails.
# Redraw it from the source of truth.
python3 "$ROOT/tools/gen-icons.py" >/dev/null

echo "== icon.icns, rebuilt from the squircle master"
SET="$(mktemp -d)/Kerotakis.iconset"
mkdir -p "$SET"
for spec in "16 16x16@1x" "32 16x16@2x" "32 32x32@1x" "64 32x32@2x" \
            "128 128x128@1x" "256 128x128@2x" "256 256x256@1x" \
            "512 256x256@2x" "512 512x512@1x" "1024 512x512@2x"; do
    set -- $spec
    rsvg-convert -w "$1" -h "$1" -o "$SET/icon_$2.png" "$ICONS/master-macos.svg"
done
iconutil -c icns "$SET" -o "$ICONS/icon.icns"
rm -rf "$(dirname "$SET")"
echo "   $(du -h "$ICONS/icon.icns" | cut -f1) icon.icns"

echo "== iOS icons flattened to RGB (Apple rejects alpha)"
python3 - "$ICONS/ios" <<'PY'
import pathlib, sys
from PIL import Image

flattened = 0
for p in sorted(pathlib.Path(sys.argv[1]).glob("*.png")):
    with Image.open(p) as im:
        if im.mode == "RGB":
            continue
        # The master is opaque everywhere, so this only drops the channel;
        # compositing onto the app's own ground keeps it true even if a
        # future master gains a soft edge.
        ground = Image.new("RGB", im.size, (0x14, 0x12, 0x0F))
        ground.paste(im, mask=im.split()[3] if im.mode == "RGBA" else None)
        ground.save(p)
        flattened += 1
print(f"   {flattened} iOS icons flattened")
PY

echo
echo "OK"
