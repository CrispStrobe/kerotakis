#!/usr/bin/env python3
"""Execute or independently recheck the frozen main-tree CLI replay."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools/fifth-independent-50'))
import run as evaluator
import review_profile as archived

FORECAST_SHA = '5c9eb5de22c2d4bcf9f11b3f62550083693af90e124779e8f1d000fc397fc2d5'


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def records(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def stage_checks(runs, paths):
    checks = {}
    checks['protocol'] = all(r['exit_code'] == 0 and not r['timeout'] and not r['json_parse_errors'] and r['final_inspection'] for r in runs.values())
    for name, stages in [('invalid', 129), ('valid', 1)]:
        steps = [r for r in records(paths[name] / 'stdout.ndjson') if (r.get('operator') or {}).get('op') == 'distil']
        checks[name + '_operator'] = len(steps) == 1 and steps[0]['operator']['stages'] == stages
        events = [e for r in steps for e in r.get('events', [])]
        refusals = [e for e in events if e.get('event') == 'not_yet_modeled']
        distilled = [e for e in events if e.get('event') == 'distilled']
        if name == 'invalid':
            checks['invalid_cause'] = len(refusals) == 1 and refusals[0].get('cause') == 'model-boundary' and (refusals[0].get('reason') or {}).get('key') == 'not-modeled.still-invalid-input' and not distilled
        else:
            checks['valid_accepted'] = not refusals and len(distilled) == 1
    checks['invalid_full_vessel_state_preserved'] = runs['blank']['vessels'] == runs['invalid']['vessels']
    for vessel, amount in [(1, 0.9), (2, 0.1)]:
        check = {'lhs': {'kind': 'species', 'variant': 'valid', 'vessel': vessel, 'species': 'water'}, 'op': 'near', 'rhs': amount, 'atol': 2e-7, 'rtol': 2e-7}
        checks[f'valid_water_v{vessel}'] = evaluator.evaluate(check, runs)['passed']
    return checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--build-binding', type=Path, required=True)
    parser.add_argument('--source-manifest', type=Path, default=ROOT / 'audits/combined-cli-replay-20261008/source.json')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--review', action='store_true')
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    forecast_path = ROOT / 'audits/main-cli-replay-20261008/forecast.json'
    assert digest(forecast_path) == FORECAST_SHA
    forecast = json.loads(forecast_path.read_text())
    source = json.loads(args.source_manifest.read_text())
    binding = json.loads(args.build_binding.read_text())
    assert source['forecast_sha256'] == FORECAST_SHA
    assert binding['compiled_source'] == source['compiled_source']
    assert binding['source_tree'] == source['source_tree']
    assert binding['source_kind'] == source['source_kind']
    if not args.review:
        assert args.binary and digest(args.binary) == binding['binary_sha256']
        args.out.mkdir(parents=True, exist_ok=False)
    else:
        assert args.report
    rows = []
    processes = 0
    for batch in forecast['sets']:
        for case in batch['cases']:
            runs = {}
            for name, script in case['variants'].items():
                path = args.out / batch['name'] / case['id'] / name
                runs[name] = archived.read_archived_run(path, script) if args.review else evaluator.execute(args.binary, script, path)
                processes += 1
            checks = [evaluator.evaluate(check, runs) for check in case['checks']]
            row = {'batch': batch['name'], 'id': case['id'], 'outcome': evaluator.outcome(case, runs, checks), 'checks': checks}
            rows.append(row)
            if not args.review:
                evaluator.save(args.out / batch['name'] / case['id'] / 'result.json', row)
                print(batch['name'], case['id'], row['outcome'], flush=True)
            else:
                assert json.loads((args.out / batch['name'] / case['id'] / 'result.json').read_text()) == row
    paths = {name: args.out / 'strict-stage' / name for name in forecast['strict_stage_control']['variants']}
    runs = {name: archived.read_archived_run(paths[name], script) if args.review else evaluator.execute(args.binary, script, paths[name]) for name, script in forecast['strict_stage_control']['variants'].items()}
    processes += len(runs)
    strict = stage_checks(runs, paths)
    counts = {batch: {outcome: sum(r['batch'] == batch and r['outcome'] == outcome for r in rows) for outcome in sorted({r['outcome'] for r in rows if r['batch'] == batch})} for batch in ['fifth', 'sixth']}
    report = {'forecast_sha256': FORECAST_SHA, 'compiled_source': binding['compiled_source'], 'source_tree': binding['source_tree'], 'source_kind': binding['source_kind'], 'binary_sha256': binding['binary_sha256'], 'build_binding_sha256': digest(args.build_binding), 'generated_lock_sha256': binding['generated_lock_sha256'], 'processes': processes, 'counts': counts, 'strict_stage_checks': strict, 'results': rows}
    assert len(rows) == source['expected_cases'] and processes == source['expected_processes']
    if args.review:
        assert json.loads((args.out / 'results.json').read_text()) == report
        with args.report.open('x') as output:
            json.dump(report, output, indent=2)
            output.write('\n')
    else:
        evaluator.save(args.out / 'results.json', report)
    print(json.dumps({'counts': counts, 'strict_stage_checks': strict, 'processes': processes}))
    accepted = {'passed', 'expected_refusal', 'qualified_agreement_with_model_notice'}
    return int(any(row['outcome'] not in accepted for row in rows) or not all(strict.values()))


if __name__ == '__main__':
    sys.exit(main())
