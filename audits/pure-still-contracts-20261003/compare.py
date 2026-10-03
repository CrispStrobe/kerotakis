"""Hosted matched CLI timings with independent pure-cut material checks."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import time

SUITE = Path(__file__).resolve().parent
LATENT = {'water': 40.657, 'ethanol': 38.58, 'methanol': 35.244}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def strict(raw):
    def number(raw):
        value = float(raw)
        if not math.isfinite(value):
            raise ValueError('nonfinite JSON')
        return value
    return json.loads(raw, parse_float=number,
                      parse_constant=lambda raw: (_ for _ in ()).throw(ValueError(raw)))


def vessel(row, ident):
    return next(v for v in row['bench']['vessels'] if v['id'] == ident)


def amount(row, ident, species):
    return sum(p['moles'] for p in vessel(row, ident)['contents'] if p['species'] == species)


def relative(actual, expected, tolerance=2e-8):
    return actual == 0 if expected == 0 else abs(actual/expected-1) <= tolerance


def inspect_result(case, rows):
    checks = []

    def check(name, passed):
        checks.append(dict(name=name, passed=bool(passed)))

    views = [r for r in rows if r['operator']['op'] == 'inspect']
    check('two inspections', len(views) == 2)
    events = [e for row in rows for e in row['events']]
    check('no solver failure or refusal', not any(e['event'] in ['solver_failed', 'not_yet_modeled', 'refused'] for e in events))
    if len(views) != 2:
        return checks
    before, after = views
    cuts = [e for e in events if e['event'] == 'distilled']
    name = case['species']
    if case['kind'] == 'mixture':
        check('one complete mixed cut', len(cuts) == 1)
        for species in ['water', 'ethanol']:
            initial = amount(before, 0, species)
            check(species+' conserved', relative(amount(after, 0, species)+amount(after, 1, species), initial))
        water, ethanol = amount(after, 1, 'water'), amount(after, 1, 'ethanol')
        initial = amount(before, 0, 'water')+amount(before, 0, 'ethanol')
        check('complete fraction', relative(water+ethanol, .01*initial))
        check('trace enriched positively', water > 0 and ethanol > 0
              and ethanol/(water+ethanol) > amount(before, 0, 'ethanol')/initial)
        return checks
    pairs = [(0, 1)] if case['kind'] == 'profile' else [(0, 1), (2, 3)]
    if case['kind'] != 'zero':
        check('all cuts complete', len(cuts) == (case['count'] if case['kind'] == 'profile' else 2))
    for source, target in pairs:
        initial = amount(before, source, name)
        received = amount(after, target, name)
        left = amount(after, source, name)
        expected_fraction = {'fraction': .2, 'full': 1.0, 'zero': 0.0, 'tiny': 1e-14}.get(case['kind'])
        if case['kind'] == 'profile':
            expected_fraction = 1-.99**case['count']
        check(str(source)+' requested amount', relative(received, initial*expected_fraction))
        check(str(source)+' residue', relative(left, initial*(1-expected_fraction)))
        check(str(source)+' budget', relative(left+received, initial))
        check(str(target)+' pure condensate', all(p['species'] == name or p['moles'] == 0 for p in vessel(after, target)['contents']))
        pair_cuts = [e for e in cuts if e['from'] == source and e['to'] == target]
        heat = sum(e['energy_kj'] for e in pair_cuts)
        check(str(target)+' material/heat agreement', relative(heat, received*LATENT[name]))
        if expected_fraction > 0:
            check(str(target)+' positive heat and amount', heat > 0 and received > 0)
        for cut in pair_cuts:
            check('pure boiling temperature fixed', cut['at'] == cut['ended'])
            check('pure cut not azeotropic', cut['azeotropic'] is False)
        if case['kind'] == 'zero':
            for ident in [source, target]:
                for field in ['contents', 'temperature', 'pressure', 'headspace']:
                    check(str(ident)+' zero request preserves '+field,
                          vessel(before, ident)[field] == vessel(after, ident)[field])
    return checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--optimized', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted comparison only; do not bypass local resource gates')
    args.out.mkdir(parents=True, exist_ok=True)
    temp = args.out/'tmp'
    temp.mkdir(exist_ok=True)
    env = dict(os.environ, TMPDIR=str(temp), KERO_WORKERS='1')
    freeze = strict((SUITE/'freeze.json').read_text())
    cases = strict((SUITE/'predictions.json').read_text())
    assert digest(SUITE/'predictions.json') == freeze['predictions_sha256']
    assert len(cases) == freeze['count'] and len({c['id'] for c in cases}) == len(cases)
    executables = {}
    receipts = {}
    for label, folder in [('baseline', args.baseline), ('optimized', args.optimized)]:
        r = strict((folder/'validation.json').read_text())
        assert r['passed'] and all(s['exit_code'] == 0 for s in r['stages'])
        assert digest(folder/'kero') == r['binary_sha256']
        executables[label] = (folder/'kero').resolve()
        receipts[label] = r
        (args.out/(label+'-validation.json')).write_text(json.dumps(r, indent=2)+'\n')
    records = []

    def execute(case, label, suffix, timed):
        script = SUITE/'scripts'/(case['id']+'.lab')
        assert script.read_text() == case['script']
        prefix = case['id']+'.'+label+'.'+suffix
        output, errors = args.out/(prefix+'.stdout'), args.out/(prefix+'.stderr')
        started = time.perf_counter()
        with output.open('wb') as a, errors.open('wb') as b:
            try:
                result = subprocess.run([str(executables[label]), 'run', str(script), '--json'],
                                        stdout=a, stderr=b, env=env, timeout=180)
                code = result.returncode
            except subprocess.TimeoutExpired:
                code = 124
        elapsed = time.perf_counter()-started
        checks = [dict(name='process completed', passed=code == 0)]
        if code == 0:
            rows = [strict(line) for line in output.read_text().splitlines()]
            checks.extend(inspect_result(case, rows))
        records.append(dict(id=case['id'], variant=label, sample=suffix, timed=timed,
                            seconds=elapsed, exit_code=code, stdout=output.name, stderr=errors.name,
                            stdout_sha256=digest(output), stderr_sha256=digest(errors), checks=checks))
        (args.out/'execution.json').write_text(json.dumps(records, indent=2)+'\n')
        print(prefix, round(elapsed, 6), 'passed='+str(all(c['passed'] for c in checks)), flush=True)

    for case in cases:
        if case['kind'] != 'profile':
            execute(case, 'optimized', 'control', False)
            continue
        for label in ['baseline', 'optimized']:
            execute(case, label, 'warmup', False)
        for sample, label in enumerate(['baseline', 'optimized', 'optimized', 'baseline']*2):
            execute(case, label, str(sample), True)
    timings = []
    for case in cases:
        if case['kind'] == 'profile':
            samples = {label: [r['seconds'] for r in records if r['id'] == case['id']
                              and r['variant'] == label and r['timed']] for label in executables}
            medians = {label: statistics.median(values) for label, values in samples.items()}
            timings.append(dict(id=case['id'], samples=samples, medians=medians,
                                baseline_over_optimized=medians['baseline']/medians['optimized']))
    result = dict(passed=all(c['passed'] for r in records for c in r['checks']),
                  checks=sum(len(r['checks']) for r in records), invocation_count=len(records),
                  predictions_sha256=freeze['predictions_sha256'],
                  executables={label: dict(commit=r['commit'], sha256=r['binary_sha256']) for label, r in receipts.items()},
                  timing_scope='Validated debug CLI executables, same hosted runner, serial ABBA repetitions after warmup; no threshold assertion or release/browser throughput claim.',
                  timings=timings, failures=[dict(id=r['id'], variant=r['variant'], sample=r['sample'], check=c['name'])
                                           for r in records for c in r['checks'] if not c['passed']])
    (args.out/'comparison.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result['passed'] else 1)


if __name__ == '__main__':
    main()
