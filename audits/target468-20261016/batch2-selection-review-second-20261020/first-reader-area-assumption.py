#!/usr/bin/env python3
"""Second independent saved batch2 assessment. No Kero executable is invoked."""
import hashlib,importlib.util,json,math,subprocess,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
SPEC=importlib.util.spec_from_file_location('saved_helpers',HERE.parent/'batch2-selection-review-20261020/review_saved.py');H=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(H)
ROOT,BASE=H.ROOT,H.BASE
H.A.update({'NH3':{'N':1,'H':3},'NH4+':{'N':1,'H':4},'CaCO3':{'Ca':1,'C':1,'O':3},'CaO':{'Ca':1,'O':1},'Ca+2':{'Ca':1},'NaHCO3':{'Na':1,'H':1,'C':1,'O':3},'Na2CO3':{'Na':2,'C':1,'O':3},'H2O2':{'H':2,'O':2},'MnO2':{'Mn':1,'O':2}});H.Q.update({'NH4+':1,'Ca+2':2});H.KEYS[:]=sorted({k for d in H.A.values() for k in d})+['charge']
IDS=('tc-002','tc-007','ab-025','ab-030','ab-035','ab-036','pq-008','ki-003','ki-004','ki-007','qa-004','gs-013')
sha=H.sha;amount=H.amounts

def carbon(v):return sum(p['moles']*H.A[p['species']].get('C',0) for p in v['contents'])
def snapshot(row):return {v['id']:v for v in row['bench']['vessels']}
def last(rows):
 d={}
 for r in rows:d.update(snapshot(r))
 return d

def catalyst_area(v,registry):
 row=registry['MnO2'];out=[]
 for lot in v.get('lots',[]):
  if lot['species']!='MnO2' or lot['phase']!='solid':continue
  diameter=lot.get('particle_size_um') or 100.;area=6*(lot['moles']*row['molar_mass']/row['density']*1e-6)/(diameter*1e-6);suspended=lot.get('suspended_fraction');contact=.35 if suspended is None else max(.35,min(1,suspended));out.append({'diameter_um':diameter,'geometric_sphere_area_m2':area,'suspended_fraction':suspended,'declared_model_contact_fraction':contact,'accessible_area_m2':area*contact})
 return out

