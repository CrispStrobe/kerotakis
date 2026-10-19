"""Authenticate saved Pages controls and numerical/source parity; no execution."""
import decimal
import hashlib
import json
import pathlib
import subprocess

H = pathlib.Path(__file__).resolve().parent
R = H.parents[2]
E = H / 'browser-evidence'
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest = json.loads((E / 'SHA256.json').read_bytes())
for name, digest in manifest.items():
    assert sha((E / name).read_bytes()) == digest, name
D = decimal.Decimal
decimal.getcontext().prec = 45
report = json.loads((E / 'actual-browser/report.json').read_bytes(), parse_float=D)
spec = report['spec']
assert report['all_checks_passed'] and len(report['checks']) == 11
assert all(c['passed'] for c in report['checks'])
assert not report['owner_credentials_supplied']
assert report['new_rest_requests'] == report['new_corpus_executions'] == 0
binding = json.loads((H / 'pages-artifact-assets.json').read_bytes())
assert spec['pages_source'] == binding['source']
for name, record in binding['assets'].items():
    observed = report['served_assets'][name]
    assert observed['status'] == 200
    assert observed['sha256'] == record['sha256'] and observed['bytes'] == record['bytes']
paths = subprocess.check_output(['git', 'diff', '--name-only',
    spec['native_runtime_source'], spec['pages_source']], cwd=R, text=True).splitlines()
assert paths == spec['runtime_equivalence_paths'] and all(p.startswith('docs/') for p in paths)
native_raw = (H.parent / 'ammonia-release-api-20261019/release-controls/uncached-native-browser-reference/response.body').read_bytes()
assert sha(native_raw) == spec['native_reference']['response_sha256']
native = json.loads(native_raw, parse_float=D)
actual = report['actual']
assert spec['pages_source'].startswith(actual['meta']['git_rev'])
assert not actual['bareCanSolve'] and actual['canSolve'] and actual['solver_hook_calls'] > 0
nv = native['observations'][-1]['bench']['vessels'][0]
pv = actual['result']['bench']['vessels'][0]
assert all(pv['solution'][key] == nv['solution'][key] for key in ['ph', 'ionic_strength'])
for key in ['Na+', 'Cl-']:
    assert abs(sum((q['moles'] for q in pv['contents'] if q['species'] == key), D(0)) - D('.037')) < D('1e-8')
assert any(str(x).strip() and D(str(x)) == 5 for row in actual['basic']['selected'] for x in row)
print(json.dumps({'pages_source': spec['pages_source'], 'saved_files_authenticated': len(manifest),
    'actual_browser_checks': 11, 'pH_and_ionic_strength_match_exactly': True,
    'interpreter_result': 5, 'new_engine_or_api_requests': 0}))
