"""Verify all saved probe streams and report execution coverage, not chemistry passes.

Usage: assess_execution.py EXECUTIONS_DIRECTORY NEW_REPORT_DIRECTORY
The caller must authenticate the hosted artifact/run association before ingestion.
"""
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
import re
import sys


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    evidence, output = map(Path, sys.argv[1:])
    root = Path(__file__).resolve().parent
    binding = json.loads((root / 'execution-binding.json').read_text())
    assert (evidence / 'binding.json').read_bytes() == (root / 'execution-binding.json').read_bytes()
    freeze = json.loads((root / 'design/freeze.json').read_text())
    assert sha(root / 'design/freeze.json') == binding['design_freeze_sha256']
    for name, digest in freeze['files'].items():
        assert sha(root / 'design' / name) == digest, name
    manifest = json.loads((root / 'design/manifest.json').read_text())['records']
    records = json.loads((evidence / 'executions.json').read_text())
    complete = json.loads((evidence / 'completed.json').read_text())
    assert complete['executed_count'] == len(records) == len(manifest) == 200
    assert complete['binary_sha256'] == binding['binary_sha256']
    assert complete['executions_sha256'] == sha(evidence / 'executions.json')
    expected = {record['id']: record for record in manifest}
    assert set(expected) == {record['id'] for record in records} and len(expected) == 200
    report = []
    for record in records:
        design = expected[record['id']]
        assert record['rank'] == design['rank']
        assert record['script_sha256'] == freeze['files'][design['script']]
        stem = f"{record['rank']:03d}-{record['id']}"
        for stream in ['stdout', 'stderr']:
            assert sha(evidence / (stem + '.' + stream)) == record[stream + '_sha256']
        observations, invalid = [], 0
        for line in (evidence / (stem + '.stdout')).read_text(errors='replace').splitlines():
            try:
                row = json.loads(line)
                if not isinstance(row, dict):
                    raise ValueError('non-object observation')
                observations.append(row)
            except ValueError:
                invalid += 1
        assert len(observations) == record['parsed_observations'] and invalid == record['invalid_json_lines']
        script = (root / 'design' / design['script']).read_text()
        commands = [line for line in script.splitlines() if line and not line.startswith('#')]
        operators = [row.get('operator', {}) for row in observations]
        expected_adds = sum(command.startswith('add ') for command in commands)
        observed_adds = sum(operator.get('op') in ('add', 'add_material') for operator in operators)
        event_counts = Counter(event.get('event', 'missing_event_type')
                               for row in observations for event in row.get('events', []))
        stderr = (evidence / (stem + '.stderr')).read_text(errors='replace')
        first_error = stderr.splitlines()[0] if stderr.splitlines() else ''
        first_error = re.sub(r'^kero: .*\.lab:(\d+): ', r'line \1: ', first_error)
        report.append({'rank': design['rank'], 'id': design['id'],
            'execution_status': record['execution_status'], 'exit_status': record['exit_status'],
            'timed_out': record['timed_out'], 'elapsed_seconds': record['elapsed_seconds'],
            'expected_add_commands': expected_adds, 'observed_add_commands': observed_adds,
            'all_add_commands_reached': observed_adds == expected_adds,
            'activation_expected': design['activation_commands'],
            'activation_observed': [operator for operator in operators if operator.get('op') in ('ignite', 'heat', 'irradiate', 'electrolyse')],
            'observation_count': len(observations), 'invalid_json_lines': invalid,
            'last_completed_operator': operators[-1] if operators else None,
            'event_counts': dict(event_counts), 'first_diagnostic': first_error[:600],
            'physical_verdict': 'unassessed; source supplies no procedure',
            'protocol_limitations': design['limitations'],
            'script_sha256': record['script_sha256'],
            'stdout_sha256': record['stdout_sha256'], 'stderr_sha256': record['stderr_sha256']})
    report.sort(key=lambda record: record['rank'])
    summary = {'schema_version': 1, 'source_head': binding['source_head'],
        'binary_sha256': binding['binary_sha256'], 'attempted_reaction_records': 200,
        'execution_counts': dict(Counter(record['execution_status'] for record in report)),
        'all_add_commands_reached': sum(record['all_add_commands_reached'] for record in report),
        'physical_pass_count': 'not_assigned',
        'limits': ['A script stops at its first refusal; 200 attempts do not mean every reagent/activation was reached',
                   'Zero exit is completed execution, not reaction validation',
                   'Review declared solvent, headspace air, activation, symbolic and catalyst limitations per case',
                   'Read protocol-review.json for independent condition-incompatibility and surrogate notes'],
        'records': report}
    output.mkdir()
    (output / 'execution-assessment.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')
    fields = ['rank', 'id', 'execution_status', 'exit_status', 'timed_out', 'all_add_commands_reached', 'first_diagnostic']
    with (output / 'execution-summary.csv').open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=fields, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(report)
    print(json.dumps({key: value for key, value in summary.items() if key != 'records'}, indent=2))


if __name__ == '__main__':
    main()
