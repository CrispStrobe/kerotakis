#!/usr/bin/env python3
"""Hosted paired comparison; retain every raw process and reject changed behavior."""
import argparse,hashlib,importlib.util,json,statistics,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
FORECAST_SHA='cbeb462995332fa55168887575301017807aff713780f97478aba95b4711c4ee'
spec=importlib.util.spec_from_file_location('evidence',ROOT/'tools/fifth-independent-50/run.py')
e=importlib.util.module_from_spec(spec);spec.loader.exec_module(e)
def records(path):return [json.loads(s) for s in path.read_text().splitlines() if s.strip()]
def equivalent(observed,path,reference):
    return (observed['exit_code']==0 and not observed['timeout'] and not observed['json_parse_errors']
            and observed['final_inspection'] and records(path/'stdout.ndjson')==reference)
def main():
    p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    fp=ROOT/'audits/startup-solid-index-20261008/comparison.json'
    if e.digest(fp)!=FORECAST_SHA:raise SystemExit('Frozen comparison changed')
    f=json.loads(fp.read_text());a.out.mkdir(parents=True,exist_ok=False)
    binaries={label:(a.build/label/'kero').resolve() for label in ['baseline','repair']}
    for label in binaries:
        assert (a.build/label/'source-commit.txt').read_text().strip()==f[label]
        assert (a.build/label/'generated-lock-sha256.txt').read_text()==(a.build/'baseline/generated-lock-sha256.txt').read_text()
    rows=[]
    for c in f['cases']:
        warm={label:e.execute(binary,c['script'],a.out/c['id']/label/'warm') for label,binary in binaries.items()}
        reference=records(a.out/c['id']/'baseline/warm/stdout.ndjson')
        good=all(equivalent(v,a.out/c['id']/label/'warm',reference) for label,v in warm.items())
        samples={label:[] for label in binaries}
        for i in range(f['samples']):
            for label in (['baseline','repair'] if i%2==0 else ['repair','baseline']):
                path=a.out/c['id']/label/f'native-{i}'
                sample=e.execute(binaries[label],c['script'],path)
                good=equivalent(sample,path,reference) and good
                samples[label].append(sample['seconds'])
        row=dict(id=c['id'],equivalent=good,samples_seconds=samples,notices={label:r['notices'] for label,r in warm.items()},
                 medians_seconds={label:statistics.median(v) if good else None for label,v in samples.items()})
        if c['id']=='F19-a':
            profiles={}
            for label,binary in binaries.items():
                path=a.out/c['id']/label;code=None;instructions=None
                try:
                    with (path/'callgrind.stdout.ndjson').open('wb') as out,(path/'callgrind.stderr.txt').open('wb') as err:
                        proc=subprocess.run(['valgrind','--tool=callgrind','--callgrind-out-file='+str(path/'callgrind.out'),str(binary),'run',str(path/'warm/input.lab'),'--json'],stdout=out,stderr=err,timeout=300)
                    code=proc.returncode
                    instructions=next(int(line.split()[1]) for line in (path/'callgrind.out').read_text().splitlines() if line.startswith('summary:'))
                    matches=code==0 and records(path/'callgrind.stdout.ndjson')==reference
                except (OSError,ValueError,StopIteration,subprocess.TimeoutExpired):matches=False
                profiles[label]=dict(exit_code=code,instructions=instructions,equivalent=matches)
            row['callgrind']=profiles
        rows.append(row);print(c['id'],'equivalent' if good else 'CHANGED',flush=True)
    report=dict(forecast_sha256=FORECAST_SHA,sources={k:f[k] for k in binaries},binary_sha256={k:e.digest(v) for k,v in binaries.items()},
                generated_lock_sha256=(a.build/'baseline/generated-lock-sha256.txt').read_text().strip(),results=rows,
                all_equivalent=all(r['equivalent'] for r in rows),
                limitations=['Whole CLI process, including startup, native initialization and serialization.','Seven interleaved samples per binary; no stable tail estimate.','Instruction counts are instrumentation measurements, separate from native timings.'])
    e.save(a.out/'comparison-results.json',report)
    return int(not report['all_equivalent'] or any(not p['equivalent'] for r in rows for p in r.get('callgrind',{}).values()))
if __name__=='__main__':sys.exit(main())
