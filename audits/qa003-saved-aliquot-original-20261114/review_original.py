"""Independent saved aliquot/titration and explicit carrier-water ledger review."""
from pathlib import Path
from decimal import Decimal as D
import json,sys
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch2-qa-003.stdout').read_text().splitlines()]
data=json.loads((root/'source-properties.json').read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};forms['base_equivalents']=forms['OH-']
water={r['property']:D(str(r['quantity']['value'])) for r in data['water_properties']};mol_per_l=water['mass_density']*1000/water['molar_mass'];keys=['H','O','Na','Cl','charge'];bound=D('1e-10');expected={};checkpoints=[];prevol={};endpoints={}
def delta(d,sp,n):
 for k,c in forms[sp].items():d[k]+=n*c
def ledger(v):
 d={k:D(0) for k in keys};assert not v.get('unresolved_materials')
 for p in v['contents']:
  n=D(str(p['moles']));assert n.is_finite() and n>=0;delta(d,p['species'],n)
 return d
def amount(v,s):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==s),D(0))
events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','gas_evolved','gas_absorbed','gas_vented'] for e in events)
limits=[e for e in events if e['event']=='not_yet_modeled'];assert len(limits)==2 and {e['vessel'] for e in limits}=={0,1}
assert all(e['reason']['key']=='not-modeled.volume-meter-on-an-open-vessel' for e in limits)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':delta(expected.setdefault(op['vessel'],{k:D(0) for k in keys}),op['species'],D(str(op['moles'])))
 if op['op']=='decant':
  f=D(str(op['fraction']));assert f==D('.25')
  src=expected[op['from']];dst=expected.setdefault(op['to'],{k:D(0) for k in keys})
  for k in keys:n=src[k]*f;src[k]-=n;dst[k]+=n
 if op['op']=='measure' and op['instrument']=='volume_meter':
  id=op['vessel'];assert not any(e['event']=='measured' for e in r['events'])
  scene=next(v for v in r['scene']['vessels'] if v['id']==id);prevol[id]=D(str(scene['liquid']['volume_l']));assert prevol[id]>0
  # This is the explicitly reported liquid volume, not a headspace-meter reading.
  v=next(v for v in r['bench']['vessels'] if v['id']==id)
  assert abs(prevol[id]-amount(v,'water')/mol_per_l)<bound
 if op['op']=='titrate':
  id=op['vessel'];assert id in prevol
  e=next(e for e in r['events'] if e['event']=='titrated');assert e['endpoint_reached'] and e['titrant']=='NaOH'
  assert abs(D(str(e['final_ph']))-D(7))<=D('.0001')
  vol=D(str(e['total_volume']));c=D(str(e['concentration']));dose=vol*c
  assert abs(sum((D(str(e['moles'])) for e in r['events'] if e['event']=='dissolved' and e['species']=='NaOH'),D(0))-dose)<bound
  delta(expected[id],'NaOH',dose);delta(expected[id],'water',vol*mol_per_l)
  endpoints[id]={'volume_l':vol,'dose_mol':dose,'step_mol':D(str(op['step']))*c,'final_ph':D(str(e['final_ph']))}
 if 'bench' not in r:continue
 for v in r['bench']['vessels']:
  actual=ledger(v);want=expected.setdefault(v['id'],{k:D(0) for k in keys});res={k:actual[k]-want[k] for k in keys}
  assert all(abs(n)<bound for n in res.values()),(i,v['id'],res)
  if v.get('solution'):
   scope=v['solution'].get('scope','complete')
   assert scope=='complete' or (scope=='solvent_only' and all(p['species']=='water' for p in v['contents']))
  checkpoints.append({'step':i,'vessel':v['id'],'residual_mol_eq':{k:str(n) for k,n in res.items()}})
assert set(endpoints)==set(prevol)=={0,1}
a=endpoints[0];b=endpoints[1]
assert abs(a['dose_mol']-3*b['dose_mol'])<=a['step_mol']+3*b['step_mol']
normal={id:e['dose_mol']/prevol[id] for id,e in endpoints.items()};resolution=sum(e['step_mol']/prevol[id] for id,e in endpoints.items())
assert abs(normal[0]-normal[1])<=resolution
assert abs(a['dose_mol']-D('.0015'))<=a['step_mol'] and abs(b['dose_mol']-D('.0005'))<=b['step_mol']
print(json.dumps({'met':2,'total':2,'criteria':{'QA-003-C1':True,'QA-003-C2':True},'solver_failures':0,'pre_titration_liquid_volume_l':{str(k):str(v) for k,v in prevol.items()},'liquid_volume_observation_source':'Actual liquid volumes on saved vessel lines at the two original measure commands; the headspace volume meter explicitly refuses open vessels. No headspace reading is substituted.','endpoint_dose_ratio':str(a['dose_mol']/b['dose_mol']),'endpoint_base_mol':{str(k):str(v['dose_mol']) for k,v in endpoints.items()},'volume_normalised_base_mol_per_l':{str(k):str(v) for k,v in normal.items()},'normalised_difference_mol_per_l':str(abs(normal[0]-normal[1])),'dosing_resolution_mol_per_l':str(resolution),'carrier_water_mol_per_l':str(mol_per_l),'known_element_charge_checks':checkpoints,'preserved_limit_keys':sorted({e['reason']['key'] for e in limits}),'scope':'Original model aliquot/stock and burette-resolution comparison using reported pre-titration liquid volumes, including titrant carrier water; no calibrated liquid-volume instrument or empirical titration precision claim.'},indent=2))
