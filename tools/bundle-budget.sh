#!/usr/bin/env bash
# PERF-001: Measure bundle and model-pack sizes.
# Reports compressed sizes and checks against budgets.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Bundle Budget Report ==="
echo ""

# Wasm module size (if built)
WASM="$ROOT/target/iphreeqc-wasm/iphreeqc.wasm"
if [ -f "$WASM" ]; then
    RAW=$(stat --printf="%s" "$WASM" 2>/dev/null || stat -f%z "$WASM")
    GZ=$(gzip -c "$WASM" | wc -c)
    echo "IPhreeqc wasm:  $(numfmt --to=iec $RAW) raw, $(numfmt --to=iec $GZ) gzipped"
    if [ "$GZ" -gt 1048576 ]; then
        echo "  WARNING: wasm exceeds 1 MiB gzipped budget"
    else
        echo "  OK: within 1 MiB budget"
    fi
else
    echo "IPhreeqc wasm:  not built (run tools/build-iphreeqc-wasm.sh)"
fi
echo ""

# Registry pack size (if compiled)
PACK="$ROOT/data/registry/registry.pack"
if [ -f "$PACK" ]; then
    RAW=$(stat --printf="%s" "$PACK" 2>/dev/null || stat -f%z "$PACK")
    GZ=$(gzip -c "$PACK" | wc -c)
    echo "Registry pack:  $(numfmt --to=iec $RAW) raw, $(numfmt --to=iec $GZ) gzipped"
else
    echo "Registry pack:  not compiled (run compile-registry)"
fi
echo ""

# Database sizes
echo "Embedded databases:"
for db in phreeqc.dat wateq4f.dat minteq.v4.dat pitzer.dat; do
    F="$ROOT/vendor/iphreeqc/database/$db"
    if [ -f "$F" ]; then
        SZ=$(stat --printf="%s" "$F" 2>/dev/null || stat -f%z "$F")
        echo "  $db: $(numfmt --to=iec $SZ)"
    fi
done
echo ""

# NASA CEA data
CEA="$ROOT/vendor/nasa-cea/thermo.inp"
if [ -f "$CEA" ]; then
    SZ=$(stat --printf="%s" "$CEA" 2>/dev/null || stat -f%z "$CEA")
    echo "NASA CEA thermo.inp: $(numfmt --to=iec $SZ)"
fi
echo ""

# Total vendored source
echo "Vendored source:"
du -sh "$ROOT/vendor/iphreeqc/src/" 2>/dev/null | awk '{print "  iphreeqc src: " $1}'
du -sh "$ROOT/vendor/my-basic/" 2>/dev/null | awk '{print "  my-basic:     " $1}'
du -sh "$ROOT/vendor/nasa-cea/" 2>/dev/null | awk '{print "  nasa-cea:     " $1}'
echo ""

echo "=== Done ==="
