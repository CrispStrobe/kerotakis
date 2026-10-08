#!/usr/bin/env python3
"""Verify and re-evaluate archived sixth-fifty evidence without running the app."""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('sixth_binding',Path(__file__).with_name('run.py'))
binding=importlib.util.module_from_spec(spec);spec.loader.exec_module(binding)
SHARED_SHA='68938926231c1958ac9a17eca65b2407afd13ee9122f74b4f4921721caf637e8'
path=ROOT/'tools/fifth-independent-50/run.py'
if binding.digest(path)!=SHARED_SHA: raise RuntimeError('Dispatched evaluator source changed')
spec=importlib.util.spec_from_file_location('frozen_evaluator',path)
evaluator=importlib.util.module_from_spec(spec);spec.loader.exec_module(evaluator)
def archived(directory,script):
    envelope=json.loads((directory/'execution.json').read_text())
    for name,field in [('input.lab','input_sha256'),('stdout.ndjson','stdout_sha256'),('stderr.txt','stderr_sha256')]:
        if binding.digest(directory/name)!=envelope[field]: raise ValueError(f'{name} hash mismatch')
    if envelope['input_sha256']!=hashlib.sha256(script.encode()).hexdigest(): raise ValueError('Input differs from frozen script')
    parsed=evaluator.decode_output((directory/'stdout.ndjson').read_bytes())
    for key in ['records','json_parse_errors','final_inspection']:
        if parsed[key]!=envelope[key]: raise ValueError(f'{key} differs from raw output')
    if not isinstance(envelope['timeout'],bool): raise ValueError('Invalid timeout flag')
    code=envelope['exit_code']
    if not (type(code) is int or code is None): raise ValueError('Invalid exit code')
    if code is None and not envelope['timeout']: raise ValueError('Missing exit without timeout')
    for key in ['seconds','child_user_cpu_seconds','child_system_cpu_seconds']:
        if not math.isfinite(envelope[key]) or envelope[key]<0: raise ValueError('Invalid process timing')
    return dict(envelope,**parsed)
def review(forecast,directory):
    original=json.loads((directory/'results.json').read_text())
    expected=dict(source_commit=forecast['source_commit'],forecast_sha256=binding.FORECAST_SHA,binary_sha256=binding.BINARY_SHA)
    for k,v in expected.items():
        if original.get(k)!=v: raise ValueError(f'{k} binding mismatch')
    if [r['id'] for r in original['results']] != [c['id'] for c in forecast['cases']]: raise ValueError('Missing, reordered or duplicate cases')
    rows=[]
    for case,recorded in zip(forecast['cases'],original['results']):
        runs={name:archived(directory/case['id']/name,script) for name,script in case['variants'].items()}
        checks=[evaluator.evaluate(c,runs) for c in case['checks']]
        outcome=evaluator.outcome(case,runs,checks)
        rebuilt=dict(id=case['id'],title=case['title'],outcome=outcome,checks=checks,
                     variants={name:{k:v for k,v in r.items() if k!='vessels'} for name,r in runs.items()})
        if rebuilt!=recorded: raise ValueError(f"{case['id']} recorded results differ from raw re-evaluation")
        individual=directory/case['id']/'result.json'
        if json.loads(individual.read_text())!=rebuilt: raise ValueError('Individual result differs from aggregate')
        rows.append(dict(id=case['id'],outcome=outcome,checks=checks,
                         execution_sha256={name:binding.digest(directory/case['id']/name/'execution.json') for name in runs}))
    counts={name:sum(r['outcome']==name for r in rows) for name in sorted({r['outcome'] for r in rows})}
    if counts!=original['counts']: raise ValueError('Outcome counts mismatch')
    return dict(**expected,integrity_verified=True,cases=len(rows),cli_processes=sum(len(c['variants']) for c in forecast['cases']),
                counts=counts,ordinary_50_of_50=counts=={'passed':50},results_sha256=binding.digest(directory/'results.json'),
                limitations=['Offline re-evaluation, no new application execution.',
                             'Execution envelopes supply exit codes and timings; input/stdout/stderr are individually hash checked.',
                             'Model qualifications remain distinct from ordinary passes.',
                             'Production source and executable must also match the independently preserved build receipt.'],results=rows)
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--forecast',type=Path,required=True);p.add_argument('--out-dir',type=Path,required=True);p.add_argument('--report',type=Path,required=True)
    a=p.parse_args()
    if binding.digest(a.forecast)!=binding.FORECAST_SHA: raise SystemExit('Frozen forecast hash mismatch')
    if a.out_dir.resolve() in a.report.resolve().parents: raise SystemExit('Report must be outside the preserved artifact')
    report=review(json.loads(a.forecast.read_text()),a.out_dir)
    with a.report.open('x') as f: json.dump(report,f,indent=2,allow_nan=False);f.write('\n')
    print(json.dumps({k:report[k] for k in ['integrity_verified','cases','cli_processes','counts','ordinary_50_of_50']}))
    return int(not report['ordinary_50_of_50'])
if __name__=='__main__':sys.exit(main())
