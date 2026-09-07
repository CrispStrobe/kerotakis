#!/usr/bin/env bash
# Assemble the static side-payload the desktop/mobile shell bundles.
#
# The bench UI is one app across every host (ROADMAP-GUI.md), and part of
# what it reads is not engine state but shipped data: the lesson library and
# the codex of experiments, fetched from `resolvePayloadBase()`. On the web
# `tools/build-web.sh` puts those one directory up from `app/`; in a Tauri
# shell there is no directory up, so the default base is `./engine/` and the
# files have to be *inside* the bundle. Vite copies `public/` verbatim into
# `dist/`, and `dist/` is what Tauri packages — so `public/engine/` is the
# shell's payload root.
#
# Without this the shipped app opens with an empty lesson picker and no
# experiment catalog, which is why it runs from `beforeBuildCommand` rather
# than being a step someone has to remember.
#
# The engine itself is NOT here: the shell links IPhreeqc natively and needs
# neither the wasm modules nor the .dat databases nor the prewarm cache.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/web/app/public/engine"

rm -rf "$OUT"
mkdir -p "$OUT/lessons"

echo "== shell payload: lessons"
cp "$ROOT"/lessons/*.lab "$OUT/lessons/"
python3 "$ROOT/tools/lessons-index.py" "$OUT/lessons"

echo "== shell payload: kids experiment catalog"
python3 "$ROOT/tools/kids-catalog.py" \
  "$ROOT/data/kids/experiments-v1.json" "$OUT/kids/index.json"

echo "== shell payload: per-step prose"
python3 "$ROOT/tools/step-prose.py" \
  "$ROOT/data/steps/step-prose-v1.json" "$OUT/steps/index.json"

echo "== shell payload: codex"
# From the repo root: `kero codex export` reads the `codex/` source tree
# relative to the working directory, and npm runs this from web/app.
cd "$ROOT"
cargo run --quiet -p kerotakis-cli -- codex export "$OUT/codex/index.json"

echo "== reviewed capability index"
python3 "$ROOT/tools/curiosity-index.py" \
  "$ROOT/tests/coverage/curiosity-v1" "$OUT/capabilities/index.json"

du -sh "$OUT"
