#!/usr/bin/env bash
# Build the browser demo: both wasm modules, the databases, and the page.
#
# Produces a directory that can be served statically — GitHub Pages, or
# `python3 -m http.server` for a local look. Requires wasm-bindgen; the
# Emscripten SDK is optional, and its absence is not an error: the page then
# runs from shipped results and says so, which is the honest degradation
# rather than a broken build.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Ask cargo where it puts things: CARGO_TARGET_DIR and .cargo/config.toml
# both move it, and a hardcoded ./target is wrong on any machine that has.
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
    | python3 -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])')"
OUT="${1:-$TARGET_DIR/web}"

rm -rf "$OUT"
mkdir -p "$OUT/db"

echo "== the bench (wasm32-unknown-unknown)"
cargo build -p kerotakis-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir "$OUT" \
    "$TARGET_DIR/wasm32-unknown-unknown/release/kerotakis_wasm.wasm"

echo "== the page"
cp "$ROOT/web/index.html" "$ROOT/web/kerotakis.mjs" "$OUT/"
cp "$ROOT/vendor/iphreeqc/database/wateq4f.dat" \
   "$ROOT/vendor/iphreeqc/database/minteq.v4.dat" \
   "$ROOT/vendor/iphreeqc/database/pitzer.dat" "$OUT/db/"

if command -v emcc >/dev/null 2>&1; then
    echo "== the aqueous engine (Emscripten)"
    "$ROOT/tools/build-iphreeqc-wasm.sh" "$TARGET_DIR/iphreeqc-wasm" >/dev/null
    cp "$TARGET_DIR/iphreeqc-wasm/iphreeqc.mjs" \
       "$TARGET_DIR/iphreeqc-wasm/iphreeqc.wasm" "$OUT/"
else
    echo "== no emcc: the page will run from shipped results only"
fi

# Pages serves what it is given; without this it would try to run the
# output through Jekyll and drop the underscore-prefixed files.
touch "$OUT/.nojekyll"

du -sh "$OUT"
ls -lh "$OUT" | sed 's/^/   /'
echo "OK: serve $OUT"
