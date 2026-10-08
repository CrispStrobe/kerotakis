#!/usr/bin/env python3
"""Hosted-only CLI experiment runner. Expectations live in the preserved forecast."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import resource
import statistics
import subprocess
import sys
import time

EXPECTED_FORECAST = '4041d15bc70b108437bf6df909cbbf892f79d737b6ef6faae7f99bbde21a15f5'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2, ensure_ascii=False, allow_nan=False) + '\n')


def execute(binary, script, directory, timeout=45):
    directory.mkdir(parents=True, exist_ok=False)
    path = directory / 'input.lab'
    path.write_text(script)
    child_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.perf_counter()
    timed_out = False
    try:
        result = subprocess.run([str(binary), 'run', str(path), '--json'], capture_output=True, timeout=timeout)
        stdout, stderr, code = result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired as error:
        stdout, stderr, code = error.stdout or b'', error.stderr or b'', None
        timed_out = True
    seconds = time.perf_counter() - started
    child_after = resource.getrusage(resource.RUSAGE_CHILDREN)
    (directory / 'stdout.ndjson').write_bytes(stdout)
    (directory / 'stderr.txt').write_bytes(stderr)
    records, parse_errors = [], []
    for number, line in enumerate(stdout.decode(errors='replace').splitlines(), 1):
        if not line.strip():
            continue
        try:
            item = json.loads(line)
            if not isinstance(item, dict):
                raise ValueError('JSON record is not an object')
            records.append(item)
        except (ValueError, json.JSONDecodeError) as error:
            parse_errors.append({'line': number, 'error': str(error)})
    snapshots = [r['bench']['vessels'] for r in records if isinstance(r.get('bench'), dict) and isinstance(r['bench'].get('vessels'), list)]
    notices = [e for r in records for e in r.get('events', []) if isinstance(e, dict) and e.get('event') in ('not_yet_modeled', 'safety_veto')]
    final_operator = records[-1].get("operator") if records else None
    envelope = dict(exit_code=code, seconds=seconds, timeout=timed_out, json_parse_errors=parse_errors,
                    child_user_cpu_seconds=child_after.ru_utime - child_before.ru_utime,
                    child_system_cpu_seconds=child_after.ru_stime - child_before.ru_stime,
                    stdout_sha256=hashlib.sha256(stdout).hexdigest(), stderr_sha256=hashlib.sha256(stderr).hexdigest(),
                    input_sha256=digest(path), records=len(records), final_inspection=bool(isinstance(final_operator, dict) and final_operator.get('op') == 'inspect'))
    save(directory / 'execution.json', envelope)
    return dict(**envelope, vessels=snapshots[-1] if snapshots else [], notices=notices)


def value(spec, runs):
    if not isinstance(spec, dict):
        return spec
    kind = spec['kind']
    if kind == 'difference':
        return value(spec['left'], runs) - value(spec['right'], runs)
    if kind == 'sum':
        return math.fsum(value(s, runs) for s in spec['terms'])
    if kind == 'ratio':
        return value(spec['left'], runs) / value(spec['right'], runs)
    run = runs[spec['variant']]
    if kind == 'exit_code':
        if run['timeout']:
            raise ValueError('Timeout is not an expected CLI refusal')
        return run['exit_code']
    if kind == 'refusal':
        return bool(run['notices'])
    selected = run['vessels'] if spec['vessel'] == 0 else [v for v in run['vessels'] if v.get('id') == spec['vessel'] - 1]
    if not selected:
        raise ValueError('Required vessel snapshot absent')
    if kind in ('species', 'solid'):
        quantities = [p['moles'] for v in selected for p in v.get('contents', [])
                      if (p.get('species') == spec.get('species') if kind == 'species' else p.get('phase') == 'solid')]
        if any(not isinstance(q, (float, int)) or not math.isfinite(q) for q in quantities):
            raise ValueError('Invalid owned amount')
        return math.fsum(quantities)
    vessel = selected[0]
    if kind == 'state':
        return vessel
    if kind == 'ph':
        solution = vessel.get('solution') or (vessel.get('resolved') or {}).get('solution') or {}
        result = solution.get('ph')
    else:
        result = vessel.get(kind)
    if not isinstance(result, (float, int)) or not math.isfinite(result):
        raise ValueError(f'Missing finite {kind} observable')
    return result


def evaluate(check, runs):
    try:
        left, right = value(check['lhs'], runs), value(check['rhs'], runs)
        op = check['op']
        if op == 'near':
            passed = math.isfinite(left) and math.isfinite(right) and abs(left - right) <= check['atol'] + check['rtol'] * abs(right)
        elif op in ('eq', 'equal'):
            passed = left == right
        elif op == 'ne':
            passed = left != right
        elif op == 'gt':
            passed = left > right
        elif op == 'lt':
            passed = left < right
        elif op == 'between':
            passed = right[0] <= left <= right[1]
        else:
            raise ValueError(f'Unknown check operator {op}')
        return dict(passed=bool(passed), observed_left=left, observed_right=right, contract=check)
    except (KeyError, TypeError, ValueError, ZeroDivisionError) as error:
        return dict(passed=False, error=str(error), contract=check)


def outcome(case, runs, checks):
    if any(r['timeout'] for r in runs.values()):
        return 'timeout'
    if any(r['json_parse_errors'] for r in runs.values()):
        return 'harness_or_json_protocol_failure'
    if any(r['exit_code'] != 0 for r in runs.values()) and case['id'] != 'F49':
        return 'cli_error_requires_author_domain_or_engine_review'
    if not all(c['passed'] for c in checks):
        return 'unmet_expectation'
    if case['id'] in ('F48', 'F49', 'F50'):
        return 'expected_refusal'
    if any(r['notices'] for r in runs.values()):
        return 'qualified_agreement_with_model_notice'
    return 'passed'


def experiments(args, forecast, out):
    results = []
    for case in forecast['cases']:
        runs = {name: execute(args.binary, script, out / case['id'] / name) for name, script in case['variants'].items()}
        checks = [evaluate(c, runs) for c in case['checks']]
        item = dict(id=case['id'], title=case['title'], outcome=outcome(case, runs, checks), checks=checks,
                    variants={name: {k: v for k, v in run.items() if k != 'vessels'} for name, run in runs.items()})
        save(out / case['id'] / 'result.json', item)
        results.append(item)
        print(case['id'], item['outcome'], flush=True)
    counts = {name: sum(r['outcome'] == name for r in results) for name in sorted({r['outcome'] for r in results})}
    save(out / 'results.json', dict(source_commit=forecast['source_commit'], forecast_sha256=digest(args.forecast), binary_sha256=digest(args.binary), counts=counts, results=results))
    return int(any(r['outcome'] not in ('passed', 'expected_refusal') for r in results))


def profile(args, forecast, out):
    results = []
    for case in forecast['cases']:
        if case['id'] not in ('F19', 'F26', 'F34', 'F39', 'F45'):
            continue
        warm = {name: execute(args.binary, script, out / case['id'] / name / 'warm') for name, script in case['variants'].items()}
        checks = [evaluate(c, warm) for c in case['checks']]
        if outcome(case, warm, checks) not in ('passed', 'qualified_agreement_with_model_notice'):
            results.append(dict(id=case['id'], status='excluded_after_observed_behavior_failure', checks=checks))
            continue
        for name, script in case['variants'].items():
            samples = [execute(args.binary, script, out / case['id'] / name / f'native-{i}') for i in range(7)]
            good = all(r['exit_code'] == 0 and not r['timeout'] and not r['json_parse_errors'] for r in samples)
            times = [r['seconds'] for r in samples]
            row = dict(id=case['id'], variant=name, status='measured' if good else 'failed_sample', samples_seconds=times,
                       median_seconds=statistics.median(times), min_seconds=min(times), max_seconds=max(times),
                       child_user_cpu_seconds=[r['child_user_cpu_seconds'] for r in samples],
                       child_system_cpu_seconds=[r['child_system_cpu_seconds'] for r in samples],
                       p95_nearest_rank_seconds=sorted(times)[math.ceil(.95 * len(times)) - 1],
                       scope='Whole CLI process: startup, native engine setup, commands, JSON serialization and pipe collection; warm filesystem cache.')
            if case['id'] in ('F19', 'F39', 'F45') and name == 'a':
                directory = out / case['id'] / name
                command = ['valgrind', '--tool=callgrind', '--callgrind-out-file=' + str(directory / 'callgrind.out'), str(args.binary), 'run', str(directory / 'warm/input.lab'), '--json']
                try:
                    with (directory / 'callgrind.stdout.ndjson').open('wb') as stdout, (directory / 'callgrind.stderr.txt').open('wb') as stderr:
                        result = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=240)
                    row['callgrind_exit_code'] = result.returncode
                    annotated = subprocess.run(['callgrind_annotate', '--inclusive=yes', '--threshold=95', str(directory / 'callgrind.out')], capture_output=True, timeout=30)
                    (directory / 'callgrind-summary.txt').write_bytes(annotated.stdout + annotated.stderr)
                    row['callgrind_annotation_exit_code'] = annotated.returncode
                except (OSError, subprocess.TimeoutExpired) as error:
                    row['callgrind_error'] = str(error)
            results.append(row)
    save(out / 'performance.json', dict(source_commit=forecast['source_commit'], forecast_sha256=digest(args.forecast), binary_sha256=digest(args.binary),
         machine=dict(platform=platform.platform(), cpu_count=os.cpu_count()),
         limitations=['Provisional single-runner baseline; no speedup claim.', 'Callgrind counts/instrumented timings are not native wall time.', 'Seven samples do not establish a stable tail-latency estimate.', 'No solver-only timing claim; process startup is included.'], results=results))
    return int(not any(r.get('status') == 'measured' for r in results))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--forecast', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--mode', choices=['experiments', 'profile'], required=True)
    args = parser.parse_args()
    if digest(args.forecast) != EXPECTED_FORECAST:
        raise SystemExit('Frozen forecast hash mismatch')
    forecast = json.loads(args.forecast.read_text())
    assert len(forecast['cases']) == 50
    args.binary = args.binary.resolve()
    args.out.mkdir(parents=True, exist_ok=False)
    return experiments(args, forecast, args.out) if args.mode == 'experiments' else profile(args, forecast, args.out)


if __name__ == '__main__':
    sys.exit(main())
