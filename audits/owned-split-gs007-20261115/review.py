"""Source-bound GS007 original and split-design authority review."""
import argparse,copy,hashlib,json,math,re,subprocess
from pathlib import Path
SOURCE='6a2a4fc916e311aadeb96abad25222b6d5aac9d7';TREE='8ac9981b82aa2d0ba8f6d203d9bee384437fe3a3'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def review_original(obs,registry):
 original=obs['original'];assert original['id']=='new120:gs-007'
 assert hashlib.sha256(original['script'].encode()).hexdigest()==original['script_sha256']
 frames=obs['frames'];commands=[x for x in original['script'].splitlines()if x and not x.startswith('#')];assert [f['command']for f in frames]==commands
 comp={r['species_id']:r for r in registry['compositions']};comp['base_equivalents']=comp['OH-'];props={(r['species_id'],r['property']):r['quantity']['value']for r in registry['phase_thermodynamics']if isinstance(r['property'],str)and r['property']in ['mass_density','molar_mass']}
 def portion_ledger(vs):
  out={}
  for v in vs:
   assert not any(v.get(k)for k in ['unresolved_materials','material_objects','surfaces','electrodes','adsorbed','exchanges','solid_solutions'])
   for p in v['contents']:
    assert math.isfinite(p['moles'])and p['moles']>=0
    c=comp[p['species']]
    for e in c['elements']:out[e['element']]=out.get(e['element'],0)+p['moles']*e['count']['value']
    out['Q']=out.get('Q',0)+p['moles']*c['net_charge']['value']
  return out
 def add(out,species,n):
  c=comp[species]
  for e in c['elements']:out[e['element']]=out.get(e['element'],0)+n*e['count']['value']
  out['Q']=out.get('Q',0)+n*c['net_charge']['value']
 residuals=[];crossings=[];warnings=[];solver_failures=[]
 for i,f in enumerate(frames):
  if i:assert f['before']==frames[i-1]['after']
  before=f['before'];after=f['after'];expected=portion_ledger(before);tokens=f['command'].split()
  if tokens[0]=='add':
   q=tokens[3]
   if q.endswith('mol'):n=float(q[:-3])
   else:assert q.endswith('mL');n=float(q[:-2])*props[tokens[2],'mass_density']/props[tokens[2],'molar_mass']
   add(expected,tokens[2],n)
  for event in f['events']:
   kind=event['event']
   if kind=='vessel_sealed':
    assert tokens[0]=='seal';assert event['vessel']==0
    trapped=event['trapped_air'];headspace=event['headspace_volume'];computed=before[0]['pressure']*headspace/(8314.462618*before[0]['temperature']);assert abs(trapped-computed)<=1e-10
    for species,fraction in [('N2',.7901),('O2',.2095),('CO2',.0004)]:add(expected,species,trapped*fraction)
   elif kind in ['gas_evolved','gas_absorbed']:
    assert event['vessel']==0 and math.isfinite(event['moles'])and event['moles']>=0
    sign=-1 if kind=='gas_evolved'else 1;add(expected,event['species'],sign*event['moles']);crossings.append(dict(command=f['command'],direction='out'if sign<0 else'in',species=event['species'],moles=event['moles']))
   elif kind=='solver_failed':solver_failures.append(event)
   elif kind=='not_yet_modeled':warnings.append(event)
  actual=portion_ledger(after);residuals.append(dict(command=f['command'],delta={e:actual.get(e,0)-expected.get(e,0)for e in set(actual)|set(expected)}))
  if tokens[0]in ['inspect','measure']:assert before==after
 rich_frame=next(f for f in frames if f['command']=='inspect v1');assert obs['rich']==rich_frame['after'][0]
 assert obs['after']==frames[-1]['after']
 def solid(v):return sum(p['moles']for p in v['contents']if p['species']=='CaCO3'and p['phase']=='solid')
 rich=solid(obs['rich']);final=solid(obs['after'][0]);dissolved=.0005-rich
 assert obs['rich_solid_mol']==rich and obs['final_solid_mol']==final and obs['dissolved_rich_mol']==dissolved
 # Same public observability gate used by the actual unchanged test.
 c1=dissolved>1e-6;c2=final>rich
 max_delta=max(abs(x)for r in residuals for x in r['delta'].values())
 return dict(criteria={'GS-007-C1':'PASS'if c1 else'FAIL','GS-007-C2':'PASS'if c2 else'FAIL'},criterion_C1=c1,criterion_C2=c2,rich_solid_mol=rich,final_solid_mol=final,rich_dissolved_Ca_mol=dissolved,rich_temperature_K=obs['rich']['temperature'],rich_pressure_Pa=obs['rich']['pressure'],final_temperature_K=obs['after'][0]['temperature'],final_pressure_Pa=obs['after'][0]['pressure'],ledger_max_abs=max_delta,ledger_pass=max_delta<=1e-10,ledger_residuals=residuals,actual_gas_crossings=crossings,solver_failures=solver_failures,model_warnings=warnings,pending_CO2_by_frame=[f['after'][0].get('pending_co2_transfer_mol',0)for f in frames],native_provenance=(obs['after'][0].get('solution')or{}).get('provenance'),source_complete=not solver_failures)
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args();o=a.evidence
 m=json.loads((o/'SHA256.json').read_text());verified=[]
 for path,h in m.items():
  file=o/path
  if file.exists():assert sha(file)==h;verified.append(path)
 assert subprocess.check_output(['git','rev-parse',SOURCE+'^{tree}'],cwd=a.repo,text=True).strip()==TREE
 binding=json.loads((o/'source-and-lock.json').read_text());assert binding['source']==SOURCE and binding['tree']==TREE
 assert sha(o/'actual-producer-Cargo.lock')==binding['cargo_lock_sha256']=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert sha(o/'compiler.stdout')=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6'
 done=json.loads((o/'COMPLETED.json').read_text());assert done['source']==SOURCE and done['tree']==TREE and not done['SP003_original_replayed']and not done['PO002_original_replayed']and not done['inventory_writer_migrated']
 assert done['split_controls_exit']==0 and done['carbon_overdraw_refusal_exit']==0
 splits=(o/'split-authority-controls.stdout').read_text();assert 'actual_sp003_step629_correction_cannot_authorize_a_scalar_writer ... ok'in splits
 binaries=json.loads((o/'native-executables.json').read_text());streams=json.loads((o/'streamed-executables.json').read_text());assert len(binaries)==3
 for row in binaries:assert row['source']==SOURCE and row['tree']==TREE and streams[row['artifact']]['actual_sha256']==row['sha256']and streams[row['artifact']]['verified']
 obs=json.loads((o/'readouts/original.json').read_text());original=json.loads(subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-phreeqc/tests/fixtures/gs007-original-20261016.json'],cwd=a.repo));assert obs['original']==original
 basis_source=subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-core/src/species.rs'],cwd=a.repo,text=True);assert 'pub const BASE_EQUIVALENTS: &str = \"base_equivalents\";' in basis_source and '.get(\"OH-\")' in basis_source
 registry=json.loads(subprocess.check_output(['git','show',SOURCE+':data/registry/registry-source-v1.json'],cwd=a.repo));result=review_original(obs,registry)
 qualification=done['gs007_exit']==0 and result['criterion_C1']and result['criterion_C2']and result['ledger_pass']and result['source_complete']
 result.update(source=SOURCE,tree=TREE,original_id='new120:gs-007',original_script_sha256=binding['script_sha256'],qualification=qualification,coverage_before=431,coverage_after=431+int(qualification),GS007_native_exit=done['gs007_exit'],split_planning_controls_passed=True,vessel_writer_migrated=False,SP003_qualified=False,actual_binary_hashes={x['name']:x['sha256']for x in binaries},verified_small_producer_files=len(verified),runtime_basis_source_sha256=hashlib.sha256(basis_source.encode()).hexdigest(),base_equivalents_scope='Runtime basis carrier borrows OH stoichiometry for elemental/charge closure, not measured free hydroxide',scope='Split planning proof is not a committed still transfer. GS007 original criteria remain unchanged; any native or inventory failure stays a refusal.')
 mutations=[]
 for name,edit in [('changed_script',lambda x:x['original'].__setitem__('script','inspect\n')),('changed_final_solid',lambda x:x.__setitem__('final_solid_mol',1.0)),('wrong_rich_stage',lambda x:x['rich'].__setitem__('temperature',100.0)),('lost_readonly_state',lambda x:x['after'][0]['contents'].clear())]:
  x=copy.deepcopy(obs);edit(x)
  try:review_original(x,registry)
  except(AssertionError,KeyError):mutations.append(name)
  else:raise AssertionError('Mutation survived '+name)
 result['mutations_rejected']=mutations
 with a.report.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
