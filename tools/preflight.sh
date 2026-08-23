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

step() { printf '\n\033[1m== %s\033[0m\n' "$1"; }

step "fmt";           cargo fmt --check
step "clippy";        cargo clippy --workspace --all-targets -- -D warnings
step "tests";         cargo test --workspace
step "no-engine";     cargo check -p kerotakis-phreeqc --no-default-features
step "wasm32";        cargo build -p kerotakis-wasm --target wasm32-unknown-unknown
step "codex lint";    cargo run --release -p kerotakis-cli -- codex lint
step "provenance";    cargo run --release -p kerotakis-cli -- provenance lint
step "sweep";         cargo run --release -p kerotakis-cli -- sweep

# CAP-14: licence bar as cargo-deny lint (2026-08-23)
if command -v cargo-deny >/dev/null 2>&1; then
  step "cargo-deny";  cargo deny check
fi

# Provenance checksums (vendored files)
if [ -f tools/provenance-lint.sh ]; then
  step "provenance checksums"; bash tools/provenance-lint.sh
fi

printf '\n\033[1;32mpreflight clean\033[0m\n'
