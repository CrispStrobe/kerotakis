"""Review one exact SP003 original; a typed atomic refusal stays FAIL."""
import argparse,copy,hashlib,json,math,re,subprocess
from pathlib import Path
SP='daeb04f0e1005da08747d371be1eb5959ee93514';TREE='9a5c1e08467964db6be1941635bf9138ee078fa2'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(observed,registry):
 original=observed['original'];assert original['id']=='new120:sp-003'
 assert hashlib.sha256(original['script'].encode()).hexdigest()==original['script_sha256']
 commands=[x for x in original['script'].splitlines()if x and not x.startswith('#')];frames=observed['frames'];assert [x['command']for x in frames]==commands
 props={(r['species_id'],r['property']):r['quantity']['value']for r in registry['phase_thermodynamics']if isinstance(r['property'],str)and r['property']in ['molar_mass','mass_density']}
 comp={r['species_id']:r for r in registry['compositions']}
 def ledger(vs):
  out={}
  for v in vs:
   assert not v.get('unresolved_materials') and not v.get('material_objects') and not v.get('pending_co2_transfer_mol',0)
   for p in v['contents']:
    n=p['moles'];assert math.isfinite(n)and n>=0;c=comp[p['species']]
    for e in c['elements']:out[e['element']]=out.get(e['element'],0)+n*e['count']['value']
    out['Q']=out.get('Q',0)+n*c['net_charge']['value']
  return out
 residuals=[]
 for i,f in enumerate(frames):
  if i:assert f['before']==frames[i-1]['after']
  expected=ledger(f['before']);bits=f['command'].split()
  if bits[0]=='add':
   assert bits[3].endswith('g');n=float(bits[3][:-1])/props[bits[2],'molar_mass'];c=comp[bits[2]]
   for e in c['elements']:expected[e['element']]=expected.get(e['element'],0)+n*e['count']['value']
   expected['Q']=expected.get('Q',0)+n*c['net_charge']['value']
  actual=ledger(f['after']);d={e:actual.get(e,0)-expected.get(e,0)for e in set(actual)|set(expected)};assert all(abs(v)<=1e-10 for v in d.values()),(f['command'],d);residuals.append(d)
  assert not any(e['event']=='solver_failed'for e in f['events'])
 one=next(f for f in frames if f['command']=='distil v1 v3 0.1 stages 1');four=next(f for f in frames if f['command']=='distil v2 v4 0.1 stages 4')
 def amounts(v):return {s:sum(p['moles']for p in v['contents']if p['species']==s)for s in ['water','ethanol']}
 one_feed=amounts(one['before'][0]);four_feed=amounts(four['before'][1]);assert one_feed==four_feed
 first=amounts(one['after'][2]);second=amounts(four['after'][3]);refusals=[e for e in four['events']if (e.get('reason')or{}).get('key')=='not-modeled.still-condensate-precision']
 assert refusals and four['before']==four['after'],'typed refusal must preserve the exact whole state'
 assert sum(first.values())>0 and sum(second.values())==0
 assert not any(e['event']=='distilled'for e in four['events'])
 assert observed['after']==frames[-1]['after'];assert frames[-1]['before']==frames[-1]['after']
 c2=abs(sum(amounts(v)['ethanol']for v in observed['after'])-10/props['ethanol','molar_mass'])<=1e-10
 assert c2
 return dict(criteria={'SP-003-C1':'FAIL','SP-003-C2':'PASS'},qualification=False,matched_feed_mol=one_feed,one_stage_product_mol=first,four_stage_product_mol=second,four_stage_source_retained_mol=amounts(four['after'][1]),four_stage_whole_bench_unchanged=True,bench_temperature_K=four['before'][1]['temperature'],bench_pressure_Pa=four['before'][1]['pressure'],model_pressure_kPa=four['before'][1]['pressure']/1000,phase_temperature_at_internal_refusal=None,ledger_max_abs=max(abs(v)for d in residuals for v in d.values()),ledger_residuals=residuals,refusal_events=refusals)
