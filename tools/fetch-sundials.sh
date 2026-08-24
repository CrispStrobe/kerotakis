#!/usr/bin/env bash
# Fetch SUNDIALS into sundials-kinetics-rs/sundials-sys/sundials_src.
#
# SUNDIALS (BSD-3-Clause, LLNL) is deliberately NOT vendored into this
# repository — the tree is ~40 MB of upstream source. Local checkouts carry
# it after a one-time run of this script; CI runs it before any job that
# builds the workspace, pinned by version and checksum so every build
# compiles the same bytes a human audited. Verified 2026-08-24: the 7.2.1
# release tarball is byte-identical to the tree the local builds use.
set -euo pipefail

VERSION="7.2.1"
SHA256="3781e3f7cdf372ca12f7fbe64f561a8b9a507b8a8b2c4d6ce28d8e4df4befbea"
URL="https://github.com/LLNL/sundials/releases/download/v${VERSION}/sundials-${VERSION}.tar.gz"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/sundials-kinetics-rs/sundials-sys/sundials_src"

if [ -f "$DEST/CMakeLists.txt" ]; then
    echo "sundials_src present — nothing to do"
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "== fetching SUNDIALS ${VERSION}"
curl -sSfL -o "$WORK/sundials.tar.gz" "$URL"
echo "${SHA256}  $WORK/sundials.tar.gz" | sha256sum -c - >/dev/null
tar xzf "$WORK/sundials.tar.gz" -C "$WORK"
mkdir -p "$(dirname "$DEST")"
mv "$WORK/sundials-${VERSION}" "$DEST"
echo "OK: $DEST"
