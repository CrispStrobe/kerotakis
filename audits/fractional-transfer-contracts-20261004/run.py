"""Hosted supplementary CLI controls for atomic fractional donor boundaries."""
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
    def number(text):
        value = float(text)
        if not math.isfinite(value):
            raise ValueError('nonfinite JSON')
        return value
    return json.loads(raw, parse_float=number,
        parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))

def stock(row, index, material):
    vessel = next(v for v in row['bench']['vessels'] if v['id'] == index)
    if material:
        return sum(p['amount'] for p in vessel['unresolved_materials'])
    return sum(p['moles'] for p in vessel['contents'] if p['species'] == 'water'
               and p['phase'] in ['liquid', 'aqueous'])

def near(actual, expected):
    return expected > 0 and math.isfinite(actual) and abs(actual / expected - 1) <= 1e-8

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only while local resource gates are closed')
    commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()
    subprocess.run([sys.executable,str(REPO/'audits/independent-next-50-20261002/run.py'),
        '--suite',str(SUITE/'cli'),'--binary',str(args.binary),
        '--binary-source-commit',commit,'--out',str(args.out)],check=True)
    receipt = strict((args.out/'execution.json').read_text())
    cases = strict((SUITE/'cli/predictions.json').read_text())
    checks = []
    def check(name, passed):
        checks.append(dict(name=name,passed=bool(passed)))
    check('frozen predictions',receipt['predictions_sha256']==digest(SUITE/'cli/predictions.json'))
    check('binary identity',receipt['binary_source_commit']==commit and receipt['binary_sha256']==digest(args.binary))
    check('complete case/mode coverage',sorted((r['id'],r['mode']) for r in receipt['runs'])==sorted((c['id'],m) for c in cases for m in ['text','json']))
    for run in receipt['runs']:
        case = next(c for c in cases if c['id']==run['id'])
        ident = run['id']+' '+run['mode']
        refusal = case['category']=='refusal'
        check(ident+' exit code',run['exit_code']==(1 if refusal else 0))
        for stream in ['stdout','stderr']:
            check(ident+' '+stream+' integrity',digest(args.out/run[stream])==run[stream+'_sha256'])
        if refusal:
            check(ident+' donor explanation','the source cannot represent the requested split' in (args.out/run['stderr']).read_text())
        if run['mode']!='json':
            continue
        rows = [strict(line) for line in (args.out/run['stdout']).read_text().splitlines()]
        views = [r for r in rows if r['operator']['op']=='inspect']
        transfers = [r for r in rows if r['operator']['op']==case['operator']]
        check(ident+' complete preflight output',len(views)==(1 if refusal else 2) and len(transfers)==(0 if refusal else 1))
        if refusal or len(views)!=2:
            continue
        before,after = views
        initial = stock(before,0,case['material'])
        received = stock(after,case['target'],case['material'])
        remaining = stock(after,0,case['material'])
        expected = initial*case['fraction']
        check(ident+' requested movement',near(received,expected))
        check(ident+' actual donor debit',near(initial-remaining,expected))
        check(ident+' requested residue',near(remaining,initial*(1-case['fraction'])))
        check(ident+' bulk conservation',near(remaining+received,initial))
    result = dict(checks=len(checks),passed=all(c['passed'] for c in checks),details=checks)
    (args.out/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='details'},indent=2))
    return 0 if result['passed'] else 1

if __name__=='__main__':
    sys.exit(main())
