#!/usr/bin/env python3
"""Fifth twelve-original saved-only review. Original protocols and criteria stay immutable."""
import hashlib,importlib.util,json,math,re,subprocess,zipfile
from decimal import Decimal,localcontext
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];BASE=ROOT/'audits/new-experiments-201-20261016/native-screen-results';D=Decimal;Z=D(0);TOL=D('1e-8');sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes(),parse_float=D)
HELPER=HERE.parent/'batch2-selection-review-fourth-20261020/review_saved.py';spec=importlib.util.spec_from_file_location('fourth',HELPER);H=importlib.util.module_from_spec(spec);spec.loader.exec_module(H)
IDS=('ec-005','ec-006','ab-028','pq-009','pq-011','pq-012','mo-002','mo-005','mo-010','qa-003','tc-003','tc-008')
def amount(v,s,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase)),Z)
def snap(r):return {v['id']:v for v in r['bench']['vessels']}
def review():
 ar=BASE/'raw-artifact.zip';ardigest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert ardigest==load(BASE/'artifact-metadata.json')['digest'].removeprefix('sha256:');binding=load(BASE/'binding.json');source=binding['engine']['source_head'];show=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=ROOT);assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip()==binding['engine']['source_tree'];regraw=show('data/registry/registry-source-v1.json');reg=json.loads(regraw,parse_float=D);comps={r['species_id']:r for r in reg['compositions']};goldraw=show('crates/kerotakis-core/tests/golden/registry.json');gold={r['key']:r for r in json.loads(goldraw)};vesselraw=show('crates/kerotakis-core/src/vessel.rs');vesselsource=vesselraw.decode();conductivity_source=show('crates/kerotakis-core/src/conductivity.rs').decode()
 optional_vectors=['unresolved_materials','material_objects','electrodes','exchanges','adsorbed','solid_solutions','surface_colours','charred_materials'];optional_objects=['soap_scum','lemon_paper_mark','surface_particles','emulsion']
 for f in optional_vectors:assert re.search(r'#\[serde\(default, skip_serializing_if = "Vec::is_empty"\)\]\s+pub '+f+r':',vesselsource)
 for f in optional_objects:assert re.search(r'#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]\s+pub '+f+r':',vesselsource)
 intake={r['original_id']:r for r in load(BASE.parent/'intake.json')};selected={r['original_target_id'] for r in load(ROOT/'audits/target468-20261016/selection-expansion-fourth-20261020/catalog-membership.json')['mapping']}
 def composition(s):
  if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
  r=comps[s];return {**{x['element']:x['count']['value'] for x in r['elements']},'charge':r['net_charge']['value']}
 def book(out,s,n):
  for k,v in composition(s).items():out[k]=out.get(k,Z)+v*n
 def atoms(v,phase=None):
  out={}
  for p in v['contents']:
   assert p['moles'].is_finite() and p['moles']>=0
   if phase is None or p['phase']==phase:book(out,p['species'],p['moles'])
  return out
 def elem(v,e,phase=None):return atoms(v,phase).get(e,Z)
 def water_energy(v):
  # Liquid273.15K reference; use phase-specific curves and a single referenced fusion step.
  return sum(float(p['moles'])*(H.sensible(gold['water'],p['phase'],273.15,float(v['temperature']))-(6010 if p['phase']=='solid' else 0)) for p in v['contents'] if p['species']=='water')
 output=[]
 with localcontext() as ctx,zipfile.ZipFile(ar) as zip:
  ctx.prec=45
  for suffix in IDS:
   key='batch2_'+suffix;case=load(BASE/'cases'/f'{key}.json');original=intake[case['id']];assert case['id'] not in selected;assert case['engine']==binding['engine'];assert case['original_acceptance_checks']==original['original_acceptance_checks'] and case['original_physical_forecast']==original['original_expected_result'];script=zip.read(key+'/experiment.lab');raw=zip.read(key+'/stdout.jsonl');err=zip.read(key+'/stderr.txt');assert script==(BASE.parent/original['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(err)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0;rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x.strip()];expected={};state={};frames=[];ledger=[];maxima={};exports=[];air=[];titrants=[];reservoir_count=0
   for i,row in enumerate(rows):
    assert set(row['bench'])=={'vessels'};before=state.copy();op=row['operator'];typ=op['op'];ev=row['events'];published=snap(row)
    if typ=='add':book(expected.setdefault(op['vessel'],{}),op['species'],op['moles'])
    elif typ=='seal':
     v=before[op['vessel']];n=Z if v['headspace']['boundary']!='open' else D(101325)*op['headspace_volume']/(D('8314.462618')*v['temperature']);air.append({'row':i,'vessel':op['vessel'],'N2_mol':n*D('.7901'),'O2_mol':n*D('.2095'),'CO2_mol':n*D('.0004')})
     for s,f in [('N2',D('.7901')),('O2',D('.2095')),('CO2',D('.0004'))]:book(expected[op['vessel']],s,n*f)
    elif typ in ('decant','filter'):
     frm,to=op['from'],op['to'];expected.setdefault(to,{})
     for p in before[frm]['contents']:
      if p['phase'] in ('liquid','aqueous'):
       n=p['moles']*op.get('fraction',D(1));book(expected[frm],p['species'],-n);book(expected[to],p['species'],n)
    elif typ=='titrate':
     e,=[e for e in ev if e['event']=='titrated'];assert e['endpoint_reached'];dose=e['total_volume']*e['concentration'];book(expected[op['vessel']],op['titrant'],dose);book(expected[op['vessel']],'water',e['total_volume']*D(997)/D('18.015'));titrants.append({'row':i,'vessel':op['vessel'],'solute_mol':dose,'carrier_water_mol':e['total_volume']*D(997)/D('18.015'),'volume_L':e['total_volume'],'endpoint_pH':e['final_ph'],'step_mol':op['step']*op['concentration']})
    else:assert typ in ('new_vessel','measure','inspect','cell','react','heat','cool'),typ
    for e in ev:
     assert e['event']!='solver_failed'
     if e['event'] in ('gas_evolved','gas_absorbed'):book(expected[e['vessel']],e['species'],e['moles']*(D(-1) if e['event']=='gas_evolved' else D(1)));exports.append({'row':i,**e})
    if set(before)-set(published):assert typ in ('measure','inspect');assert all(v==before[k] for k,v in published.items())
    if typ in ('measure','inspect','cell'):assert all(v['elapsed_seconds']==before[k]['elapsed_seconds'] for k,v in published.items() if k in before)
    state.update(published)
    for vid,v in state.items():
     assert not v['surfaces'];assert all(not v.get(f) for f in optional_vectors);assert all(v.get(f) is None for f in optional_objects);assert v['nuclides']=={'inventory':{}} and not any(v['foam'].values()) and v['pending_co2_transfer_mol']==0;assert v['resolved']=={'solution':None,'valid':False};reservoir_count+=1;actual=atoms(v);ext=expected.setdefault(vid,{});res={k:actual.get(k,Z)-ext.get(k,Z) for k in actual.keys()|ext.keys()}
     for k,d in res.items():maxima[k]=max(maxima.get(k,Z),abs(d))
     ledger.append({'row':i,'vessel':vid,'residuals':res})
    frames.append((row,before,state.copy()))
   material_ok=all(x<=TOL for x in maxima.values());assert material_ok or suffix=='mo-005',(suffix,maxima);events=[e for r in rows for e in r['events']];criteria=[True,True];metrics={};notes=[];hold=None
   if suffix=='ec-005':
    vals=[]
    for row,before,after in frames:
     if row['operator']['op']!='cell':continue
     assert before==after;pot={};acts=[]
     for vid,v in after.items():
      ion,e0=('Zn+2',-.7618) if vid==0 else ('Cu+2',.3419);activity=next(s['activity'] for s in v['solution']['species'] if s['name']==ion);pot[vid]=e0+8.31446261815324*float(v['temperature'])/(2*96485.33212)*math.log(float(activity));acts.append(activity)
     e,=[e for e in row['events'] if e['event']=='cell_voltage'];pred=pot[e['cathode']]-pot[e['anode']];assert abs(pred-float(e['volts']))<1e-12;vals.append({'solved_activities':acts,'independent_signed_voltage_V':pred,'actual_voltage_V':e['volts'],'T_K':[v['temperature'] for v in after.values()]})
    assert len(vals)==2 and vals[0]['actual_voltage_V']!=vals[1]['actual_voltage_V'];metrics={'states':vals,'input_NaNO3_mol':D('.02')};notes=['Actual activity/speciation and temperatures determine the shift. No prescribed sign from analytical concentration alone; slow Zn-water limit is unrelated to nonconsuming potential readback.']
   elif suffix=='ec-006':
    vals=[]
    for row,_,after in frames:
     if row['operator']['op']=='add' and row['operator']['species']=='CuSO4':
      v=after[0];metal=amount(v,'Cu','solid');oxidized=D('.0005')-amount(v,'Zn','solid');assert abs(metal-oxidized)<D('1e-10') and metal<=D('.0005')+TOL;vals.append({'Cu_metal_mol':metal,'Zn_oxidation_mol':oxidized,'remaining_Zn_metal_mol':amount(v,'Zn','solid'),'remaining_Cu2_mol':amount(v,'Cu+2')})
    assert len(vals)==3 and vals[0]['Cu_metal_mol']>0 and vals[-1]['remaining_Cu2_mol']>0 and abs(vals[-1]['Cu_metal_mol']-D('.0005'))<=TOL;metrics={'successive_Cu_doses':vals};notes=['Finite donor exhaustion, not a complete kinetic or corrosion claim. Named initial Zn-water dissolution/rate limits do not negate later independently accounted Cu displacement.']
   elif suffix=='ab-028':
    ph={v:[e['value'] for e in events if e['event']=='measured' and e['instrument']=='ph_meter' and e['vessel']==v] for v in (0,1)};assert all(len(x)==4 for x in ph.values());increments={v:[b-a for a,b in zip(ph[v],ph[v][1:])] for v in ph};assert increments[0][1]<increments[1][1] and increments[0][2]<increments[1][2];pools=[]
    for row,_,after in frames:
     if row['operator']['op']=='measure' and row['operator']['instrument']=='ph_meter':
      v=after[row['operator']['vessel']];pools.append({'vessel':v['id'],'acetate_mol':amount(v,'CH3COOH')+amount(v,'CH3COO-'),'P_mol':elem(v,'P'),'phosphate_proton_inventory_mol':sum(amount(v,s)*n for s,n in [('H3PO4',3),('H2PO4-',2),('HPO4-2',1)]),'phosphate_species_mol':{s:amount(v,s) for s in ['H3PO4','H2PO4-','HPO4-2','PO4-3']},'T_K':v['temperature']})
    assert all(abs(p['acetate_mol']-(D('.001') if p['vessel']==0 else D('.002')))<=TOL and abs(p['P_mol']-(D('.001') if p['vessel']==0 else Z))<=TOL for p in pools);metrics={'actual_pH':ph,'equal_0_5mmol_step_delta_pH':increments,'separate_pools':pools};notes=['Compare incremental pH spans against each own starting pH and actual temperatures; separate phosphate proton equivalents and acetate, without assuming equal total formal buffer capacities.']
   elif suffix=='pq-009':
    vals=[{'vessel':v['id'],'Ba_total_mol':elem(v,'Ba'),'BaSO4_solid_mol':amount(v,'BaSO4','solid'),'Ba_aqueous_mol':amount(v,'Ba+2'),'all_solid_portions':[p for p in v['contents'] if p['phase']=='solid']} for v in state.values()];assert all(abs(v['Ba_total_mol']-(D('.0004') if v['vessel']==0 else D('.0008')))<=TOL for v in vals);assert 'not-modeled.supersaturated-no-registry-solid' in case['named_model_limits'];criteria=[True,False];hold='missing-barium-carbonate-phase';metrics={'arms':vals};notes=['Sulfate allocation is real, but missing Witherite/BaCO3 excludes the competing carbonate phase required by the original question. Barium closure does not make unsupported carbonate chemistry pass.']
   elif suffix=='pq-011':
    initial=next(after for row,_,after in frames if row['operator']['op']=='inspect');later=state;vals=[]
    for vid,v in initial.items():
     acid=amount(v,'CH3COOH');hydroxide=amount(v,'Cu(OH)2','solid');vals.append({'vessel':vid,'pH':v['solution']['ph'],'acid_acetate_mol':acid,'acetate_anion_mol':amount(v,'CH3COO-'),'CuOH2_solid_mol':hydroxide,'other_solids':[p for p in v['contents'] if p['phase']=='solid' and p['species']!='Cu(OH)2'],'total_Cu_mol':elem(v,'Cu')})
    assert vals[0]['CuOH2_solid_mol']>D('.00019') and vals[1]['CuOH2_solid_mol']==0 and amount(later[1],'Cu(OH)2','solid')>D('.00019');assert all(abs(elem(v,'Cu')-D('.0002'))<=TOL for v in later.values());metrics={'after_equal_0_4mmol_base_pulses':vals,'later_buffered_pH':later[1]['solution']['ph'],'later_buffered_CuOH2_mol':amount(later[1],'Cu(OH)2','solid'),'later_buffered_Cu_total_mol':elem(later[1],'Cu')};notes=['Metastable supported hydroxide/basic-sulfate domain only. Explicit Tenorite formation-temperature exclusion is retained, not a claim of full oxide nucleation/phase equilibrium. Acid/acetate and all other Cu solids stay accounted.']
   elif suffix=='pq-012':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');solid=before[0];afterfilter=after[0];final=state[0];phase='octacalcium_phosphate';initialP=elem(solid,'P','solid');retainedP=elem(afterfilter,'P','solid');assert amount(solid,phase,'solid')>0 and initialP>0 and abs(retainedP-initialP)<=TOL;assert elem(final,'P','solid')==0 and abs(elem(final,'P')-retainedP)<=TOL;metrics={'actual_captured_phase':phase,'phase_composition':composition(phase),'captured_phase_mol':amount(solid,phase,'solid'),'solid_P_before_filter_mol':initialP,'solid_P_after_filter_mol':retainedP,'final_acidified_P_mol':elem(final,'P'),'final_filtrate_P_mol':elem(state[1],'P'),'final_species':final['contents']};notes=['Actual octacalcium-phosphate capture and acid remobilisation only. Withheld Hydroxylapatite/beta-Ca3(PO4)2 metastability remains explicit; no automatic crystalline-equilibrium/nucleation claim.']
   elif suffix=='mo-002':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='react');loss=amount(before[0],'ethanol')-amount(after[0],'ethanol');gain=amount(after[0],'CH3COOH')+amount(after[0],'CH3COO-');oxygen=amount(before[0],'O2')-amount(after[0],'O2');assert loss>0 and abs(gain-loss)<=TOL and abs(oxygen-loss)<=TOL;assert len(titrants)==2;treat,blank=sorted(titrants,key=lambda t:t['vessel']);diff=treat['solute_mol']-blank['solute_mol'];assert abs(diff-loss)<=max(treat['step_mol'],blank['step_mol']);end=[state[v] for v in (0,1)];carboncharge=lambda v:amount(v,'HCO3-')+2*amount(v,'CO3-2');basis=lambda v:amount(v,'base_equivalents')-amount(v,'H+');corrected=diff+amount(end[0],'CH3COOH')-(carboncharge(end[0])-carboncharge(end[1]))-(basis(end[0])-basis(end[1]));assert abs(corrected-loss)<=TOL;metrics={'ethanol_loss_mol':loss,'acetate_gain_mol':gain,'O2_consumed_mol':oxygen,'initial_total_O2_mol':amount(before[0],'O2'),'actual_air':air,'titrant_stages':titrants,'blank_corrected_base_mol':diff,'endpoint_carbonate_and_protonation_corrected_base_mol':corrected,'blank_retained_ethanol_mol':amount(state[1],'ethanol')};notes=['Requested stoichiometric oxidation template, not spontaneous mechanism or calibrated rate. Initial air oxygen, carbonic acidity, endpoint protonation and reagent water remain separately accounted.']
   elif suffix=='mo-005':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');gases=[p for p in before[0]['contents'] if p['phase']=='gas'];assert gases and not material_ok;criteria=[False,True];hold='historical-filter-gas-ownership';metrics={'retained_solid_identity':[p for p in state[0]['contents'] if p['phase']=='solid'],'silently_removed_filter_gases':gases,'initial_air':air,'final_vessel_C_mol':{k:elem(v,'C') for k,v in state.items()}};notes=['Glucose/carbonate questions have positive partial evidence, but original closed N/O and possibly traceC budgets fail when historical filter silently drops headspace. Later accepted gas-preserving filter may address; no new current-source failure claimed.']
   elif suffix=='mo-010':
    ph=[(row,after[0]) for row,_,after in frames if row['operator']['op']=='measure' and row['operator']['instrument']=='conductivity_meter'];vals=[]
    for row,v in ph:
     e,=[e for e in row['events'] if e['event']=='measured'];assert abs(elem(v,'Na')-D('.005'))<=TOL and abs(elem(v,'Cl')-D('.005'))<=TOL;vals.append({'T_K':v['temperature'],'conductivity_uS_cm':e['value'],'water_mol':amount(v,'water'),'scene_layer_volume_L':[x['volume_l'] for x in row['scene']['vessels'][0]['layers']]})
    assert '25 °C, water' in conductivity_source and 'pub fn specific_conductance(info: &SolutionInfo)' in conductivity_source;criteria=[True,False];hold='conductivity-temperature-authority';metrics={'states':vals,'source_basis':'25C limiting ionic mobilities plus concentration correction; no explicit temperature transport argument.','volume_instrument_limit':'Original volume_meter on open vessel refused; independent retained water and scene layers provide ownership control, not a fabricated instrument reading.'};notes=['Small conductivity changes alone do not establish in-situ temperature-dependent transport or actual temperature compensation. Source reference table and measured temperatures expose the missing authority; original physical-temperature experiment is held.']
   elif suffix=='qa-003':
    prior=next(before for row,before,after in frames if row['operator']['op']=='titrate');vals=[]
    for t in titrants:
     vid=t['vessel'];v=prior[vid];volume=amount(v,'water')*D('18.015')/D(997);vals.append({'vessel':vid,'actual_base_mol':t['solute_mol'],'inventory_derived_pretitration_L':volume,'dose_per_L':t['solute_mol']/volume})
    assert len(vals)==2 and abs(vals[0]['actual_base_mol']/vals[1]['actual_base_mol']-3)<D('.001');criteria=[True,False];hold='original-liquid-volume-instrument-protocol';metrics={'aliquots':vals,'actual_failed_volume_instrument_events':[e for e in events if e['event']=='not_yet_modeled'],'available_scene_layer_prevolumes':[[{'vessel':v['id'],'layer_volumes_L':[x['volume_l'] for x in v['layers']]} for v in row['scene']['vessels']] for row in rows if row['operator']['op']=='measure' and row['operator']['instrument']=='volume_meter']};notes=['Scene layer volumes and canonical water prove aliquot proportions, but do not replace the failed volume-meter instrument requested by the original measured-volume criterion. No protocol rewrite.']
   elif suffix=='tc-003':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='add' and f[0]['operator'].get('at')==D('333.15'));cold=before[0];final=after[0];ice=amount(cold,'water','solid');warmn=row['operator']['moles'];warmT=row['operator']['at'];inputwarm=float(warmn)*H.sensible(gold['water'],'liquid',273.15,float(warmT));coldH=water_energy(cold);finalH=water_energy(final);res=finalH-coldH-inputwarm;assert ice>0 and cold['temperature']==D('273.15') and amount(final,'water','solid')==0;assert abs(res)<.1;weighted=(amount(cold,'water')*cold['temperature']+warmn*warmT)/(amount(cold,'water')+warmn);assert final['temperature']<weighted;inferred=(inputwarm-finalH+float(amount(cold,'water','liquid'))*H.sensible(gold['water'],'liquid',273.15,float(cold['temperature'])))/float(ice);metrics={'initial_ice_mol':ice,'initial_liquid_water_mol':amount(cold,'water','liquid'),'initial_T_K':cold['temperature'],'warm_input_mol':warmn,'warm_input_T_K':warmT,'actual_final_T_K':final['temperature'],'liquid_only_weighted_T_K':weighted,'independent_cold_enthalpy_J':coldH,'independent_warm_enthalpy_J':inputwarm,'independent_final_enthalpy_J':finalH,'whole_mixing_energy_residual_J':res,'inferred_fusion_J_mol':inferred};notes=['Pure-water phase-specific sensible/latent balance; actual starting ice fraction at273.15K, so no subzero solid warming is omitted. No empirical calorimeter calibration.']
   elif suffix=='tc-008':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='add' and f[0]['operator']['species']=='NaCl');b=before[0];v=after[0];n=row['operator']['moles'];q=water_energy(v)-water_energy(b);incoming=float(n)*H.sensible(gold['NaCl'],'solid',273.15,float(row['operator']['at']));dissolution=float(n)*gold['NaCl']['dissolution_enthalpy_kj']*1000;res=q-incoming+dissolution;criteria=[amount(v,'water','solid')<amount(b,'water','solid') and v['temperature']<b['temperature'],abs(res)<.1];hold=None if all(criteria) else 'historical-brine-ice-energy-ownership';metrics={'before_ice_mol':amount(b,'water','solid'),'after_ice_mol':amount(v,'water','solid'),'before_T_K':b['temperature'],'after_T_K':v['temperature'],'undissolved_NaCl_mol':amount(v,'NaCl','solid'),'salt_dissolution_energy_J':dissolution,'independent_water_phase_enthalpy_change_J':q,'incoming_salt_sensible_enthalpy_J':incoming,'thermal_closure_residual_J':res,'actual_phase_events':[e for e in row['events'] if e['event']=='state_changed']};notes=['Include actual salt dissolution and all ice/liquid enthalpy, not cooling/melting signs alone. Historical phase-energy failure, if present, is not automatically a current-source defect.']
   else:raise AssertionError(suffix)
   output.append({'id':case['id'],'proposed_candidate':all(criteria) and material_ok,'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],criteria,strict=True)],'metrics':metrics,'limitations':notes,'hold_or_gap':hold,'named_model_limits':case['named_model_limits'],'source':case['engine'],'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(err),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt'],'material_budget_pass':material_ok,'material_maxima':maxima,'per_step_vessel_material_charge':ledger,'boundary_exports':exports,'initial_air':air,'titrant_carrier':titrants,'noncontents_reservoirs':{'checked_snapshots':reservoir_count,'all_optional_owned_vectors_empty':optional_vectors,'all_optional_owned_objects_none':optional_objects,'surfaces_empty':True,'nuclides_empty':True,'foam_zero':True,'pending_CO2_zero':True,'resolved_mirror_empty':True}})
 return {'schema':'kerotakis.batch2.fifth-saved-review.v1','cases':output,'raw_archive_sha256':ardigest,'registry_sha256':sha(regraw),'thermal_fixture_sha256':sha(goldraw),'helper_sha256':sha(HELPER.read_bytes()),'source_blobs_sha256':{p:sha(show(p)) for p in ['crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/vessel.rs','crates/kerotakis-core/src/solve.rs','crates/kerotakis-core/src/conductivity.rs','crates/kerotakis-core/src/displacement.rs','crates/kerotakis-core/src/states.rs','crates/kerotakis-core/src/heat_capacity.rs']},'original_lock_qualification':binding['lock_qualification'],'new_engine_or_HTTP_requests':0,'catalog_selection_assigned':False,'material_tolerance_mol':TOL}
if __name__=='__main__':print(json.dumps(review(),indent=2,default=str))
