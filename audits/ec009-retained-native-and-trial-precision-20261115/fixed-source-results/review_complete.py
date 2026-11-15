"""Complete source-bound original review; every failed acceptance stays failed."""
import argparse,copy,hashlib,json,math,re,subprocess
from pathlib import Path
SOURCE="78e0be4b29f9d42f66aa4db3bc92f38583aeff1b"
TREE="b7e4dd5254af62f0290f06a7441d0aed25b81569"
FIXTURE="adee22f70af455490364760fbefd78ce120f9a6ad0f0a3be00387442b17554ca"
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def load(path):return json.loads(path.read_text())
def amount(v,s):return sum(p['moles'] for p in v['contents'] if p['species']==s)

def check(stage,fixture,comp):
 assert stage['physical_source']=='4bd93add656c936b0724673c7ea8332cf21e906f' and stage['historical_native_binary_sha256'] is None
 assert 'prefix was not rerun' in stage['stage'] and stage['error'] is None
 assert stage['before']==fixture['before_titration'] and stage['operator']==fixture['original_titration']
 assert len(stage['before'])==len(stage['after'])==1
 assert not any(e['event']=='solver_failed' for e in stage['events'])
 titre=next(e for e in stage['events'] if e['event']=='titrated');assert titre['concentration']==.02 and 0<titre['steps']<=200 and titre['endpoint_reached'] is True
 volume=titre['total_volume'];assert math.isfinite(volume) and volume>0
 additions={'KMnO4':volume*.02,'water':volume*1000*comp['water_density']['quantity']['value']/comp['water_molar_mass']['quantity']['value']}
 def parts(v):
  out={}
  assert not v.get('unresolved_materials',[]) and not v.get('material_objects',[]) and v.get('pending_co2_transfer_mol',0)==0
  for p in v['contents']:
   assert math.isfinite(p['moles']) and p['moles']>=0
   row=comp['compositions'][p['species']];f={**{r['element']:r['count']['value'] for r in row['elements']},'Q':row['net_charge']['value']}
   for e,n in f.items():out[e]=out.get(e,0)+n*p['moles']
  return out
 before=stage['before'][0];after=stage['after'][0];expected=parts(before)
 for s,moles in additions.items():
  row=comp['compositions'][s]
  for a in row['elements']:expected[a['element']]=expected.get(a['element'],0)+a['count']['value']*moles
 actual=parts(after);ledgers={e:actual.get(e,0)-expected.get(e,0) for e in set(expected)|set(actual)}
 assert all(abs(x)<=1e-10 for x in ledgers.values()),ledgers
 fe2=amount(before,'Fe+2')-amount(after,'Fe+2');fe3=amount(after,'Fe+3')-amount(before,'Fe+3');assert fe2>0 and fe3>0 and abs(fe2-fe3)<=1e-10
 mn2=amount(after,'Mn+2');mn7=amount(after,'MnO4-');mno2=amount(after,'MnO2');assert mn2>0 and mno2==0 and abs(mn2+mn7-additions['KMnO4'])<=1e-10
 assert abs(fe3-5*mn2)<=1e-10,'no unowned electrons may masquerade as an approximate titre'
 return dict(criteria={'EC-009-C1':True,'EC-009-C2':True},actual_burette_l=volume,actual_mn2_mol=mn2,actual_mn7_mol=mn7,actual_mno2_mol=mno2,fe2_lost_mol=fe2,fe3_formed_mol=fe3,electron_residual_eq=fe3-5*mn2,ledger=ledgers,max_abs_residual=max(map(abs,ledgers.values())))

def model_boundary(stage):
 return [e for e in stage['events']if e.get('reason',{}).get('key')in ['not-modeled.redox-coupling-stood-down','not-modeled.redox-pair-unpriced']]

