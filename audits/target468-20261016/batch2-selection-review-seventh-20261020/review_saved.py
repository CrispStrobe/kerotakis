#!/usr/bin/env python3
"""Eight disjoint originals: authenticate historical raw bytes and independently review saved physics."""
import argparse,hashlib,json,re,subprocess,zipfile
from decimal import Decimal,localcontext
from pathlib import Path
D=Decimal;Z=D(0);TOL=D('1e-8');IDS=('ab-032','pq-005','ec-008','se-002','se-007','mo-007','mo-008','mo-009');sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes(),parse_float=D)
def amount(v,s,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase)),Z)
def review(root):
 base=root/'audits/new-experiments-201-20261016/native-screen-results';ar=base/'raw-artifact.zip';digest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert digest==load(base/'artifact-metadata.json')['digest'].removeprefix('sha256:');binding=load(base/'binding.json');engine=binding['engine'];source=engine['source_head'];assert source=='c44692053a94c59dcf62e7fb67549f84c7d18f83';assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=root,text=True).strip()==engine['source_tree'];show=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=root)
 paths=['crates/kerotakis-core/src/'+p for p in ['bench.rs','vessel.rs','solve.rs','kinetics.rs','curated.rs','spectrum.rs','starch_iodine.rs','clock.rs','indicator.rs','solution_optics.rs','instrument.rs','states.rs','heat_capacity.rs']];paths += ['crates/kerotakis-phreeqc/src/aqueous.rs','crates/kerotakis-thermo/src/vle.rs','crates/kerotakis-thermo/src/batch.rs'];blobs={p:show(p) for p in paths};bench=blobs[paths[0]].decode();vsource=blobs[paths[1]].decode();kinetics=blobs[paths[3]].decode();curated=blobs[paths[4]].decode();regraw=show('data/registry/registry-source-v1.json');reg=json.loads(regraw,parse_float=D);comps={r['species_id']:r for r in reg['compositions']};goldraw=show('crates/kerotakis-core/tests/golden/registry.json');gold={r['key']:r for r in json.loads(goldraw,parse_float=D)};intake={r['original_id']:r for r in load(base.parent/'intake.json')};selected={r['original_target_id'] for r in load(root/'audits/target468-20261016/selection-sulfur-filter-beneficiary-20261020/catalog-membership.json')['mapping']}
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
    elif typ=='seal':
     v=before[op['vessel']];n=Z if v['headspace']['boundary']!='open' else D(101325)*op['headspace_volume']/(D('8314.462618')*v['temperature']);rec={'row':i,'vessel':op['vessel'],'N2_mol':n*D('.7901'),'O2_mol':n*D('.2095'),'CO2_mol':n*D('.0004')};air.append(rec)
     for s,f in [('N2',D('.7901')),('O2',D('.2095')),('CO2',D('.0004'))]:book(expected[op['vessel']],s,n*f)
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
    else:assert typ in ('new_vessel','wait','inspect','measure','heat','stir'),typ
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
   material=all(v<=TOL for v in maximum.values());full=material;criteria=[True,True];metrics={};notes=[];hold=None
   if suffix=='ab-032':
    readings=[]
    for i,(r,before,after) in enumerate(frames):
     if r['operator']['op']!='measure' or r['operator']['instrument']!='eyes':continue
     vid=r['operator']['vessel'];v=after[vid];observed,=[e for e in r['events'] if e['event']=='observed'];ind='methyl_orange' if vid==0 else 'phenolphthalein';pka=D('3.47') if vid==0 else D('9.4');ph=v['solution']['ph'];f=1/(1+10**float(pka-ph));scene=next(v for v in r['scene']['vessels'] if v['id']==vid)['liquid'];assert scene['path_length_cm']==4 and observed['appearance']['cloudiness']==0;assert amount(v,ind)==D('1e-7');acidtotal=amount(v,'CH3COOH')+amount(v,'CH3COO-');assert abs(acidtotal-D('.001'))<D('1e-10');readings.append({'row':i,'vessel':vid,'indicator':ind,'indicator_mol':amount(v,ind),'acid_acetate_pool_mol':acidtotal,'indicator_fraction_of_acid_pool':amount(v,ind)/acidtotal,'simultaneous_pH':ph,'T_K':v['temperature'],'reported_liquid_volume_L':scene['volume_l'],'same_path_cm':scene['path_length_cm'],'actual_RGB':observed['appearance']['liquid'],'actual_words':observed['appearance']['words'],'independent_base_form_fraction_from_source_pKa':f})
    mo=[r for r in readings if r['vessel']==0];pp=[r for r in readings if r['vessel']==1];assert len(mo)==len(pp)==3;assert all(a['simultaneous_pH']==b['simultaneous_pH'] for a,b in zip(mo,pp));assert mo[0]['independent_base_form_fraction_from_source_pKa']<.5 and mo[1]['independent_base_form_fraction_from_source_pKa']>.94;assert pp[1]['independent_base_form_fraction_from_source_pKa']<.0001 and pp[2]['independent_base_form_fraction_from_source_pKa']>.07;assert mo[0]['actual_RGB']!=mo[1]['actual_RGB'] and pp[1]['actual_RGB']!=pp[2]['actual_RGB'];assert all(r['indicator_fraction_of_acid_pool']<D('.000101') for r in readings);metrics={'actual_eye_readings_and_simultaneous_pH':readings,'simultaneous_recorded_pH_readings':[e for r in rows for e in r['events'] if e['event']=='measured' and e['instrument']=='ph_meter']};notes=['Bounded indicator-pH teaching response at1µM nominal dose and common4cm path only. Actual words remain orange/colourless: no invented human-visible endpoint or exact detection threshold. ActualRGB changes are paired with own simultaneouspH, not nominaldose alone.','Methyl-orange ionic chemical mapping is absent; its traceinventory is0.01% of formalacid, so no quantitative acid/base/speciation calibration is claimed. Source indicatorpKa and optical spectra, distinct from component speciation, determine this bounded colour response.']
   elif suffix=='pq-005':
    heat,before,after=next(f for f in frames if f[0]['operator']['op']=='heat');b=before[0];v=after[0];n=amount(b,'water');lost=amount(b,'water')-amount(v,'water');exps=sum((e['moles'] for e in heat['events'] if e['event']=='gas_evolved' and e['species']=='water'),Z);assert abs(lost-exps)<D('1e-14');assert elem(v,'C')==0 and amount(v,'CaCO3')==0 and abs(elem(v,'Ca')-D('.001'))<TOL;Cp=gold['water'];sens=float(n)*sensible(Cp,'liquid',float(b['temperature']),float(v['temperature']));latent_row=next(r for r in reg['phase_thermodynamics'] if r['species_id']=='water' and r['property']=='enthalpy_of_vaporisation');latent=float(latent_row['quantity']['value']);assert latent==40650;assert all(gold[p['species']]['heat_capacity']==0 for p in v['contents'] if p['species']!='water');latentenergy=float(lost)*latent;energyres=sens+latentenergy+float(v['excess_enthalpy_j']-b['excess_enthalpy_j'])-float(heat['operator']['energy']);assert abs(energyres)<.1;feed=next(before[1] for r,before,after in frames if r['operator']['op']=='filter');retained=elem(state[1],'Ca','solid');filtrate=elem(state[2],'Ca');assert retained>D('.00098') and filtrate<D('.00002') and abs(retained+filtrate-D('.001'))<TOL;metrics={'heated_arm_Ca_mol':elem(v,'Ca'),'heated_arm_C_mol':elem(v,'C'),'heated_arm_CaCO3_mol':amount(v,'CaCO3'),'heated_water_export_mol':lost,'before_T_K':b['temperature'],'after_T_K':v['temperature'],'heat_input_J':heat['operator']['energy'],'independent_liquid_sensible_heat_J':sens,'independent_exported_vapour_latent_heat_J':latentenergy,'whole_phase_heat_residual_J':energyres,'carbonate_feed_Ca_mol':elem(feed,'Ca'),'retained_solid_Ca_mol':retained,'filtrate_all_phase_Ca_mol':filtrate,'filtrate_all_phase_C_mol':elem(state[2],'C'),'final_contents':{k:v['contents'] for k,v in state.items()}};notes=['Original calcium/carbon/phase question, not a settled boilingpH claim. Heat35kJ is independently reconciled with actual phase-specificCp and exportedwater latentheat; aqueous ionCp iszero in this declared model.','Tiny0.185nmol CaCO3 forms anew in filtrate after filtering; it is included in totalfiltrateCa, not presented as transferred oldsolid. No complete hardnessremoval or quantitative carbonate-nucleation calibration claim.']
   elif suffix=='ec-008':
    assert amount(state[0],'Fe+3')==D('.0004') and amount(state[0],'I2')==0 and amount(state[1],'I2')==0;criteria=[False,True];hold='iodide-iron-redox-unsupported';metrics={'arms':[{'vessel':k,'FeIII_mol':amount(v,'Fe+3'),'FeII_mol':amount(v,'Fe+2'),'I2_mol':amount(v,'I2'),'KI_mol':amount(v,'KI'),'all_O2_mol':amount(v,'O2'),'canonical_contents':v['contents'],'solved_iron_species':[s for s in v['solution']['species'] if 'Fe' in s['name']]} for k,v in state.items()]};notes=['Both actual modeled inventories containnoO2 and no otheraddedoxidant beyondFeIII in firstarm. No physicaloxygenexclusion claimed for an unsealedbeaker. Ferric reduction/iodineproduction never occurs; full iron/iodine closure anddifferent ironcolour do not repair missing redox/speciation support.']
   elif suffix=='se-002':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');assert amount(before[0],'naphthalene','aqueous')==D('.000005') and amount(before[0],'naphthalene','solid')==0 and amount(after[1],'naphthalene','aqueous')==D('.000005');assert state[2]['contents']==[];criteria=[True,False];hold='extraction-distribution-and-product-observer-unavailable';metrics={'prefilter_dissolved_naphthalene_mol':amount(before[0],'naphthalene','aqueous'),'final_feed_naphthalene_mol':amount(state[1],'naphthalene'),'final_feed_Na_mol':elem(state[1],'Na'),'final_feed_Cl_mol':elem(state[1],'Cl'),'organic_product_contents':state[2]['contents'],'actual_extract_events':[e for r in rows if r['operator']['op']=='extract' for e in r['events']],'chromatograph_events':[e for r in rows if r['operator']['op']=='measure' and r['operator']['instrument']=='chromatograph' for e in r['events']]};notes=['Dissolved startingnaphthalene andfiltertransfer are proven, but extractionrefuses beforeaddinghexane ortransferringanycomponent. Emptyorganicproduct andunavailablechromatograph are not physicalpartitionpredictions. No unearned extractionenergy or partitionfactor closure.']
   elif suffix=='se-007':
    cuts=[]
    for r,before,after in frames:
     if r['operator']['op']!='distil':continue
     op=r['operator'];src,dst=op['from'],op['to'];events=[e for e in r['events'] if e['event']=='distilled'];cuts.append({'from':src,'to':dst,'actual_events':r['events'],'source_before_contents':before[src]['contents'],'source_after_contents':after[src]['contents'],'product_contents':after[dst]['contents'],'actual_source_T_before_K':before[src]['temperature'],'actual_source_T_after_K':after[src]['temperature'],'actual_product_T_K':after[dst]['temperature']})
     if events:
      e,=events;assert 'pub const WATER_HVAP_KJ_PER_MOL: f64 = 40.657;' in blobs['crates/kerotakis-thermo/src/vle.rs'].decode();computed=amount(after[dst],'water')*D('40.657');assert abs(computed-e['energy_kj'])<D('1e-8');cuts[-1]['independent_declared_burner_condenser_latent_energy_kJ']=computed
    assert amount(state[2],'water')>0 and elem(state[2],'Na')==0 and elem(state[2],'Cl')==0 and not state[3]['contents'];criteria=[False,False];hold='acetic-acid-distillation-properties-unavailable';metrics={'actual_cuts':cuts,'total_retained_acetate_mol':amount(state[1],'CH3COOH')+amount(state[1],'CH3COO-')};notes=['Salt/watercut materialanddeclaredlatentenergy reconcile: burnerandcondenser supply/removeheat outsidevesselledger. This is not a closedadiabaticwholeapparatusenergy prediction.','Aceticacidarm refuses beforeanytransfer because reviewed vapourpressure/latentheat aremissing; receiverempty. NeitherlowpHnoracidcarryoverexists. Do not substitute saltcontrol or inferquantitativeseparationfactor.']
   elif suffix in ('mo-007','mo-008','mo-009'):
    actual=[]
    for i,(r,before,after) in enumerate(frames):
     if r['operator']['op']!='measure':continue
     vid=r['operator']['vessel'];v=after[vid];scene=next((q for q in r.get('scene',{}).get('vessels',[]) if q['id']==vid),None);liquid=scene.get('liquid') if scene else None;actual.append({'row':i,'vessel':vid,'instrument':r['operator']['instrument'],'actual_events':r['events'],'reported_liquid':liquid,'total_Cu_mol':elem(v,'Cu'),'actual_solved_copper_species':[s for s in v.get('solution',{}).get('species',[]) if 'Cu' in s['name']],'actual_solid_portions':[p for p in v['contents'] if p['phase']=='solid']})
    uv=[r for r in actual if r['instrument']=='spectrophotometer'];assert all(not any(e['event']=='measured' for e in r['actual_events']) for r in uv);assert all(any(e['event']=='not_yet_modeled' and e['reason']['key']=='not-modeled.incomplete-absorbance' for e in r['actual_events']) for r in uv);metrics={'actual_observations':actual,'no_absorbance_reading_available':True,'no_linear_fit_or_extrapolated_recovery':True}
    if suffix=='mo-007':
     volumes=[r['reported_liquid']['volume_l'] for r in uv];assert volumes==[D('.1'),D('.2'),D('.4')];assert all(abs(r['total_Cu_mol']-D('.001'))<TOL for r in uv);criteria=[True,False];hold='copper-complex-spectral-coverage';metrics['actual_reported_volumes_L']=volumes;notes=['Actualvolumes0.1/0.2/0.4L andcommon4cm path arecontrols, not substituteabsorbance. Sulfatecomplexation changes actualsolvedfreeCu/CuSO4/CuOH fractions; missingcomplexspectra blocks requestedfeaturecomparisonandlinearityfit.']
    elif suffix=='mo-008':
     criteria=[False,False];hold='copper-spectrum-and-original-liquid-volume-instrument';metrics['original_volume_instrument_events']=[r for r in actual if r['instrument']=='volume_meter'];notes=['EveryrequestedUVvisandvolumeinstrumentresponsefails. Scenevolumes can independentlyexplainwaterownership butcannotreplaceoriginalmeasuredvolumecriterion. Knowninitial0.5mmolCuandlateradditionsdonotvalidate anycalibrationintercept/recoverywithoutactualabsorbance.']
    else:
     row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');b=before[0];v=after[1];assert amount(b,'SiO2','solid')>D('.0016') and amount(after[0],'SiO2','solid')==amount(b,'SiO2','solid');assert abs(amount(v,'SiO2','aqueous')-amount(b,'SiO2','aqueous'))<D('1e-12') and abs(elem(v,'Cu')-elem(b,'Cu'))<D('1e-10');eyes=[r for r in actual if r['instrument']=='eyes'];assert len(eyes)==2;looks=[next(e['appearance'] for e in r['actual_events'] if e['event']=='observed') for r in eyes];assert looks[0]['cloudiness']>D('.3') and looks[1]['cloudiness']==0 and looks[0]['liquid']==looks[1]['liquid'];metrics.update({'solid_SiO2_retained_mol':amount(after[0],'SiO2','solid'),'aqueous_SiO2_transferred_mol':amount(v,'SiO2','aqueous'),'Cu_transferred_mol':elem(v,'Cu'),'actual_cloudiness_before_after':[l['cloudiness'] for l in looks],'actual_liquid_RGB_before_after':[l['liquid'] for l in looks]});notes=['Bounded materialseparationandactualeyesclaritydemonstration only. StartingSiO2solidisreal; actualeyescloudinessfallswhileblueRGBpersists. SpectrophotometerrefusesmissingCucomplexspectra, so noabsorbance/turbidity/scatteringcalibrationorlinearopticalclaim.','OriginalconditionalforecastandC2explicitlyallowrecognizingunimplementedscattering. Actualsourceabsorbanceusesdissolvedspeciesonly; oxideparticles/adsorptionarenot assignedinventedspectralcrosssections. No absenceofparticlesisinferredfrommissingUVvis.']
   else:raise AssertionError(suffix)
   assert len(criteria)==len(case['original_acceptance_checks']);output.append({'id':case['id'],'proposed_candidate':all(criteria) and full and hold is None,'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],criteria,strict=True)],'metrics':metrics,'limitations':notes,'hold_or_gap':hold,'named_model_limits':case['named_model_limits'],'source':engine,'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(err),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt'],'material_budget_pass':full,'material_maxima':maximum,'per_step_vessel_material_charge':ledger,'actual_clocks_temperatures_pH':clocks,'boundary_exports':exports,'initial_air':air,'transfers':transfers,'noncontents_reservoirs':{'checked_snapshots':reservoirs,'all_optional_owned_vectors_empty':vectors,'all_optional_owned_objects_none':objects,'surfaces_empty':True,'nuclides_empty':True,'foam_zero':True,'pending_CO2_external_credits':pending,'resolved_mirror_empty':True}})
 return {'schema':'kerotakis.batch2.seventh-saved-review.v1','cases':output,'raw_archive_sha256':digest,'registry_sha256':sha(regraw),'thermal_registry_fixture_sha256':sha(goldraw),'source_blobs_sha256':{p:sha(b) for p,b in blobs.items()},'original_lock_qualification':binding['lock_qualification'],'new_engine_or_HTTP_requests':0,'catalog_selection_assigned':False,'material_tolerance_mol':TOL}
def cp(row,phase,T):
 sets=[p for p in row['heat_capacity_polys'] if p['phase']==phase]
 if not sets:return float(row['heat_capacity'])
 qsets=sets[0]['intervals'];T=max(float(qsets[0]['t_min']),min(T,float(qsets[-1]['t_max'])));q=next(q for q in qsets if float(q['t_min'])<=T<=float(q['t_max']));assert q['form']=='nasa9';a=list(map(float,q['coefficients']));return 8.31446261815324*(a[0]/T**2+a[1]/T+a[2]+a[3]*T+a[4]*T**2+a[5]*T**3+a[6]*T**4)
def sensible(row,phase,a,b):
 n=400;h=(b-a)/n;return h/3*sum((1 if i in (0,n) else 4 if i%2 else 2)*cp(row,phase,a+i*h) for i in range(n+1))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[3]);a=p.parse_args();print(json.dumps(review(a.repo),indent=2,default=str))
