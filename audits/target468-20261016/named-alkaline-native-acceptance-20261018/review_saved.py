"""Authenticate saved outputs only; never launch builds, requests or retries."""
import ast
import datetime
import hashlib
import json
import pathlib
import re
import subprocess

B = pathlib.Path(__file__).resolve().parent
P = json.loads((B / 'frozen-plan.json').read_bytes())
R = B.parents[2]
sha = lambda raw: hashlib.sha256(raw).hexdigest()
get = lambda source, path: subprocess.check_output(['git', 'show', source + ':' + path], cwd=R)

assert (B / 'COLLECTED.json').exists(), 'Wait for the existing once-only collector'
C = json.loads((B / 'COLLECTED.json').read_bytes())
for path, record in C['files'].items():
    raw = (B / 'output' / path).read_bytes()
    assert sha(raw) == record['sha256'] and len(raw) == record['bytes'], path
roots = list((B / 'output').rglob('SHA256.json'))
assert len(roots) == 1
E = roots[0].parent
manifest = json.loads(roots[0].read_bytes())
for path, digest in manifest.items():
    assert sha((E / path).read_bytes()) == digest, path
worker = (B / 'upload/native_controls.py').read_bytes()
assert sha(worker) == P['uploaded_worker_sha256']
# Provider metadata stays private; its original digest is frozen in P.
if (B / 'upload/kernel-metadata.json').exists():
    assert sha((B / 'upload/kernel-metadata.json').read_bytes()) == P['metadata_sha256']
assert json.loads((E / 'STARTED.json').read_bytes())['script_sha256'] == sha(worker)
embedded = ast.literal_eval(ast.parse(worker).body[0].value)
assert all(P[k] == value for k, value in embedded.items())
for source_key in ('baseline', 'fix'):
    source = P[source_key]
    assert subprocess.check_output(['git', 'rev-parse', source + '^{tree}'], cwd=R, text=True).strip() == P[source_key + '_tree']
for path, digest in P['fix_source_sha256'].items():
    assert sha(get(P['fix'], path)) == digest, path
for path, digest in P['frozen_control_sha256'].items():
    assert sha(get(P['baseline'], path)) == sha(get(P['fix'], path)) == digest, path
changed = subprocess.check_output(['git', 'diff', '--name-only', P['accepted_runtime_base'], P['baseline']], cwd=R, text=True).splitlines()
assert set(changed) == set(P['frozen_control_sha256']), changed
records = []
for kind in ('baseline', 'fix'):
    for index, argv in enumerate(P[kind + '_commands']):
        label = kind + '-' + str(index)
        path = E / (label + '.json')
        if not path.exists():
            continue
        record = json.loads(path.read_bytes())
        assert record['argv'] == argv
        for stream in ('stdout', 'stderr'):
            assert sha((E / (label + '.' + stream)).read_bytes()) == record[stream + '_sha256']
        records.append(record)
result = {'reviewed_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'source': P['fix'], 'tree': P['fix_tree'], 'baseline': P['baseline'], 'authenticated_files': len(manifest), 'authenticated_commands': records, 'new_engine_or_api_requests': 0, 'scientific_selections': 0}
if (E / 'STOPPED.json').exists():
    assert not (E / 'FIX-CONTROLS-PASSED.json').exists()
    result.update(accepted=False, first_stop=json.loads((E / 'STOPPED.json').read_bytes()))
    output = B / 'ROOT-FIRST-FAILURE.json'
else:
    assert (E / 'COMPLETED.json').exists() and (E / 'FIX-CONTROLS-PASSED.json').exists()
    assert len(records) == 3 and all(r['exit_status'] == 0 for r in records)
    previous = B.parent / 'named-alkaline-first-failure-20261018'
    assert sha((previous/'ROOT-FIRST-FAILURE.json').read_bytes()) == P['previous_authenticated_failure_sha256']
    assert sha((previous/'raw/SHA256.json').read_bytes()) == P['baseline_manifest_sha256']
    first = json.loads((previous/'ROOT-FIRST-FAILURE.json').read_bytes())
    assert not first['accepted'] and first['source'] == P['previous_fix']
    lock = sha((E / 'fix-Cargo.lock').read_bytes())
    assert lock == P['expected_producer_lock_sha256']
    proof = json.loads((E / 'FIX-CONTROLS-PASSED.json').read_bytes())
    assert proof['actual_producer_lock_sha256'] == lock and proof['source'] == P['fix'] and proof['tree'] == P['fix_tree']
    counts = {}
    for index, path in [(2, 'crates/kerotakis-core/tests/named_alkaline.rs'), (1, 'crates/kerotakis-phreeqc/tests/named_hydrolysis.rs')]:
        raw = (E / ('fix-' + str(index) + '.stdout')).read_text()
        required = re.findall(r'#\[test\]\s*fn (\w+)\(', get(P['fix'], path).decode())
        assert len(required) == (9 if index == 2 else 3)
        for name in required:
            assert 'test ' + name + ' ... ok' in raw, name
        results = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', raw)
        assert results and all(int(failed) == 0 for _, failed, _ in results)
        counts[str(index)] = {'passed': sum(int(n) for n, _, _ in results), 'ignored': sum(int(n) for _, _, n in results), 'required_new_tests': required}
        assert counts[str(index)]['passed'] >= (2137 if index == 2 else 607)
        assert counts[str(index)]['ignored'] == 0
    stderr = (E / 'fix-1.stderr').read_text()
    groups = {prefix: [json.loads(line.split(prefix + ' ', 1)[1]) for line in stderr.splitlines() if line.startswith(prefix + ' ')] for prefix in ('NAMED_ALKALINE_STATE', 'NAMED_ALKALINE_ACID', 'NAMED_ALKALINE_NEGATIVE')}
    assert [len(rows) for rows in groups.values()] == [4, 2, 3]
    result.update(accepted=True, actual_producer_lock_sha256=lock, test_counts=counts, native_control_groups=groups, remaining_acceptance='Independent chemical/proton/counterion review; current prerequisites; full33 and platform/protected integration; delivery and original OR006 replay. No scientific selection.')
    output = B / 'ROOT-CONTROLS-AUTHENTICATED.json'
if output.exists():
    previous_result = json.loads(output.read_bytes())
    assert {k:v for k,v in previous_result.items() if k != 'reviewed_utc'} == {k:v for k,v in result.items() if k != 'reviewed_utc'}
else:
    output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ('source', 'authenticated_files', 'accepted', 'scientific_selections')}))
