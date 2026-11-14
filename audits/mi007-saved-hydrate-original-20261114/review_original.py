"""Independent saved hydrate-input, dissolution and known inventory review."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,hashlib
root=Path(sys.argv[1]);repo=Path(sys.argv[2]);registry=repo/'data/registry/registry-source-v1.json';data=json.loads(registry.read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};forms['base_equivalents']=forms['OH-'];keys=['H','O','Cu','S','Cl','charge'];bound=D('1e-10')
rows=[json.loads(x) for x in (root/'batch3-diverse-mi-007.stdout').read_text().splitlines()];expected={};checks=[];events=[e for r in rows for e in r.get('events',[])]
assert not any(e['event'] in ['solver_failed','gas_absorbed','gas_evolved'] for e in events)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':
  d=expected.setdefault(op['vessel'],{k:D(0) for k in keys})
  for k,c in forms[op['species']].items():d[k]+=D(str(op['moles']))*c
 if 'bench' not in r:continue
 for v in r['bench']['vessels']:
  d=expected.setdefault(v['id'],{k:D(0) for k in keys});actual={k:D(0) for k in keys}
  assert not v.get('unresolved_materials')
  for p in v['contents']:
   n=D(str(p['moles']));assert n.is_finite() and n>=0
   for k,c in forms[p['species']].items():actual[k]+=n*c
  residual={k:actual[k]-d[k] for k in keys};assert all(abs(x)<bound for x in residual.values()),(i,v['id'],residual)
  checks.append({'step':i,'vessel':v['id'],'residual_mol_eq':{k:str(x) for k,x in residual.items()}})
def amount(v,key):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==key),D(0))
final=rows[-1]['bench']['vessels'];assert len(final)==2
for v in final:
 assert abs(amount(v,'Cu+2')-D('0.0008'))<bound and abs(amount(v,'SO4-2')-D('0.0002'))<bound
 assert abs(amount(v,'H+')-D('0.0003'))<bound
 assert not any(p['phase']=='solid' and p['moles']>1e-12 for p in v['contents'])
 assert v['solution'] and v['solution'].get('scope','complete')=='complete'
water_difference=amount(final[0],'water')-amount(final[1],'water');assert abs(water_difference-D('0.0002'))<bound
m={e['vessel']:D(str(e['value'])) for e in events if e['event']=='measured' and e['instrument']=='balance'};mass_difference=m[0]-m[1];assert abs(mass_difference-D('0.0002')*D('18.015'))<bound
assert forms['langite']['Cu']==forms['brochantite']['Cu']==4 and forms['langite']['S']==forms['brochantite']['S']==1
assert forms['langite']['H']-forms['brochantite']['H']==2 and forms['langite']['O']-forms['brochantite']['O']==1
warnings=[e for e in events if e['event']=='not_yet_modeled'];assert warnings;assert all(e.get('reason',{}).get('key')=='not-modeled.supersaturated-below-formation-temperature' for e in warnings)
print(json.dumps({'criteria':{'question':True,'inventory':True,'coverage':True},'met':3,'total':3,'solver_failures':0,'known_element_charge_checks':checks,'registered_hydration_difference':'one H2O formula unit; reference registry/database formulas only','preacid_balance_g':{str(k):str(v) for k,v in m.items()},'mass_difference_g':str(mass_difference),'expected_hydration_mass_difference_g':'0.003603','final_water_difference_mol':str(water_difference),'final_copper_mol':[str(amount(v,'Cu+2')) for v in final],'final_sulfate_mol':[str(amount(v,'SO4-2')) for v in final],'final_acid_basis_mol':[str(amount(v,'H+')) for v in final],'preserved_model_limit_keys':sorted({e['reason']['key'] for e in warnings}),'registry_sha256':hashlib.sha256(registry.read_bytes()).hexdigest(),'scope':'Declared reference-phase formula and actually observed acid dissolution only; no mineral phase-transition, identical-rate or natural-crystal hydration claim.'},indent=2))
