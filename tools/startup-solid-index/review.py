#!/usr/bin/env python3
"""Revalidate the complete paired artifact without executing either binary."""
import argparse,hashlib,importlib.util,json,statistics,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/fifth-independent-50'))
import review_profile as archived
import run as evaluator
FORECAST_SHA='cbeb462995332fa55168887575301017807aff713780f97478aba95b4711c4ee'
def digest(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
def records(path):return [json.loads(s) for s in path.read_text().splitlines() if s.strip()]
def check_run(path,script,reference):
 r=archived.read_archived_run(path,script)
 assert r['exit_code']==0 and not r['timeout'] and not r['json_parse_errors'] and r['final_inspection']
 assert records(path/'stdout.ndjson')==reference,'Complete output changed'
 return r

def review(directory):
 fp=ROOT/'audits/startup-solid-index-20261008/comparison.json';assert digest(fp)==FORECAST_SHA
 f=json.loads(fp.read_text());build=directory/'paired-build';out=directory/'paired-comparison'
 original=json.loads((out/'comparison-results.json').read_text());assert original['all_equivalent']
 assert original['sources']=={label:f[label] for label in ['baseline','repair']}
 lock=digest(build/'baseline/Cargo.lock');binaries={};sources={}
 for label in ['baseline','repair']:
  p=build/label;assert (p/'source-commit.txt').read_text().strip()==f[label]
  assert (p/'source-tree.txt').read_text().strip()==git('rev-parse',f[label]+'^{tree}').decode().strip()
  assert not (p/'status-before.txt').read_text().strip()
  assert digest(p/'Cargo.lock')==lock==(p/'generated-lock-sha256.txt').read_text().strip()
  binaries[label]=digest(p/'kero');assert binaries[label]==original['binary_sha256'][label]
  assert binaries[label]==(p/'binary-sha256.txt').read_text().split()[0]
  assert '2 passed; 0 failed' in (p/'identity-tests.log').read_text()
  assert '10 passed; 0 failed' in (p/'derivation-tests.log').read_text()
  sources[label]=dict(commit=f[label],tree=(p/'source-tree.txt').read_text().strip(),
   evidence_sha256={name:digest(p/name) for name in ['Cargo.lock','submodules.txt','identity-tests.log','derivation-tests.log','build.log']})
 assert original['generated_lock_sha256']==lock
 assert (build/'harness/commit.txt').read_text().strip().startswith('8f863e65')
 for line in (build/'harness/sha256.txt').read_text().splitlines():
  sha,name=line.split(maxsplit=1);assert hashlib.sha256(git('show','8f863e65:'+name.lstrip('*'))).hexdigest()==sha
 rows=[]
 assert [r['id'] for r in original['results']]==[c['id'] for c in f['cases']]
 for c,old in zip(f['cases'],original['results']):
  reference=records(out/c['id']/'baseline/warm/stdout.ndjson');samples={};hashes={}
  for label in ['baseline','repair']:
   p=out/c['id']/label;check_run(p/'warm',c['script'],reference)
   runs=[check_run(p/f'native-{i}',c['script'],reference) for i in range(7)]
   samples[label]=[r['seconds'] for r in runs]
   hashes[label]=[digest(p/'warm/execution.json')]+[digest(p/f'native-{i}/execution.json') for i in range(7)]
   assert samples[label]==old['samples_seconds'][label]
   assert statistics.median(samples[label])==old['medians_seconds'][label]
  a,b=map(lambda l:statistics.median(samples[l]),['baseline','repair'])
  row=dict(id=c['id'],complete_output_equivalent=True,execution_sha256=hashes,samples_seconds=samples,
   median_seconds=dict(baseline=a,repair=b),observed_median_reduction_percent=100*(a-b)/a)
  if 'callgrind' in old:
   profiles={}
   for label in ['baseline','repair']:
    p=out/c['id']/label;assert old['callgrind'][label]['exit_code']==0
    assert records(p/'callgrind.stdout.ndjson')==reference
    with (p/'callgrind.out').open() as handle:
     count=next(int(line.split()[1]) for line in handle if line.startswith('summary:'))
    assert count==old['callgrind'][label]['instructions']
    profiles[label]=dict(instructions=count,files_sha256={name:digest(p/name) for name in ['callgrind.out','callgrind.stdout.ndjson','callgrind.stderr.txt']})
   row['callgrind']=profiles
   row['observed_instruction_reduction_percent']=100*(profiles['baseline']['instructions']-profiles['repair']['instructions'])/profiles['baseline']['instructions']
  rows.append(row)
 return dict(run=37775230032,forecast_sha256=FORECAST_SHA,source_bindings=sources,binary_sha256=binaries,generated_lock_sha256=lock,
  original_results_sha256=digest(out/'comparison-results.json'),all_outputs_equivalent=True,workloads=len(rows),native_processes=96,results=rows,
  limitations=['Single hosted runner; seven interleaved samples per binary do not establish stable tails or cross-machine gains.','Native wall time includes CLI startup, engine initialization and serialization.','Callgrind instruction counts are separate from native timings.','Source/tree and clean-status bindings cover manifests; explicit manifest hash files were not collected by the dispatched builder.'])
def main():
 p=argparse.ArgumentParser();p.add_argument('--artifact',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args()
 if a.artifact.resolve() in a.report.resolve().parents:raise SystemExit('Report must be outside preserved artifact')
 r=review(a.artifact)
 with a.report.open('x') as f:json.dump(r,f,indent=2,allow_nan=False);f.write('\n')
 print(json.dumps({k:r[k] for k in ['all_outputs_equivalent','workloads','native_processes']}))
 for row in r['results']:print(row['id'],round(row['observed_median_reduction_percent'],2),'% lower median',row.get('observed_instruction_reduction_percent'))
if __name__=='__main__':main()
