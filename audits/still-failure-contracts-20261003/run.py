"""Frozen CLI contracts for precise, atomic binary still refusals."""
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
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''):
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

def near(actual, expected, tolerance=1e-8):
    return expected > 0 and math.isfinite(actual) and abs(actual / expected - 1) <= tolerance

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only while local disk headroom is unavailable')
    commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()
    subprocess.run([sys.executable,str(REPO/'audits/independent-next-50-20261002/run.py'),
        '--suite',str(SUITE),'--binary',str(args.binary),'--binary-source-commit',commit,
        '--out',str(args.out)],check=True)
    receipt = strict((args.out/'execution.json').read_text())
    cases = strict((SUITE/'predictions.json').read_text())
    checks = []
    classifications = {}
    def check(name, passed):
        checks.append(dict(name=name,passed=bool(passed)))
    check('frozen predictions',receipt['predictions_sha256']==digest(SUITE/'predictions.json'))
    check('binary identity',receipt['binary_source_commit']==commit and receipt['binary_sha256']==digest(args.binary))
    check('all cases in both modes',sorted((r['id'],r['mode']) for r in receipt['runs'])==sorted((c['id'],m) for c in cases for m in ['text','json']))
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
        check(ident+' complete output',len(rows)==7 and len(views)==2 and len(operations)==1)
        if len(views)!=2 or len(operations)!=1:
            continue
        before,after = views
        events = operations[0]['events']
        cuts = [e for e in events if e['event']=='distilled']
        if case['category']!='supported':
            refusals = [e for e in events if e['event']=='not_yet_modeled']
            check(ident+' one atomic refusal',not cuts and len(refusals)==1 and len(events)==1)
            check(ident+' entire vessels unchanged',before['bench']['vessels']==after['bench']['vessels'])
            if not refusals:
                continue
            event = refusals[0]
            reason = event.get('reason') or {}
            check(ident+' model boundary cause',event.get('cause')=='model-boundary')
            check(ident+' legacy key preserved',reason.get('key')=='not-modeled.no-complete-still-cut')
            check(ident+' zero transfer disclosed','No cut was transferred' in event.get('what',''))
            failure = next((slot.get('phrase',{}).get('phrase',{}) for name,slot in reason.get('slots',[]) if name=='failure'),{})
            code = failure.get('key','').removeprefix('not-modeled.still-failure.')
            classifications[ident] = code
            if case['category']=='exploratory':
                allowed = ['phase-evaluation','composition-precision','condensate-precision','residue-precision','energy-precision','integration-limit','incomplete-cut','request-precision']
                check(ident+' explicit explored failure owner',code in allowed)
            else:
                check(ident+' predicted failure owner',code==case['category'])
            check(ident+' readable cause agrees with JSON',bool(failure.get('en')) and failure['en'] in event.get('what',''))
            continue
        check(ident+' one completed cut',len(cuts)==1)
        check(ident+' no operation failure or refusal',not any(e['event'] in ['not_yet_modeled','solver_failed'] for e in events))
        for species in ['water','ethanol']:
            source = amount(after,0,species)
            received = amount(after,1,species)
            initial = amount(before,0,species)
            check(ident+' '+species+' positive separate stocks',source>0 and received>0)
            check(ident+' '+species+' relative budget',near(source+received,initial))
        water = amount(after,1,'water')
        ethanol = amount(after,1,'ethanol')
        original = amount(before,0,'water')+amount(before,0,'ethanol')
        check(ident+' complete fraction',near(water+ethanol,case['fraction']*original))
        if cuts:
            check(ident+' latent account',near(cuts[0]['energy_kj'],water*40.657+ethanol*38.58))
        if ident=='S06':
            check(ident+' staged enrichment',ethanol/(water+ethanol)>amount(before,0,'ethanol')/original)
    result = dict(checks=len(checks),passed=all(c['passed'] for c in checks),classifications=classifications,details=checks)
    (args.out/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='details'},indent=2))
    return 0 if result['passed'] else 1

if __name__=='__main__':
    sys.exit(main())
