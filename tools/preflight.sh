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

LIGHT=false
for arg in "$@"; do
  case "$arg" in
    --light) LIGHT=true ;;
  esac
done

step() { printf '\n\033[1m== %s\033[0m\n' "$1"; }

step "fmt";           cargo fmt --check
step "clippy";        cargo clippy --workspace --all-targets -- -D warnings
step "no-engine";     cargo check -p kerotakis-phreeqc --no-default-features

if $LIGHT; then
  printf '\n\033[1;32mpreflight --light clean\033[0m\n'
  exit 0
fi

step "tests";         cargo test --workspace
step "wasm32";        cargo build -p kerotakis-wasm --target wasm32-unknown-unknown
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

printf '\n\033[1;32mpreflight clean\033[0m\n'
