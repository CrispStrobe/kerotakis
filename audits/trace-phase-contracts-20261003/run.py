"""Hosted frozen CLI controls for trace preservation and atomic refusal."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys

SUITE = Path(__file__).resolve().parent
REPO = SUITE.parents[1]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def strict(raw):
    def number(value):
        result = float(value)
        if not math.isfinite(result):
            raise ValueError('nonfinite JSON')
        return result
    return json.loads(raw, parse_float=number,
        parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))

def amount(row, vessel, species):
    v = next(v for v in row['bench']['vessels'] if v['id'] == vessel)
    return sum(p['moles'] for p in v['contents'] if p['species'] == species)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only while local resource headroom is unavailable')
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    subprocess.run([sys.executable, str(REPO/'audits/independent-next-50-20261002/run.py'),
        '--suite', str(SUITE), '--binary', str(args.binary), '--binary-source-commit', commit,
        '--out', str(args.out)], check=True)
    receipt = strict((args.out/'execution.json').read_text())
    cases = strict((SUITE/'predictions.json').read_text())
    checks = []
    def check(name, passed):
        checks.append(dict(name=name, passed=bool(passed)))
    check('frozen predictions', receipt['predictions_sha256'] == digest(SUITE/'predictions.json'))
    check('binary identity', receipt['binary_source_commit'] == commit
        and receipt['binary_sha256'] == digest(args.binary))
    check('all controls in both modes', sorted((r['id'],r['mode']) for r in receipt['runs'])
        == sorted((c['id'],mode) for c in cases for mode in ['text','json']))
    for record in receipt['runs']:
        ident = record['id']
        check(ident+' '+record['mode']+' completed', record['exit_code'] == 0)
        for stream in ['stdout','stderr']:
            check(ident+' '+record['mode']+' '+stream+' integrity',
                digest(args.out/record[stream]) == record[stream+'_sha256'])
        if record['mode'] != 'json':
            continue
        rows = [strict(line) for line in (args.out/record['stdout']).read_text().splitlines()]
        views = [r for r in rows if r['operator']['op'] == 'inspect']
        check(ident+' two inspections', len(views) == 2)
        distillation = [r for r in rows if r['operator']['op'] == 'distil']
        check(ident+' one operation', len(distillation) == 1)
        if len(views) != 2 or len(distillation) != 1:
            continue
        before, after = views
        events = distillation[0]['events']
        cuts = [e for e in events if e['event'] == 'distilled']
        if ident in ['T01','T02']:
            check(ident+' no completed cut', not cuts)
            check(ident+' explicit atomic refusal', any(e['event'] == 'not_yet_modeled'
                and 'No cut was transferred' in e.get('what','') for e in events))
            check(ident+' entire vessel state preserved',
                before['bench']['vessels'] == after['bench']['vessels'])
            continue
        check(ident+' no failure or refusal anywhere', not any(e['event'] in
            ['solver_failed','not_yet_modeled','refused'] for r in rows for e in r['events']))
        check(ident+' completed cut', len(cuts) == 1)
        if len(cuts) != 1:
            continue
        names = ['water','methanol'] if ident == 'T03' else ['ethanol']
        for name in names:
            initial = amount(before,0,name)
            check(ident+' '+name+' conserved', abs((amount(after,0,name)
                + amount(after,1,name))/initial-1) < 1e-8)
            check(ident+' '+name+' positive overhead', amount(after,1,name) > 0)
        received = sum(amount(after,1,name) for name in names)
        initial = sum(amount(before,0,name) for name in names)
        check(ident+' requested fraction', abs(received/(initial*0.01)-1) < 1e-8)
        if ident == 'T03':
            check(ident+' trace enrichment', amount(after,1,'methanol')/received
                > amount(before,0,'methanol')/initial)
        else:
            check(ident+' analytic latent heat', abs(cuts[0]['energy_kj']/0.3858-1) < 1e-10)
    result = dict(checks=len(checks), passed=all(c['passed'] for c in checks), details=checks)
    (args.out/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(checks=result['checks'],passed=result['passed'],
        failures=[c for c in checks if not c['passed']]),indent=2))
    return 0 if result['passed'] else 1

if __name__ == '__main__':
    sys.exit(main())
