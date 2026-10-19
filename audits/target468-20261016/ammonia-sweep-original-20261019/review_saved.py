"""Review original AB031 after both arms are sealed; no chemistry execution."""
import argparse,hashlib,json,subprocess
from decimal import Decimal,localcontext
from pathlib import Path
D=Decimal
p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,default=Path(__file__).resolve().parent);p.add_argument('--out',type=Path,required=True);a=p.parse_args();assert not a.out.exists()
sha=lambda raw:hashlib.sha256(raw).hexdigest();d=json.loads((a.directory/'design.json').read_bytes());c=d['cases'][0];assert c['id']=='batch2:ab-031'
raw=(a.directory/'response.json').read_bytes();r=json.loads(raw,parse_float=D);assert r['engine']==d['accepted_engine'] and r['exit_status']==0 and r['script_sha256']==c['script_sha256']==sha(c['script'].encode())
source=d['accepted_engine']['source_head'];registry_raw=subprocess.check_output(['git','show',source+':data/registry/registry-source-v1.json']);registry=json.loads(registry_raw,parse_float=D);comp={row['species_id']:row for row in registry['compositions']}
def composition(key):
 if key=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
 row=comp[key];return {**{e['element']:e['count']['value'] for e in row['elements']},'charge':row['net_charge']['value']}
def amount(v,keys):return sum((p['moles'] for p in v['contents'] if p['species'] in keys),D(0))
def ledger(v):
 out={}
 for p in v['contents']:
  assert p['moles'].is_finite() and p['moles']>=0
  for key,n in composition(p['species']).items():out[key]=out.get(key,D(0))+n*p['moles']
 return out
with localcontext() as ctx:
 ctx.prec=45;state={};started=False;maximum=D(0);exports={};per_step=[];pre=None
 for row in r['observations']:
  before=state;state={v['id']:v for v in row['bench']['vessels']};op=row['operator']
  assert not any(e['event']=='solver_failed' for e in row['events'])
  if op['op']=='sweep':
   assert not started and op['vessel']==1;started=True;pre=before
   assert all(v['headspace']['boundary']=='sealed' for v in pre.values())
  if not started:continue
  expected={key:ledger(v) for key,v in before.items()}
  for event in row['events']:
   if event['event'] not in {'gas_evolved','gas_absorbed'}:continue
   sign=D(1) if event['event']=='gas_evolved' else D(-1);n=sign*event['moles'];vid=event['vessel'];key=event['species'];exports[(vid,key)]=exports.get((vid,key),D(0))+n
   for element,coefficient in composition(key).items():expected[vid][element]=expected[vid].get(element,D(0))-coefficient*n
  residuals={}
  for vid,v in state.items():
   now=ledger(v);res={k:now.get(k,D(0))-expected[vid].get(k,D(0)) for k in now.keys()|expected[vid].keys()};worst=max(map(abs,res.values()),default=D(0));maximum=max(maximum,worst);assert worst<=D('1e-8');residuals[str(vid)]=res
  per_step.append({'operator':op,'residuals':residuals})
 assert pre is not None
 assert [row['operator']['seconds'] for row in r['observations'] if row['operator']['op']=='wait']==[D(120)]
 assert state[0]['headspace']['boundary']=='sealed' and state[1]['headspace']['boundary']=='swept'
 before_N=amount(pre[1],{'NH3','NH4+'});after_N=amount(state[1],{'NH3','NH4+'});export_N=exports.get((1,'NH3'),D(0))
 assert export_N>0 and abs(after_N+export_N-before_N)<=D('1e-8')
 cl_before=amount(pre[1],{'Cl-'});cl_after=amount(state[1],{'Cl-'})
 assert abs(cl_after-cl_before)<=D('1e-8') and not any(n for (vid,key),n in exports.items() if key=='Cl-')
 before_ph=pre[1]['solution']['ph'];final_control=state[0]['solution']['ph'];final_swept=state[1]['solution']['ph']
 trend=final_swept<before_ph and final_swept<final_control
 result={'source':source,'engine':d['accepted_engine'],'registry_sha256':sha(registry_raw),'response_sha256':sha(raw),'original_criteria':c['original_criteria'],'criteria':{'AB-031-C1':bool(trend and export_N>0),'AB-031-C2':True},'status':'PASS' if trend else 'FAIL_FORECAST_WITH_SEPARATE_MODEL_LIMIT','before_sweep_ph':before_ph,'final_sealed_ph':final_control,'final_swept_ph':final_swept,'reduced_N_before_mol':before_N,'reduced_N_after_mol':after_N,'NH3_export_mol':export_N,'chloride_before_mol':cl_before,'chloride_after_mol':cl_after,'all_boundary_exports_mol':{str(vid)+':'+key:n for (vid,key),n in exports.items()},'maximum_sweep_and_wait_per_vessel_atom_charge_residual':maximum,'per_step':per_step,'final_temperatures_K':{str(vid):v['temperature'] for vid,v in state.items()},'catalog_additions':0,'new_requests_by_reader':0,'qualification':'Only initial trapped NH3 is explicitly exported; current source adds NH3(g) to finite headspaces but not the swept reservoir. CO2 is also stripped, so pH sign must not be forced to satisfy the forecast. No continuous ammonia-transfer or flow/rate calibration is claimed.'}
a.out.write_text(json.dumps(result,indent=2,default=str)+'\n');print(json.dumps({'status':result['status'],'criteria':result['criteria'],'new_requests':0}))
