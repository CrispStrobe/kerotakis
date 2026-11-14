"""Original open-boundary cryogen comparison, retaining full phase/vent ownership."""
from pathlib import Path
from decimal import Decimal as D
import json,sys
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch3-diverse-th-001.stdout').read_text().splitlines()];data=json.loads((root/'source-properties.json').read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};mw={k:D(str(v)) for k,v in data['molar_masses'].items()};keys=['H','O','C','N','charge'];bound=D('1e-10');expected={};checks=[];doses={};vents={};thermo={};ph={};phase_events=[]
assert forms['dry_ice']==forms['CO2'] and forms['liquid_nitrogen']==forms['N2']
events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_absorbed','reacted'] for e in events)
assert any(e['event']=='hazard_warning' and e['rule']=='liquid-nitrogen-handling' for e in events)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':
  id=op['vessel'];want=expected.setdefault(id,{k:D(0) for k in keys});n=D(str(op['moles']))
  for k,c in forms[op['species']].items():want[k]+=n*c
  if op['species'] in ['dry_ice','liquid_nitrogen']:
   doses[id]=(op['species'],n);assert abs(n*mw[op['species']]-D('.1'))<bound
   phase=next(e for e in r['events'] if e['event']=='state_changed');assert phase['species']==op['species'] and D(str(phase['moles']))==n
   assert phase['to']=='gas' and phase['kind']==('sublimation' if id==0 else 'boiling')
   phase_events.append(phase)
 for e in r.get('events',[]):
  if e['event']=='gas_evolved':
   id=e['vessel'];n=D(str(e['moles']));assert id in doses and n==doses[id][1]
   assert e['species']==('CO2' if id==0 else 'N2');assert id not in vents;vents[id]=e
   for k,c in forms[e['species']].items():expected[id][k]-=n*c
  if e['event']=='measured':
   id=e['vessel'];v=next(v for v in r['bench']['vessels'] if v['id']==id)
   if e['instrument']=='thermometer':thermo[id]=D(str(e['value']));assert abs(thermo[id]-(D(str(v['temperature']))-D('273.15')))<bound
   elif e['instrument']=='ph_meter':ph.setdefault(id,[]).append(D(str(e['value'])));assert abs(D(str(e['value']))-D(str(v['solution']['ph'])))<bound
 if 'bench' not in r:continue
 for v in r['bench']['vessels']:
  assert v['headspace']['boundary']=='open' and not v.get('unresolved_materials') and v.get('pending_co2_transfer_mol',0)==0
  actual={k:D(0) for k in keys}
  for p in v['contents']:
   n=D(str(p['moles']));assert n.is_finite() and n>=0
   for k,c in forms[p['species']].items():actual[k]+=n*c
   assert p['species']=='water' and p['phase']=='liquid'
  want=expected.get(v['id'],{k:D(0) for k in keys});res={k:actual[k]-want[k] for k in keys};assert all(abs(n)<bound for n in res.values()),(i,v['id'],res)
  if v.get('solution'):
   info=v['solution'];assert info['scope']=='solvent_only'
   assert {p['name'] for p in info['species']}=={'H+','OH-'}
   ions={p['name']:D(str(p['molality'])) for p in info['species']};assert ions['H+']==ions['OH-']
  checks.append({'step':i,'vessel':v['id'],'residual_mol_eq':{k:str(n) for k,n in res.items()}})
assert set(doses)==set(vents)==set(thermo)==set(ph)=={0,1};assert all(len(x)==2 for x in ph.values())
assert thermo[0]<thermo[1]<25 and all(abs(x-D(7))<D('.01') for xs in ph.values() for x in xs)
assert len(phase_events)==2
final=next(r['bench']['vessels'] for r in reversed(rows) if 'bench' in r);assert all(v['elapsed_seconds']==120 for v in final)
print(json.dumps({'met':3,'total':3,'criteria':{k:True for k in ['question','inventory','coverage']},'solver_failures':0,'known_element_charge_checks':checks,'phase_changes':phase_events,'explicit_vent_mol':{str(k):{'species':v['species'],'moles':str(v['moles'])} for k,v in vents.items()},'thermometer_c':{str(k):str(v) for k,v in thermo.items()},'reported_ph_before_after_wait':{str(k):[str(x) for x in v] for k,v in ph.items()},'retained_co2_and_nitrogen_mol':'0; the complete input doses leave through explicit open-boundary events','scope':'Original equal-mass cooling/phase/vent comparison with explicit solvent-only pH coverage. No retained-CO2 acidification, bubble-to-liquid mass-transfer kinetics, universal cooling efficiency or cryogen handling prediction is qualified.'},indent=2))
