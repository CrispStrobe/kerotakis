"""Original pressure/phase inventory and cleared-window/reference-property inspection."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,math,hashlib,re
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch2-tc-005.stdout').read_text().splitlines()];data=json.loads((root/'source-compositions.json').read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};keys=['H','O','C','N','charge'];bound=D('1e-10');expected={};checks=[];regulated=set();pv=[];routes={};apparatus={};gauges={};thermo={};energy={};admitted={}
excerpts=(root/'source-boundary-excerpts.json').read_bytes();b=json.loads((root/'binding.json').read_text());assert hashlib.sha256(excerpts).hexdigest()==b['boundary_excerpts_sha256'];assert 'volume move to hold a set' in excerpts.decode();boundary_code=''.join(p['text'] for p in json.loads(excerpts));gas_r=float(re.search(r'const R_LITRE_PASCAL: f64 = ([0-9_.]+);',boundary_code).group(1).replace('_',''))
events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented'] for e in events)
def add(d,sp,n):
 for k,c in forms[sp].items():d[k]+=n*c
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':add(expected.setdefault(op['vessel'],{k:D(0) for k in keys}),op['species'],D(str(op['moles'])))
 if op['op']=='regulate':
  id=op['vessel'];regulated.add(id);e=next(e for e in r['events'] if e['event']=='vessel_pressure_controlled');n=D(str(e['trapped_gas']));admitted[id]=str(n)
  assert e['initial_volume']==.1 and e['pressure']==(100000 if id==0 else 200000)
  for sp,f in [('N2','.7901'),('O2','.2095'),('CO2','.0004')]:add(expected[id],sp,n*D(f))
 for e in r.get('events',[]):
  if e['event']=='boiling_point_routed':routes[e['vessel']]=e
  elif e['event']=='transition_point_read':apparatus[e['vessel']]=e['reading']
  elif e['event']=='energy_transferred':
   assert e['delivered_j']==e['requested_j']==8000 and e['capped'] is False;energy[e['vessel']]=e
  elif e['event']=='measured':
   if e['instrument']=='pressure_gauge':gauges[e['vessel']]=e['value']
   elif e['instrument']=='thermometer':thermo[e['vessel']]=e['value']
 if 'bench' not in r:continue
 for v in r['bench']['vessels']:
  id=v['id'];actual={k:D(0) for k in keys};assert not v.get('unresolved_materials')
  for p in v['contents']:
   n=D(str(p['moles']));assert n.is_finite() and n>=0;add(actual,p['species'],n)
  res={k:actual[k]-expected.get(id,{k:D(0) for k in keys})[k] for k in keys};assert all(abs(n)<bound for n in res.values()),(i,id,res)
  checks.append({'step':i,'vessel':id,'residual_mol_eq':{k:str(n) for k,n in res.items()}})
  if id in regulated:
   head=v['headspace'];assert head['boundary']=='pressure_controlled' and head['pressure']==v['pressure']==(100000 if id==0 else 200000)
   n=sum((p['moles'] for p in v['contents'] if p['phase']=='gas'),0.0);pred=n*gas_r*v['temperature']/v['pressure'];err=abs(pred-head['volume']);assert err<1e-10
   pv.append({'step':i,'vessel':id,'volume_l':head['volume'],'pv_volume_residual_l':err})
assert set(routes)==set(apparatus)==set(gauges)==set(thermo)==set(energy)=={0,1}
assert routes[0]['route']=='cleared_correlation' and routes[0]['pressure_kpa']==gauges[0]==100
assert routes[1]['route']=='pressure_outside_cleared_window' and routes[1]['pressure_kpa']==gauges[1]==200 and routes[1]['shifted_by']==0
assert abs(routes[0]['boiling']-273.15-thermo[0])<1e-10 and abs(routes[1]['boiling']-273.15-thermo[1])<1e-10
for reading in apparatus.values():assert reading['verdict']=='not_isolated' and reading['value_c'] is None and reading['lowest_component_c']==100
final=next(r['bench']['vessels'] for r in reversed(rows) if 'bench' in r);phases={}
for v in final:
 id=v['id'];phases[id]=[p for p in v['contents'] if p['species']=='water'];assert {p['phase'] for p in phases[id]}=={'liquid','gas'} and all(p['moles']>0 for p in phases[id]);assert v['headspace']['volume']>.7
print(json.dumps({'met':2,'total':2,'criteria':{'TC-005-C1':True,'TC-005-C2':True},'solver_failures':0,'known_element_charge_checks':checks,'pressure_controlled_volume_checks':pv,'admitted_room_air_mol':admitted,'operative_pressure_kpa':gauges,'reported_thermometer_c':thermo,'boiling_routes':routes,'boiling_apparatus':apparatus,'reported_energy_allocation':energy,'final_water_phases':phases,'scope':'Original actual pressure/phase-inventory and property-domain inspection only. Variable-volume owned gas, not a fixed100mL or atmospheric reservoir. The200kPa route is outside its cleared correlation and uses a flagged reference fallback. No validated2bar boiling law, high-pressure plateau or calorimetric calibration is qualified; no numeric apparatus reading is invented.'},indent=2))
