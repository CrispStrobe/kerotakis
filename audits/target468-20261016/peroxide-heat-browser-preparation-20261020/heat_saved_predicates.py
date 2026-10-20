"""Independent saved-state budgets. Synthetic tests are not browser observations."""
import math,re
SOURCE='18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a'
PROTOCOL='register lv3\nadd v1 water 10mol\nadd v1 H2O2 0.1mol\nadd v1 KI 0.001mol\n'
CONTINUATION='wait 1s\n'
FORMULAE={'water':({'H':2,'O':1},0),'H2O2':({'H':2,'O':2},0),'O2':({'O':2},0),'KI':({'K':1,'I':1},0),'K+':({'K':1},1),'I-':({'I':1},-1),'H+':({'H':1},1),'OH-':({'H':1,'O':1},-1)}
def number(x):
 assert type(x)in(int,float)and math.isfinite(x),'finite typed number required'
 return x
def near(a,b,tol=1e-9):assert abs(a-b)<=tol,(a,b)
def pools(v):
 assert type(v['id'])is int and v['id']==0 and type(v['contents'])is list
 result={}
 for p in v['contents']:
  assert p['species']in FORMULAE and p['phase']in ['solid','liquid','aqueous','gas']
  n=number(p['moles']);assert n>=0;result[p['species']]=result.get(p['species'],0)+n
 for field in ['temperature','pressure','elapsed_seconds','free_proton','free_hydroxide','excess_enthalpy_j','solute_charge','pending_co2_transfer_mol']:number(v[field])
 assert v['temperature']>0 and v['pressure']>0 and v['elapsed_seconds']>=0 and v['free_proton']>=0 and v['free_hydroxide']>=0
 assert v['pending_co2_transfer_mol']==0,'unspent external CO2 amount'
 if v['co2_partial_pressure_atm']is not None:assert number(v['co2_partial_pressure_atm'])>=0
 assert v['thermal_mode']=='adiabatic','protocol thermal mode changed'
 # NuclideLedger is a serialized struct even when its BTreeMap is empty.
 assert type(v['nuclides'])is dict and v['nuclides']=={'inventory':{}},'unexpected nuclide inventory or schema'
 for field in ['surfaces','unresolved_materials','material_objects','surface_colours','charred_materials','electrodes','exchanges','adsorbed','solid_solutions']:
  assert v.get(field,[])==[] and type(v.get(field,[]))is list,('unexpected owned reservoir',field)
 for field in ['soap_scum','lemon_paper_mark','surface_particles','emulsion','step_start']:
  assert v.get(field)is None,('unexpected owned reservoir',field)
 assert type(v['foam'])is dict and set(v['foam'])=={'trapped_gas_liters','volume_liters','peak_volume_liters'}
 for field,n in v['foam'].items():assert number(n)==0,('unexpected foam pool',field)
 # Derived solution/readback caches and provenance lots are not second material pools.
 # Still reject nonfinite numeric values anywhere in their actual serialized trees.
 def finite_tree(value):
  if type(value)in(int,float):number(value)
  elif type(value)is dict:
   for child in value.values():finite_tree(child)
  elif type(value)is list:
   for child in value:finite_tree(child)
 for field in ['solution','resolved','lots']:finite_tree(v.get(field))
 return result
def ledger(pool):
 out={k:0. for k in ['H','O','K','I','charge']}
 for species,n in pool.items():
  formula,z=FORMULAE[species];out['charge']+=z*n
  for atom,count in formula.items():out[atom]+=count*n
 return out
