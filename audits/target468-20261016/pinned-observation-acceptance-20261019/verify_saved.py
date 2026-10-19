"""Authenticate published saved CPU proof without executing chemistry."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib
import yaml

base = Path(__file__).resolve().parent
repo = base.parents[2]
sha = lambda data: hashlib.sha256(data).hexdigest()
for name, digest in json.loads((base / 'evidence-sha256.json').read_text()).items():
    assert sha((base / name).read_bytes()) == digest, name
proof = base / 'producer'
p = json.loads((proof / 'frozen-plan.json').read_text())
r = json.loads((base / 'ROOT-ACCEPTED.json').read_text())
source = p['reviewed_head']
get = lambda path: subprocess.check_output(['git', 'show', source + ':' + path], cwd=repo)
assert r['source'] == source and r['tree'] == p['tree']
assert sha(get('.github/workflows/chemistry-audit.yml')) == p['workflow_sha256']
workflow = yaml.safe_load(get('.github/workflows/chemistry-audit.yml'))
for path, digest in p['source_native_manifest_files_sha256'].items():
    assert sha(get(path)) == digest, path
omitted = json.loads((base / 'retained-binary-archives.json').read_text())
for path, digest in json.loads((proof / 'SHA256.json').read_text()).items():
    if path in omitted:
        assert omitted[path]['sha256'] == digest
    else:
        assert sha((proof / path).read_bytes()) == digest, path
assert len(p['jobs']) == 33
steps = 0
for job in p['jobs']:
    original = workflow['jobs'][job['key']]
    expected = [(i, step['run'].replace('${{ matrix.family }}', job['family'] or '').replace('${{ github.sha }}', source))
                for i, step in enumerate(original['steps']) if 'run' in step]
    assert [(step['index'], step['run']) for step in job['steps']] == expected
    name = job['key'] + ('-' + job['family'] if job['family'] else '')
    for step in job['steps']:
        stem = proof / name / ('step-' + str(step['index']))
        result = json.loads(stem.with_suffix('.json').read_text())
        assert result['exit_status'] == 0
        assert result['argv'] == ['bash', '-e', '-o', 'pipefail', '-c', step['run']]
        for stream in ['stdout', 'stderr']:
            assert sha(stem.with_suffix('.' + stream).read_bytes()) == result[stream + '_sha256']
        steps += 1
assert steps == 142
for i, minimum in [(2, 620), (3, 2143), (4, 2)]:
    record = json.loads((proof / ('combined-native-' + str(i) + '.json')).read_text())
    assert record['exit_status'] == 0 and record['argv'] == p['focused_native_commands'][i]
    raw = (proof / ('combined-native-' + str(i) + '.stdout')).read_text()
    counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', raw)
    assert counts and all(int(f) == 0 and int(skip) == 0 for _, f, skip in counts)
    assert sum(int(n) for n, _, _ in counts) >= minimum
report = json.loads((proof / 'additional-gate-1.stdout').read_text())
assert len(report['prompts']) == 500 and report['unmet_requirements'] == 0 and not report['failures'] and not report['baseline_drift']
original = tomllib.loads(subprocess.check_output(['git', 'show', '8681037e:tests/coverage/curiosity-v1/baseline.toml'], cwd=repo, text=True))
before = {row['id']: row for row in original['observation']}
changes = []
for row in report['prompts']:
    actual = {'id': row['id'], 'owning_task': row['owning_task'], 'outcome': row['observed'], 'reason_code': row['reason_code']}
    if before[row['id']] != actual:
        changes.append({'id': row['id'], 'before': before[row['id']], 'after': actual})
assert sorted(changes, key=lambda x: x['id']) == sorted(p['reviewed_baseline_migration']['reviewed_changes'], key=lambda x: x['id'])
for case in p['primary_event_diagnostics']:
    name = 'primary-' + case['id']
    assert (proof / (name + '.lab')).read_text() == '\n'.join(case['script']) + '\n'
    rows = [json.loads(line) for line in (proof / (name + '.stdout')).read_text().splitlines() if line]
    assert len(rows) == len(case['script'])
    kinds = [event['event'] for row in rows for event in row['events']]
    assert 'observed' in kinds and 'not_yet_modeled' in kinds
    assert set(kinds) <= {'added', 'material_added', 'vessel_created', 'vessel_removed', 'stirred', 'solution_routed', 'not_yet_modeled', 'observed'}
assert sha((proof / 'producer-Cargo.lock').read_bytes()) == p['expected_producer_lock_sha256'] == r['engine']['actual_producer_lock_sha256']
assert len(r['law_reports']) == 29 and r['all_original_steps_and_named_laws_pass']
print(json.dumps({'source': source, 'public_hashes_and_original_steps_verified': True, 'original_jobs': 33, 'steps': steps,
                  'coverage_requirements': 500, 'new_primary_traces': 3, 'scientific_catalog_additions': 0,
                  'retained_archives_rehashed_in_this_text_only_review': False, 'deployment_or_integration_claim': False}))