def review():
 ar=BASE/'raw-artifact.zip';digest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert digest==json.loads((BASE/'artifact-metadata.json').read_text())['digest'].removeprefix('sha256:');binding=json.loads((BASE/'binding.json').read_text());source=binding['engine']['source_head'];assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip()==binding['engine']['source_tree']
 reg_raw=subprocess.check_output(['git','show',source+':crates/kerotakis-core/tests/golden/registry.json'],cwd=ROOT);reg={r['key']:r for r in json.loads(reg_raw)};assert reg['water']['molar_mass']==18.015 and reg['water']['density']==.997
 intake={r['original_id']:r for r in json.loads((BASE.parent/'intake.json').read_text())};selected={r['original_target_id'] for r in json.loads((ROOT/'audits/target468-20261016/hydrolysis-beneficiaries-outcome-20261019/catalog-membership.json').read_text())['mapping']};out=[]
 with zipfile.ZipFile(ar) as z:
  for suffix in IDS:
   key='batch2_'+suffix;case=json.loads((BASE/'cases'/f'{key}.json').read_text());original=intake[case['id']];assert case['id'] not in selected;assert case['engine']==binding['engine'];assert case['original_acceptance_checks']==original['original_acceptance_checks'] and case['original_physical_forecast']==original['original_expected_result']
   script=z.read(key+'/experiment.lab');raw=z.read(key+'/stdout.jsonl');stderr=z.read(key+'/stderr.txt');assert script==(BASE.parent/original['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(stderr)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0
   rows=[json.loads(x) for x in raw.splitlines() if x.strip()];expected={};prior={};maxima=H.zero();exports=[];seals=[];titrants=[];ledger=[];clocks=[]
   for ix,row in enumerate(rows):
    op=row['operator'];typ=op['op'];assert typ in ('add','new_vessel','seal','inspect','cool','measure','decant','dilute','grind','wait','titrate'),typ
    if typ=='add':H.book(expected.setdefault(op['vessel'],H.zero()),op['species'],op['moles'])
    elif typ=='dilute':H.book(expected[op['vessel']],'water',op['volume']*.997*1000/18.015)
    elif typ=='seal':
     n=101325*op['headspace_volume']/(8314.462618*prior[op['vessel']]['temperature']);target=expected[op['vessel']]
     for sp,f in [('N2',.7901),('O2',.2095),('CO2',.0004)]:H.book(target,sp,n*f)
     seals.append({'row':ix,'vessel':op['vessel'],'air_mol':n,'air_C_mol':n*.0004})
    elif typ=='decant':
     frm,to=op['from'],op['to'];expected.setdefault(to,H.zero())
     for p in prior[frm]['contents']:
      if p['phase'] in ('liquid','aqueous'):
       n=op['fraction']*p['moles'];H.book(expected[frm],p['species'],-n);H.book(expected[to],p['species'],n)
    for e in row['events']:
     assert e['event'] not in ('solver_failed','not_yet_modeled','gas_absorbed')
     if e['event']=='gas_evolved':H.book(expected[e['vessel']],e['species'],-e['moles']);exports.append({'row':ix,**e})
     if e['event']=='titrated':
      assert e['endpoint_reached'];n=e['total_volume']*e['concentration'];H.book(expected[e['vessel']],op['titrant'],n);H.book(expected[e['vessel']],'water',e['total_volume']*.997*1000/18.015);titrants.append({'row':ix,'vessel':e['vessel'],'solute_mol':n,'carrier_water_mol':e['total_volume']*.997*1000/18.015,'endpoint_pH':e['final_ph']})
    current=snapshot(row)
    if set(prior)-set(current):assert typ=='inspect' and not row['events'];assert all(v==prior[k] for k,v in current.items())
    if typ=='wait':
     for vid,v in current.items():
      before=prior[vid];assert abs(v['elapsed_seconds']-before['elapsed_seconds']-op['seconds'])<1e-12;clocks.append({'row':ix,'vessel':vid,'from_seconds':before['elapsed_seconds'],'to_seconds':v['elapsed_seconds']})
    prior.update(current)
    for vid,v in prior.items():
     assert not v.get('unresolved_materials') and not v.get('surfaces');actual=H.vectors(v);exp=expected.setdefault(vid,H.zero());res={k:actual[k]-exp[k] for k in H.KEYS}
     for k,d in res.items():maxima[k]=max(maxima[k],abs(d));assert abs(d)<=H.TOL,(suffix,ix,vid,k,d)
     ledger.append({'row':ix,'vessel':vid,'maximum_absolute_residual_mol':max(map(abs,res.values()))})
   ev=[e for r in rows for e in r['events']];met=[True,True];notes=[];observed={};gap=None
   if suffix=='tc-002':
    vals=[]
    for i,r in enumerate(rows):
     if r['operator']['op']!='decant':continue
     op=r['operator'];before=snapshot(rows[i-1]);v0,v1=before[op['to']],before[op['from']];after=snapshot(r)[op['to']];q=sum(amount(v,'water')*H.sensible(reg['water'],'liquid',v['temperature'],after['temperature']) for v in (v0,v1));vals.append({'vessel':op['to'],'mixing_corrected_sensible_energy_J':q,'final_acid_basis_mol':amount(after,'H+'),'final_hydroxide_basis_mol':amount(after,'base_equivalents')})
    assert all(abs(v['final_hydroxide_basis_mol'])<1e-12 for v in vals);assert abs(vals[0]['final_acid_basis_mol']-.0005)<1e-8 and abs(vals[1]['final_acid_basis_mol']-.0015)<1e-8;met=[False,True];observed={'mixes':vals};gap='transfer-neutralisation-enthalpy';notes=['Same missing mixing-neutralisation energy as originalTC009. Correct limiting-base material budget does not prove its thermal forecast.']
   elif suffix=='tc-007':
    vals=[]
    for vid,v in prior.items():
     solid=amount(v,'water','solid');liquid=amount(v,'water','liquid');energy=(5000,20000)[vid];assert solid>0 and liquid>0 and abs(v['temperature']-273.15)<1e-9;vals.append({'vessel':vid,'ice_mol':solid,'liquid_mol':liquid,'temperature_K':v['temperature'],'energy_removed_J':energy,'inferred_fusion_J_mol':energy/solid})
    assert abs(vals[1]['ice_mol']/vals[0]['ice_mol']-4)<1e-10;assert all(abs(x['inferred_fusion_J_mol']-6010)<1e-5 for x in vals);observed={'coexisting_phases':vals};notes=['The energy convention is phase-latent retained energy at the shared melting temperature, not solely thermometer or sensible heat.']
   elif suffix=='ab-025':
    ph={v:[e['value'] for e in ev if e['event']=='measured' and e['instrument']=='ph_meter' and e['vessel']==v] for v in (0,1)};assert all(len(p)==3 for p in ph.values());spans={v:ph[v][-1]-ph[v][0] for v in ph};assert 0<spans[0]<spans[1];v=prior[0];acetate=amount(v,'CH3COOH')+amount(v,'CH3COO-');nitrogen=amount(v,'NH3')+amount(v,'NH4+');assert abs(acetate-.001)<1e-10 and abs(nitrogen-.0012)<1e-10;observed={'actual_pH':ph,'pH_span_80_to_120_percent':spans,'acetate_mol':acetate,'reduced_N_mol':nitrogen,'actual_temperatures_K':{vid:v['temperature'] for vid,v in prior.items()}};notes=['Different actual thermal histories retained; bounded broad-endpoint comparison, not calibrated pKa or equal-temperature titration curves.']
   elif suffix=='ab-030':
    states=[snapshot(r)[0] for r in rows if r['operator']['op']=='measure'];solids=[amount(v,'CaCO3','solid') for v in states];assert len(solids)==4 and all(a>b for a,b in zip(solids,solids[1:]));assert solids[-1]==0 and amount(states[-1],'H+')>.00049;assert abs(carbon(states[-1])+sum(e['moles'] for e in exports)-.001)<1e-8;observed={'carbonate_solid_mol_after_acid_pulses':solids,'actual_pH':[v['solution']['ph'] for v in states],'final_acid_basis_mol':amount(states[-1],'H+'),'retained_C_mol':carbon(states[-1]),'exported_CO2_mol':sum(e['moles'] for e in exports)};notes=['Open-boundary carbon exports are counted explicitly; carbonate does not buffer indefinitely.']
   elif suffix in ('ab-035','gs-013'):
    vals=[{'vessel':vid,'final_C_mol':carbon(v),'CO2_gas_mol':amount(v,'CO2','gas'),'aqueous_CO2_mol':amount(v,'CO2(aq)'),'carbonate_mol':amount(v,'CO3-2'),'pH':v['solution']['ph']} for vid,v in prior.items()];met=[True,False];gap='original-open-before-seal-carbon-budget';observed={'final_arms':vals,'original_reagent_C_mol':.001,'trapped_air':seals,'prior_exports':exports}
    assert any(e['moles']>1e-5 for e in exports);notes=['Direction of acid/base or earlygas response holds, but bicarbonate loses substantial carbon before seal. Original1mmol-per-closed-vessel forecast fails; cannot replace it with a post-loss inventory criterion.']
    if suffix=='ab-035':assert vals[0]['CO2_gas_mol']>vals[1]['CO2_gas_mol'] and vals[1]['carbonate_mol']>vals[0]['carbonate_mol']
    else:
     middle=next(snapshot(r) for r in rows if r['operator']['op']=='inspect');assert amount(middle[1],'CO2','gas')>amount(middle[0],'CO2','gas')
   elif suffix=='ab-036':
    vals=[]
    for vid,v in prior.items():
     air=next(s['air_C_mol'] for s in seals if s['vessel']==vid);ca=sum(p['moles']*H.A[p['species']].get('Ca',0) for p in v['contents']);assert abs(ca-.0005)<1e-10;assert abs(amount(v,'H+')-.0002)<1e-7;assert abs(carbon(v)-air-(.0005 if vid==1 else 0))<1e-8;vals.append({'vessel':vid,'total_Ca_mol':ca,'acid_basis_mol':amount(v,'H+'),'total_C_mol':carbon(v),'trapped_air_C_mol':air,'reagent_derived_C_mol':carbon(v)-air,'CO2_gas_mol':amount(v,'CO2','gas')})
    assert vals[1]['CO2_gas_mol']>100*vals[0]['CO2_gas_mol'];observed={'arms':vals};notes=['Oxide arm contains small legitimate trapped-air carbon. Subtract that explicit carbon before comparing reagent-derived gas; no invented carbonate source.']
   elif suffix=='pq-008':
    states=[snapshot(r)[0] for r in rows if r['operator']['op']=='measure'];vals=[]
    for v in states:
     conductivity=next(e['value'] for r in rows if snapshot(r).get(0)==v for e in r['events'] if e['event']=='measured');aq=amount(v,'Na+');kg=amount(v,'water')*.018015;vals.append({'solid_NaCl_mol':amount(v,'NaCl','solid'),'aqueous_Na_mol':aq,'molality_mol_kgw':aq/kg,'conductivity_uS_cm':conductivity,'temperature_K':v['temperature'],'Na_activity':next(s['activity'] for s in v['solution']['species'] if s['name']=='Na+'),'Cl_activity':next(s['activity'] for s in v['solution']['species'] if s['name']=='Cl-')})
    assert len(vals)==4 and vals[0]['solid_NaCl_mol']>vals[1]['solid_NaCl_mol']>vals[2]['solid_NaCl_mol']>0 and vals[3]['solid_NaCl_mol']==0;assert max(v['molality_mol_kgw'] for v in vals[:3])/min(v['molality_mol_kgw'] for v in vals[:3])<1.01 and vals[3]['molality_mol_kgw']<vals[2]['molality_mol_kgw'];observed={'dilution_series':vals};notes=['Saturated Pitzer activities and actualtemperature retained; conductivity is separately reported, not equated to molarity.']
   elif suffix in ('ki-003','ki-004','ki-007'):
    waits=[(i,r) for i,r in enumerate(rows) if r['operator']['op']=='wait'];rates=[]
    for i,r in waits:
     for vid,v in snapshot(r).items():
      before=snapshot(rows[i-1])[vid];initial=amount(before,'H2O2');final=amount(v,'H2O2');k=-math.log(final/initial)/r['operator']['seconds'];loss=initial-final;assert loss>0;assert abs((amount(v,'water')-amount(before,'water'))-loss)<1e-10;assert abs((amount(v,'O2')-amount(before,'O2'))-loss/2)<1e-10;assert abs(amount(v,'MnO2')-amount(before,'MnO2'))<1e-15;rates.append({'row':i,'vessel':vid,'actual_start_T_K':before['temperature'],'actual_end_T_K':v['temperature'],'initial_H2O2_mol':initial,'remaining_H2O2_mol':final,'fraction_lost':loss/initial,'apparent_first_order_k_s-1':k,'MnO2_mol':amount(v,'MnO2'),'solid_lots':[l for l in v.get('lots',[]) if l['species']=='MnO2'],'initial_area':catalyst_area(before,reg),'final_area':catalyst_area(v,reg)})
    observed={'rates':rates}
    if suffix=='ki-003':assert abs(sum(a['geometric_sphere_area_m2'] for a in rates[0]['initial_area'])/sum(a['geometric_sphere_area_m2'] for a in rates[1]['initial_area'])-1)<1e-12;met=[False,True];gap='protocol-confound-surface-availability-versus-nominal-area';notes=['Nominalmass/diameter equality does not survive different gravity-settling histories; observed matched-time fractional losses differ substantially. Preserve original rather than ignore availability.']
    elif suffix=='ki-004':
     grind=[(i,r) for i,r in enumerate(rows) if r['operator']['op']=='grind'];i,r=grind[-1];before=snapshot(rows[i-1])[0];after=snapshot(r)[0];assert before['contents']==after['contents'] and before['elapsed_seconds']==after['elapsed_seconds'];assert rates[1]['apparent_first_order_k_s-1']>5*rates[0]['apparent_first_order_k_s-1'];observed['grind_inventory_unchanged']=True;assert abs(sum(a['geometric_sphere_area_m2'] for a in rates[1]['initial_area'])/sum(a['geometric_sphere_area_m2'] for a in rates[0]['initial_area'])-10)<1e-10;notes=['Actualremaining substrate and fullMnO2mass retained; grinding changes subsequent availability, never retrospectively changes product.']
    else:
     assert rates[1]['actual_start_T_K']<rates[0]['actual_start_T_K']-9 and rates[1]['apparent_first_order_k_s-1']<rates[0]['apparent_first_order_k_s-1'];assert abs(rates[0]['MnO2_mol']-rates[1]['MnO2_mol'])<1e-15;assert all(a['declared_model_contact_fraction']==.35 for rate in rates for a in rate['initial_area']+rate['final_area']);observed['source_Ea_only_expected_cool_over_warm_direction']=math.exp(-58000/8.31446261815324*(1/rates[1]['actual_start_T_K']-1/rates[0]['actual_start_T_K']));notes=['Apparentcoefficient calculated from separateactualinventoryintervals, not eventthresholds or absoluteconsumption. Cooling and gravity availability histories retained; no calibratedrateclaim.']
   elif suffix=='qa-004':
    assert len(titrants)==2;p=titrants[0]['solute_mol'];t=p+titrants[1]['solute_mol'];observed={'first_stage_acid_mol':p,'additional_second_stage_acid_mol':titrants[1]['solute_mol'],'cumulative_acid_mol':t,'inferred_carbonate_mol':p,'inferred_bicarbonate_mol':t-2*p};assert t-2*p<0;met=[True,False];gap='open-carbonate-two-endpoint-protocol';notes=['Separateactualdoses andcumulativeT correctlyrecorded; originalnonnegative0.3/0.4mmolreconstructionfails. Do not hide openCO2 exports or replace original0.3mmolPforecast.']
   out.append({'id':case['id'],'proposed_candidate':all(met),'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],met,strict=True)],'observed':observed,'max_per_vessel_per_step_atom_charge_residual_mol':maxima,'per_step_ledger':ledger,'clock_checks':clocks,'boundary_exports':exports,'titrant_carrier_checks':titrants,'named_model_limits':case['named_model_limits'],'limitations':notes,'shared_gap':gap,'source':case['engine'],'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(stderr),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt']})
 return {'schema':'kerotakis.batch2.second-saved-review.v1','cases':out,'raw_artifact_sha256':digest,'source_registry_sha256':sha(reg_raw),'saved_helper_sha256':sha((HERE.parent/'batch2-selection-review-20261020/review_saved.py').read_bytes()),'source_blobs_sha256':{path:sha(subprocess.check_output(['git','show',source+':'+path],cwd=ROOT)) for path in ['crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/kinetics.rs','crates/kerotakis-core/src/states.rs','crates/kerotakis-core/src/heat_capacity.rs','crates/kerotakis-core/tests/golden/registry.json']},'new_engine_executions':0,'selection_performed':False,'tolerance_mol':H.TOL,'lock_qualification':binding['lock_qualification']}
if __name__=='__main__':print(json.dumps(review(),indent=2,allow_nan=False))
