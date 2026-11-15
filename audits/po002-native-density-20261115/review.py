"""Independent PO002 original criteria and exact source/artifact review."""
import argparse,copy,hashlib,json,math,re,subprocess
from pathlib import Path
SOURCE='5db4a1260112c79b7ed610e5a8399b151dfd7363'
TREE='cbbc417da7002f9494efa30aa9f316928be8f980'
SCRIPT='e54dce875e64a5386efcb1c469ab6c7d1b7bb2104dd33144597ba1608f4a66d0'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(observed,registry):
 original=observed['original'];assert original['id']=='batch3-diverse:po-002'
 assert hashlib.sha256(original['script'].encode()).hexdigest()==SCRIPT
 commands=[x for x in original['script'].splitlines() if x and not x.startswith('#')]
 frames=observed['frames'];assert [f['command']for f in frames]==commands
 comp={r['species_id']:r for r in registry['compositions']}
 props={(r['species_id'],r['property']):r['quantity']['value'] for r in registry['phase_thermodynamics'] if isinstance(r['property'],str) and r['property']in ['mass_density','molar_mass']}
 def ledger(vessels):
  out={}
  for v in vessels:
   assert not v.get('unresolved_materials') and not v.get('material_objects') and not v.get('pending_co2_transfer_mol',0)
   for p in v['contents']:
    assert math.isfinite(p['moles']) and p['moles']>=0
    c=comp[p['species']]
    for e in c['elements']:out[e['element']]=out.get(e['element'],0)+p['moles']*e['count']['value']
    out['Q']=out.get('Q',0)+p['moles']*c['net_charge']['value']
  return out
 residuals=[];densities=[];unsupported=[]
 for i,f in enumerate(frames):
  if i:assert f['before']==frames[i-1]['after']
  expected=ledger(f['before']);tokens=f['command'].split()
  if tokens[0]=='add':
   species=tokens[2];amount=tokens[3]
   if amount.endswith('mL'):grams=float(amount[:-2])*props[species,'mass_density']
   else:assert amount.endswith('g');grams=float(amount[:-1])
   n=grams/props[species,'molar_mass'];c=comp[species]
   for e in c['elements']:expected[e['element']]=expected.get(e['element'],0)+n*e['count']['value']
   expected['Q']=expected.get('Q',0)+n*c['net_charge']['value']
  else:assert f['before']==f['after'] or tokens[0]=='new'
  actual=ledger(f['after']);delta={e:actual.get(e,0)-expected.get(e,0)for e in set(actual)|set(expected)}
  assert all(abs(v)<=1e-10 for v in delta.values()),(f['command'],delta)
  residuals.append(delta)
  for event in f['events']:
   assert event['event']!='solver_failed',event
   if event['event']=='measured'and event['unit']=='g/mL':densities.append(event['value'])
   if event['event']=='not_yet_modeled':unsupported.append(event)
 assert observed['after']==frames[-1]['after'] and len(densities)==2
 water,brine=densities;ps=props['PS','mass_density'];pet=props['PET','mass_density']
 assert all(math.isfinite(x)for x in [water,brine,ps,pet])
 assert water<ps<brine<pet,(water,ps,brine,pet)
 for v in observed['after']:
  for species in ['PS','PET']:
   portions=[p for p in v['contents']if p['species']==species]
   assert len(portions)==1 and portions[0]['phase']=='solid'
   assert abs(portions[0]['moles']*props[species,'molar_mass']-0.5)<=1e-10
 v=observed['after'][1];info=v['solution'];w=info['native_density']
 assert info.get('scope','complete')=='complete' and 'pitzer' in w['dataset']
 assert w['kg_per_l']==brine and w['temperature']==v['temperature']and w['pressure']==v['pressure']
 assert w['liquid_contents']==[p for p in v['contents']if p['phase']in ['liquid','aqueous']]
 # The model result must not coexist with the explicit missing-ion-volume warning.
 assert not any((e.get('reason')or{}).get('key')=='not-modeled.density-of-the-dissolved-ions' for e in unsupported)
 # Other warnings remain visible for scientific scope review, not discarded.
 return dict(density_water_g_ml=water,density_brine_g_ml=brine,density_ps_g_ml=ps,density_pet_g_ml=pet,ledger_max_abs=max(abs(x)for d in residuals for x in d.values()),ledger_residuals=residuals,unsupported_events=unsupported)
def check_refusal(negative):
 frames=negative['frames'];assert [f['command']for f in frames]==['add v1 water 50mL','add v1 KCl 8g','measure v1 density']
 for f in frames:assert not any(e['event']=='solver_failed'for e in f['events'])
 final=frames[-1];v=final['after'][0]
 assert v['solution'].get('native_density') is None
 assert any(p['species']=='K+'and p['moles']>0 for p in v['contents'])
 assert any((e.get('reason')or{}).get('key')=='not-modeled.density-of-the-dissolved-ions' for e in final['events'])
 return dict(native_density_authority=False,missing_ion_volume_warning_retained=True,not_a_supported_density_measurement=True)

