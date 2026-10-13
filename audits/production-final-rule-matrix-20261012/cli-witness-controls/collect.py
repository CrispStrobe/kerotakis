#!/usr/bin/env python3
"""Run frozen CLI discovery controls once and retain failures and route evidence."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def finite(value):
    if isinstance(value, float):
        assert math.isfinite(value), 'nonfinite output'
    elif isinstance(value, dict):
        for item in value.values():
            finite(item)
    elif isinstance(value, list):
        for item in value:
            finite(item)


def check(case, stdout, stderr, exit_code):
    problems = []
    if exit_code != case['expected_exit']:
        problems.append('exit differs from forecast')
    if stderr:
        problems.append('successful-run forecast has stderr')
    rows = []
    try:
        for line in stdout.splitlines():
            row = json.loads(line, parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))
            finite(row)
            assert isinstance(row, dict) and all(k in row for k in ['step', 'operator', 'events', 'bench'])
            assert isinstance(row['events'], list) and isinstance(row['bench']['vessels'], list)
            rows.append(row)
    except (ValueError, AssertionError, KeyError, TypeError) as exc:
        problems.append('invalid structured output: ' + str(exc))
    if len(rows) != case['expected_rows']:
        problems.append('row inventory differs from forecast')
    warnings = [e for r in rows for e in r['events'] if e.get('event') == 'hazard_warning' and e.get('rule') == case['target_rule']]
    if bool(warnings) != case['expected_target_warning']:
        problems.append('target warning differs from forecast')
    if any(e.get('severity') != case['expected_severity'] or not e.get('hazard') or not e.get('real_world') for e in warnings):
        problems.append('target warning metadata incomplete or severity differs')
    return dict(passed=not problems, unmet=problems, rows=len(rows), target_warnings=warnings,
                observed_events=[dict(row=i, operator=r['operator'], events=r['events']) for i, r in enumerate(rows)],
                final_bench=rows[-1]['bench'] if rows else None,
                final_callback_attribution='unresolved: warning presence alone does not establish diagnostic origin')


def gate(script):
    """Private operating policy stays outside the portable frozen collector."""
    import sys
    return json.loads(subprocess.check_output([sys.executable, str(script)]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--binary-sha256', required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--resource-gate', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    assert __debug__ and not args.out.exists()
    blob = (HERE / 'manifest.json').read_bytes()
    assert sha(blob) == args.manifest_sha256
    assert sha(args.binary.read_bytes()) == args.binary_sha256
    manifest = json.loads(blob)
    args.out.mkdir(parents=True)
    outcomes = []
    for case in manifest['cases']:
        folder = args.out / case['id']
        folder.mkdir()
        script = HERE / case['script']
        assert sha(script.read_bytes()) == case['script_sha256']
        argv = [str(args.binary.resolve()), 'run', str(script.resolve()), '--json']
        resources = gate(args.resource_gate)
        (folder / 'command.json').write_text(json.dumps(dict(argv=argv, cwd=str(args.source), resources=resources), indent=2) + '\n')
        (folder / 'input.lab').write_bytes(script.read_bytes())
        try:
            result = subprocess.run(argv, cwd=args.source, capture_output=True, timeout=manifest['timeout_seconds'])
            stdout, stderr, code = result.stdout, result.stderr, result.returncode
        except subprocess.TimeoutExpired as exc:
            stdout, stderr, code = exc.stdout or b'', exc.stderr or b'', None
        (folder / 'stdout.log').write_bytes(stdout)
        (folder / 'stderr.log').write_bytes(stderr)
        observation = check(case, stdout, stderr, code)
        observation.update(id=case['id'], exit=code, stdout_sha256=sha(stdout), stderr_sha256=sha(stderr))
        (folder / 'result.json').write_text(json.dumps(observation, indent=2) + '\n')
        outcomes.append(observation)
        (args.out / 'checkpoint.json').write_text(json.dumps(dict(completed=len(outcomes), outcomes=outcomes), indent=2) + '\n')
    assert sha(args.binary.read_bytes()) == args.binary_sha256
    summary = dict(head=manifest['head'], source_tree=manifest['source_tree'], binary_sha256=args.binary_sha256,
                   manifest_sha256=args.manifest_sha256, cases=len(outcomes), passed=sum(x['passed'] for x in outcomes),
                   unmet_cases=[x['id'] for x in outcomes if not x['passed']], outcomes=outcomes,
                   accepted_final_callback_coverage=False, accepted_empirical_accuracy=False)
    (args.out / 'result.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({k: v for k, v in summary.items() if k != 'outcomes'}))


if __name__ == '__main__':
    main()
