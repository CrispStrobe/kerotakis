"""Review the original light/dark boundary experiment, without a photochemistry claim."""
from pathlib import Path
from decimal import Decimal as D
import json,sys
root=Path(sys.argv[1]); repo=Path(sys.argv[2])
rows=[json.loads(x) for x in (root/'batch3-diverse-bd-003.stdout').read_text().splitlines()]
reg=json.loads((root/'source-compositions.json').read_text())
forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in reg['compositions']}
keys=['H','O','C','N','charge'];bound=D('1e-10')
def ledger(v):
 out={k:D(0) for k in keys}
 assert not v.get('unresolved_materials')
 for p in v['contents']:
  n=D(str(p['moles']));assert n.is_finite() and n>=0
  for k,c in forms[p['species']].items():out[k]+=n*c
 return out
def amount(v,s):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==s),D(0))
events=[e for r in rows for e in r.get('events',[])]
assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented'] for e in events)
# Seal admits room air explicitly; start each closed ledger immediately after its seal.
baselines={};start={};checks=[];expected={}
for i,r in enumerate(rows):
 if 'bench' not in r:continue
 op=r['operator']
 if op['op']=='add':
  d=expected.setdefault(op['vessel'],{k:D(0) for k in keys})
  for k,c in forms[op['species']].items():d[k]+=D(str(op['moles']))*c
 if op['op']=='seal':
  air=next(e for e in r['events'] if e['event']=='vessel_sealed')
  d=expected[op['vessel']]
  for sp,fraction in [('N2','0.7901'),('O2','0.2095'),('CO2','0.0004')]:
   for k,c in forms[sp].items():d[k]+=D(str(air['trapped_air']))*D(fraction)*c

  v=next(v for v in r['bench']['vessels'] if v['id']==op['vessel']);baselines[v['id']]=ledger(v);start[v['id']]=v
 for v in r['bench']['vessels']:
  actual=ledger(v);wanted=expected.get(v['id'],{k:D(0) for k in keys});res={k:actual[k]-wanted[k] for k in keys}
  assert all(abs(x)<bound for x in res.values()),(i,v['id'],res)
  if v['id'] in baselines:
   assert v['headspace']['boundary']=='sealed'
   assert v['solution'].get('scope','complete')=='complete'
  checks.append({'step':i,'vessel':v['id'],'residual_mol_eq':{k:str(x) for k,x in res.items()}})
final=next(r['bench']['vessels'] for r in reversed(rows) if 'bench' in r)
assert len(final)==3
irr=[e for e in events if e['event']=='irradiated'];assert {e['vessel'] for e in irr}=={0,2} and len(irr)==2
assert all(e['photolysis_coupled'] is False and e['wavelength_nm']==254 and e['irradiance_w_m2']==1 for e in irr)
reactions=[e for e in events if e['event']=='reacted'];assert len(reactions)==2 and {e['vessel'] for e in reactions}=={0,1}
assert all(e['reaction']=='peroxide-decomposition' and e['catalyst'] is None and e['seconds']==600 for e in reactions)
assert reactions[0]['moles']==reactions[1]['moles']
conversions=[]
for v in final:
 id=v['id'];delta=amount(start[id],'H2O2')-amount(v,'H2O2');conversions.append(str(delta))
 if id<2:
  assert abs(amount(start[id],'H2O2')-D('0.0001'))<bound
  extent=D(str(next(e['moles'] for e in reactions if e['vessel']==id)))
  assert abs(delta-2*extent)<bound and delta>0
  assert abs(amount(v,'O2')-amount(start[id],'O2')-extent)<bound
 else:assert amount(v,'H2O2')==0
assert abs(D(conversions[0])-D(conversions[1]))<bound
press={e['vessel']:D(str(e['value'])) for e in events if e['event']=='measured' and e['instrument']=='pressure_gauge'}
assert set(press)=={0,1,2} and press[0]==press[1] and press[0]>press[2]
print(json.dumps({'met':4,'total':4,'criteria':{k:True for k in ['question','inventory','coverage','controls']},'solver_failures':0,'photolysis_coupled':False,'peroxide_consumed_mol':conversions,'pressure_kpa':{str(k):str(v) for k,v in press.items()},'addition_air_and_closed_checks':checks,'scope':'Original known-boundary comparison only: irradiation is stored, thermal peroxide conversion matches the dark arm. No supported light-dependent conversion, quantum yield or radical prediction is qualified.'},indent=2))
