#!/usr/bin/env python3
"""Recheck archived profile evidence; never executes the app or changes inputs."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics

import run as runner

ACCEPTED = ('passed', 'qualified_agreement_with_model_notice')


def read_archived_run(directory, script):
    envelope = json.loads((directory / 'execution.json').read_text())
    for name, field in [('input.lab', 'input_sha256'), ('stdout.ndjson', 'stdout_sha256'), ('stderr.txt', 'stderr_sha256')]:
        if runner.digest(directory / name) != envelope[field]:
            raise ValueError(f'Archived {name} hash mismatch')
    if envelope['input_sha256'] != hashlib.sha256(script.encode()).hexdigest():
        raise ValueError('Archived input differs from frozen forecast')
    observed = runner.decode_output((directory / 'stdout.ndjson').read_bytes())
    for field in ('records', 'json_parse_errors', 'final_inspection'):
        if observed[field] != envelope[field]:
            raise ValueError(f'Archived {field} disagrees with raw output')
    for field in ('seconds', 'child_user_cpu_seconds', 'child_system_cpu_seconds'):
        if not math.isfinite(envelope[field]) or envelope[field] < 0:
            raise ValueError(f'Invalid archived {field}')
    return dict(envelope, **observed)


def review(forecast, directory):
    original = json.loads((directory / 'performance.json').read_text())
    if original['source_commit'] != forecast['source_commit'] or original['forecast_sha256'] != runner.EXPECTED_FORECAST:
        raise ValueError('Original profile source/forecast binding mismatch')
    if not isinstance(original['binary_sha256'], str) or len(original['binary_sha256']) != 64:
        raise ValueError('Original binary binding absent')
    rows = []
    for case in forecast['cases']:
        if case['id'] not in ('F19', 'F26', 'F34', 'F39', 'F45'):
            continue
        try:
            warm = {name: read_archived_run(directory / case['id'] / name / 'warm', script)
                    for name, script in case['variants'].items()}
            checks = [runner.evaluate(c, warm) for c in case['checks']]
            warm_outcome = runner.outcome(case, warm, checks)
            if warm_outcome not in ACCEPTED:
                rows.append(dict(id=case['id'], status='excluded_after_observed_behavior_failure', warm_outcome=warm_outcome, checks=checks))
                continue
            for name, script in case['variants'].items():
                path = directory / case['id'] / name
                samples = [read_archived_run(path / f'native-{i}', script) for i in range(7)]
                validations = runner.validate_samples(case, warm, name, samples)
                accepted = all(v['outcome'] in ACCEPTED for v in validations)
                originals = [r for r in original['results'] if r['id'] == case['id'] and r.get('variant') == name]
                if len(originals) != 1:
                    raise ValueError('Missing or duplicate original profile row')
                old = originals[0]
                times = [sample['seconds'] for sample in samples]
                if old.get('samples_seconds') != times:
                    raise ValueError('Original sample timings disagree with execution envelopes')
                row = dict(id=case['id'], variant=name, status='measured' if accepted else 'failed_sample',
                           original_status=old['status'], warm_outcome=warm_outcome, validations=validations,
                           samples_seconds=times, median_seconds=statistics.median(times) if accepted else None,
                           execution_sha256=[runner.digest(path / f'native-{i}' / 'execution.json') for i in range(7)])
                if case['id'] in ('F19', 'F39', 'F45') and name == 'a':
                    # The dispatched harness stores profiler exits in performance.json,
                    # and raw instrumented output beside the native samples.
                    instrumented = dict(warm[name], exit_code=old.get('callgrind_exit_code'), timeout=bool(old.get('callgrind_error')))
                    instrumented.update(runner.decode_output((path / 'callgrind.stdout.ndjson').read_bytes()))
                    validation = runner.validate_samples(case, warm, name, [instrumented])[0]
                    profile_ok = (validation['outcome'] in ACCEPTED
                                  and old.get('callgrind_annotation_exit_code') == 0
                                  and (path / 'callgrind.out').stat().st_size > 0
                                  and (path / 'callgrind-summary.txt').stat().st_size > 0)
                    row.update(callgrind_status='passed' if profile_ok else 'failed', callgrind_validation=validation,
                               callgrind_sha256={file: runner.digest(path / file) for file in
                                                ('callgrind.out', 'callgrind-summary.txt', 'callgrind.stdout.ndjson', 'callgrind.stderr.txt')})
                rows.append(row)
        except (OSError, KeyError, ValueError, TypeError) as error:
            rows.append(dict(id=case['id'], status='failed_sample', error=str(error)))
    return dict(source_commit=original['source_commit'], forecast_sha256=original['forecast_sha256'],
                binary_sha256=original['binary_sha256'], original_performance_sha256=runner.digest(directory / 'performance.json'),
                limitations=['Offline validation of archived evidence; no app execution or new measurements.',
                             'Original execution envelopes provide exit codes and timings, bound here by hashes.',
                             'Checks compare each sample to the other warm variants; model notices remain qualified.',
                             'Original whole-process and seven-sample limitations still apply; no speedup claim.'],
                failed=runner.profile_failed(rows), results=rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--forecast', type=Path, required=True)
    parser.add_argument('--profile-dir', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if runner.digest(args.forecast) != runner.EXPECTED_FORECAST:
        raise SystemExit('Frozen forecast hash mismatch')
    if args.profile_dir.resolve() in args.report.resolve().parents:
        raise SystemExit('Write the review outside the preserved artifact directory')
    report = review(json.loads(args.forecast.read_text()), args.profile_dir)
    with args.report.open('x') as out:
        json.dump(report, out, indent=2, allow_nan=False)
        out.write('\n')
    return int(report['failed'])


if __name__ == '__main__':
    raise SystemExit(main())
