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
# Stamp the build revision into hello's git_rev (option_env in the wasm).
export KEROTAKIS_GIT_REV="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || true)"
cargo build -p kerotakis-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir "$OUT" \
    "$TARGET_DIR/wasm32-unknown-unknown/release/kerotakis_wasm.wasm"

if command -v wasm-opt >/dev/null 2>&1; then
    echo "== wasm-opt -Oz"
    BEFORE=$(stat --printf="%s" "$OUT/kerotakis_wasm_bg.wasm" 2>/dev/null \
             || stat -f%z "$OUT/kerotakis_wasm_bg.wasm")
    wasm-opt -Oz -o "$OUT/kerotakis_wasm_bg.wasm" "$OUT/kerotakis_wasm_bg.wasm"
    AFTER=$(stat --printf="%s" "$OUT/kerotakis_wasm_bg.wasm" 2>/dev/null \
            || stat -f%z "$OUT/kerotakis_wasm_bg.wasm")
    echo "   $BEFORE → $AFTER bytes ($(( (BEFORE - AFTER) * 100 / BEFORE ))% reduction)"
else
    echo "== no wasm-opt: skipping size optimization"
fi

echo "== the page"
cp "$ROOT/web/index.html" "$ROOT/web/privacy.html" \
   "$ROOT/web/privacy.de.html" "$ROOT/web/kerotakis.mjs" \
   "$ROOT/web/manifest.webmanifest" "$ROOT/web/manifest.de.webmanifest" \
   "$ROOT/web/icon.svg" \
   "$ROOT/web/icon-192.png" "$ROOT/web/icon-512.png" \
   "$ROOT/web/icon-maskable-512.png" "$ROOT/web/apple-touch-icon.png" \
   "$ROOT/web/screenshot-wide.png" "$ROOT/web/screenshot-narrow.png" "$OUT/"
cp "$ROOT/vendor/iphreeqc/database/wateq4f.dat" \
   "$ROOT/vendor/iphreeqc/database/minteq.v4.dat" \
   "$ROOT/vendor/iphreeqc/database/pitzer.dat" "$OUT/db/"

echo "== lessons for the app's player"
mkdir -p "$OUT/lessons"
cp "$ROOT"/lessons/*.lab "$OUT/lessons/"
python3 "$ROOT/tools/lessons-index.py" "$OUT/lessons"

echo "== reviewed capability index"
python3 "$ROOT/tools/curiosity-index.py" \
  "$ROOT/tests/coverage/curiosity-v1" "$OUT/capabilities/index.json"

echo "== codex export"
cargo run -p kerotakis-cli -- codex export "$OUT/codex/index.json"

echo "== quest export"
mkdir -p "$OUT/quests"
cargo run -p kerotakis-cli -- quest export "$OUT/quests/index.json"
echo "== species pack (DATA-010)"
mkdir -p "$OUT/packs"
cargo run -p kerotakis-cli -- pack export "$OUT/packs/registry.pack"
python3 - "$OUT/packs" <<'PYMANIFEST'
import hashlib, json, os, sys
d = sys.argv[1]
packs = []
for f in sorted(os.listdir(d)):
    if not f.endswith(".pack"):
        continue
    data = open(os.path.join(d, f), "rb").read()
    packs.append({"file": f, "sha256": hashlib.sha256(data).hexdigest(), "size": len(data)})
open(os.path.join(d, "index.json"), "w").write(json.dumps({"packs": packs}, indent=1))
print(f"   manifest: {len(packs)} pack(s)")
PYMANIFEST

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
    # `tools/build-shell-payload.sh` stages the lessons and the codex in
    # web/app/public/engine/ for the packaged shells, and Vite copies
    # public/ verbatim into dist/. On the web those files already exist one
    # directory up, where this script put them and where the app looks for
    # them — so a leftover copy would ship the same 700 KB twice and enter
    # the service worker's precache twice. The shells rebuild it on demand.
    rm -rf "$ROOT/web/app/public/engine"
    (cd "$ROOT/web/app" \
        && { # emsdk_env.sh prepends its bundled ancient node; the app
             # needs a modern npm — prefer the system one when the PATH
             # winner is prehistoric.
             if [ "$(npm --version | cut -d. -f1)" -lt 9 ] && [ -x /usr/bin/npm ]; then
                 export PATH="/usr/bin:$PATH"
             fi; } \
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
