"""Original inventory comparison and source-bound runtime energy interpretation."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,math,hashlib,re
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'new120-sp-004.stdout').read_text().splitlines()];props={p['species_id']:p for p in json.loads((root/'source-properties.json').read_text())};b=json.loads((root/'binding.json').read_text());h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert h(root/'source-properties.json')==b['source_properties_sha256'] and h(root/'source-excerpts.json')==b['source_excerpts_sha256'];source=json.loads((root/'source-excerpts.json').read_text());code={k:''.join(e['text'] for e in v['excerpts']) for k,v in source.items()};assert 'return en' in code['i18n'] and 'Self::interpolate(self.t(key, en), vars)' in code['i18n'];assert 'register lv3' in (root/'original.lab').read_text();assert 'if !model.is_empty()' in code['render'];template=re.search(r'"event.distilled.lv3",\s*"([^"]+)"',code['render']).group(1);assert 'latent heat' in template and 'outside the vessel ledger' in template;assert 'externally' in code['bench'] and 'source\'s sensible' in code['bench'];constant=lambda k:D(re.search(r'pub const '+k+r': f64 = ([0-9.]+)',code['vle']).group(1));lv={'water':constant('WATER_HVAP_KJ_PER_MOL'),'ethanol':constant('ETHANOL_HVAP_KJ_PER_MOL')}
keys=['C','H','O','charge'];expected={};checks=[];mass_checks=[];transfers=[];bound=D('1e-10')
def amount(v,s):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==s),D(0))
def add(t,s,n):
 for k,c in props[s]['atoms'].items():t[k]+=n*D(c)
 t['charge']+=n*D(props[s]['charge'])
for i,r in enumerate(rows):
 op=r['operator'];events=r.get('events',[]);assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented','reacted'] for e in events)
 if op['op']=='add':add(expected.setdefault(op['vessel'],{k:D(0) for k in keys}),op['species'],D(str(op['moles'])))
 if op['op']=='distil':
  es=[e for e in events if e['event']=='distilled'];assert len(es)==1;e=es[0];src=op['from'];dst=op['to'];assert (src,dst) in [(0,2),(1,3)] and e['from']==src and e['to']==dst;assert e['energy_kj']==op['energy']/1000==(10 if src==0 else 20) and e['stages']==op['stages']==1 and e['azeotropic'] is False and not e.get('model');assert len(e['components'])==2 and {x[0] for x in e['components']}=={'water','ethanol'}
  budget_kj=D(0);collected=D(0);before=rows[i-1]['bench']['vessels'][src];after=r['bench']['vessels'][src];receiver=r['bench']['vessels'][dst]
  assert before['temperature']==after['temperature']==receiver['temperature']==343.15;assert e['at']>before['temperature'] and e['ended']>e['at'];assert not rows[i-1]['bench']['vessels'][dst]['contents']
  for species,n in e['components']:
   n=D(str(n));assert n>0 and n==D(str(e[species]));assert amount(before,species)-amount(after,species)>0;assert abs(amount(before,species)-amount(after,species)-n)<bound;assert amount(receiver,species)==n
   add(expected.setdefault(src,{k:D(0) for k in keys}),species,-n);add(expected.setdefault(dst,{k:D(0) for k in keys}),species,n);budget_kj+=n*lv[species];collected+=n*D(str(props[species]['molar_mass']))
  assert abs(budget_kj-D(str(e['energy_kj'])))<D('1e-10')
  fields={'from':f'v{src+1}','to':f'v{dst+1}','stages':str(e['stages']),'at':f"{e['at']:.2f}",'ended':f"{e['ended']:.2f}",'energy_kj':f"{e['energy_kj']:.2f}",'azeotrope':''};derived=template.format(**fields)
  transfers.append({'from':src,'to':dst,'declared_latent_budget_kj':e['energy_kj'],'collected_mol':{s:str(n) for s,n in e['components']},'collected_mass_g':str(collected),'reference_latent_sum_kj':str(budget_kj),'latent_arithmetic_residual_kj':str(budget_kj-D(str(e['energy_kj']))),'vessel_temperature_k':before['temperature'],'reported_boil_start_k':e['at'],'reported_boil_end_k':e['ended'],'source_derived_runtime_explanation':derived})
 for v in r['bench']['vessels']:
  id=v['id'];assert not any(v.get(k) for k in ['unresolved_materials','material_objects','surfaces','exchanges','electrodes','solid_solutions','adsorbed']);assert v['pending_co2_transfer_mol']==0;actual={k:D(0) for k in keys};mass=D(0)
  for p in v['contents']:
   n=D(str(p['moles']));assert n.is_finite() and n>=0 and p['phase']=='liquid';add(actual,p['species'],n);mass+=n*D(str(props[p['species']]['molar_mass']))
  res={k:actual[k]-expected.get(id,{k:D(0) for k in keys})[k] for k in keys};assert all(abs(n)<bound for n in res.values());checks.append({'step':i,'vessel':id,'residual_mol_eq':{k:str(n) for k,n in res.items()}})
  if 'scene' in r:
   scene=next(x for x in r['scene']['vessels'] if x['id']==id);mr=mass-D(str(scene['mass_g']));assert abs(mr)<D('1e-10');mass_checks.append({'step':i,'vessel':id,'known_mass_g':str(mass),'scene_mass_residual_g':str(mr)})
  else:
   assert op['op']=='inspect', 'Unexpected missing scene for a state-changing operation'
assert len(transfers)==2 and D(transfers[1]['collected_mass_g'])>D(transfers[0]['collected_mass_g'])>0
final=rows[-1]['bench']['vessels'];left={}
for src,dst in [(0,2),(1,3)]:
 for s,n in [('water',D(100)),('ethanol',D(10))]:assert abs((amount(final[src],s)+amount(final[dst],s))*D(str(props[s]['molar_mass']))-n)<D('1e-10')
 left[src]=str(sum((amount(final[src],s)*D(str(props[s]['molar_mass'])) for s in props),D(0)))
assert D(left[0])>D(left[1]);ratio=D(transfers[1]['collected_mass_g'])/D(transfers[0]['collected_mass_g']);assert ratio!=2
print(json.dumps({'met':2,'total':2,'criteria':{'SP-004-C1':True,'SP-004-C2':True},'solver_failures':0,'known_atom_charge_checks':checks,'known_mass_checks':mass_checks,'transfers':transfers,'remaining_feed_mass_g':left,'collected_mass_ratio_20_to_10':str(ratio),'runtime_explanation_key':'event.distilled.lv3','runtime_template':template,'explanation_evidence':'Exact selected source LV3 template plus actual saved empty-model Distilled event slots. Displayed explanations are explicitly source-derived decoding; historical JSON has no captured rendered prose and no new CLI observation is claimed.','scope':'Original budget-dependent transfer/feed comparison and declared runtime energy-boundary interpretation only. The command budget is withdrawn-condensate reference latent energy; burner/reflux/sensible heat is external and not computed in vessel ledgers. Boil-event temperatures are not vessel thermometer observations. No apparatus total-energy closure, empirical VLE/recovery or calorimetry is qualified. Physical components counted once, not duplicated by legacy event mirrors. Historical executable checksum remains absent.'},indent=2))
