"""Frozen CLI controls for zero-debit refusals and changing microscopic donors."""
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
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(65536), b''):
            h.update(block)
    return h.hexdigest()

def strict(raw):
    def number(value):
        result = float(value)
        if not math.isfinite(result):
            raise ValueError('nonfinite JSON')
        return result
    return json.loads(raw, parse_float=number,
        parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))

def amount(row, index, species):
    vessel = next(v for v in row['bench']['vessels'] if v['id'] == index)
    return sum(p['moles'] for p in vessel['contents'] if p['species'] == species)

def near(actual, expected, tolerance=1e-10):
    return expected > 0 and math.isfinite(actual) and abs(actual / expected - 1) <= tolerance

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only while local resource gates are closed')
    commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()
    subprocess.run([sys.executable,str(REPO/'audits/independent-next-50-20261002/run.py'),
        '--suite',str(SUITE),'--binary',str(args.binary),'--binary-source-commit',commit,
        '--out',str(args.out)],check=True)
    receipt = strict((args.out/'execution.json').read_text())
    cases = strict((SUITE/'predictions.json').read_text())
    checks = []
    def check(name, passed):
        checks.append(dict(name=name,passed=bool(passed)))
    check('frozen predictions',receipt['predictions_sha256']==digest(SUITE/'predictions.json'))
    check('binary identity',receipt['binary_source_commit']==commit and receipt['binary_sha256']==digest(args.binary))
    check('complete case/mode coverage',sorted((r['id'],r['mode']) for r in receipt['runs'])==sorted((c['id'],m) for c in cases for m in ['text','json']))
    for run in receipt['runs']:
        ident = run['id']
        case = next(c for c in cases if c['id']==ident)
        check(ident+' '+run['mode']+' completed',run['exit_code']==0)
        for stream in ['stdout','stderr']:
            check(ident+' '+run['mode']+' '+stream+' integrity',digest(args.out/run[stream])==run[stream+'_sha256'])
        if run['mode']!='json':
            continue
        rows = [strict(line) for line in (args.out/run['stdout']).read_text().splitlines()]
        views = [r for r in rows if r['operator']['op']=='inspect']
        operations = [r for r in rows if r['operator']['op']=='distil']
        check(ident+' complete output',len(rows)==6 and len(views)==2 and len(operations)==1)
        if len(views)!=2 or len(operations)!=1:
            continue
        before,after = views
        events = operations[0]['events']
        cuts = [e for e in events if e['event']=='distilled']
        if case['category']=='donor-refusal':
            refusal = [e for e in events if e['event']=='not_yet_modeled']
            check(ident+' one atomic refusal',not cuts and len(events)==1 and len(refusal)==1)
            check(ident+' complete physical vessels unchanged',before['bench']['vessels']==after['bench']['vessels'])
            if refusal:
                check(ident+' explicit donor cause',refusal[0].get('cause')=='model-boundary' and (refusal[0].get('reason') or {}).get('key')=='not-modeled.unrepresentable-still-donor')
                check(ident+' zero transfer disclosure','No cut was transferred' in refusal[0].get('what',''))
            continue
        check(ident+' one complete cut',len(cuts)==1)
        check(ident+' no solver failure or refusal',not any(e['event'] in ['not_yet_modeled','solver_failed'] for e in events))
        remaining = amount(after,0,case['species'])
        received = amount(after,1,case['species'])
        initial = amount(before,0,case['species'])
        check(ident+' supplying donor changes',0 < remaining < initial)
        check(ident+' complete requested amount',near(received,initial*case['fraction']))
        check(ident+' bulk component budget',near(remaining+received,initial,1e-12))
        if cuts:
            check(ident+' latent account',near(cuts[0]['energy_kj'],received*case['latent_kj']))
            reported = sum(q for species,q in cuts[0]['components'] if species==case['species'])
            check(ident+' event/receiver amount agree',near(reported,received))
    result = dict(checks=len(checks),passed=all(c['passed'] for c in checks),details=checks)
    (args.out/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='details'},indent=2))
    return 0 if result['passed'] else 1

if __name__=='__main__':
    sys.exit(main())
