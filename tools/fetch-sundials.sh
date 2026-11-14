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

if [ "${1:-}" = "--archive-sha256" ] && [ "$#" -eq 1 ]; then
    printf '%s\n' "$SHA256"
    exit 0
fi
if [ "$#" -ne 0 ]; then
    echo "usage: $0 [--archive-sha256]" >&2
    exit 2
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/sundials-kinetics-rs/sundials-sys/sundials_src"

# Local callers retain the existing source-tree fast path. CI supplies an
# archive cache and must verify those bytes even when the source tree exists.
if [ -z "${SUNDIALS_ARCHIVE_CACHE:-}" ] && [ -f "$DEST/CMakeLists.txt" ]; then
    echo "sundials_src present — nothing to do"
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

ARCHIVE="${SUNDIALS_ARCHIVE_CACHE:-$WORK/sundials.tar.gz}"
if [ ! -f "$ARCHIVE" ]; then
    echo "== fetching SUNDIALS ${VERSION}"
    # Retry transient network/server failures, with bounded connection and
    # transfer times. Never publish an unverified download into the cache.
    curl -sSfL --connect-timeout 15 --max-time 120 \
        --retry 3 --retry-delay 2 --retry-max-time 180 --retry-connrefused \
        -o "$WORK/sundials.tar.gz" "$URL"
    echo "${SHA256}  $WORK/sundials.tar.gz" | sha256sum -c - >/dev/null
    if [ "$ARCHIVE" != "$WORK/sundials.tar.gz" ]; then
        mkdir -p "$(dirname "$ARCHIVE")"
        # Cache and temporary directories may be on different filesystems;
        # publish with a rename in the cache directory after copying.
        CACHED_PART="$(mktemp "${ARCHIVE}.XXXXXX")"
        trap 'rm -rf "$WORK"; rm -f "${CACHED_PART:-}"' EXIT
        cp "$WORK/sundials.tar.gz" "$CACHED_PART"
        mv "$CACHED_PART" "$ARCHIVE"
    fi
fi
# Cache hits are untrusted until this check passes. A mismatch fails closed
# and preserves the offending archive for diagnosis; it is never extracted.
echo "${SHA256}  $ARCHIVE" | sha256sum -c - >/dev/null
if [ -f "$DEST/CMakeLists.txt" ]; then
    echo "sundials_src present — nothing to do"
    exit 0
fi
tar xzf "$ARCHIVE" -C "$WORK"
mkdir -p "$(dirname "$DEST")"
mv "$WORK/sundials-${VERSION}" "$DEST"
echo "OK: $DEST"
