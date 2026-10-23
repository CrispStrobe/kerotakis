"""Recheck published focused outcomes and source bindings; no engine execution."""
from pathlib import Path
import hashlib,json,re,subprocess
B=Path(__file__).resolve().parent;R=B.parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
manifest=read(B/'SHA256.json');assert {str(p.relative_to(B)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in B.rglob('*') if p.is_file() and p.name!='SHA256.json'}==manifest
accepted=read(B/'acceptance.json');assert accepted['passed_controls']==5 and not accepted['full_fleet_platform_integration_and_product_accepted'] and accepted['scientific_selections']==0
assert accepted['phase_energy_absolute_tolerance_j']==0.001 and accepted['material_absolute_tolerance_mol']==1e-10
seen=set();passed=0
for group in accepted['groups']:
 d=B/group['name'];review=read(d/'parent-saved-review.json');binding=read(d/'binding-projection.json');completed=read(d/'completed.json');original=read(d/'original-producer-manifest.json')
 assert sha(d/'parent-saved-review.json')==group['parent_review_sha256'] and sha(d/'original-producer-manifest.json')==group['producer_manifest_sha256']
 assert binding['source']==review['source']==accepted['source']==completed['runtime_source'] and binding['tree']==review['tree']==accepted['tree']==completed['runtime_tree']
 assert sha(B/'actual-producer-Cargo.lock')==binding['actual_lock_sha256']==review['actual_lock_sha256']
 for path,row in binding['source_pins'].items():
  actual=subprocess.check_output(['git','show',binding['source']+':'+path],cwd=R)
  assert hashlib.sha256(actual).hexdigest()==row['sha256'],path
 expected={(row['target'],test) for row in binding['tracked_targets'] for test in row['tests']}
 assert len(expected)==group['passed'] and not(seen&expected);seen|=expected
 outcomes={(row['target'],row['test']):row for row in completed['actual_outcomes']};assert set(outcomes)==expected and completed['all_named_controls_collected']
 records=read(d/'command-projections.json');assert len(records)==len(expected)
 for record in records:
  key=(record['target'],record['test']);assert key in expected
  outcome=outcomes[key];assert outcome['exit_status']==record['exit_status']==0 and outcome['complete']
  assert record['exact_arguments_after_executable']==['--exact',record['test'],'--nocapture','--test-threads=1']
  k=record['record'];assert original[k+'.json']['sha256']==record['raw_command_record_sha256']
  for suffix in ['.stdout','.stderr']:
   f=d/(k+suffix);assert original[k+suffix]=={'sha256':sha(f),'bytes':f.stat().st_size}
   assert record[suffix[1:]+'_sha256']==sha(f)
  raw=(d/(k+'.stdout')).read_text();assert re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',raw,re.M)==[(record['test'],'ok')]
  listed=re.findall(r'^(\S+): test$',(d/(record['target']+'-list.stdout')).read_text(),re.M)
  assert tuple(map(int,re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',raw)[0]))==(1,0,0,0,len(listed)-1)
  meta=read(d/(record['target']+'-binary.json'));assert meta['sha256']==record['actual_test_executable_sha256']==review['actual_executables'][record['target']]['sha256']
  assert original[record['target']+'-binary.json']['sha256']==sha(d/(record['target']+'-binary.json'))
  passed+=1
assert passed==5
print(json.dumps({'verified_focused_controls':passed,'source':accepted['source'],'scientific_selections':0,'full_fleet_platform_and_product_acceptance':False,'no_engine_test_api_or_provider_execution':True}))
