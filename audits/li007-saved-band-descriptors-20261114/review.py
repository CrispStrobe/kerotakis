"""Read authenticated saved LI007 descriptor comparison; no product execution."""
import json,pathlib,hashlib,math,copy
R=pathlib.Path(__file__).resolve().parent
SOURCE='4bd93add656c936b0724673c7ea8332cf21e906f'
def load(n):return json.loads((R/n).read_text())
def sha(n):return hashlib.sha256((R/n).read_bytes()).hexdigest()
def assess(frames,contract):
 assert len(frames)==16 and all(f['output_sequence']==i for i,f in enumerate(frames))
 recipe=contract['recipe'];assert recipe['id']=='cosmetic/broad-spectrum-sunscreen' and recipe['version']==1
 role=next(r for r in recipe['roles'] if r['kind']=='uv_attenuation')
 assert role['spf']==30 and role['uva_protection_factor']==10 and role['film_mg_per_cm2']==2
 assert 'PENDING REVIEW' in role['source'] and recipe['confidence']=='surrogate'
 assert 'says nothing' in role['boundary']
 comp=contract['water_composition'];elements={r['element']:r['count']['value'] for r in comp['elements']};assert elements=={'H':2,'O':1} and comp['net_charge']['value']==0
 mw=contract['water_molar_mass']['quantity']['value'];assert mw==18.015
 budget=[0.0]*4;opaque=[0.0]*4;expected_mass=[0.0]*4;checks=[];mass_checks=[]
 for i,f in enumerate(frames):
  op=f['operator'];events=f['events'];assert not any(e['event'] in ['solver_failed','reacted','reaction_occurred','gas_transferred','evaporated'] for e in events)
  if op['op']=='add_material':
   v=op['vessel'];assert op['material']=='sunscreen' and op['total_amount']==1 and v in [0,1,2]
   e=next(e for e in events if e['event']=='material_added');assert e['recipe_id']==recipe['id'] and e['recipe_version']==1 and e['total_amount']==1 and e['basis']=='mass_fraction'
   assert len(e['components'])==1 and e['components'][0]['species']=='water' and e['components'][0]['basis_amount']==.6 and e['unresolved_amount']==.4
   budget[v]+=.6/mw;opaque[v]+=.4;expected_mass[v]+=1
  elif op['op']=='add':
   assert op['species']=='water' and op['vessel']==3 and abs(op['moles']*mw-1)<1e-14
   budget[3]+=op['moles'];expected_mass[3]+=1
  for v in f['bench']['vessels']:
   vi=v['id'];assert not v.get('material_objects',[]) and v.get('pending_co2_transfer_mol',0)==0
   assert all(p['species']=='water' and p['phase']=='liquid' and math.isfinite(p['moles']) and p['moles']>=0 for p in v['contents'])
   water=sum(p['moles'] for p in v['contents']);unresolved=v.get('unresolved_materials',[])
   assert all(p['recipe_id']==recipe['id'] and p['recipe_version']==1 and p['material']=='sunscreen' and p['basis']=='mass_fraction' for p in unresolved)
   retained=sum(p['amount'] for p in unresolved);assert abs(retained-opaque[vi])<=1e-10
   mass_delta=water*mw+retained-expected_mass[vi];assert abs(mass_delta)<=1e-10;mass_checks.append(dict(sequence=i,vessel=vi,residual_g=mass_delta,unresolved_g=retained))
   for element,n in [('H',2),('O',1),('Q',0)]:
    delta=(water-budget[vi])*n;assert abs(delta)<=1e-10;checks.append(dict(sequence=i,vessel=vi,element=element,residual=delta))
  if i>=7:assert f['bench']==frames[6]['bench'],'all irradiation/look/inspection leaves complete bench exact'
 irradiations=[f for f in frames if f['operator']['op']=='irradiate'];assert len(irradiations)==4
 descriptors=[]
 for f,vi,wavelength,factor,band in zip(irradiations,[0,1,2,3],[305,365,550,305],[30,10,None,None],['UV-B','UV-A',None,None]):
  op=f['operator'];assert op['vessel']==vi and op['wavelength_nm']==wavelength and op['irradiance_w_m2']==1
  light=next(e for e in f['events'] if e['event']=='irradiated');assert light['vessel']==vi and light['wavelength_nm']==wavelength and light['irradiance_w_m2']==1 and light['photolysis_coupled'] is False
  readings=[e for e in f['events'] if e['event']=='uv_attenuated']
  if factor:
   assert len(readings)==1;e=readings[0];assert e['vessel']==vi and e['wavelength_nm']==wavelength and e['band']==band and abs(e['transmitted_fraction']-1/factor)<1e-15 and e['material']==recipe['name'] and e['mechanism']==role['mechanism']
   descriptors.append(dict(wavelength_nm=wavelength,band=band,transmitted_fraction=e['transmitted_fraction'],scope='declared teaching descriptor; empirical provenance pending'))
  else:assert readings==[]
 assert frames[-1]['bench']['vessels'][3].get('unresolved_materials',[])==[]
 return dict(criteria={k:True for k in ['question','inventory','coverage','controls']},band_descriptors=descriptors,visible_transmitted_fraction=None,water_control_transmitted_fraction=None,absence_is_not_full_transmission=True,known_element_charge_checks=len(checks),max_abs_known_residual=max(abs(c['residual']) for c in checks),mass_ownership_checks=len(mass_checks),max_abs_mass_residual_g=max(abs(c['residual_g']) for c in mass_checks),per_sample_unresolved_g=.4,whole_bench_unchanged_across_irradiation_and_observation=True,ledger=checks,mass_ledger=mass_checks)
