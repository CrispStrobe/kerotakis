"""Independently verify saved API evidence and compare it with verified CLI rows.

Usage: assess_api.py API_EXECUTIONS CLI_EXECUTIONS NEW_REPORT_DIRECTORY
No requests, execution or scientific pass assignment occurs here.
"""
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
import statistics
import sys


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    evidence, cli, output = map(Path, sys.argv[1:])
    root = Path(__file__).resolve().parent
    binding = json.loads((evidence / 'binding.json').read_text())
    assert sha(evidence / 'frozen-runner.py') == binding['runner_sha256'] == sha(root / 'run_api.py')
    assert sha(root / 'design/freeze.json') == binding['design_freeze_sha256']
    freeze = json.loads((root / 'design/freeze.json').read_text())
    for name, digest in freeze['files'].items():
        assert sha(root / 'design' / name) == digest
    assert sha(cli / 'binding.json') == binding['cli_binding_sha256']
    assert sha(cli / 'executions.json') == binding['cli_executions_sha256']
    baseline = {r['id']: r for r in json.loads((cli / 'executions.json').read_text())}
    records = json.loads((root / 'design/manifest.json').read_text())['records']
    executions = json.loads((evidence / 'executions.json').read_text())
    completion = json.loads((evidence / 'completed.json').read_text())
    assert completion['records'] == len(records) == len(executions) == 200
    assert sha(evidence / 'executions.json') == completion['executions_sha256']
    assert json.loads((evidence / 'initial-health.json').read_text()) == binding['health']
    for file in evidence.glob('health-after-*.json'):
        assert json.loads(file.read_text()) == binding['health']
    expected = {r['id']: r for r in records}
    assert len({r['id'] for r in executions}) == 200 and set(expected) == set(baseline)
    reviewed = []
    attempts = 0
    for result in sorted(executions, key=lambda r: r['rank']):
        record = expected[result['id']]
        assert record['rank'] == result['rank']
        script_path = root / 'design' / record['script']
        assert sha(script_path) == result['script_sha256'] == freeze['files'][record['script']]
        stem = f"{result['rank']:03d}-{result['id']}"
        assert json.loads((evidence / (stem + '.result.json')).read_text()) == result
        for attempt in range(1, result['attempts'] + 1):
            attempt_record = json.loads((evidence / f'{stem}.attempt-{attempt:02d}.json').read_text())
            assert attempt_record['rank'] == result['rank'] and attempt_record['id'] == result['id']
            assert attempt_record['script_sha256'] == result['script_sha256']
            assert sha(evidence / f'{stem}.attempt-{attempt:02d}.body') == attempt_record['response_sha256']
            if attempt < result['attempts']:
                assert attempt_record['http_status'] == 429
            attempts += 1
        body_path = evidence / result['response_file']
        assert body_path.name == f"{stem}.attempt-{result['attempts']:02d}.body"
        assert sha(body_path) == result['response_sha256']
        observations = []
        if result['http_status'] == 200:
            response = json.loads(body_path.read_text())
            assert response['engine'] == binding['health']['engine']
            assert response['physical_verdict'] == 'not_assigned' and response['script_sha256'] == result['script_sha256']
            assert response['exit_status'] == result['exit_status'] and response['execution_status'] == result['execution_status']
            observations = response['observations']
        before = baseline[result['id']]
        assert before['script_sha256'] == result['script_sha256']
        for stream in ('stdout', 'stderr'):
            assert sha(cli / (stem + '.' + stream)) == before[stream + '_sha256']
        cli_rows = [json.loads(line) for line in (cli / (stem + '.stdout')).read_text().splitlines()]
        same_execution = result['exit_status'] == before['exit_status'] and (
            result['execution_status'] == 'completed' and before['execution_status'] == 'completed'
            or result['execution_status'] == 'refused' and before['execution_status'] == 'nonzero_exit')
        equal = result['http_status'] == 200 and observations == cli_rows
        assert result['execution_matches_cli'] == same_execution and result['observations_equal_cli'] == equal
        adds = sum(line.startswith('add ') for line in script_path.read_text().splitlines())
        observed = sum(row.get('operator', {}).get('op') in ('add', 'add_material') for row in observations)
        assert result['all_add_commands_reached'] == (adds == observed)
        reviewed.append({**result, 'expected_add_commands': adds, 'observed_add_commands': observed,
            'cli_observations': len(cli_rows), 'response_file': body_path.name})
    counts = dict(Counter(r['execution_status'] for r in reviewed))
    assert counts == completion['execution_counts']
    assert sum(r['execution_matches_cli'] for r in reviewed) == completion['execution_matches_cli']
    assert sum(r['observations_equal_cli'] for r in reviewed) == completion['observations_equal_cli']
    times = [r['elapsed_seconds'] for r in reviewed]
    summary = {'schema_version': 1, 'origin': binding['origin'], 'api_revision': binding['health']['revision'],
        'engine': binding['health']['engine'], 'records': len(reviewed), 'http_attempts': attempts,
        'execution_counts': counts, 'http_counts': dict(Counter(str(r['http_status']) for r in reviewed)),
        'execution_matches_cli': sum(r['execution_matches_cli'] for r in reviewed),
        'observations_equal_cli': sum(r['observations_equal_cli'] for r in reviewed),
        'all_add_commands_reached': sum(r['all_add_commands_reached'] for r in reviewed),
        'request_seconds': {'median': statistics.median(times), 'maximum': max(times), 'sum': sum(times)},
        'binding_sha256': sha(evidence / 'binding.json'), 'completion_sha256': sha(evidence / 'completed.json'),
        'assessment_tool_sha256': sha(Path(__file__)), 'physical_verdict': 'not_assigned',
        'limitations': ['Unchanged generated contact probes, not source-supplied validated laboratory procedures',
            'A refused script stops early; attempting200 does not mean every reagent/activation was reached',
            'Matching observations establishes agreement with this pinned CLI execution, not scientific correctness'],
        'records_detail': reviewed}
    output.mkdir()
    (output / 'api-assessment.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2, allow_nan=False) + '\n')
    fields = ['rank', 'id', 'http_status', 'execution_status', 'exit_status', 'observations', 'execution_matches_cli',
        'observations_equal_cli', 'all_add_commands_reached', 'first_diagnostic']
    with (output / 'api-summary.csv').open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=fields, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(reviewed)
    print(json.dumps({k: v for k, v in summary.items() if k != 'records_detail'}, indent=2))


if __name__ == '__main__':
    main()