def main():
 a=argparse.ArgumentParser();a.add_argument('--evidence',type=Path,required=True);a.add_argument('--repo',type=Path,required=True);a.add_argument('--report',type=Path,required=True);args=a.parse_args();o=args.evidence
 manifest=json.loads((o/'SHA256.json').read_text());verified=[]
 for rel,digest in manifest.items():
  p=o/rel
  if p.exists():assert sha(p)==digest;verified.append(rel)
 binding=json.loads((o/'source-and-lock.json').read_text());assert binding['source']==SOURCE and binding['tree']==TREE
 assert subprocess.check_output(['git','rev-parse',SOURCE+'^{tree}'],cwd=args.repo,text=True).strip()==TREE
 assert sha(o/'actual-producer-Cargo.lock')==binding['cargo_lock_sha256']=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert sha(o/'compiler.stdout')=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6'
 complete=json.loads((o/'COMPLETED.json').read_text());steps={r['name']:r for r in complete['steps']}
 for name in ['native-original-and-refusal','particle-census-controls','core-test-compilation']:assert steps[name]['exit']==0,(name,steps[name]['exit'])
 assert complete['source']==SOURCE and complete['tree']==TREE and complete['all_commands_passed']
 binary=json.loads((o/'native-executable.json').read_text());assert len(binary)==1 and binary[0]['source']==SOURCE
 receipt=json.loads((o/'streamed-native-executable.json').read_text());assert receipt['source']==SOURCE and receipt['actual_sha256']==binary[0]['sha256']and receipt['sha256_verified_by_streaming_actual_artifact']
 registry=json.loads(subprocess.check_output(['git','show',SOURCE+':data/registry/registry-source-v1.json'],cwd=args.repo))
 observed=json.loads((o/'readouts/original.json').read_text());original=json.loads(subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-phreeqc/tests/fixtures/po002-original-20261016.json'],cwd=args.repo));assert observed['original']==original
 result=check(observed,registry);mutations=[]
 assert not result['unsupported_events']
 negative=json.loads((o/'readouts/unsupported-electrolyte.json').read_text());result['KCl_refusal']=check_refusal(negative)
 trace=(o/'native-original-and-refusal.stderr').read_text()
 rho=float(re.findall(r'^   solution_density\s+([0-9.eE+-]+)',trace,re.M)[-1])
 native_t=re.findall(r'^    temp\s+([0-9.eE+-]+)',trace,re.M)[-1]
 v=observed['after'][1]
 assert rho==result['density_brine_g_ml']==v['solution']['native_density']['kg_per_l']
 assert native_t==format(v['temperature']-273.15,'.4f')
 assert v['pressure']==101325.0
 result['native_readback']=dict(RHO_kg_l=rho,same_numeric_units_as_g_ml=True,input_temperature_c=native_t,bound_vessel_temperature_K=v['temperature'],bound_pressure_Pa=v['pressure'],temperature_precision='Existing unchanged four-decimal Celsius native-input formatter',placeholder_substitution=False)
 result['criteria']={r['id']:'PASS'for r in original['acceptance_checks']}
 for name,edit in [('false_KCl_authority',lambda x:x['frames'][-1]['after'][0]['solution'].__setitem__('native_density',{})),('missing_KCl_refusal',lambda x:x['frames'][-1]['events'].__setitem__(slice(None),[e for e in x['frames'][-1]['events']if e['event']!='not_yet_modeled']))]:
  x=copy.deepcopy(negative);edit(x)
  try:check_refusal(x)
  except AssertionError:mutations.append(name)
  else:raise AssertionError('Mutation survived: '+name)
 for name,edit in [('wrong_density',lambda x:x['frames'][10]['events'].clear()),('stale_temperature',lambda x:x['after'][1]['solution']['native_density'].__setitem__('temperature',300)),('stale_pressure',lambda x:x['after'][1]['solution']['native_density'].__setitem__('pressure',100000)),('lost_salt',lambda x:x['after'][1]['contents'].clear()),('solver_failure',lambda x:x['frames'][-1]['events'].append({'event':'solver_failed'})),('changed_original',lambda x:x['original'].__setitem__('script','inspect\n'))]:
  x=copy.deepcopy(observed);edit(x)
  try:check(x,registry)
  except(AssertionError,KeyError):mutations.append(name)
  else:raise AssertionError('Mutation survived: '+name)
 result.update(original_id=original['id'],source=SOURCE,tree=TREE,original_script_sha256=SCRIPT,native_executable_sha256=receipt['actual_sha256'],cargo_lock_sha256=binding['cargo_lock_sha256'],focused_pass=True,mutations_rejected=mutations,verified_files=len(verified),qualification=True,coverage_before=430,coverage_after=431,delivery_acceptance_complete=False,scope='All unchanged original criteria PASS on the exact native candidate source; density-only feasibility, not empirical calibration or particle separation. Full source/platform/fleet/integration acceptance remains separate.')
 with args.report.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