def main():
 p=argparse.ArgumentParser();p.add_argument('--evidence',type=Path,required=True);p.add_argument('--repo',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args();o=a.evidence
 manifest=json.loads((o/'SHA256.json').read_text());verified=[]
 for name,h in manifest.items():
  file=o/name
  if file.exists():assert sha(file)==h;verified.append(name)
 source=json.loads((o/'sp003-source-and-lock.json').read_text());assert source['source']==SP and source['tree']==TREE;assert subprocess.check_output(['git','rev-parse',SP+'^{tree}'],cwd=a.repo,text=True).strip()==TREE
 completed=json.loads((o/'COMPLETED.json').read_text());assert completed['sp003_source']==SP and completed['sp003_tree']==TREE
 assert completed['sp003_exit']==101 and completed['unchanged_precision_controls_exit']==0
 stdout=(o/'sp003-original-and-controls.stdout').read_text();assert 'test independently_supported_small_cut_and_impossible_composition_controls ... ok'in stdout and 'test unchanged_original_sp003_requires_both_equal_cut_products ... FAILED'in stdout
 assert sha(o/'actual-producer-Cargo.lock')==source['cargo_lock_sha256']=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert sha(o/'compiler.stdout')==source['compiler_sha256']=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6'
 exe=next(x for x in json.loads((o/'native-executables.json').read_text())if x['name']=='sp003_original');assert exe['source']==SP and exe['tree']==TREE
 actual=json.loads((o/'streamed-executables.json').read_text());assert actual[exe['artifact']]['actual_sha256']==exe['sha256']and actual[exe['artifact']]['verified']
 registry=json.loads(subprocess.check_output(['git','show',SP+':data/registry/registry-source-v1.json'],cwd=a.repo));obs=json.loads((o/'readouts/original.json').read_text());original=json.loads(subprocess.check_output(['git','show',SP+':crates/kerotakis-core/tests/fixtures/sp003-original-20261016.json'],cwd=a.repo));assert obs['original']==original
 result=check(obs,registry);trace=(o/'sp003-original-and-controls.stderr').read_text();rows=re.findall(r'^still-condensate-refusal (.+)$',trace,re.M);assert len(rows)==1
 fields={k:float(v)for k,v in (item.split('=',1)for item in rows[0].split())};assert fields['stages']==4 and fields['de']>0 and fields['dw']>0 and fields['e']>0 and fields['w']>0
 assert fields['e_over']+fields['de']==fields['e_over'] or fields['w_over']+fields['dw']==fields['w_over']
 assert fields['remaining']>0
 # Internal pot amounts are provisional: the complete operator was refused.
 result.update(source=SP,tree=TREE,original_script_sha256=source['script_sha256'],internal_provisional_refusal=fields,internal_trial_was_not_committed=True,native_executable_sha256=exe['sha256'],verified_producer_files=len(verified),coverage_before=431,coverage_after=431,ordinary_PR884_acceptance_exit=completed['pr884_focused_exit'],scope='Original C1 FAIL, not an integration/chemistry pass; native Rust core physics actual execution. Current public cut/portion API owns overhead scalars and cannot retain this small positive component separately from accumulated overhead without a representation change.')
 mutations=[]
 for name,edit in [('fake_product',lambda x:x['after'][3]['contents'].append({'species':'ethanol','phase':'liquid','moles':1e-6})),('lost_stock',lambda x:x['after'][1]['contents'].clear()),('removed_refusal',lambda x:next(f for f in x['frames']if f['command']=='distil v2 v4 0.1 stages 4')['events'].clear()),('changed_protocol',lambda x:x['original'].__setitem__('script','inspect\n'))]:
  x=copy.deepcopy(obs);edit(x)
  try:check(x,registry)
  except(AssertionError,KeyError):mutations.append(name)
  else:raise AssertionError('Mutation survived '+name)
 result['mutations_rejected']=mutations
 with a.report.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
