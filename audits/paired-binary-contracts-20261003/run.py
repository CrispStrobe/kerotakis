"""Replay frozen paired-binary CLI controls with trace-relative accounting."""
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

def near(actual, expected, tolerance=1e-8):
    return math.isfinite(actual) and expected > 0 and abs(actual/expected-1) <= tolerance

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only while local resource headroom is unavailable')
    commit = subprocess.check_output(['git','rev-parse','HEAD'], cwd=REPO,text=True).strip()
    subprocess.run([sys.executable,str(REPO/'audits/independent-next-50-20261002/run.py'),
        '--suite',str(SUITE),'--binary',str(args.binary),'--binary-source-commit',commit,
        '--out',str(args.out)], check=True)
    receipt = strict((args.out/'execution.json').read_text())
    cases = strict((SUITE/'predictions.json').read_text())
    checks = []
    def check(name, passed):
        checks.append(dict(name=name,passed=bool(passed)))
    check('frozen predictions',receipt['predictions_sha256']==digest(SUITE/'predictions.json'))
    check('binary identity',receipt['binary_source_commit']==commit
        and receipt['binary_sha256']==digest(args.binary))
    check('all controls in both modes',sorted((r['id'],r['mode']) for r in receipt['runs'])
        ==sorted((c['id'],mode) for c in cases for mode in ['text','json']))
    for record in receipt['runs']:
        ident=record['id']
        check(ident+' '+record['mode']+' completed',record['exit_code']==0)
        for stream in ['stdout','stderr']:
            check(ident+' '+record['mode']+' '+stream+' integrity',
                digest(args.out/record[stream])==record[stream+'_sha256'])
        if record['mode']!='json':
            continue
        rows=[strict(line) for line in (args.out/record['stdout']).read_text().splitlines()]
        views=[r for r in rows if r['operator']['op']=='inspect']
        operations=[r for r in rows if r['operator']['op']=='distil']
        check(ident+' two inspections',len(views)==2)
        check(ident+' one operation',len(operations)==1)
        if len(views)!=2 or len(operations)!=1:
            continue
        before,after=views
        events=operations[0]['events']
        cuts=[e for e in events if e['event']=='distilled']
        if ident in ['B04','B05','B07','B09']:
            check(ident+' no completed cut',not cuts)
            check(ident+' explicit receiver refusal',any(e['event']=='not_yet_modeled'
                and 'receiver' in e.get('what','') and 'No cut was transferred' in e.get('what','')
                for e in events))
            check(ident+' complete vessel state preserved',
                before['bench']['vessels']==after['bench']['vessels'])
            continue
        check(ident+' no failure or refusal anywhere',not any(e['event'] in
            ['solver_failed','not_yet_modeled','refused'] for r in rows for e in r['events']))
        check(ident+' one completed cut',len(cuts)==1)
        if len(cuts)!=1:
            continue
        names=['ethanol'] if ident=='B06' else ['water'] if ident=='B08' else ['water','ethanol']
        received={name:amount(after,1,name)-amount(before,1,name) for name in names}
        for name in names:
            initial=amount(before,0,name)+amount(before,1,name)
            check(ident+' '+name+' trace-relative budget',near(
                amount(after,0,name)+amount(after,1,name),initial))
            check(ident+' '+name+' positive received increment',received[name]>0)
        cut=cuts[0]
        latent=sum(received[name]*(40.657 if name=='water' else 38.58) for name in names)
        check(ident+' latent account matches received material',near(cut['energy_kj'],latent))
        if ident=='B02':
            check(ident+' requested positive heat completed',near(cut['energy_kj'],0.1,1e-10))
        else:
            fraction=1.0 if ident=='B08' else 0.01
            initial=sum(amount(before,0,name) for name in names)
            check(ident+' fraction completed',near(sum(received.values()),initial*fraction))
        if ident in ['B01','B02','B03']:
            check(ident+' water cut has a positive residue',amount(after,0,'water')>0)
            check(ident+' water event matches explicit receiver trace',near(cut['water'],received['water']))
        if ident=='B06':
            check(ident+' analytic pure heat',near(cut['energy_kj'],0.3858,1e-10))
        if ident=='B08':
            check(ident+' donor fully emptied',amount(after,0,'water')==0)
    result=dict(checks=len(checks),passed=all(c['passed'] for c in checks),details=checks)
    (args.out/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(checks=result['checks'],passed=result['passed'],
        failures=[c for c in checks if not c['passed']]),indent=2))
    return 0 if result['passed'] else 1

if __name__=='__main__':
    sys.exit(main())