def main():
 hashes=load('selected-producer-hashes.json')
 for n,h in hashes.items():
  if (R/n).exists():assert sha(n)==h
 binding=load('source-and-lock.json');assert binding['source']==SOURCE and binding['tree']=='3e1471c481b9ef6f91a962039e3158ab0df2763d' and binding['cargo_lock_sha256']==hashes['actual-producer-Cargo.lock']
 assert load('release-cli.json')['exit']==0 and load('STARTED.json')['source']==SOURCE
 original=load('original.json');assert sha('batch3-diverse-li-007.lab')==original['script_sha256'] and (R/'batch3-diverse-li-007.lab').read_text()==original['script']
 frames=[json.loads(l) for l in (R/'batch3-diverse-li-007.stdout').read_text().splitlines() if l.startswith('{')];contract=load('pinned-source-contract.json');assert contract['source']==SOURCE
 source=(R/'pinned-uv.rs').read_text();assert '(280.0..320.0)' in source and '(320.0..400.0)' in source and 'return Vec::new()' in source
 result=assess(frames,contract);mutations=[]
 edits=[('wrong_UVB',lambda f,c:f[7]['events'][1].__setitem__('transmitted_fraction',.1)),('wrong_UVA',lambda f,c:f[8]['events'][1].__setitem__('transmitted_fraction',1/30)),('invent_visible_reading',lambda f,c:f[9]['events'].append(copy.deepcopy(f[7]['events'][1]))),('invent_blank_reading',lambda f,c:f[10]['events'].append(copy.deepcopy(f[7]['events'][1]))),('lost_unresolved',lambda f,c:f[15]['bench']['vessels'][0]['unresolved_materials'].clear()),('lost_water',lambda f,c:f[15]['bench']['vessels'][0]['contents'][0].__setitem__('moles',0)),('wrong_film',lambda f,c:c['recipe']['roles'][0].__setitem__('film_mg_per_cm2',1)),('hide_pending_provenance',lambda f,c:c['recipe']['roles'][0].__setitem__('source','validated')),('invent_photolysis',lambda f,c:f[7]['events'][0].__setitem__('photolysis_coupled',True)),('solver_failure',lambda f,c:f[7]['events'].append(dict(event='solver_failed')))]
 for name,edit in edits:
  f=copy.deepcopy(frames);c=copy.deepcopy(contract);edit(f,c)
  try:assess(f,c)
  except (AssertionError,StopIteration):mutations.append(dict(case=name,rejected=True))
  else:raise AssertionError(name+' survived')
 result.update(source=SOURCE,tree=binding['tree'],historical_native_binary_sha256=None,cargo_lock_sha256=binding['cargo_lock_sha256'],output_sha256=sha('batch3-diverse-li-007.stdout'),reader_mutations=mutations,chemical_replay=False,empirical_optical_calibration=False,unresolved_elemental_composition=None)
 with (R/'review-result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['ledger','mass_ledger','reader_mutations']},indent=2))
if __name__=='__main__':main()
