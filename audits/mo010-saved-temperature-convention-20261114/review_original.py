"""Saved original material and conductivity-convention review; no temperature calibration."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,re,math,hashlib
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch2-mo-010.stdout').read_text().splitlines()];data=json.loads((root/'source-properties.json').read_text());code=(root/'source-conductivity.rs').read_text();binding=json.loads((root/'binding.json').read_text());assert hashlib.sha256(code.encode()).hexdigest()==binding['source_conductivity_sha256']
forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};water={r['property']:D(str(r['quantity']['value'])) for r in data['water_properties']};keys=['H','O','Na','Cl','charge'];bound=D('1e-10');expected={k:D(0) for k in keys};checks=[];measurements=[];temperatures=[];volumes=[]
# Read the fixed coefficients from the actual historical source, not a later build.
table=code.split('pub const LIMITING_CONDUCTIVITY:')[1].split('];',1)[0];lambdas={k:float(v) for k,v in re.findall(r'\("([^"]+)", ([0-9.]+)\)',table)}
a=float(re.search(r'const FIT_SQRT: f64 = ([0-9.]+);',code).group(1));b=float(re.search(r'const FIT_LINEAR: f64 = (-?[0-9.]+);',code).group(1));assert 'at 25 °C' in code
function=code.split('pub fn specific_conductance(info: &SolutionInfo) -> Estimate {')[1].split('/// The 25 °C')[0]
assert 'temperature' not in function and 'let c_mol_l = *molality;' in function
all_events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','gas_evolved','gas_absorbed','gas_vented'] for e in all_events)
limits=[e for e in all_events if e['event']=='not_yet_modeled'];assert len(limits)==3 and all(e['reason']['key']=='not-modeled.volume-meter-on-an-open-vessel' for e in limits)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':
  for k,c in forms[op['species']].items():expected[k]+=D(str(op['moles']))*c
 if 'bench' not in r:continue
 assert len(r['bench']['vessels'])==1;v=r['bench']['vessels'][0];assert not v.get('unresolved_materials');actual={k:D(0) for k in keys}
 for p in v['contents']:
  n=D(str(p['moles']));assert n.is_finite() and n>=0
  for k,c in forms[p['species']].items():actual[k]+=n*c
 res={k:actual[k]-expected[k] for k in keys};assert all(abs(n)<bound for n in res.values()),(i,res)
 checks.append({'step':i,'residual_mol_eq':{k:str(n) for k,n in res.items()}})
 if i>0:
  assert v['solution'].get('scope','complete')=='complete'
  amounts={p['species']:D(str(p['moles'])) for p in v['contents']};assert amounts['Na+']==amounts['Cl-']==D('.005')
  assert abs(amounts['water']*water['molar_mass']-D('99.7'))<bound
 for scene in r.get('scene',{}).get('vessels',[]):
  liquid=scene.get('liquid')
  if liquid:volumes.append(D(str(liquid['volume_l'])))
 for e in r.get('events',[]):
  if e['event']!='measured':continue
  if e['instrument']=='thermometer':
   assert abs(e['value']-(v['temperature']-273.15))<1e-10;temperatures.append(e['value'])
  if e['instrument']=='conductivity_meter':
   assert e['unit']=='µS/cm';info=v['solution'];ionic=info['ionic_strength'];assert 0<ionic<.1
   ions=[p for p in info['species'] if '+' in p['name'] or '-' in p['name']];assert all(p['name'] in lambdas for p in ions)
   raw=sum(lambdas[p['name']]*p['molality']*1000 for p in ions);factor=1/(1+a*math.sqrt(ionic)+b*ionic);pred=raw*factor
   assert abs(pred-e['value'])<1e-8
   measurements.append({'temperature_c':v['temperature']-273.15,'observed_us_cm':e['value'],'reconstructed_us_cm':pred,'ionic_strength_mol_kg':ionic,'absolute_residual_us_cm':abs(pred-e['value'])})
assert len(temperatures)==len(measurements)==3 and volumes and all(x==D('.1') for x in volumes)
assert temperatures[1]>temperatures[0]+20 and abs(temperatures[2]-temperatures[0])<1e-8
assert abs(measurements[2]['observed_us_cm']-measurements[0]['observed_us_cm'])<1e-8
print(json.dumps({'met':2,'total':2,'criteria':{'MO-010-C1':True,'MO-010-C2':True},'solver_failures':0,'known_element_charge_checks':checks,'retained_each_salt_ion_mol':'.005','retained_water_g':'99.7','reported_liquid_volume_l':'.1','thermometer_c':temperatures,'conductivity_reconstruction':measurements,'temperature_convention':'Fixed 25 C limiting ionic mobilities and fixed concentration correction applied to temperature-dependent solved species/I. No mobility-temperature law or empirical automatic temperature compensation. Molality approximates molarity in the stated dilute domain.','interpretation':'Tiny signal variation reflects solved speciation/I, not validated in-situ ion mobility; real temperature dependence is a coverage limitation, not a negative chemical finding.','preserved_limit_keys':sorted({e['reason']['key'] for e in limits})},indent=2))
