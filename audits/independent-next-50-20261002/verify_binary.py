"""Bind a reused CLI to its successful hosted validation receipt."""
import hashlib
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
receipt = json.loads((root / 'validation.json').read_text())
assert receipt['commit'] == sys.argv[2], 'validation commit mismatch'
assert receipt['passed'] is True, 'validation did not pass'
assert receipt['stages'] and all(stage['exit_code'] == 0 for stage in receipt['stages'])
h = hashlib.sha256()
with (root / 'kero').open('rb') as stream:
    for block in iter(lambda: stream.read(1024 * 1024), b''):
        h.update(block)
assert h.hexdigest() == receipt['binary_sha256'], 'executable does not match validated artifact'
print('Verified executable from', receipt['commit'], 'with', receipt['total_rust_tests_passed'], 'passing Rust tests')
