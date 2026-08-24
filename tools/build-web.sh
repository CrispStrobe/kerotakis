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
cp "$ROOT/web/index.html" "$ROOT/web/kerotakis.mjs" \
   "$ROOT/web/manifest.webmanifest" "$ROOT/web/icon.svg" "$OUT/"
cp "$ROOT/vendor/iphreeqc/database/wateq4f.dat" \
   "$ROOT/vendor/iphreeqc/database/minteq.v4.dat" \
   "$ROOT/vendor/iphreeqc/database/pitzer.dat" "$OUT/db/"

echo "== pre-warmed lessons and R1 acceptance states"
cargo run -p kerotakis-cli -- prewarm "$ROOT"/lessons/*.lab \
    -o "$OUT/results.postcard"

# (The service worker is stamped at the end of this script, once the app's
# content-hashed assets exist and can enter its precache list.)

if command -v emcc >/dev/null 2>&1; then
    echo "== the aqueous engine (Emscripten)"
    # Key the CMake build dir to this checkout's path: a shared target dir
    # serving several worktrees otherwise holds a cache generated from one
    # source path and refuses every other (found the hard way, three
    # parallel sessions in).
    WASM_DIR="$TARGET_DIR/iphreeqc-wasm-$(printf %s "$ROOT" | shasum | cut -c1-8)"
    "$ROOT/tools/build-iphreeqc-wasm.sh" "$WASM_DIR" >/dev/null
    cp "$WASM_DIR/iphreeqc.mjs" "$WASM_DIR/iphreeqc.wasm" "$OUT/"
else
    echo "== no emcc: the page will run from shipped results only"
fi

# The bench app (web/app, GUI-010): built when node is available, served
# under app/ beside the console page. Its engine base points one level up,
# where this script already put the wasm modules and databases — the app
# and the console share one engine payload.
if command -v npm >/dev/null 2>&1; then
    echo "== the bench app (web/app)"
    (cd "$ROOT/web/app" \
        && npm ci --no-audit --no-fund >/dev/null \
        && node tools/licence-lint.mjs \
        && npx vitest run --silent >/dev/null \
        && VITE_ENGINE_BASE="../" npm run build >/dev/null)
    cp -r "$ROOT/web/app/dist" "$OUT/app"
else
    echo "== no npm: skipping the bench app; the console page still works"
fi

# The service worker's cache is versioned by commit, so every deploy
# retires the previous cache and an unchanged deploy keeps it warm. Stamped
# last so the precache list includes the app's hashed assets.
STAMP="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || date +%s)"
APP_ASSETS="$(cd "$OUT" && find app -type f 2>/dev/null | sort | sed 's|.*|"&"|' | paste -sd, -)"
sed -e "s|__KERO_CACHE__|$STAMP|" \
    -e "s|\"__KERO_APP_ASSETS__\"|${APP_ASSETS:-\"__no_app__\"}|" \
    "$ROOT/web/sw.js" > "$OUT/sw.js"

# Vercel serves the same payload; the config only tightens caching for the
# app's content-hashed assets.
cp "$ROOT/web/vercel.json" "$OUT/vercel.json"

# Pages serves what it is given; without this it would try to run the
# output through Jekyll and drop the underscore-prefixed files.
touch "$OUT/.nojekyll"

du -sh "$OUT"
ls -lh "$OUT" | sed 's/^/   /'
echo "OK: serve $OUT"
