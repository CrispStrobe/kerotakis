#!/usr/bin/env bash
# Everything CI checks that can be checked without emscripten or a browser.
#
# Written because three pushes in one week went out green on `cargo test`
# and red on CI, every one of them for the same reason: the workspace has
# builds that native testing never touches. `kerotakis-phreeqc` compiles
# with and without its `engine` feature, and the featureless build is what
# the browser runs — so a helper taking `&mut Phreeqc`, or a struct field
# the JS bridge cannot supply, breaks a target that `cargo test` is not
# looking at.
#
# The wasm32 target was installed on this machine the whole time. Knowing
# that CI builds for wasm is not the same as building for wasm.
#
# Not a substitute for CI: the emscripten IPhreeqc build, the wasm-bindgen
# bridge test and the headless demo still only run there.
set -euo pipefail
cd "$(dirname "$0")/.."

# One heavy gate at a time on this box: 7.6 GiB of RAM does not hold
# three sessions' full preflights, and stacking them is how the
# 2026-08-25 near-OOM happened. The lock lives on the shared volume, so
# every session (and only this machine — CI runners have no
# /mnt/volume1 and skip it) queues here instead of thrashing swap.
if [ -d /mnt/volume1 ]; then
    # The lock fd is inherited by every child, and a compile daemon that
    # outlives this script then holds the gate forever — observed
    # 2026-08-29, when a preflight-spawned sccache server kept the flock
    # and every later session queued behind a script that had long since
    # exited. Two defences: start the daemon FIRST so no child of ours
    # ever becomes it, and close the fd for each step's children via the
    # step() wrapper below.
    command -v sccache >/dev/null && sccache --start-server >/dev/null 2>&1 || true
    exec 9>/mnt/volume1/.kero-build-lock
    if ! flock -n 9; then
        echo "preflight: another session holds the build gate — waiting…"
        flock 9
    fi
fi

LIGHT=false
# Clippy is on by default: a preflight that checks less than CI is how the
# "green locally, red on CI" problem this script exists to prevent comes
# back. CI passes --no-clippy because its own `Test (native)` matrix
# already runs the identical command on both platforms, so preflight's
# run is a third pass that buys nothing.
CLIPPY=true
for arg in "$@"; do
  case "$arg" in
    --light) LIGHT=true ;;
    --no-clippy) CLIPPY=false ;;
  esac
done

step() { printf '\n\033[1m== %s\033[0m\n' "$1"; }
# Run a gate step with the lock fd closed for its children, so nothing a
# step spawns can inherit — and outlive us holding — the build gate.
gated() { "$@" 9>&-; }

step "fmt";           gated cargo fmt --check
if $CLIPPY; then
  step "clippy";      gated cargo clippy --workspace --all-targets -- -D warnings
else
  step "clippy";      echo "skipped (--no-clippy; the Test matrix runs it on both platforms)"
fi
step "no-engine";     gated cargo check -p kerotakis-phreeqc --no-default-features

if $LIGHT; then
  printf '\n\033[1;32mpreflight --light clean\033[0m\n'
  exit 0
fi

step "tests";         gated cargo test --workspace
step "wasm32";        gated cargo build -p kerotakis-wasm -p kerotakis-scene-physics --target wasm32-unknown-unknown
step "BRD-071 evaluator"; python3 -m unittest tools.tests.test_brd071_evaluate
step "BRD-072 evaluator"; python3 -m unittest tools.tests.test_brd072_evaluate
# The i18n gates. Seconds each, and each one caught something real while
# this was being built: a key shared by two different sentences (which
# renders the WRONG sentence, not a missing one), a placeholder nothing
# fills (which renders as literal `{name}` on screen), a catalogue drifting
# behind the source it translates, and a codex slug the map de-slugs into a
# dictionary that has no word for it (which renders English inside German).
step "i18n catalogue"; python3 tools/codex-locale-lint.py --check
step "i18n engine";    python3 tools/engine-locale-lint.py --check
step "i18n vocabulary"; python3 tools/i18n-engine-vocabulary-lint.py --check
step "i18n vocabulary self-test"; python3 -m unittest tools/test_i18n_engine_vocabulary.py
step "i18n holes";     python3 tools/i18n-holes-lint.py --check
step "i18n surfaces";  python3 tools/i18n-surface-lint.py --check
step "i18n slugs";     python3 tools/i18n-slug-lint.py --check
step "codex lint";    cargo run --release -p kerotakis-cli -- codex lint
step "provenance";    cargo run --release -p kerotakis-cli -- provenance lint
step "sweep";         cargo run --release -p kerotakis-cli -- sweep
# CAP-13: every curated structure's InChIKey recomputed by the official
# IUPAC library must reproduce the registry key. The C build is cached,
# so the steady-state cost is seconds.
step "inchi identity"; cargo test -q -p kerotakis-org --features native-inchi --test native_identity
# EXP-0: a quest that could lie, block, or corridor fails the gate.
step "quest lint";     cargo run --release -p kerotakis-cli -- quest lint

# CAP-14: licence bar as cargo-deny lint (2026-08-23)
if command -v cargo-deny >/dev/null 2>&1; then
  step "cargo-deny";  cargo deny check
fi

# Provenance checksums (vendored files)
if [ -f tools/provenance-lint.sh ]; then
  step "provenance checksums"; bash tools/provenance-lint.sh
fi

# The committed icons must still match the mark they were drawn from —
# every store, browser tab and home screen reads these, and a hand-edited
# PNG that no longer matches its source is invisible until an upload is
# rejected.
#
# Local only, deliberately. It needs librsvg, which CI does not install,
# and pinning committed bytes against a differently-versioned rasteriser on
# another platform would fail for reasons that have nothing to do with the
# artwork. Whoever regenerates the icons has librsvg by definition, and
# this is the gate on them.
if command -v rsvg-convert >/dev/null 2>&1; then
  step "icons"; python3 tools/gen-icons.py --check
else
  echo "   (icons: rsvg-convert absent, skipping the icon check)"
fi

printf '\n\033[1;32mpreflight clean\033[0m\n'
