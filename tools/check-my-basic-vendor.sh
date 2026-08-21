#!/usr/bin/env bash
# Verify the pinned MY-BASIC payload and exercise the disabled-I/O build.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENDOR="$ROOT/vendor/my-basic"
BUILD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/kerotakis-my-basic.XXXXXX")"
trap 'rm -rf "$BUILD_DIR"' EXIT

if command -v sha256sum >/dev/null 2>&1; then
    sha256_file() { sha256sum "$1" | awk '{print $1}'; }
else
    sha256_file() { shasum -a 256 "$1" | awk '{print $1}'; }
fi

verify_hash() {
    local path="$1"
    local expected="$2"
    local actual
    actual="$(sha256_file "$VENDOR/$path")"
    if [ "$actual" != "$expected" ]; then
        echo "MY-BASIC vendor hash mismatch: $path" >&2
        echo "expected $expected" >&2
        echo "actual   $actual" >&2
        exit 1
    fi
}

verify_hash LICENSE 8156be51115207259348d3e553ae9b61ab305b8fe3d74ff358cb4d3fe992b6c3
verify_hash my_basic.c 669994f258dec9efd1a4e93f03d406cd5087ce9096d28825439bc58a76dfc846
verify_hash my_basic.h 3a72d47d50c8f56084682a36d2a54252fdac33d481f0a5a6b065d8d2f44a35ea

CC_BIN="${CC:-cc}"
"$CC_BIN" -std=c99 -DMB_DISABLE_LOAD_FILE \
    -I"$VENDOR" \
    "$VENDOR/my_basic.c" \
    "$ROOT/tools/my-basic-smoke.c" \
    -lm \
    -o "$BUILD_DIR/my-basic-smoke"

"$BUILD_DIR/my-basic-smoke"
echo "OK: pinned MY-BASIC hashes and disabled-I/O smoke checks passed."
