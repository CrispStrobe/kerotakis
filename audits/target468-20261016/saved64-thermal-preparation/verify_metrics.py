"""Saved thermal snapshots with independent bounded energy arithmetic."""
import argparse,copy,hashlib,json,math,re,subprocess,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83';sha=lambda b:hashlib.sha256(b).hexdigest();p=argparse.ArgumentParser();p.add_argument('--record',action='store_true');a=p.parse_args();projection=json.loads((HERE.parent/'saved64-first-screen/material-projection.json').read_text());screen={c['id']:c for c in json.loads((HERE.parent/'saved64-first-screen/first-screen.json').read_text())['cases']};raw_registry=subprocess.check_output(['git','show',SOURCE+':data/registry/registry-source-v1.json'],cwd=ROOT);registry=json.loads(raw_registry);route=subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-core/src/phase_route.rs'],cwd=ROOT);prop={(r['species_id'],r['property']):r['quantity'] for r in registry['phase_thermodynamics'] if isinstance(r['property'],str)};curves={(r['species_id'],r['phase']):r for r in registry['heat_capacity_polynomials']};fusion,vap=route.decode().split('pub const FUSION_ENTHALPIES:',1)[1].split('pub const VAPORISATION_ENTHALPIES:',1);parse=lambda s:{k:1000*float(v) for k,v in re.findall(r'species: "([^"\n]+)",\s*(?://[^\n]*\n\s*)?kj_per_mol: ([0-9.]+)',s)};fus=parse(fusion);vap=parse(vap);R=8.31446261815324

def cp(s,phase,T):
 curve=curves.get((s,phase))
 if not curve:return prop[s,'molar_heat_capacity']['value']
 intervals=curve['intervals'];T=max(intervals[0]['t_min_k'],min(intervals[-1]['t_max_k'],T));iv=next(iv for iv in intervals if iv['t_min_k']<=T<=iv['t_max_k']);c=iv['coefficients']
 if curve['form']=='nasa9':return R*math.fsum([c[0]/T**2,c[1]/T,c[2],c[3]*T,c[4]*T**2,c[5]*T**3,c[6]*T**4])
 assert curve['form']=='shomate';t=T/1000;return math.fsum([c[0],c[1]*t,c[2]*t*t,c[3]*t**3,c[4]/t**2])
def integral(s,phase,A,B,steps=2048):
 if B<A:return -integral(s,phase,B,A,steps)
 if A==B:return 0.
 h=(B-A)/steps;return h/3*math.fsum((1 if i in [0,steps] else 4 if i%2 else 2)*cp(s,phase,A+i*h) for i in range(steps+1))
def amt(v,s,phase=None):return sum(q['moles'] for q in v['contents'] if q['species']==s and (phase is None or q['phase']==phase))
def metric(v):return {k:v[k] for k in ['id','temperature','elapsed_seconds','contents','headspace','excess_enthalpy_j','solution'] if k in v}
archive=ROOT/'audits/target468-20261016/common-engine-completion/results/raw-artifact.zip';assert sha(archive.read_bytes())==projection['raw_artifact_sha256'];records=[]
with zipfile.ZipFile(archive) as z:
 for n in range(1,17):
  cid=f'new120:th-{n:03}';c=screen[cid];prefix=cid.replace(':','_');assert sha(z.read(prefix+'/experiment.lab'))==c['original_script_sha256'];raw=z.read(prefix+'/stdout.jsonl');assert sha(raw)==c['saved_execution_receipt']['stdout_sha256'];rows=[json.loads(l) for l in raw.splitlines()];state={};timeline=[];energy=[]
  for i,row in enumerate(rows):
   op=row['operator'];before=copy.deepcopy(state.get(op.get('vessel')));state.update({v['id']:v for v in row['bench']['vessels']})
   if op['op'] in ['heat','cool','inspect','mix','add']:
    timeline.append({'row':i,'operator':op,'vessels':[metric(v) for v in row['bench']['vessels']],'events':row['events']})
   if before is None or op['op'] not in ['heat','cool']:continue
   after=state[op['vessel']];q=op['energy']*(1 if op['op']=='heat' else -1);calc=None
   if n==1:
    s='water' if op['vessel']==0 else 'ethanol';m=amt(before,s);e=m*integral(s,'liquid',before['temperature'],after['temperature']);coarse=m*integral(s,'liquid',before['temperature'],after['temperature'],1024);calc={'method':'unchanged liquid inventory; integrate pinned Cp(T)','species':s,'sensible_J':e,'quadrature_refinement_delta_J':e-coarse,'requested_J':q,'accounted_J':e,'residual_J':e-q}
   elif n==3:
    latent=(amt(after,'water','liquid')-amt(before,'water','liquid'))*prop['water','enthalpy_of_fusion']['value'];calc={'method':'water phase coexistence at273.15K; liquid-mole change times fusion enthalpy','requested_J':q,'accounted_J':latent,'residual_J':latent-q}
   elif n in [11,12,13,16]:
    s={11:'naphthalene',12:'CH3COOH',13:'ethanol',16:'Zn'}[n];phase='solid' if n in [11,16] else 'liquid';m=amt(before,s);sensible=m*integral(s,phase,before['temperature'],after['temperature']);latent=(amt(after,s,'liquid')-amt(before,s,'liquid'))*fus[s] if n!=13 else (amt(before,s)-amt(after,s))*vap[s];e=sensible+latent;calc={'method':'initial-phase sensible integral plus actual phase-fraction latent change; open ethanol export counted','species':s,'requested_J':q,'sensible_J':sensible,'latent_J':latent,'accounted_J':e,'residual_J':e-q,'note':'A large mismatch is retained as evidence, never a fitted property or repaired forecast.'}
   if calc:energy.append({'row':i,**calc})
  mixing=None
  if n==2:
   v=state[0];water=amt(v,'water');copper=amt(v,'Cu');T=v['temperature'];waterQ=water*integral('water','liquid',293.15,T);copperQ=copper*integral('Cu','solid',373.15,T);mixing={'reported_T_K':T,'initial_water_K':293.15,'initial_Cu_K':373.15,'water_heat_gain_J':waterQ,'copper_heat_change_J':copperQ,'adiabatic_residual_J':waterQ+copperQ,'scope':'Pinned Cp(T) with documented endpoint clamping, not a mass-weighted or constant-Cp estimate.'}
  records.append({'id':cid,'script_sha256':c['original_script_sha256'],'stdout_sha256':sha(raw),'original_criteria':c['original_criteria'],'timeline':timeline,'bounded_energy_arithmetic':energy,'mixing_arithmetic':mixing,'formal_selection':'not_assigned'})
report={'source':SOURCE,'registry_sha256':sha(raw_registry),'phase_route_sha256':sha(route),'archive_sha256':projection['raw_artifact_sha256'],'cases':records,'new_engine_or_api_executions':0,'catalog_additions':0,'qualification':'Independent saved-output arithmetic within declared property/model domains. Composite Simpson integration uses pinned NASA9/Shomate Cp curves and source endpoint clamping, with constant legacy Cp only where no curve is present. All original raw thermal and solver events retained. No universal energy closure, calorimetric calibration or current-source reproduction is claimed; uncomputed mixture/reaction/export domains remain pending.'};out=HERE/'metrics.json'
if a.record:
 assert not out.exists();out.write_text(json.dumps(report,indent=2,allow_nan=False)+'\n')
else:assert json.loads(out.read_text())==report,'Thermal projection changed'
print(json.dumps({'thermal_cases':len(records),'bounded_heat_calculations':sum(len(c['bounded_energy_arithmetic']) for c in records),'new_executions':0,'new_selections':0}))
