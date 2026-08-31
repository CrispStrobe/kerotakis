#!/usr/bin/env bash
# BRD-030 spike: fetch feos's published parameter files.
#
# These files are NOT committed. They are third-party parameter tables
# transcribed from journal publications, and clearing them for shipping is
# BRD-031's job, not this spike's. The spike only needs to *run* against
# them once to produce the discrepancy fixtures, so it downloads them into a
# gitignored directory and records a checksum so the run is reproducible.
#
# Provenance of each file is recorded in
# provenance/brd-030-feos-spike.md § Parameter provenance.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
DEST="$HERE/parameters"
BASE="https://raw.githubusercontent.com/feos-org/feos/main/parameters/pcsaft"

mkdir -p "$DEST"
for f in esper2023.json rehner2023_binary.json gross2001.json gross2002.json \
         gross2002_binary.json gross2005_literature.json gross2006.json \
         README.md literature.bib; do
    echo "fetching $f" >&2
    curl -sSfL -o "$DEST/$f" "$BASE/$f"
done

( cd "$DEST" && sha256sum ./*.json ./*.md ./*.bib > SHA256SUMS )
cat "$DEST/SHA256SUMS"
