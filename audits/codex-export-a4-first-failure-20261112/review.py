#!/usr/bin/env python3
"""Review the retained first export and its successor without running the engine."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
original = gzip.decompress((root / 'codex-export.original.json.gz').read_bytes())
actual = gzip.decompress((root / 'codex-export.actual.json.gz').read_bytes())
assert hashlib.sha256(original).hexdigest() == '0b988525416a63ccc1d171ea10b8e0af6fa08577781ce4904310514be3f182c1'
assert hashlib.sha256(actual).hexdigest() == '7845441d2193288071501e9746d82770ad1fc006790ac9811eb0cb512bea7bf9'
before, after = json.loads(original), json.loads(actual)
assert before.keys() == after.keys()
assert before['models'] == after['models'] and before['concepts'] == after['concepts']
a, b = before['reactions'], after['reactions']
assert len(a) == len(b) == 131
changed = {}
for x, y in zip(a, b):
    assert x['id'] == y['id']
    if x != y:
        changed[x['id']] = {k for k in x.keys() | y.keys() if x.get(k) != y.get(k)}
assert changed == {
    'baking-soda-vents-carbon-dioxide': {'expect', 'equation', 'registers', 'provenance'},
    'bicarbonate-plays-both-sides': {'expect', 'registers', 'provenance'},
    'the-antacid-runs-on-arithmetic': {'expect', 'registers', 'provenance'},
}
assert (root.parent.parent / 'crates/kerotakis-codex/tests/golden/codex-export.json').read_bytes() == actual
print('PASS: only the three reviewed codex entries changed; golden equals CI export')
