"""Read unchanged KI003 comparison and actual particle-size activity coupling."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,math,hashlib,re
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch2-ki-003.stdout').read_text().splitlines()];binding=json.loads((root/'binding.json').read_text());props={p['species_id']:p for p in json.loads((root/'source-properties.json').read_text())};h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert h(root/'source-properties.json')==binding['source_properties_sha256'];assert h(root/'source-excerpts.json')==binding['source_excerpts_sha256'];source=json.loads((root/'source-excerpts.json').read_text());code={k:''.join(e['text'] for e in v['excerpts']) for k,v in source.items()};number=lambda text,key:float(re.search(r'\b'+key+r': f64 = ([0-9_.]+)',text).group(1).replace('_',''))
gas_r=number(code['constants'],'GAS_CONSTANT');kin=code['kinetics'];parse=lambda key:float(re.search(r'\b'+key+r': ([0-9_.e]+)',kin).group(1).replace('_',''));a=parse('pre_exponential');ea=float(re.search(r'species: "MnO2",\s*activation_energy: ([0-9_\.]+)',kin).group(1).replace('_',''));reference_area=parse('reference_area_m2');default_d=parse('default_diameter_um')
assert '.particle_size_um' in kin and 'unwrap_or(default_diameter_um)' in kin and '.clamp(0.35, 1.0)' in kin and 'rate *= activity' in kin
assert code['clock'].index('Box::new(GravityClock)')<code['clock'].index('Box::new(CuratedKineticsClock)');assert a==5.6e7 and ea==58000 and reference_area==.001 and default_d==100
expected={};checks=[];ground={};intervals=[];budget=D('1e-10');seen_10=False;seen_30=False;keys=['H','O','Mn','charge'];catalyst={};clocks={}
def moles(v,s):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==s),D(0))
def add(t,sp,n):
 for k,c in props[sp]['atoms'].items():t[k]+=n*D(c)
 t['charge']+=n*D(props[sp]['charge'])
def lot(v):
 lots=[l for l in v['lots'] if l['species']=='MnO2'];assert len(lots)==1;return lots[0]
for i,r in enumerate(rows):
 op=r['operator'];events=r.get('events',[]);assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented'] for e in events)
 if op['op']=='add':
  id=op['vessel'];add(expected.setdefault(id,{k:D(0) for k in keys}),op['species'],D(str(op['moles'])))
  if op['species']=='H2O2':assert op['moles']==.001
  if op['species']=='MnO2':
   catalyst[id]=D(str(op['moles']));assert abs(catalyst[id]*D(str(props['MnO2']['molar_mass']))-D('.001' if id==0 else '.01'))<D('1e-15')
 for v in r['bench']['vessels']:
  id=v['id'];assert not any(v.get(k) for k in ['unresolved_materials','material_objects','surfaces','exchanges','adsorbed','electrodes','solid_solutions']);assert v['pending_co2_transfer_mol']==0
  actual={k:D(0) for k in keys}
  for p in v['contents']:
   n=D(str(p['moles']));assert n.is_finite() and n>=0;add(actual,p['species'],n)
  res={k:actual[k]-expected.get(id,{k:D(0) for k in keys})[k] for k in keys};assert all(abs(n)<budget for n in res.values()),(i,id,res);checks.append({'step':i,'vessel':id,'residual_mol_eq':{k:str(x) for k,x in res.items()}})
  if id in catalyst:assert moles(v,'MnO2')==catalyst[id]
 if op['op']=='grind':
  id=op['vessel'];e=next(e for e in events if e['event']=='ground');v=r['bench']['vessels'][id];l=lot(v);d=10 if id==0 else 100;assert e['diameter_um']==l['particle_size_um']==d and e['rate_coupled'] is True and l['suspended_fraction']==1
  area=6*float(catalyst[id])*props['MnO2']['molar_mass']/props['MnO2']['density_g_ml']*1e-6/(d*1e-6);assert abs(area-e['surface_area_m2'])<1e-15;ground[id]={'diameter_um':d,'nominal_area_m2':area,'default_area_m2':area*d/default_d}
 if op['op']!='wait':continue
 dt=op['seconds'];old={v['id']:v for v in rows[i-1]['bench']['vessels']};assert dt in [10,20]
 paired=[]
 for v in r['bench']['vessels']:
  id=v['id'];before=old[id];l=lot(v);assert l['particle_size_um']==ground[id]['diameter_um'];settle=next((e for e in events if e['event']=='gravity_settled' and e['vessel']==id),None)
  if settle is None:
   assert lot(before)['suspended_fraction']==l['suspended_fraction']==0, 'Only an already settled, unchanged lot may omit the settling event'
  else:
   sep=settle['separations'];assert len(sep)==1;s=sep[0];assert s['species']=='MnO2' and not s['particle_size_assumed'] and s['particle_diameter_um']==l['particle_size_um'];assert abs(l['suspended_fraction']-lot(before)['suspended_fraction']*(1-s['separated_fraction']))<1e-14
  clocks[id]=clocks.get(id,0)+dt;assert v['elapsed_seconds']==clocks[id];n0=moles(before,'H2O2');n=moles(v,'H2O2');extent=(n0-n)/2;react=next(e for e in events if e['event']=='reacted' and e['vessel']==id);gas=next(e for e in events if e['event']=='gas_produced' and e['vessel']==id);assert react['reaction']==gas['reaction']=='peroxide-decomposition' and gas['species']=='O2' and react['seconds']==dt
  assert abs(extent-D(str(react['moles'])))<budget and abs(extent-D(str(gas['moles'])))<budget;assert abs(moles(v,'O2')-moles(before,'O2')-extent)<budget;assert abs(moles(v,'water')-moles(before,'water')-2*extent)<budget
  contact=max(.35,min(1,l['suspended_fraction']));activity=ground[id]['nominal_area_m2']/reference_area*contact;k=a*math.exp(-ea/(gas_r*before['temperature']))*activity;pred=float(n0)*math.exp(-2*k*dt);assert abs(pred-float(n))<float(budget)
  default_activity=ground[id]['default_area_m2']/reference_area*contact;default_pred=float(n0)*math.exp(-2*a*math.exp(-ea/(gas_r*before['temperature']))*default_activity*dt)
  if id==0:assert abs(default_pred-float(n))>100*float(budget)
  paired.append({'vessel':id,'elapsed_seconds':v['elapsed_seconds'],'fraction_consumed':str((D('.001')-n)/D('.001')),'remaining_h2o2_mol':str(n),'suspended_fraction':l['suspended_fraction'],'contact_factor':contact,'activity_multiplier':activity,'interval_start_temperature_k':before['temperature'],'analytic_remaining_mol':pred,'analytic_residual_mol':pred-float(n),'default_diameter_prediction_mol':default_pred})
 assert paired[0]['elapsed_seconds']==paired[1]['elapsed_seconds'];assert D(paired[0]['fraction_consumed'])>D(paired[1]['fraction_consumed'])>0;assert paired[0]['activity_multiplier']>paired[1]['activity_multiplier'];intervals.append(paired)
assert set(ground)=={0,1} and len(intervals)==2 and clocks=={0:30,1:30};assert abs(ground[0]['nominal_area_m2']-ground[1]['nominal_area_m2'])<1e-15
print(json.dumps({'met':2,'total':2,'criteria':{'KI-003-C1':True,'KI-003-C2':True},'solver_failures':0,'known_atom_charge_checks':checks,'ground_geometry':ground,'paired_consumption_and_activity':intervals,'source_rate_parameters':{'pre_exponential':a,'activation_energy_j_mol':ea,'gas_constant':gas_r,'reference_area_m2':reference_area},'forecast_status':'Original nominal-area forecast preserved; similar consumption is not confirmed. Faster settling reduces the larger particles\u2019 modeled contact, so equal nominal area does not imply equal activity during these unstirred waits.','scope':'Original matched-time fractional consumption comparison and explicit diameter-to-activity inspection. Analytic checks verify the installed editorial teaching law, not empirical kinetics, BET/pore area, suspension/contact calibration, calorimetry or realistic catalyst performance. O2 remains in physical contents; gas_produced is not an external sink. Solvent-only pH is not used as a peroxide measurement.'},indent=2))
