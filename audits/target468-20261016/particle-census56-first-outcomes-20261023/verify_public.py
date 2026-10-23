"""Verify published projections and actual source IDs; no engine/archive execution."""
from pathlib import Path
import hashlib,json,re,subprocess,math
B=Path(__file__).resolve().parent;R=B.parents[2]
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
m=read(B/'SHA256.json');assert m=={str(p.relative_to(B)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in B.rglob('*') if p.is_file() and p.name!='SHA256.json'}
b=read(B/'binding-projection.json');original=read(B/'original-producer-manifest.json');done=read(B/'completed.json');review=read(B/'parent-small-review.json')
assert b['source']==done['runtime_source']==review['source']=='283c3bfcd01e5e6bf6527bc91c9d9de1f923062d'
assert subprocess.check_output(['git','--no-replace-objects','rev-parse',b['source']+'^{tree}'],cwd=R,text=True).strip()==b['tree']==done['runtime_tree']==review['tree']
assert b['actual_named_passes']==review['actual_observed_passes']==56 and not b['full_1536_sweep_passed'] and not b['full_source_platform_product_accepted'] and b['scientific_selections']==0
assert b['test_executable_archive_bodies_independently_rehashed'] is False and not review['local_archive_bodies_read']
assert sha(B/'actual-producer-Cargo.lock')==b['actual_lock_sha256']
if 'actual_lock_sha256' in review:assert review['actual_lock_sha256']==b['actual_lock_sha256']
assert sha(B/'original-producer-manifest.json')==review['manifest_sha256']
assert b['closed_raw_members']==len(original)+1==288
assert done['all_named_controls_collected'] and done['expected_named_controls']==56 and done['actual_target_count']==8
for public,raw in [('completed.json','COMPLETED.json'),('actual-producer-Cargo.lock','actual-producer-Cargo.lock'),('source-before.json','source-before.json'),('source-after.json','source-after.json')]:
 f=B/public;assert original[raw]=={'sha256':sha(f),'bytes':f.stat().st_size}
for path,row in b['source_pins'].items():assert hashlib.sha256(subprocess.check_output(['git','--no-replace-objects','show',b['source']+':'+path],cwd=R)).hexdigest()==row['sha256'],path
for label in ['source-before','source-after']:
 proof=read(B/(label+'.json'));assert proof['runtime_source']==b['source'] and proof['runtime_tree']==b['tree'] and proof['tracked_source_unchanged'] and proof['observer_overlays']=={}
 assert proof['actual_source_pins']=={k:v['sha256'] for k,v in b['source_pins'].items()}
 assert proof['actual_producer_lock_sha256']==b['actual_lock_sha256']
expected={(t['target'],n) for t in b['targets'] for n in t['tests']};outcomes={(r['target'],r['test']):r for r in done['actual_outcomes']};records=read(B/'test-command-projections.json')
assert len(expected)==len(outcomes)==len(done['actual_outcomes'])==len(records)==56 and set(outcomes)=={(r['target'],r['test']) for r in records}==expected
for record in records:
 name=record['target'];test=record['test'];k=record['record'];assert outcomes[(name,test)]['record']==k;assert record['exit_status']==outcomes[(name,test)]['exit_status']==0 and outcomes[(name,test)]['complete']
 assert record['arguments_after_executable']==['--exact',test,'--nocapture','--test-threads=1']
 assert original[k+'.json']['sha256']==record['original_command_sha256']
 for suffix in ['.stdout','.stderr']:
  f=B/(k+suffix);assert original[k+suffix]=={'sha256':sha(f),'bytes':f.stat().st_size} and sha(f)==record[suffix[1:]+'_sha256']
 raw=(B/(k+'.stdout')).read_text();assert re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',raw,re.M)==[(test,'ok')]
 listed=re.findall(r'^(\S+): test$',(B/(name+'-list.stdout')).read_text(),re.M)
 assert original[name+'-list.stdout']=={'sha256':sha(B/(name+'-list.stdout')),'bytes':(B/(name+'-list.stdout')).stat().st_size}
 assert tuple(map(int,re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',raw)[0]))==(1,0,0,0,len(listed)-1)
 meta=read(B/(name+'-binary.json'));assert meta['runtime_source']==b['source'] and meta['runtime_tree']==b['tree'] and meta['actual_producer_lock_sha256']==b['actual_lock_sha256']
 assert original[name+'-binary.json']['sha256']==sha(B/(name+'-binary.json')) and original[name+'.gz']=={'sha256':meta['gzip_sha256'],'bytes':meta['gzip_bytes']}
for record in read(B/'cli-command-projections.json'):
 k=record['record'];assert record['arguments_after_executable']==(['sweep'] if k=='full-1536-sweep' else ['run','audits/phase-particle-precision-20261023/corrected-cli-protocol.kero','--json']);assert original[k+'.json']['sha256']==record['original_command_sha256']
 for suffix in ['.stdout','.stderr']:
  f=B/(k+suffix);assert original[k+suffix]=={'sha256':sha(f),'bytes':f.stat().st_size} and sha(f)==record[suffix[1:]+'_sha256']
 assert record['exit_status']==(0 if k=='original-cycle-cli' else 1)
raw=(B/'full-1536-sweep.stdout').read_text();assert '1536 states — 1536 ran' in raw and '2 violation(s)' in raw
for case in ['Na2CO3 0.02mol','NaHCO3 0.05mol']:assert case in raw
rows=[json.loads(line) for line in (B/'original-cycle-cli.stdout').read_text().splitlines() if line.startswith('{')]
assert not [e for row in rows for e in row.get('events',[]) if e.get('event')=='solver_failed']
v=rows[-1]['bench']['vessels'][0];assert v['solution']['particle_molality']>0 and math.isfinite(v['temperature'])
last=next(e for row in reversed(rows) for e in reversed(row.get('events',[])) if e.get('event')=='state_changed' and e.get('species')=='water');assert abs(v['temperature']-last['at'])<=1e-6
print(json.dumps({'observed_named_passes':56,'exact_original_cli_converged':True,'full1536_sweep_passed':False,'full_sweep_failures':2,'public_projection_only':True,'native_test_archive_bodies_independently_rehashed':False,'full_source_and_product_accepted':False,'scientific_selections':0}))
