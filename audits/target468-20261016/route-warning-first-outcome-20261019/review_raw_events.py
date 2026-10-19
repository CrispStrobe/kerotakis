"""Check source-bound diagnostic events and owned atoms without running an engine."""
import argparse,hashlib,json,subprocess
from decimal import Decimal,localcontext
from pathlib import Path
D=Decimal
p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,default=Path(__file__).resolve().parent);p.add_argument('--out',type=Path,required=True);a=p.parse_args();assert not a.out.exists()
sha=lambda raw:hashlib.sha256(raw).hexdigest();plan=json.loads((a.directory/'raw-event-plan.json').read_bytes());source=plan['source']
registry_raw=subprocess.check_output(['git','show',source+':data/registry/registry-source-v1.json']);registry=json.loads(registry_raw,parse_float=D);compositions={r['species_id']:r for r in registry['compositions']}
def composition(key):
 row=compositions[key];return {**{e['element']:e['count']['value'] for e in row['elements']},'charge':row['net_charge']['value']}
def ledger(v):
 out={}
 for p in v['contents']:
  n=D(p['moles']);assert n.is_finite() and n>=0
  for k,c in composition(p['species']).items():out[k]=out.get(k,D(0))+c*n
 return out
def amount(v,key):return sum((D(p['moles']) for p in v['contents'] if p['species']==key),D(0))
results=[]
with localcontext() as ctx:
 ctx.prec=45
 for case in plan['cases']:
  folder=a.directory/'raw-events'/case['id'];raw=(folder/'stdout.ndjson').read_bytes();receipt=json.loads((folder/'completion.json').read_bytes());assert receipt['stdout_sha256']==sha(raw) and receipt['exit_status']==0 and receipt['attempts']==1
  script='\n'.join(case['script'])+'\n';assert sha(script.encode())==json.loads((folder/'issued.json').read_bytes())['script_sha256']
  rows=[json.loads(x,parse_float=D) for x in raw.decode().splitlines()];old={};maximum=D(0);per_step=[];events=[]
  for row in rows:
   assert len(row['bench']['vessels'])==1;v=row['bench']['vessels'][0];op=row['operator'];expected=old.copy();events.extend(row['events'])
   assert not any(e['event']=='solver_failed' for e in row['events'])
   if op['op']=='add':
    for k,c in composition(op['species']).items():expected[k]=expected.get(k,D(0))+c*op['moles']
   else:assert op['op'] in {'stir','wait','measure'}
   actual=ledger(v);res={k:actual.get(k,D(0))-expected.get(k,D(0)) for k in actual.keys()|expected.keys()};maximum=max(maximum,max(map(abs,res.values()),default=D(0)));assert maximum<=D('1e-8');per_step.append({'op':op['op'],'residual':res});old=actual
  visual=any(e['event']=='observed' for e in events);assert visual
  if case['id'] in ['bio-034','bio-041']:
   assert not any(e['event'] in {'reacted','fermented','enzyme_hydrolysed'} for e in events)
   assert any(e['event']=='not_yet_modeled' for e in events)
   finding='Only handling, unsupported-model notes and appearance are observed; there is no calculated transformation. This visual grade does not prove the requested chemistry.';metrics={}
  else:
   assert case['id']=='bio-112';reactions={e['reaction']:e for e in events if e['event']=='reacted'};bleach=reactions['peroxide-melanin-bleach']['moles'];decomposition=reactions['peroxide-decomposition']['moles'];final=rows[-1]['bench']['vessels'][0]
   assert bleach>0 and decomposition>0
   assert abs(D('.001')-amount(final,'hair_pigment')-bleach)<=D('1e-12')
   assert abs(amount(final,'hair_pigment_ox')-bleach)<=D('1e-12')
   assert abs(D('.01')-amount(final,'H2O2')-bleach-2*decomposition)<=D('1e-12')
   assert abs(amount(final,'water')-bleach-2*decomposition)<=D('1e-12')
   assert abs(amount(final,'O2')-decomposition)<=D('1e-12')
   finding='Finite declared teaching-model arithmetic changes the owned pigment inventory. Rates are editorial; this is not empirical hair chemistry or aqueous-medium calibration.';metrics={'bleaching_extent_mol':bleach,'decomposition_extent_mol':decomposition,'pigment_remaining_mol':amount(final,'hair_pigment'),'oxidised_pigment_mol':amount(final,'hair_pigment_ox')}
  results.append({'id':case['id'],'raw_sha256':sha(raw),'finding':finding,'metrics':metrics,'maximum_per_step_atom_charge_residual':maximum,'per_step':per_step})
out={'source':source,'binary_sha256':plan['binary_sha256'],'actual_local_runtime_libraries':plan['actual_local_runtime_libraries'],'registry_sha256':sha(registry_raw),'cases':results,'new_engine_executions_by_reviewer':0,'scientific_catalog_additions':0,'qualification':'Three separately frozen diagnostic probes explain missing primary events in the failed classifier report. This does not rerun the whole corpus or qualify original experiments.'};a.out.write_text(json.dumps(out,indent=2,default=str)+'\n');print(json.dumps({'reviewed_diagnostics':len(results),'new_executions_by_reviewer':0,'catalog_additions':0}))
