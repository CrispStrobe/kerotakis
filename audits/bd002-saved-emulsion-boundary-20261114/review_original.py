"""Original independent emulsion/conversion teaching-role and physical-pool review."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,math,re,hashlib
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch3-diverse-bd-002.stdout').read_text().splitlines()];data=json.loads((root/'source-properties.json').read_text());mw={k:D(str(v)) for k,v in data['molar_masses'].items()};bound=D('1e-10');water={};enzyme={};opaque={};mass={};checks=[];converted={0:D(0),1:D(0)};hist={0:[],1:[]}
code=(root/'source-enzyme.rs').read_text();binding=json.loads((root/'binding.json').read_text());assert hashlib.sha256(code.encode()).hexdigest()==binding['source_enzyme_sha256'];mw['lipase']=D(re.search(r'enzyme_species\(\s*"lipase",\s*([0-9_.]+),',code).group(1).replace('_',''));assert 'no molecular-formula claim' in code
recipes={r['canonical_key']:r for r in data['material_recipes']}
events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented'] for e in events)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op'] in ['add','add_material']:
  id=op['vessel'];water.setdefault(id,D(0));enzyme.setdefault(id,D(0));opaque.setdefault(id,{});mass.setdefault(id,D(0))
  if op['op']=='add':
   n=D(str(op['moles']));assert op['species'] in ['water','lipase'];mass[id]+=n*mw[op['species']]
   if op['species']=='water':water[id]+=n
   else:enzyme[id]+=n;assert abs(n*mw['lipase']-D('.05'))<bound
  else:
   recipe=recipes[op['material']];assert op['basis']=='mass_fraction' and op['recipe_id']==recipe['id'] and op['recipe_version']==recipe['version']==1
   e=next(e for e in r['events'] if e['event']=='material_added');total=D(str(op['total_amount']));mass[id]+=total;unresolved=D(str(e['unresolved_amount']));opaque[id][op['material']]=unresolved
   assert abs(unresolved-total*D(str(recipe['unresolved_fraction']['lower'])))<bound
   for p in e['components']:
    assert p['species']=='water';water[id]+=D(str(p['moles']));assert abs(D(str(p['moles']))*mw['water']-D(str(p['basis_amount'])))<bound
   assert abs(sum((D(str(p['basis_amount'])) for p in e['components']),D(0))+unresolved-total)<bound
 for e in r.get('events',[]):
  if e['event']=='stirred':assert e['rate_coupled'] is False and e['rpm']==500 and e['seconds']==30
  if e['event']=='enzyme_hydrolysed':
   id=e['vessel'];assert e['family']=='lipase' and e['material']=='vegetable oil';converted[id]+=D(str(e['hydrolysed_mass_g']));assert abs(converted[id]-D(str(e['converted_fraction']))*opaque[id]['vegetable_oil'])<bound
   hist[id].append({'seconds':e['seconds'],'converted_fraction':e['converted_fraction'],'increment_g':e['hydrolysed_mass_g']})
 if 'bench' not in r:continue
 for v in r['bench']['vessels']:
  id=v['id'];amounts={p['species']:D(str(p['moles'])) for p in v['contents']};assert set(amounts)<= {'water','lipase'}
  rw=amounts.get('water',D(0))-water.get(id,D(0));re=amounts.get('lipase',D(0))-enzyme.get(id,D(0));assert abs(rw)<bound and abs(re)<bound
  assert all(p['phase']==('liquid' if p['species']=='water' else 'aqueous') and p['moles']>=0 for p in v['contents'])
  us={u['material']:u for u in v.get('unresolved_materials',[])};assert set(us)==set(opaque.get(id,{}))
  for material,n in opaque.get(id,{}).items():
   u=us[material];assert D(str(u['amount']))==n and u['basis']=='mass_fraction' and u['recipe_id']==recipes[material]['id'] and u['recipe_version']==1
   if u.get('enzyme_hydrolysis'):
    h=u['enzyme_hydrolysis'];assert h['family']=='lipase' and h['carried_enzyme_denatured'] is False;assert abs(D(str(h['converted_fraction']))*n-converted[id])<bound
  actual_mass=amounts.get('water',D(0))*mw['water']+amounts.get('lipase',D(0))*mw['lipase']+sum(opaque.get(id,{}).values(),D(0));rm=actual_mass-mass.get(id,D(0));assert abs(rm)<bound
  checks.append({'step':i,'vessel':id,'known_water_H_residual_mol':str(2*rw),'known_water_O_residual_mol':str(rw),'known_charge_residual_eq':'0','opaque_enzyme_residual_mol':str(re),'total_mass_residual_g':str(rm)})
 for scene in r.get('scene',{}).get('vessels',[]):assert abs(D(str(scene['mass_g']))-mass.get(scene['id'],D(0)))<bound
assert hist[0]==hist[1] and [h['seconds'] for h in hist[0]]==[30,1800] and converted[0]==converted[1]>0
em=[e for e in events if e['event']=='emulsion_changed'];assert len(em)==2 and all(e['vessel']==0 and e['half_life_seconds']==300 for e in em)
assert abs(em[1]['to_dispersed_fraction']-em[0]['to_dispersed_fraction']*2**(-1800/300))<1e-12
assert abs(em[0]['to_dispersed_fraction']-.552)<1e-12 and abs(em[1]['to_dispersed_fraction']-.008625)<1e-12
post=rows[10]['scene']['vessels'];assert any(n['key']=='scene.emulsion' for n in post[0]['notes']);assert any(n['key']=='scene.material-layer' for n in post[1]['notes']) and not any(n['key']=='scene.emulsion' for n in post[1]['notes'])
final=next(r['bench']['vessels'] for r in reversed(rows) if 'bench' in r);assert all(v['elapsed_seconds']==1830 for v in final)
print(json.dumps({'met':4,'total':4,'criteria':{k:True for k in ['question','inventory','coverage','controls']},'solver_failures':0,'physical_pool_checks':checks,'expected_total_mass_g':{str(k):str(v) for k,v in mass.items()},'equal_conversion_history':hist[0],'conversion_subset_g':{str(k):str(v) for k,v in converted.items()},'emulsion_events':em,'scope':'Original independent teaching-role comparison only. Oil/soap remain conserved opaque material pools; conversion is state metadata, not extra product mass. No molecular hydrolysis stoichiometry, droplet-area rate law, detergent compatibility or real-lipase kinetic calibration is qualified.'},indent=2))
