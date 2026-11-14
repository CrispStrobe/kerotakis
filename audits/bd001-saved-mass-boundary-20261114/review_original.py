"""Original model-boundary comparison with independently conserved opaque polymer."""
from pathlib import Path
from decimal import Decimal as D
import json,sys
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch3-diverse-bd-001.stdout').read_text().splitlines()];data=json.loads((root/'source-properties.json').read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};mw={k:D(str(v)) for k,v in data['molar_masses'].items()};keys=['H','O','Na','Cl','charge'];expected={};opaque={};mass={};bound=D('1e-10');checks=[];swelling=[]
events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented'] for e in events)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op'] in ['add','add_material']:
  id=op['vessel'];want=expected.setdefault(id,{k:D(0) for k in keys});mass.setdefault(id,D(0))
  if op['op']=='add':
   n=D(str(op['moles']));mass[id]+=n*mw[op['species']]
   for k,c in forms[op['species']].items():want[k]+=n*c
  else:
   assert op['material']=='instant_snow_powder' and op['basis']=='mass_fraction' and D(str(op['total_amount']))==D('.2')
   e=next(e for e in r['events'] if e['event']=='material_added');assert not e['components'] and e['unresolved_amount']==.2
   opaque[id]=D('.2');mass[id]+=D('.2')
 if 'bench' in r:
  for v in r['bench']['vessels']:
   actual={k:D(0) for k in keys}
   for p in v['contents']:
    n=D(str(p['moles']));assert n.is_finite() and n>=0
    for k,c in forms[p['species']].items():actual[k]+=n*c
   want=expected.get(v['id'],{k:D(0) for k in keys});res={k:actual[k]-want[k] for k in keys};assert all(abs(n)<bound for n in res.values())
   unresolved=v.get('unresolved_materials',[])
   if v['id'] in opaque:
    assert len(unresolved)==1;u=unresolved[0];assert u['material']=='instant_snow_powder' and u['basis']=='mass_fraction' and D(str(u['amount']))==opaque[v['id']]
    assert u['recipe_id']=='school/sodium-polyacrylate-instant-snow' and u['recipe_version']==1
   else:assert not unresolved
   checks.append({'step':i,'vessel':v['id'],'residual_mol_eq':{k:str(n) for k,n in res.items()}})
 for scene in r.get('scene',{}).get('vessels',[]):
  id=scene['id'];assert abs(D(str(scene['mass_g']))-mass.get(id,D(0)))<bound
  if i>=5:
   note=next(n for n in scene['notes'] if n['key']=='scene.swelling');slots={k:D(v['number']['text']) for k,v in note['slots']};assert slots['water']==20 and slots['ratio']==100
   swelling.append({'step':i,'vessel':id,'retained_water_g':str(slots['water']),'ratio_g_per_g':str(slots['ratio'])})
assert swelling and {s['vessel'] for s in swelling}=={0,1}
cond={e['vessel']:e for e in events if e['event']=='measured' and e['instrument']=='conductivity_meter'};assert set(cond)=={0,1} and cond[1]['value']>cond[0]['value']*1000
assert cond[1]['note_reason']['key']=='measured.conductivity-past-the-dilute-limit'
final=next(r['bench']['vessels'] for r in reversed(rows) if 'bench' in r)
for v in final:
 amounts={p['species']:D(str(p['moles'])) for p in v['contents']};assert abs(amounts['water']*mw['water']-30)<bound
 if v['id']==1:assert abs(amounts['Na+']-D('0.01711068904744794'))<bound and amounts['Na+']==amounts['Cl-']
 else:assert 'Na+' not in amounts and 'Cl-' not in amounts
print(json.dumps({'met':4,'total':4,'criteria':{k:True for k in ['question','inventory','coverage','controls']},'solver_failures':0,'known_element_charge_checks':checks,'opaque_polymer_g':{str(k):str(v) for k,v in opaque.items()},'final_total_mass_g':{str(k):str(v) for k,v in mass.items()},'post_salt_swelling':swelling,'conductivity_us_cm':{str(k):v['value'] for k,v in cond.items()},'preserved_conductivity_limit':cond[1]['note_reason']['key'],'scope':'Original intentionally designed boundary comparison: fixed100g/g retention persists after salt, despite changed ionic environment. Salt-sensitive real-gel swelling is not qualified. Retained water is a subset of total solvent, not extra water; opaque polymer has no invented atom formula.'},indent=2))