def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--outputs',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args();o=a.outputs;repo=a.repo;prior=repo/'audits/ec009-first-charge-repair-failure-20261114'
 manifest=load(o/'SHA256.json');verified=[]
 for name,digest in manifest.items():
  path=o/name
  if path.is_file():assert sha(path)==digest,name;verified.append(name)
 fixture=load(o/'native-pair-fixture.json');assert sha(o/'native-pair-fixture.json')==FIXTURE
 binding=load(o/'source-and-lock.json');started=load(o/'STARTED.json');complete=load(o/'COMPLETED.json')
 assert started['source']==binding['source']==complete['source']==SOURCE and started['tree']==binding['tree']==complete['tree']==TREE
 assert not complete['preparation_prefix_replayed']
 assert sha(o/'actual-producer-Cargo.lock')==binding['cargo_lock_sha256']=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert sha(o/'compiler.stdout')=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6'
 producer_script=repo/'crates/kerotakis-phreeqc/tests/ec009_charge_continuation.rs'
 # The candidate need not be checked out in the broad audit workspace.
 actual_tree=subprocess.check_output(['git','rev-parse',SOURCE+'^{tree}'],cwd=repo,text=True).strip();assert actual_tree==TREE
 prefix=(prior/'ec009-historical.stdout').read_bytes();assert (o/'ec009-historical.stdout').read_bytes()==prefix
 frames=[json.loads(line)for line in prefix.decode().splitlines()if line.startswith('{')];assert fixture['before_titration']==frames[2]['bench']['vessels']and fixture['original_titration']==frames[3]['operator']
 cache=load(o/'cache-reuse.json');assert cache['current_source']==SOURCE and cache['cache_source']=='838a590a31c94d86ed6faace782a85360b3e469c'and cache['cache_sha256']=='c42c3c91b33a6d0a3fdcc913821615bb04da5dde0f128d49d688ba209da7513e'
 composition=load(prior/'source-compositions.json');registry=subprocess.check_output(['git','show',SOURCE+':data/registry/registry-source-v1.json'],cwd=repo);assert hashlib.sha256(registry).hexdigest()=='8741b6f338bf68ca8c4acfa8eac6a121daa35b512ccb9f05d2508e3df34baa44';assert registry==subprocess.check_output(['git','show',composition['source']+':data/registry/registry-source-v1.json'],cwd=repo)
 exes=load(o/'native-continuation-executable.json');assert len(exes)==1;exe=exes[0];assert exe['source']==SOURCE and exe['tree']==TREE and exe['cargo_lock_sha256']==binding['cargo_lock_sha256']
 stream=load(o/'streamed-native-executable.json');assert stream['actual_sha256']==stream['expected_sha256']==exe['sha256']==manifest['native-continuation-test'] and stream['source']==SOURCE and stream['bytes']>0
 results={}
 for name,count in [('fixed-precision',3),('fixed-ledger-negatives',3),('basis-charge-controls',3),('native-positive-controls',6),('frontend-observation-controls',2),('last-state-diagnostic',1),('native-charge-controls',1)]:
  text=(o/(name+'.stdout')).read_text();record=load(o/(name+'.json'));results[name]=dict(exit=record['exit'],expected_passes=count,passed=record['exit']==0 and f'test result: ok. {count} passed; 0 failed;'in text)
 stage=load(o/'readouts/titration.json');physical=check(stage,fixture,composition)
 observations={}
 for name,canonical in [('spectrum.json','measure v1 uvvis'),('inspect.json','inspect')]:
  obs=load(o/'readouts'/name);unchanged=obs['before']==obs['after']==stage['after'];assert unchanged
  assert obs['canonical_command']==canonical
  if canonical=='inspect':assert obs['operator'] is None and obs['events']==[]
  else:assert obs['operator'] is not None
  assert not any(e['event']=='solver_failed'for e in obs['events']);observations[name]=dict(unchanged=True,canonical_command=canonical)
 saved=json.loads(subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-phreeqc/tests/fixtures/ec009-after-first-precision-20261115.json'],cwd=repo))
 last=load(o/'readouts/last-state-diagnostic.json');last_binding=load(o/'last-state-binding.json')
 assert last['saved_source']==saved['source']==last_binding['saved_source']=='838a590a31c94d86ed6faace782a85360b3e469c'
 assert last['saved_titration_sha256']==saved['titration_sha256']==last_binding['saved_titration_sha256']
 assert last['before']==saved['after'][0] and last['error'] is None
 assert model_boundary(last),'actual negative-control refusal must be present'
 last_stock_drift={species:amount(last['after'],species)-amount(last['before'],species)for species in ['Fe+2','Fe+3','Mn+2','MnO4-']}
 assert all(abs(x)<=1e-10 for x in last_stock_drift.values()),last_stock_drift
 boundary=model_boundary(stage);terminal=[]
 for event in boundary:
  match=re.search(r'([0-9.]+e[-+]?[0-9]+) mol of electron-equivalents',event.get('what',''))
  if match:terminal.append(float(match[1]))
 mutations=[]
 edits=[('wrong_source',lambda d:d.__setitem__('physical_source',SOURCE)),('wrong_prefix_binary',lambda d:d.__setitem__('historical_native_binary_sha256',exe['sha256'])),('charge_drift',lambda d:next(p for p in d['after'][0]['contents']if p['species']=='Fe+2').__setitem__('moles',1e-6)),('solver_failure',lambda d:d['events'].append(dict(event='solver_failed'))),('wrong_dose',lambda d:next(e for e in d['events']if e['event']=='titrated').__setitem__('total_volume',.1))]
 edits += [('wrong_ferric_product',lambda d:next(p for p in d['after'][0]['contents']if p['species']=='Fe+3').__setitem__('moles',0)),('invent_MnO2_endpoint',lambda d:d['after'][0]['contents'].append(dict(species='MnO2',phase='solid',moles=1e-5))),('lost_water',lambda d:next(p for p in d['after'][0]['contents']if p['species']=='water').__setitem__('moles',1)),('unreached_endpoint',lambda d:next(e for e in d['events']if e['event']=='titrated').__setitem__('endpoint_reached',False)),('changed_preparation',lambda d:d['before'][0].__setitem__('temperature',300)),('unspent_pending_owner',lambda d:d['after'][0].__setitem__('pending_co2_transfer_mol',1e-6)),('unowned_opaque_stock',lambda d:d['after'][0].__setitem__('unresolved_materials',[dict(amount=.1)]))]
 for name,edit in edits:
  bad=copy.deepcopy(stage);edit(bad)
  try:check(bad,fixture,composition)
  except(AssertionError,KeyError,StopIteration):mutations.append(name)
  else:raise AssertionError(name+' survived')
 # This is the original global source/command gate AND the scientific model
 # gate, not an exemption for an expected unsupported terminal state.
 qualified=complete['all_commands_passed'] and all(x['passed']for x in results.values())and not boundary and all(physical['criteria'].values())
 result=dict(source=SOURCE,tree=TREE,qualified=qualified,coverage_before=430,coverage_after=430+int(qualified),all_required_commands_passed=complete['all_commands_passed'],focused_results=results,physical_stage_predicates=physical,actual_observations=observations,last_state_stock_drift=last_stock_drift,model_boundary_accepted=not boundary,terminal_refused_electron_eq=terminal,unchanged_electron_budget=1e-12,refused_model_events=boundary,reader_mutations_rejected=mutations,verified_small_producer_files=len(verified),continuation_executable_sha256_streamed=stream['actual_sha256'],prefix_native_binary_sha256=None,no_criterion_or_tolerance_relaxation=True)
 with a.report.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
