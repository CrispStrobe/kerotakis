#!/usr/bin/env bash
# LIC-005: provenance lint — verify that every vendored file has a provenance
# record with valid checksums, every runtime source has an approved licence,
# and no oracle output has leaked into shipping paths.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/provenance/sources.toml"
ERRORS=0

err() { echo "FAIL: $*" >&2; ERRORS=$((ERRORS + 1)); }
ok()  { echo "  ok  $*"; }

echo "=== Provenance lint ==="

# 1. Manifest exists
if [ ! -f "$MANIFEST" ]; then
    err "provenance/sources.toml not found"
    exit 1
fi
ok "manifest exists"

# 2. Every checksum entry matches the actual file
echo "--- Checksum verification ---"
while IFS= read -r line; do
    path=$(echo "$line" | sed -n 's/^path = "\(.*\)"/\1/p')
    [ -z "$path" ] && continue
    sha=$(sed -n "/^path = \"${path//\//\\/}\"/{ n; s/^sha256 = \"\(.*\)\"/\1/p; }" "$MANIFEST")
    [ -z "$sha" ] && continue
    if [ ! -f "$ROOT/$path" ]; then
        err "checksum entry for $path but file does not exist"
        continue
    fi
    actual=$(sha256sum "$ROOT/$path" | awk '{print $1}')
    if [ "$actual" != "$sha" ]; then
        err "$path: expected $sha, got $actual"
    else
        ok "$path checksum matches"
    fi
done < "$MANIFEST"

# 3. No runtime source uses a non-allowlisted licence
echo "--- Licence check ---"
ALLOWED_RUNTIME="MIT|Apache-2.0|BSD-2-Clause|BSD-3-Clause|CC0-1.0|ISC|Zlib|LicenseRef-USGS"
while IFS= read -r line; do
    licence=$(echo "$line" | sed -n 's/^licence = "\(.*\)"/\1/p')
    [ -z "$licence" ] && continue
    lane=$(sed -n "/^licence = \"${licence//\//\\/}\"/{
        # look backwards for lane
        x; p; d
    }" "$MANIFEST")
done < "$MANIFEST"
# Simpler approach: parse with python
python3 - "$MANIFEST" "$ALLOWED_RUNTIME" << 'PYEOF'
import sys, re
manifest = open(sys.argv[1]).read()
allowed = sys.argv[2]

# Find all [[source]] blocks
blocks = re.split(r'\[\[source\]\]', manifest)[1:]  # skip preamble
for block in blocks:
    lines = block.strip().split('\n')
    fields = {}
    for line in lines:
        m = re.match(r'(\w+)\s*=\s*"([^"]*)"', line)
        if m:
            fields[m.group(1)] = m.group(2)
    if not fields:
        continue
    lane = fields.get('lane', '')
    licence = fields.get('licence', '')
    sid = fields.get('id', '?')
    if 'runtime' in lane.lower():
        if not re.match(allowed, licence):
            print(f"FAIL: runtime source {sid} has non-allowlisted licence: {licence}", file=sys.stderr)
            sys.exit(1)
        else:
            print(f"  ok  {sid}: {licence} (runtime)")
    else:
        print(f"  ok  {sid}: {licence} ({lane})")
PYEOF
if [ $? -ne 0 ]; then ERRORS=$((ERRORS + 1)); fi

# 4. No oracle output in shipping paths
echo "--- Oracle leakage check ---"
if find "$ROOT/crates" "$ROOT/web" "$ROOT/data" -name "*.oracle" -o -name "*oracle_output*" 2>/dev/null | grep -q .; then
    err "oracle output files found in shipping paths"
else
    ok "no oracle leakage"
fi

# 5. Vendored sources have provenance records
echo "--- Vendored source coverage ---"
for dir in "$ROOT/vendor/iphreeqc" "$ROOT/vendor/my-basic" "$ROOT/vendor/nasa-cea"; do
    name=$(basename "$dir")
    if ! grep -q "\"$name" "$MANIFEST" 2>/dev/null && ! grep -q "$name" "$MANIFEST" 2>/dev/null; then
        err "vendored directory $name has no provenance record"
    else
        ok "$name has provenance record"
    fi
done

echo ""
if [ $ERRORS -gt 0 ]; then
    echo "FAILED: $ERRORS error(s)"
    exit 1
else
    echo "All provenance checks passed."
fi
