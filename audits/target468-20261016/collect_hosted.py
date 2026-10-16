#!/usr/bin/env python3
"""Collect one terminal bounded run and verify every source/script/stream receipt."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

HERE = Path(__file__).resolve().parent
REPO = 'CrispStrobe/kerotakis'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def gh_json(*args):
    return json.loads(subprocess.check_output(['gh', *args]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lane', choices=['common-engine-completion', 'curriculum200-common-engine'], required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    lane = HERE / args.lane
    dispatch = json.loads((lane / 'corrected-dispatch.json').read_text())
    run_id = dispatch['run_id']
    run = gh_json('run', 'view', str(run_id), '--repo', REPO, '--json', 'status,conclusion,headSha,url')
    assert run['status'] == 'completed', 'Run remains active; do not collect or duplicate it.'
    assert run['conclusion'] == 'success', 'Preserve and diagnose job failure before result collection.'
    assert run['headSha'] == dispatch['workflow_head']
    design_bytes = (lane / 'native-screen-design.json').read_bytes()
    design = json.loads(design_bytes)
    assert sha(design_bytes) == dispatch['design_sha256']
    assert sha((lane / 'hosted_native_screen.py').read_bytes()) == dispatch['runner_sha256']
    listing = gh_json('api', f'repos/{REPO}/actions/runs/{run_id}/artifacts')
    assert listing['total_count'] == len(listing['artifacts']) == 1
    artifact, = listing['artifacts']
    prefix = 'remaining267' if args.lane == 'common-engine-completion' else 'curriculum200'
    assert artifact['name'] == f'{prefix}-native-{run_id}' and not artifact['expired']
    assert 0 < artifact['size_in_bytes'] <= 128 * 1024**2
    raw = subprocess.check_output(['gh', 'api', f'repos/{REPO}/actions/artifacts/{artifact["id"]}/zip'])
    assert sha(raw) == artifact['digest'].removeprefix('sha256:')
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'raw-artifact.zip').write_bytes(raw)
    (args.out / 'artifact-metadata.json').write_text(json.dumps(artifact, indent=2) + '\n')
    with zipfile.ZipFile(args.out / 'raw-artifact.zip') as zipped:
        assert len(zipped.namelist()) == len(set(zipped.namelist()))
        assert sum(i.file_size for i in zipped.infolist()) <= 512 * 1024**2
        # Read fixed names only; never extract arbitrary archive paths.
        binding = json.loads(zipped.read('binding.json'))
        producer = json.loads(zipped.read('producer-manifest.json'))
        completion = json.loads(zipped.read('completion.json'))
        executions = json.loads(zipped.read('execution.json'))
        assert json.loads(zipped.read('design.json')) == design
        for name in ('design_sha256', 'runner_sha256'):
            assert binding[name] == dispatch[name]
        assert binding['workflow_commit'] == dispatch['workflow_head']
        assert binding['input_checkout_commit'] == dispatch['input_checkout_commit']
        assert str(binding['run_id']) == str(run_id)
        assert binding['engine'] == executions['engine'] == design['engine']
        for key, value in design['engine'].items():
            assert producer[key] == value
        rows = executions['results']
        assert len(rows) == completion['attempts'] == len(design['cases'])
        assert [r['id'] for r in rows] == [c['id'] for c in design['cases']]
        assert completion['retries'] == 0
        cases = []
        for case, result in zip(design['cases'], rows, strict=True):
            base = case['id'].replace(':', '_')
            script = zipped.read(base + '/experiment.lab')
            stdout = zipped.read(base + '/stdout.jsonl')
            stderr = zipped.read(base + '/stderr.txt')
            assert json.loads(zipped.read(base + '/execution.json')) == result
            assert sha(script) == result['script_sha256'] == case['selected_script_sha256']
            assert script == (lane / case['selected_script']).read_bytes()
            assert sha(stdout) == result['stdout_sha256']
            assert sha(stderr) == result['stderr_sha256']
            events = Counter()
            limits = Counter()
            parse_error = None
            count = 0
            try:
                for line in stdout.splitlines():
                    if not line.strip():
                        continue
                    obj = json.loads(line, parse_constant=lambda _: (_ for _ in ()).throw(ValueError('Nonfinite JSON')))
                    assert isinstance(obj, dict)
                    count += 1
                    for event in obj.get('events', []):
                        events[event['event']] += 1
                        if event['event'] == 'not_yet_modeled':
                            limits[str(event.get('reason', event.get('detail', 'unspecified')))] += 1
            except (ValueError, UnicodeError) as exc:
                parse_error = type(exc).__name__
            assert count == result['parsed_observations']
            assert parse_error == result['stdout_parse_error']
            cases.append({**result, 'events': dict(events), 'named_limits': dict(limits),
                          'physical_verdict': 'not_assigned', 'engine': design['engine']})
    assert dict(Counter(r['status'] for r in rows)) == completion['counts']
    summary = {'run': run, 'run_id': run_id, 'engine': design['engine'], 'cases': cases,
               'completion': completion, 'binding': binding, 'raw_artifact_sha256': sha(raw),
               'verified_exact_scripts_streams_and_receipts': len(cases),
               'local_engine_executions': 0, 'api_execution_requests': 0,
               'physical_verdict': 'not_assigned'}
    (args.out / 'verified-results.json').write_text(json.dumps(summary, indent=2, allow_nan=False) + '\n')
    print(json.dumps({'run': run_id, 'verified_attempts': len(cases), 'counts': completion['counts'],
                      'stdout_parse_errors': completion['stdout_parse_errors'], 'physical_verdict': 'not_assigned'}, indent=2))


if __name__ == '__main__':
    main()