def validate_case(raw,locale):
 assert type(raw['meta']['git_rev'])is str and len(raw['meta']['git_rev'])>=7 and SOURCE.startswith(raw['meta']['git_rev'])
 assert type(raw['before_calls'])is int and type(raw['after_calls'])is int and raw['after_calls']>=raw['before_calls']>=0
 a,b=raw['before'],raw['after'];assert len(a['vessels'])==len(b['vessels'])==1
 assert a['spills']==b['spills'] and a['broken_vessels']==b['broken_vessels'],'unaccounted loss boundary'
 av,bv=a['vessels'][0],b['vessels'][0];assert av['headspace']==bv['headspace']=={'boundary':'open'}
 pa,pb=pools(av),pools(bv);near(pa.get('water',0),10);near(pa.get('H2O2',0),.1);near(pa.get('KI',0),.001);near(pa.get('O2',0),0)
 near(av['elapsed_seconds'],0);near(bv['elapsed_seconds']-av['elapsed_seconds'],1)
 assert raw['preparation']['bench']['vessels']==a['vessels'] and raw['result']['bench']['vessels']==b['vessels'],'owned snapshots disagree'
 steps=raw['result']['steps'];assert len(steps)==1 and steps[0]['operator']=={'op':'wait','seconds':1}
 events=steps[0]['events'];heats=[];gas=0;extent=0;export={};rendered=steps[0]['rendered'];assert type(rendered)is list and all(type(x)is str for x in rendered)
 assert not any('peroxide-decomposition'in line or ' released 'in line for line in rendered),'English kinetic leakage'
 for e in events:
  if 'vessel'in e:assert type(e['vessel'])is int and e['vessel']==0
  kind=e['event']
  if kind=='reaction_heat_released':
   assert e['reaction']=='peroxide-decomposition';q=number(e['energy_j']);assert q>0;heats.append(q)
  elif kind=='gas_produced':
   assert e['reaction']=='peroxide-decomposition'and e['species']=='O2';n=number(e['moles']);assert n>0;near(number(e['rate_moles_per_second']),n);gas+=n
  elif kind=='reacted':
   assert e['reaction']=='peroxide-decomposition';n=number(e['moles']);assert n>0;near(number(e['seconds']),1);extent+=n
  elif kind=='gas_evolved':
   assert e['species']in ['O2','water'];n=number(e['moles']);assert n>0;export[e['species']]=export.get(e['species'],0)+n
  elif kind=='evaporated':
   n=number(e['moles']);assert n>0;export['water']=export.get('water',0)+n
  else:
   assert kind not in ['gas_absorbed','gas_contained','org_reacted','distilled','fermented','enzyme_hydrolysed','transferred','drained','spilled','added'],'unowned source/sink'
 consumed=pa.get('H2O2',0)-pb.get('H2O2',0);assert 1e-8<consumed<pa['H2O2'];near(gas,consumed/2);near(extent,consumed/2)
 near(pb.get('O2',0)-pa.get('O2',0)+export.get('O2',0),consumed/2);near(pb.get('water',0)-pa.get('water',0)+export.get('water',0),consumed);near(pb.get('KI',0),pa['KI'])
 final=dict(pb)
 for key,n in export.items():final[key]=final.get(key,0)+n
 la,lb=ledger(pa),ledger(final)
 for atom in la:near(la[atom],lb[atom])
 assert heats;heat=sum(heats);ratio=heat/consumed;assert abs(ratio-98480)<=790,('literature heat basis',ratio)
 label,verb=('Peroxidzersetzung','setzte')if locale=='de'else('Décomposition du peroxyde','a libéré')
 heatlines=[line for line in rendered if label in line and verb in line and ' J' in line];assert len(heatlines)==len(heats)
 observed=[]
 for line in heatlines:
  assert 'peroxide-decomposition'not in line and ' released 'not in line
  if locale=='de':assert 'frei'in line
  match=re.search(r'([0-9]+,[0-9]{2}) J',line);assert match,'localized two-decimal joules required';observed.append(float(match[1].replace(',','.')))
 for value,q in zip(observed,heats):assert abs(value-q)<=.005000001
 return {'consumed_H2O2_mol':consumed,'O2_produced_mol':gas,'exported_mol':export,'heat_j':heat,'joules_per_consumed_mol':ratio,'reference_band_J_per_mol':[97690,99270],'atoms_charge_before':la,'atoms_charge_after_plus_exports':lb,'elapsed_seconds':1,'localized_heat_lines':heatlines,'temperature_before_K':av['temperature'],'temperature_after_K':bv['temperature']}
def validate_report(r,binding):
 assert binding['expected_engine_source']==SOURCE and r['binding']==binding and r['protocol']==PROTOCOL and r['continuation']==CONTINUATION
 assert r['no_owner_credentials_supplied']is True and r['scientific_selections']==0 and r['browser']['web_security_disabled']is False
 assert r['client']['status']==200 and r['client']['sha256']==binding['client_sha256']
 required={'iphreeqc.mjs','iphreeqc.wasm','kerotakis.mjs','kerotakis_wasm.js','kerotakis_wasm_bg.wasm'};assert set(binding['frontend_assets'])==set(r['assets'])==required
 for name,digest in binding['frontend_assets'].items():assert r['assets'][name]['status']==200 and r['assets'][name]['sha256']==digest and number(r['assets'][name]['bytes'])>0
 assert set(r['raw'])=={'de','fr'}
 return {locale:validate_case(r['raw'][locale],locale)for locale in ['de','fr']}
