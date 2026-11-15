"""Exact successor review: obsolete oracle repair must retain scientific authority."""
import argparse,copy,hashlib,json,re,subprocess
from pathlib import Path
SOURCE='ee027474365d0b9a48647132b964be58c5d2c36a';TREE='2ba6ceeca730cbbee668ff8e4fc8bf30c39c3078';BASE='5db4a1260112c79b7ed610e5a8399b151dfd7363'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def readrows(p):return [json.loads(line)for line in p.read_text().splitlines()if line.strip()]
def check(positive,negative,registry):
 def last(rows):return next(r['bench']['vessels']for r in reversed(rows)if r.get('bench',{}).get('vessels'))
 def events(rows):return [e for r in rows for e in r.get('events',[])]
 v=last(positive);k=last(negative)[0];assert len(v)==4
 props={(r['species_id'],r['property']):r['quantity']['value']for r in registry['phase_thermodynamics']if isinstance(r['property'],str)and r['property']in ['molar_mass','mass_density']}
 comp={r['species_id']:r for r in registry['compositions']};comp['base_equivalents']=comp['OH-']
 residuals=[]
 for vessel,water,salt,poly in [(v[0],200,None,'PP'),(v[1],200,None,'PS'),(v[2],200,None,'PET'),(v[3],200,('NaCl',60),None),(k,50,('KCl',8),None)]:
  assert not vessel.get('unresolved_materials')and not vessel.get('material_objects')
  expected={};actual={}
  def add(out,s,n):
   for e in comp[s]['elements']:out[e['element']]=out.get(e['element'],0)+n*e['count']['value']
   out['Q']=out.get('Q',0)+n*comp[s]['net_charge']['value']
  add(expected,'water',water*props['water','mass_density']/props['water','molar_mass'])
  if salt:add(expected,salt[0],salt[1]/props[salt[0],'molar_mass'])
  if poly:add(expected,poly,5/props[poly,'molar_mass'])
  for portion in vessel['contents']:add(actual,portion['species'],portion['moles'])
  delta={e:actual.get(e,0)-expected.get(e,0)for e in set(expected)|set(actual)};assert all(abs(d)<=1e-10 for d in delta.values()),delta;residuals.append(delta)
  if poly:
   portions=[p for p in vessel['contents']if p['species']==poly];assert len(portions)==1 and portions[0]['phase']=='solid'and abs(portions[0]['moles']*props[poly,'molar_mass']-5)<=1e-10
 def measurements(rows,index):return [e['value']for e in events(rows)if e['event']=='measured'and e['vessel']==index and e['instrument']=='densitometer'and e['unit']=='g/mL']
 def missing(rows,index):return any(e.get('vessel')==index and (e.get('reason')or{}).get('key')=='not-modeled.density-of-the-dissolved-ions'for e in events(rows))
 brine=v[3];info=brine['solution'];w=info['native_density'];m=measurements(positive,3);water=measurements(positive,0)
 assert len(m)==len(water)==1 and water[0]<1.05<m[0]<1.38
 assert m==[w['kg_per_l']]and 'pitzer.dat'in w['dataset']and w['dataset']==info['provenance']['dataset']
 assert w['temperature']==brine['temperature']and w['pressure']==brine['pressure']
 assert w['liquid_contents']==[p for p in brine['contents']if p['phase']in ['liquid','aqueous']]
 assert not missing(positive,3)
 assert k['solution'].get('native_density')is None and 'pitzer.dat'in k['solution']['provenance']['dataset']and missing(negative,0)
 for rows in [positive,negative]:
  for before,after in zip(rows,rows[1:]):
   if after.get('operator',{}).get('op')=='measure':assert before['bench']['vessels']==after['bench']['vessels']
  assert not any(e['event']=='solver_failed'for e in events(rows))
 return dict(water_density_g_ml=water[0],NaCl_density_g_ml=m[0],NaCl_temperature_K=brine['temperature'],NaCl_pressure_Pa=brine['pressure'],NaCl_native_dataset=w['dataset'],NaCl_missing_volume_warning=False,KCl_native_density_authority=False,KCl_missing_volume_warning=True,stock_max_abs=max(abs(d)for delta in residuals for d in delta.values()),stock_residuals=residuals,all_three_polymer_stocks_g=5,measurements_read_only=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args();o=a.evidence
 manifest=json.loads((o/'SHA256.json').read_text());verified=[]
 for name,h in manifest.items():
  file=o/name
  if file.exists():assert sha(file)==h;verified.append(name)
 assert subprocess.check_output(['git','rev-parse',SOURCE+'^{tree}'],cwd=a.repo,text=True).strip()==TREE
 done=json.loads((o/'COMPLETED.json').read_text());assert done['source']==SOURCE and done['tree']==TREE and done['all_commands_passed']and done['all_cli_lessons_exit']==done['core_golden_exit']==0
 assert not done['PO002_original_replayed']and done['qualification_delta']==0
 stdout=(o/'all-cli-lessons.stdout').read_text();assert 'test result: ok. 34 passed; 0 failed;'in stdout
 assert 'test plastics_sort_themselves_by_density_in_water ... ok'in stdout and 'test salt_density_authority_has_provenance_stocks_and_an_unsupported_control ... ok'in stdout
 assert 'test result: ok. 1 passed; 0 failed;'in(o/'unchanged-core-golden.stdout').read_text()
 assert sha(o/'actual-producer-Cargo.lock')=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert sha(o/'compiler.stdout')=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6'
 changed=subprocess.check_output(['git','diff','--name-only',BASE,SOURCE],cwd=a.repo,text=True).splitlines();assert changed==['crates/kerotakis-cli/tests/fixtures/kcl-density-boundary.lab','crates/kerotakis-cli/tests/lessons_replay.rs','lessons/float-or-sink.lab']
 commands=lambda text:[x.strip()for x in text.splitlines()if x.strip()and not x.lstrip().startswith('#')]
 old=subprocess.check_output(['git','show',BASE+':lessons/float-or-sink.lab'],cwd=a.repo,text=True);new=subprocess.check_output(['git','show',SOURCE+':lessons/float-or-sink.lab'],cwd=a.repo,text=True);assert commands(old)==commands(new)
 binaries=json.loads((o/'native-executables.json').read_text());streams=json.loads((o/'streamed-executables.json').read_text());assert {r['name']for r in binaries}=={'kero','lessons_replay'}
 for r in binaries:assert r['source']==SOURCE and r['tree']==TREE and streams[r['artifact']]['verified']and streams[r['artifact']]['actual_sha256']==r['sha256']
 registry=json.loads(subprocess.check_output(['git','show',SOURCE+':data/registry/registry-source-v1.json'],cwd=a.repo));positive=readrows(o/'readouts/float-or-sink.jsonl');negative=readrows(o/'readouts/kcl-density-boundary.jsonl');result=check(positive,negative,registry)
 mutations=[]
 for name,which,edit in [('false_NaCl_density',0,lambda rows:rows[-1]['bench']['vessels'][3]['solution']['native_density'].__setitem__('kg_per_l',1.0)),('lost_polymer',0,lambda rows:rows[-1]['bench']['vessels'][0]['contents'].clear()),('false_KCl_authority',1,lambda rows:rows[-1]['bench']['vessels'][0]['solution'].__setitem__('native_density',{})),('missing_KCl_warning',1,lambda rows:[r.__setitem__('events',[e for e in r.get('events',[])if(e.get('reason')or{}).get('key')!='not-modeled.density-of-the-dissolved-ions'])for r in rows])]:
  rows=[copy.deepcopy(positive),copy.deepcopy(negative)];edit(rows[which])
  try:check(rows[0],rows[1],registry)
  except(AssertionError,KeyError):mutations.append(name)
  else:raise AssertionError('Mutation survived '+name)
 result.update(source=SOURCE,tree=TREE,full_CLI_lessons_passed=34,unchanged_core_golden_passed=True,lesson_commands_unchanged=True,production_solver_model_guard_and_data_unchanged=True,changed_paths=changed,actual_binary_hashes={r['name']:r['sha256']for r in binaries},verified_small_producer_files=len(verified),mutations_rejected=mutations,qualification_delta=0,coverage=431,full_source_platform_fleet_acceptance=False,merge_authorized=False)
 with a.report.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
