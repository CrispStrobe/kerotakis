#!/usr/bin/env python3
"""Eight disjoint originals: authenticate historical raw bytes and independently review saved physics."""
import argparse,hashlib,json,re,subprocess,zipfile
from decimal import Decimal,localcontext
from pathlib import Path
D=Decimal;Z=D(0);TOL=D('1e-8');IDS=('pq-001','pq-004','ec-009','se-008','tc-001','tc-005','pq-010','ki-008');sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes(),parse_float=D)
def amount(v,s,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase)),Z)
def review(root):
 base=root/'audits/new-experiments-201-20261016/native-screen-results';ar=base/'raw-artifact.zip';digest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert digest==load(base/'artifact-metadata.json')['digest'].removeprefix('sha256:');binding=load(base/'binding.json');engine=binding['engine'];source=engine['source_head'];assert source=='c44692053a94c59dcf62e7fb67549f84c7d18f83';assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=root,text=True).strip()==engine['source_tree'];show=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=root)
 paths=['crates/kerotakis-core/src/'+p for p in ['bench.rs','vessel.rs','solve.rs','kinetics.rs','curated.rs','spectrum.rs','starch_iodine.rs','clock.rs','indicator.rs','solution_optics.rs','instrument.rs','states.rs','heat_capacity.rs']];paths += ['crates/kerotakis-phreeqc/src/aqueous.rs','crates/kerotakis-thermo/src/vle.rs','crates/kerotakis-thermo/src/batch.rs'];blobs={p:show(p) for p in paths};bench=blobs[paths[0]].decode();vsource=blobs[paths[1]].decode();kinetics=blobs[paths[3]].decode();curated=blobs[paths[4]].decode();regraw=show('data/registry/registry-source-v1.json');reg=json.loads(regraw,parse_float=D);comps={r['species_id']:r for r in reg['compositions']};goldraw=show('crates/kerotakis-core/tests/golden/registry.json');gold={r['key']:r for r in json.loads(goldraw,parse_float=D)};intake={r['original_id']:r for r in load(base.parent/'intake.json')};selected={r['original_target_id'] for r in load(root/'audits/target468-20261016/selection-expansion-seventh-20261020/catalog-membership.json')['mapping']}
 vectors=['unresolved_materials','material_objects','electrodes','exchanges','adsorbed','solid_solutions','surface_colours','charred_materials'];objects=['soap_scum','lemon_paper_mark','surface_particles','emulsion']
 for f in vectors:assert re.search(r'#\[serde\(default, skip_serializing_if = "Vec::is_empty"\)\]\s+pub '+f+r':',vsource)
 for f in objects:assert re.search(r'#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]\s+pub '+f+r':',vsource)
 assert 'if vessel.pending_co2_transfer_mol.abs() > kerotakis_core::clock::CO2_TRANSFER_FLOOR_MOL' in blobs['crates/kerotakis-phreeqc/src/aqueous.rs'].decode()
 def comp(s):
  if s=='catalase':return {} # Mass/amount tracer only; registry C placeholder is NOT a protein formula.
  if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
  r=comps[s];return {**{e['element']:e['count']['value'] for e in r['elements']},'charge':r['net_charge']['value']}
 def book(d,s,n):
  for k,v in comp(s).items():d[k]=d.get(k,Z)+v*n
 def atoms(v,phase=None):
  d={}
  for p in v['contents']:
   assert p['moles']>=0 and p['moles'].is_finite()
   if phase is None or p['phase']==phase:book(d,p['species'],p['moles'])
  return d
 def elem(v,k,phase=None):return atoms(v,phase).get(k,Z)
 def oxid(v):return amount(v,'I2')+amount(v,'I3-') # Each I3- contains one I2 oxidant equivalent and one iodide.
 def netacid(v):return amount(v,'H+')-amount(v,'base_equivalents')
 output=[]
 with localcontext() as ctx,zipfile.ZipFile(ar) as archive:
  ctx.prec=45
  for suffix in IDS:
   key='batch2_'+suffix;case=load(base/'cases'/f'{key}.json');o=intake[case['id']];assert case['id'] not in selected;assert case['engine']==engine;assert case['original_acceptance_checks']==o['original_acceptance_checks'] and case['original_physical_forecast']==o['original_expected_result'];script=archive.read(key+'/experiment.lab');raw=archive.read(key+'/stdout.jsonl');err=archive.read(key+'/stderr.txt');assert script==(base.parent/o['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(err)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0;rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x.strip()];state={};expected={};frames=[];ledger=[];maximum={};air=[];exports=[];transfers=[];clocks=[];reservoirs=0;pending=[]
   for i,row in enumerate(rows):
    assert set(row['bench'])=={'vessels'};before=state.copy();op=row['operator'];typ=op['op'];published={v['id']:v for v in row['bench']['vessels']};events=row['events']
    if typ=='add':book(expected.setdefault(op['vessel'],{}),op['species'],op['moles'])
    elif typ=='dilute':book(expected[op['vessel']],'water',op['volume']*D(997)/D('18.015'))
    elif typ in ('seal','regulate'):
     v=before[op['vessel']];n=Z if v['headspace']['boundary']!='open' else (op['pressure'] if typ=='regulate' else D(101325))*op.get('headspace_volume',op.get('initial_volume'))/(D('8314.462618')*v['temperature']);rec={'row':i,'vessel':op['vessel'],'N2_mol':n*D('.7901'),'O2_mol':n*D('.2095'),'CO2_mol':n*D('.0004')};air.append(rec)
     for s,f in [('N2',D('.7901')),('O2',D('.2095')),('CO2',D('.0004'))]:book(expected[op['vessel']],s,n*f)
    elif typ=='titrate':
     t,=[e for e in events if e['event']=='titrated'];dose=t['total_volume']*t['concentration'];book(expected[op['vessel']],op['titrant'],dose);book(expected[op['vessel']],'water',t['total_volume']*D(997)/D('18.015'))
    elif typ in ('filter','decant','magnet'):
     frm,to=op['from'],op['to'];expected.setdefault(to,{});moved=[]
     for p in before[frm]['contents']:
      eligible=(p['phase'] in ('liquid','aqueous')) if typ!='magnet' else (p['phase']=='solid' and gold[p['species']]['magnetic'])
      if eligible:
       n=p['moles']*op.get('fraction',D(1));book(expected[frm],p['species'],-n);book(expected[to],p['species'],n);moved.append({'species':p['species'],'moles':n,'phase':p['phase']})
     transfers.append({'row':i,'operation':typ,'from':frm,'to':to,'independently_expected_portions':moved,'original_donor_gases':[p for p in before[frm]['contents'] if p['phase']=='gas'],'actual_donor_gases':[p for p in published[frm]['contents'] if p['phase']=='gas']})
    elif typ=='distil':
     frm,to=op['from'],op['to'];expected.setdefault(to,{})
     if any(e['event']=='distilled' for e in events):
      # This original salt arm contains only water as a volatile component.
      assert all(p['species'] in ('water','Na+','Cl-') for p in before[frm]['contents']);n=amount(before[frm],'water','liquid')*op['fraction'];book(expected[frm],'water',-n);book(expected[to],'water',n)
     else:assert published==before
    elif typ=='extract':
     assert any(e['event']=='not_yet_modeled' and e['reason']['key']=='not-modeled.no-distribution-coefficient' for e in events);assert published==before
    else:assert typ in ('new_vessel','wait','inspect','measure','heat','cool','stir'),typ
    for e in events:
     assert e['event']!='solver_failed'
     if e['event'] in ('gas_evolved','gas_absorbed'):
      # A finite open add's gas_absorbed event describes the already-booked dose, not an additional reservoir.
      dose_event=typ=='add' and e['event']=='gas_absorbed' and e['species']==op['species'] and e['vessel']==op['vessel']
      if not dose_event:book(expected[e['vessel']],e['species'],e['moles']*(D(-1) if e['event']=='gas_evolved' else D(1)))
      exports.append({'row':i,**e,'already_owned_finite_addition':dose_event})
    if set(before)-set(published):assert typ in ('inspect','measure') and all(v==before[k] for k,v in published.items())
    state.update(published)
    for vid,v in state.items():
     if vid in before:
      elapsed=v['elapsed_seconds']-before[vid]['elapsed_seconds'];assert elapsed==(op['seconds'] if typ=='wait' or (typ=='stir' and vid==op['vessel']) else Z)
     else:elapsed=Z;assert v['elapsed_seconds']==0
     clocks.append({'row':i,'vessel':vid,'op':typ,'elapsed_seconds':v['elapsed_seconds'],'delta_seconds':elapsed,'temperature_K':v['temperature'],'pH':v['solution']['ph'] if v.get('solution') else None});assert not v['surfaces'];assert all(not v.get(f) for f in vectors);assert all(v.get(f) is None for f in objects);assert v['nuclides']=={'inventory':{}} and not any(v['foam'].values()) and v['resolved']=={'solution':None,'valid':False};pending.append({'row':i,'vessel':vid,'unapplied_external_CO2_credit_mol':v['pending_co2_transfer_mol']});reservoirs+=1;actual=atoms(v);ext=expected.setdefault(vid,{});res={k:actual.get(k,Z)-ext.get(k,Z) for k in actual.keys()|ext.keys()}
     for k,d in res.items():maximum[k]=max(maximum.get(k,Z),abs(d))
     ledger.append({'row':i,'vessel':vid,'residuals':res})
    frames.append((row,before,state.copy()))
   material=all(v<=TOL for v in maximum.values());full=material and suffix!='ki-008';criteria=[True,True];metrics={};notes=[];hold=None
   if suffix=='pq-001':
    stages=[]
    for r,before,after in frames:
     if r['operator']['op']!='inspect':continue
     v=after[0];kg=v['solution']['solvent_kg'];native=[s for s in v['solution']['species'] if 'Fe' in s['name'] and 'SCN' in s['name']];bound=sum((s['molality']*kg for s in native),Z);stages.append({'pH':v['solution']['ph'],'T_K':v['temperature'],'native_Fe_SCN_complexes':native,'bound_complex_Fe_mol_from_native_molality':bound,'analytical_Fe_mol':elem(v,'Fe'),'SCN_component_mol':amount(v,'SCN-')})
    assert len(stages)==2 and 0<stages[0]['bound_complex_Fe_mol_from_native_molality']<stages[1]['bound_complex_Fe_mol_from_native_molality'];assert all(abs(s['analytical_Fe_mol']-D('.0001'))<TOL for s in stages);uv=[{'operator':r['operator'],'events':r['events']} for r in rows if r['operator']['op']=='measure'];assert all(not any(e['event']=='measured' for e in r['events']) for r in uv);criteria=[True,True];hold='original_FeSCN_optical_forecast_unavailable';metrics={'ligand_stages':stages,'actual_UVvis_responses':uv};notes=['Both literal inventory/complex-identification criteria have positive evidence, but original forecast also requires characteristic optical response. All actual UVvis readings refuse incomplete native spectra; explicit complex identity does not replace unavailable optical evidence. Native diagnostic quantities are distribution estimates, never additional molecular inventory.']
   elif suffix=='pq-004':
    final=state[0];criteria=[False,abs(sum((elem(v,'Ag') for v in state.values()),Z)-D('.0005'))<TOL];assert amount(final,'AgI')==0 and amount(final,'KI')==D('.0006');metrics={'final_AgCl_solid_mol':amount(final,'AgCl','solid'),'final_AgI_mol':amount(final,'AgI'),'unchanged_KI_mol':amount(final,'KI'),'all_phase_Ag_mol':sum((elem(v,'Ag') for v in state.values()),Z),'all_final_contents':{k:v['contents'] for k,v in state.items()}};hold='iodide_silver_phase_exchange_unrepresented';notes=['Small AgCl dissolution after rewetting is not conversion to AgI. No iodide uptake/AgI production occurs; registered KI and a conserved silver total do not establish the missing exchange.']
   elif suffix=='ec-009':
    r,before,after=next(f for f in frames if f[0]['operator']['op']=='titrate');op=r['operator'];t,=[e for e in r['events'] if e['event']=='titrated'];v=after[0];loss=amount(before[0],'Fe+2')-amount(v,'Fe+2');gain=amount(v,'Fe+3')-amount(before[0],'Fe+3');mnII=amount(v,'Mn+2');electrons=loss-5*mnII;dose=t['total_volume']*t['concentration'];assert loss>0 and abs(loss-gain)<TOL;assert abs(electrons)>D('1e-6');criteria=[False,True];hold='redox_titration_owned_charge_electron_mismatch';metrics={'actual_titrated_event':t,'titrant_mol':dose,'titrant_carrier_water_mol':t['total_volume']*D(997)/D('18.015'),'actual_FeII_loss_mol':loss,'actual_FeIII_gain_mol':gain,'actual_MnII_mol':mnII,'remaining_MnO4_mol':amount(v,'MnO4-'),'MnO2_solid_mol':amount(v,'MnO2','solid'),'Fe_electrons_minus_Mn_electron_demand_mol':electrons,'canonical_net_charge_mol':atoms(v).get('charge',Z),'actual_final_contents':v['contents'],'original_colour_endpoint_vs_UVvis':[e for row in rows if row['operator']['op']=='measure' for e in row['events']]};notes=['Actual nominal10.1mL endpoint includes0.2mmol MnII plus2µmol residualpermanganate, but FeII oxidation supplies1.10924µmol fewer electrons than that MnII demands. Complete charge/H ledgers diagnose this mismatch; endpoint success and approximate5:1 dose cannot qualify inconsistent owned chemistry. Native redoxstooddown/analytical-charge warnings remain preserved.']
   elif suffix=='se-008':
    solid=state[0];filtrate=state[1];assert not solid['contents'] and amount(filtrate,'water')>0;criteria=[False,False];hold='original_cooling_does_not_produce_crystals';heat=[]
    for r,bef,aft in frames:
     if r['operator']['op'] not in ('heat','cool'):continue
     b=bef[0];v=aft[0];delta=float(amount(b,'water'))*sensible(gold['water'],'liquid',float(b['temperature']),float(v['temperature']));target=float(r['operator']['energy'])*(1 if r['operator']['op']=='heat' else -1);heat.append({'op':r['operator'],'before_T_K':b['temperature'],'after_T_K':v['temperature'],'phase_specific_water_sensible_J':delta,'prescribed_energy_J':target,'source_sensible_residual_J':delta-target});assert abs(delta-target)<.1
    metrics={'feed_KNO3_mol':D('50')/gold['KNO3']['molar_mass'],'feed_NaCl_mol':D('5')/gold['NaCl']['molar_mass'],'retained_solid_contents':solid['contents'],'final_filtrate_K_mol':elem(filtrate,'K'),'final_filtrate_NO3_mol':amount(filtrate,'NO3-'),'final_filtrate_Na_mol':elem(filtrate,'Na'),'final_filtrate_Cl_mol':elem(filtrate,'Cl'),'final_temperature_K':filtrate['temperature'],'actual_heat_steps':heat};notes=['All original salt remains dissolved after cooling to45.29875C; retained solid is empty, so purity and crystal recovery are undefined. This saved protocol is inconclusive for its predicted selectivecrystallisation. No unsupported claim that this proves an appbug or real mixed-salt saturation curve; source/property/domain inspection is separate.']
   elif suffix=='tc-001':
    vals=[]
    for i,(r,before,after) in enumerate(frames):
     if r['operator']['op']!='measure':continue
     e,=[e for e in r['events'] if e['event']=='measured'];vals.append({'row':i,'instrument':r['operator']['instrument'],'vessel':e['vessel'],'value':e['value'],'unit':e['unit']})
    calcium=amount(state[0],'Ca+2');assert calcium==D('.001') and state[0]['contents']==state[1]['contents'];cal={vid:[v['value']*1000 for v in vals if v['vessel']==vid and v['instrument']=='calorimeter'] for vid in (0,1)};assert len(cal[0])==len(cal[1])==2;assert abs(cal[1][-1]-cal[0][-1])>100;assert gold['CaO']['dissolution_enthalpy_kj'] is None;criteria=[False,True];hold='route_dependent_missing_CaO_thermal_authority';metrics={'actual_instrument_readings':vals,'calorimeter_actual_J':cal,'final_route_difference_J':cal[1][-1]-cal[0][-1],'independent_retained_water_sensible_J':{vid:float(amount(v,'water'))*sensible(gold['water'],'liquid',298.15,float(v['temperature'])) for vid,v in state.items()},'same_final_phase_species_contents':state[0]['contents'],'CaO_declared_dissolution_enthalpy_kJ_mol':gold['CaO']['dissolution_enthalpy_kj'],'CaO_provenance':gold['CaO']['provenance']};notes=['Exact same final retainedcomposition, identicalinputs andnoboundaryexports do not closeHess-law energy: acidfirst/CaO~0J versusCaOfirst/acid103.822J. Full route instrument histories and sensibleenergies are compared, not only lastincrement. Source explicitly withholds slakingenthalpy; no missing heat or calibrated reference is invented.']
   elif suffix=='tc-005':
    pressures=[state[v]['pressure'] for v in (0,1)];assert pressures==[D(100000),D(200000)];routed=[e for r in rows for e in r['events'] if e['event']=='boiling_point_routed'];assert len(routed)==2 and routed[1]['route']=='pressure_outside_cleared_window';heat=[]
    for r,bef,aft in frames:
     if r['operator']['op']!='heat':continue
     vid=r['operator']['vessel'];b=bef[vid];v=aft[vid];water_sensible=float(amount(b,'water'))*sensible(gold['water'],'liquid',float(b['temperature']),float(v['temperature']));latent=float(amount(v,'water','gas')-amount(b,'water','gas'))*40650;gas_sensible=sum(float(amount(b,s,'gas'))*sensible(gold[s],'gas',float(b['temperature']),float(v['temperature'])) for s in ['N2','O2','CO2']);heat.append({'vessel':vid,'before_T_K':b['temperature'],'after_T_K':v['temperature'],'water_sensible_J':water_sensible,'canonical_water_vapour_latent_J':latent,'preexisting_air_sensible_J':gas_sensible,'declared8kJ_minus_these_terms_J':8000-water_sensible-latent-gas_sensible,'retained_water_vapour_mol':amount(v,'water','gas'),'actual_pressure_Pa':v['pressure'],'actual_gas_boundary_events':[e for e in r['events'] if e['event'] in ('gas_evolved','gas_absorbed')]})
    criteria=[True,False];hold='pressure_outside_reviewed_boiling_domain';metrics={'initial_pressure_controlled_air':air,'actual_heat_arms':heat,'actual_boiling_routes':routed,'actual_transition_instrument_responses':[e for r in rows for e in r['events'] if e['event']=='transition_point_read'],'actual_pressures_Pa':pressures,'actual_final_gas_inventory':{vid:[p for p in v['contents'] if p['phase']=='gas'] for vid,v in state.items()}};notes=['Regulatorinitialisation trapsair atactualspecifiedpressure; all gas andwater remaincanonical andnoexplicit gas export occurs. That bookkeeping is not validatedphysicalregulator venting.','Two-bar phase route explicitly reports pressure_outside_cleared_window andfalls backnear1atm100C. The transitioninstrument returnsnotisolated/no numericvalue. Directional tinyTchange cannot qualify an operative2barboiling/enthalpy prediction; no unclearedAntoineextrapolation or physicalcalibration is added.']
   elif suffix=='pq-010':
    inspected=[after[0] for r,bef,after in frames if r['operator']['op']=='inspect'];b=inspected[0];final=state[0];carbsolid=[p for p in b['contents'] if p['phase']=='solid' and comp(p['species']).get('C',Z)>0];assert not carbsolid;CO2=sum((e['moles'] for e in exports if e['event']=='gas_evolved' and e['species']=='CO2'),Z);criteria=[False,material];hold='original_required_silver_carbonate_solid_absent';metrics={'preacid_solid_portions':[p for p in b['contents'] if p['phase']=='solid'],'preacid_solid_C_atoms_mol':elem(b,'C','solid'),'final_allphase_silver_mol':elem(final,'Ag'),'explicit_CO2_export_mol':CO2,'final_carbonate_pool_mol':sum((amount(final,s) for s in ['CO2(aq)','HCO3-','CO3-2']),Z),'final_acetate_pool_mol':amount(final,'CH3COOH')+amount(final,'CH3COO-'),'before_final_AgCl_mol':[amount(v,'AgCl','solid') for v in [b,final]]};notes=['OnlyAgClsolid existsbeforeacid, withno carbonatebearingAgsolid. Acidreleasesdissolvedcarbonatecarbonbutdoesnotdemonstrateselectivesolidcarbonate removal; originalC1explicitlymakesabsenceinconclusive. All actualCO2exports andsilverredistribution arebudgeted.']
   elif suffix=='ki-008':
    r,bef,aft=next(f for f in frames if f[0]['operator']['op']=='wait');arms=[]
    for vid,b in bef.items():
     v=aft[vid];loss=amount(b,'H2O2')-amount(v,'H2O2');oxygen=amount(v,'O2')-amount(b,'O2');water=amount(v,'water')-amount(b,'water');assert abs(loss-2*oxygen)<D('1e-12') and abs(water-loss)<D('1e-12');assert amount(b,'MnO2')==amount(v,'MnO2') and amount(b,'catalase')==amount(v,'catalase');arms.append({'vessel':vid,'actual_seconds':v['elapsed_seconds']-b['elapsed_seconds'],'initial_T_K':b['temperature'],'final_T_K':v['temperature'],'H2O2_loss_mol':loss,'conversion_fraction':loss/amount(b,'H2O2'),'O2_gain_mol':oxygen,'water_gain_mol':water,'MnO2_tracer_retained_mol':amount(v,'MnO2'),'catalase_amount_tracer_retained_mol':amount(v,'catalase')})
    symbolic=[];expected_enzyme={}
    for i,(row,before,after) in enumerate(frames):
     op=row['operator']
     if op['op']=='add' and op['species']=='catalase':expected_enzyme[op['vessel']]=expected_enzyme.get(op['vessel'],Z)+op['moles']
     for vid,v in after.items():
      portions=[p for p in v['contents'] if p['species']=='catalase'];assert all(p['phase']=='aqueous' for p in portions);owned=sum((p['moles'] for p in portions),Z);coefficient=owned-expected_enzyme.get(vid,Z);assert coefficient==0;symbolic.append({'row':i,'vessel':vid,'input_catalase_mol':expected_enzyme.get(vid,Z),'actual_owned_catalase_aqueous_mol':owned,'unknown_atom_and_charge_vector_coefficient_residual_mol':coefficient})
    full=material and all(p['unknown_atom_and_charge_vector_coefficient_residual_mol']==0 for p in symbolic);assert '.max_by(|a, b| a.2.total_cmp(&b.2))' in kinetics;assert arms[1]['conversion_fraction']==arms[2]['conversion_fraction'];criteria=[True,full];hold=None;metrics={'three_actual_conversion_arms':arms,'every_step_symbolic_enzyme_conservation':symbolic,'symbolic_proof':'For every unknown enzyme elemental/charge coordinate A, contribution to conservation residual is A*(owned_enzyme_moles-input_enzyme_moles)=A*0 exactly. No absolute molecular formula is asserted.','exact_source_combination_rule':'Maximum effective active pathway strength, activity*exp(-Ea/RT), relative to uncatalysed baseline; no multiplication of catalyst factors.','catalase_formula_placeholder':gold['catalase']['formula'],'catalase_molar_mass_g_mol':gold['catalase']['molar_mass'],'catalase_declared_domain':gold['catalase']['provenance']};notes=['Actualfractional conversions andexplicitmaxpathway rule are reported. Equal nearcompleteenzymearms do not identify interactionparameters. MnO2 andenzymeamounttracers persist; H/O/Mn atomsclose forrepresentedmolecules. Registry Cplaceholder isnotproteincomposition; no absoluteenzymeformulaorchargeisinvented. Identicalaqueousenzymeidentity/amountatEVERYstepvessel establishesconservationforANYunknownfixedenzymeatom/chargevectorbyzero symboliccoefficient, withnohiddenreservoir. No independentinhibition/calibration claim.']
   else:raise AssertionError(suffix)
   assert len(criteria)==len(case['original_acceptance_checks']);output.append({'id':case['id'],'proposed_candidate':all(criteria) and full and hold is None,'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],criteria,strict=True)],'metrics':metrics,'limitations':notes,'hold_or_gap':hold,'named_model_limits':case['named_model_limits'],'source':engine,'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(err),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt'],'material_budget_pass':full,'declared_nonprotein_molecular_budget_pass':material,'unknown_absolute_molecular_formula':['catalase'] if suffix=='ki-008' else [],'unknown_formula_conservation_proven_symbolically':suffix=='ki-008' and full,'material_maxima':maximum,'per_step_vessel_material_charge':ledger,'actual_clocks_temperatures_pH':clocks,'boundary_exports':exports,'initial_air':air,'transfers':transfers,'noncontents_reservoirs':{'checked_snapshots':reservoirs,'all_optional_owned_vectors_empty':vectors,'all_optional_owned_objects_none':objects,'surfaces_empty':True,'nuclides_empty':True,'foam_zero':True,'pending_CO2_external_credits':pending,'resolved_mirror_empty':True}})
 return {'schema':'kerotakis.batch2.eighth-saved-review.v1','cases':output,'raw_archive_sha256':digest,'registry_sha256':sha(regraw),'thermal_registry_fixture_sha256':sha(goldraw),'source_blobs_sha256':{p:sha(b) for p,b in blobs.items()},'original_lock_qualification':binding['lock_qualification'],'new_engine_or_HTTP_requests':0,'catalog_selection_assigned':False,'material_tolerance_mol':TOL}
def cp(row,phase,T):
 sets=[p for p in row['heat_capacity_polys'] if p['phase']==phase]
 if not sets:return float(row['heat_capacity'])
 qsets=sets[0]['intervals'];T=max(float(qsets[0]['t_min']),min(T,float(qsets[-1]['t_max'])));q=next(q for q in qsets if float(q['t_min'])<=T<=float(q['t_max']));assert q['form']=='nasa9';a=list(map(float,q['coefficients']));return 8.31446261815324*(a[0]/T**2+a[1]/T+a[2]+a[3]*T+a[4]*T**2+a[5]*T**3+a[6]*T**4)
def sensible(row,phase,a,b):
 n=400;h=(b-a)/n;return h/3*sum((1 if i in (0,n) else 4 if i%2 else 2)*cp(row,phase,a+i*h) for i in range(n+1))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[3]);a=p.parse_args();print(json.dumps(review(a.repo),indent=2,default=str))
