#!/usr/bin/env python3
"""Fourth independent twelve-original saved review; no engine, network or selections."""
import hashlib,json,math,subprocess,zipfile
from decimal import Decimal,localcontext
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];BASE=ROOT/'audits/new-experiments-201-20261016/native-screen-results';D=Decimal;Z=D(0);TOL=D('1e-8');sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes(),parse_float=D)
IDS=('ab-027','ab-034','pq-006','pq-007','ec-003','ec-004','ec-007','ec-010','ec-012','qa-005','qa-006','se-003')
def amount(v,s,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase)),Z)
def snap(r):return {v['id']:v for v in r['bench']['vessels']}
def activity(v,s):return next(p['activity'] for p in v['solution']['species'] if p['name']==s)
def sensible(row,phase,a,b):
 sets=[x for x in row['heat_capacity_polys'] if x['phase']==('liquid' if phase=='aqueous' else phase)]
 if not sets:return row['heat_capacity']*(b-a)
 def cp(T):
  intervals=sets[0]['intervals'];T=max(intervals[0]['t_min'],min(T,intervals[-1]['t_max']));q=next(x for x in intervals if x['t_min']<=T<=x['t_max']);assert q['form']=='nasa9';v=q['coefficients'];return 8.31446261815324*(v[0]/T**2+v[1]/T+v[2]+v[3]*T+v[4]*T**2+v[5]*T**3+v[6]*T**4)
 N=400;h=(b-a)/N;return h/3*sum((1 if i in (0,N) else 4 if i%2 else 2)*cp(a+i*h) for i in range(N+1))
def review():
 ar=BASE/'raw-artifact.zip';ardigest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert ardigest==load(BASE/'artifact-metadata.json')['digest'].removeprefix('sha256:');binding=load(BASE/'binding.json');source=binding['engine']['source_head'];assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip()==binding['engine']['source_tree'];show=lambda path:subprocess.check_output(['git','show',source+':'+path],cwd=ROOT)
 goldraw=show('crates/kerotakis-core/tests/golden/registry.json');gold={r['key']:r for r in json.loads(goldraw)};assert gold['water']['molar_mass']==18.015 and gold['water']['density']==.997
 regraw=show('data/registry/registry-source-v1.json');reg=json.loads(regraw,parse_float=D);comps={x['species_id']:x for x in reg['compositions']};intake={r['original_id']:r for r in load(BASE.parent/'intake.json')};selected={r['original_target_id'] for r in load(ROOT/'audits/target468-20261016/selection-partition-beneficiaries-20261020/catalog-membership.json')['mapping']}
 def composition(s):
  if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
  r=comps[s];return {**{e['element']:e['count']['value'] for e in r['elements']},'charge':r['net_charge']['value']}
 def book(v,s,n):
  for key,c in composition(s).items():v[key]=v.get(key,Z)+n*c
 def ledger(v,phase=None):
  total={}
  for p in v['contents']:
   assert p['moles'].is_finite() and p['moles']>=0
   if phase is None or p['phase']==phase:book(total,p['species'],p['moles'])
  return total
 def elem(v,e,phase=None):return ledger(v,phase).get(e,Z)
 output=[]
 with localcontext() as context,zipfile.ZipFile(ar) as zipped:
  context.prec=45
  for suffix in IDS:
   key='batch2_'+suffix;case=load(BASE/'cases'/f'{key}.json');original=intake[case['id']];assert case['id'] not in selected;assert case['engine']==binding['engine'];assert case['original_acceptance_checks']==original['original_acceptance_checks'] and case['original_physical_forecast']==original['original_expected_result'];script=zipped.read(key+'/experiment.lab');raw=zipped.read(key+'/stdout.jsonl');stderr=zipped.read(key+'/stderr.txt');assert script==(BASE.parent/original['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(stderr)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0
   rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x.strip()];expected={};state={};perstep=[];maxima={};exports=[];titrants=[];frames=[];boundaries=[]
   for i,row in enumerate(rows):
    op=row['operator'];typ=op['op'];before=state.copy();events=row['events'];published=snap(row)
    if typ=='add':book(expected.setdefault(op['vessel'],{}),op['species'],op['moles'])
    elif typ=='seal':
     v=before[op['vessel']];n=Z if v['headspace']['boundary']!='open' else D(101325)*op['headspace_volume']/(D('8314.462618')*v['temperature']);boundaries.append({'row':i,'vessel':op['vessel'],'air_mol':n})
     for s,f in [('N2',D('.7901')),('O2',D('.2095')),('CO2',D('.0004'))]:book(expected[op['vessel']],s,n*f)
    elif typ=='dilute':book(expected[op['vessel']],'water',op['volume']*D('997')/D('18.015'))
    elif typ=='filter':
     frm,to=op['from'],op['to'];expected.setdefault(to,{})
     for part in before[frm]['contents']:
      if part['phase'] in ('aqueous','liquid'):book(expected[frm],part['species'],-part['moles']);book(expected[to],part['species'],part['moles'])
    elif typ=='titrate':
     event,=[e for e in events if e['event']=='titrated'];assert event['endpoint_reached'];dose=event['total_volume']*event['concentration'];book(expected[op['vessel']],op['titrant'],dose);book(expected[op['vessel']],'water',event['total_volume']*D(997)/D('18.015'));titrants.append({'row':i,'solute_mol':dose,'volume_L':event['total_volume'],'final_pH':event['final_ph'],'step_L':op['step'],'carrier_water_mol':event['total_volume']*D(997)/D('18.015')})
    else:assert typ in ('new_vessel','measure','inspect','cell','electrolyse','heat','cool'),typ
    for e in events:
     assert e['event']!='solver_failed'
     if e['event'] in ('gas_evolved','gas_absorbed'):book(expected[e['vessel']],e['species'],e['moles']*(D(-1) if e['event']=='gas_evolved' else D(1)));exports.append({'row':i,**e})
    if set(before)-set(published):assert typ in ('inspect','measure');assert all(v==before[k] for k,v in published.items())
    if typ in ('cell','measure','inspect') and before:assert all(v['elapsed_seconds']==before[k]['elapsed_seconds'] for k,v in published.items() if k in before)
    state.update(published)
    for vid,v in state.items():
     assert not v.get('unresolved_materials') and not v.get('surfaces');actual=ledger(v);ext=expected.setdefault(vid,{});residual={k:actual.get(k,Z)-ext.get(k,Z) for k in actual.keys()|ext.keys()}
     for k,d in residual.items():maxima[k]=max(maxima.get(k,Z),abs(d))
     perstep.append({'row':i,'vessel':vid,'residuals':residual})
    frames.append((row,before,state.copy()))
   assert all(d<=TOL for d in maxima.values()),(suffix,maxima)
   events=[e for r in rows for e in r['events']];criteria=[True,True];metrics={};notes=[];hold=None
   if suffix=='ab-027':
    measures=[(r,snap(r)[0]) for r in rows if r['operator']['op']=='measure'];ph=[e['value'] for e in events if e['event']=='measured' and e['instrument']=='ph_meter'];cond=[e['value'] for e in events if e['event']=='measured' and e['instrument']=='conductivity_meter'];assert all(abs(amount(v,'CH3COOH')+amount(v,'CH3COO-')-D('.002'))<=D('1e-12') for r,v in measures);assert len(ph)==4 and len(cond)==3 and cond[0]<cond[1]<cond[2];assert max(abs(ph[i]-ph[0]) for i in (2,3))<D('.02');assert abs(amount(state[0],'Na+')-D('.0018'))<=TOL and abs(amount(state[0],'Cl-')-D('.0008'))<=TOL;metrics={'actual_pH':ph,'conductivity_uS_cm':cond,'final_Na_mol':amount(state[0],'Na+'),'final_Cl_mol':amount(state[0],'Cl-'),'actual_temperatures_K':[v['temperature'] for r,v in measures]};notes=['Acetatepoolfixedandcompleteionsaccounted; actualtemperature/ionicstrengthchange retained. SimilarpHdoesnotmakeconductivityequal.']
   elif suffix=='ab-034':
    vals=[]
    for vid,v in state.items():
     species=[s for s in v['solution']['species'] if 'PO4' in s['name']];vals.append({'vessel':vid,'pH':v['solution']['ph'],'T_K':v['temperature'],'total_P_mol':elem(v,'P'),'solved_phosphate_species':species,'canonical_phosphate_mol':{s:amount(v,s) for s in ['H3PO4','H2PO4-','HPO4-2','PO4-3']}});assert abs(elem(v,'P')-D('.001'))<=TOL
    assert amount(state[0],'H2PO4-')>D('.0009') and amount(state[1],'HPO4-2')>D('.0009') and amount(state[2],'PO4-3')>amount(state[1],'PO4-3');assert vals[0]['pH']<vals[1]['pH']<vals[2]['pH'];metrics={'arms':vals};notes=['Actualsolvedspeciationandtemperaturesretained; notallneutralisedsaltsareassignedpH7.']
   elif suffix=='pq-006':
    vals=[{'vessel':vid,'Ca_solid_mol':elem(v,'Ca','solid'),'Ca_aqueous_mol':elem(v,'Ca','aqueous'),'Mg_solid_mol':elem(v,'Mg','solid'),'Mg_aqueous_mol':elem(v,'Mg','aqueous')} for vid,v in state.items()];assert vals[0]['Mg_solid_mol']>D('.0004') and vals[1]['Ca_solid_mol']>D('.0004');assert vals[1]['Mg_solid_mol']>0;metrics={'fraction_elemental_budgets':vals};notes=['SecondfractionhasrealMgcarryover; donorMgsolidandsecondCaCO3areidentified, notcalledquantitativelypure. Finalsolutiontraceprecipitatesareincluded.']
   elif suffix=='pq-007':
    first=[elem(state[v],'Ag') for v in (2,3)];wash=[elem(state[v],'Ag') for v in (4,5)];retained=[elem(state[v],'Ag') for v in (0,1)];assert wash[1]<wash[0];assert all(abs(first[i]+wash[i]+retained[i]-D('.0001'))<=TOL for i in (0,1));metrics={'first_filtrate_Ag_mol':first,'second_wash_Ag_mol':wash,'retained_Ag_mol':retained,'total_recovery_fractions':[r/D('.0001') for r in retained],'actual_temperature_K':[state[v]['temperature'] for v in (0,1,4,5)]};notes=['Includesinitialfiltrationlossandfinitecommonionwashloss, notonlylastsolidmass. Noassertionofperfectretention.']
   elif suffix in ('ec-003','ec-004'):
    cells=[]
    for row,before,after in frames:
     if row['operator']['op']!='cell':continue
     assert before==after;pred={}
     for vid,v in after.items():
      if suffix=='ec-003':ion,e0,n=('Zn+2',-.7618,2) if vid==0 else ('Cu+2',.3419,2)
      else:ion,e0,n='Ag+',.7996,1
      act=float(activity(v,ion));pred[vid]=e0+8.31446261815324*float(v['temperature'])/(n*96485.33212)*math.log(act)
     cell_events=[e for e in row['events'] if e['event']=='cell_voltage'];rec={'actualT_K':[v['temperature'] for v in after.values()],'independent_signed_b_minus_a_V':pred[1]-pred[0],'actual_events':row['events'],'free_ion_activities':[activity(v,'Zn+2' if suffix=='ec-003' and vid==0 else 'Cu+2' if suffix=='ec-003' else 'Ag+') for vid,v in after.items()]}
     for e in cell_events:assert abs(pred[e['cathode']]-pred[e['anode']]-float(e['volts']))<1e-12
     cells.append(rec)
    metrics={'cell_states':cells}
    if suffix=='ec-003':assert len(cells)==2 and cells[0]['independent_signed_b_minus_a_V']!=cells[1]['independent_signed_b_minus_a_V'];notes=['Dissolvedactivities/complexation/precipitatesandactualtemperaturesretained. NamedslowZnwatercorrosionlimitirrelevanttoinstantaneousnonconsumingNernstcomparison.']
    else:assert amount(state[1],'AgCl','solid')>0 and activity(state[1],'Ag+')<activity(state[0],'Ag+');criteria=[True,False];hold='electrode-low-free-ion-observability';notes=['PostprecipitationfreeAg+ispositivebutmodelreturnsno_cellinstead ofavoltage. IndependentNernstpotentialisnotasubstitutefororiginalmissinginstrumentobservable. Do nottreatbelowdisplaythresholdasphysicalzeroions.']
   elif suffix=='ec-007':
    before=next(before[0] for row,before,after in frames if row['operator']['op']=='add' and row['operator']['species']=='Zn');v=state[0];ag=amount(v,'Ag','solid');zn=D('.0003')-amount(v,'Zn','solid');assert ag>0 and zn>0 and abs(ag-2*zn)<=D('1e-12');assert amount(v,'AgCl','solid')<amount(before,'AgCl','solid');assert abs(elem(v,'Ag')-D('.0004'))<=TOL;metrics={'formed_Ag_mol':ag,'oxidized_Zn_mol':zn,'remaining_AgCl_mol':amount(v,'AgCl','solid'),'initial_AgCl_solid_mol':amount(before,'AgCl','solid'),'AgCl_solid_loss_mol':amount(before,'AgCl','solid')-amount(v,'AgCl','solid'),'initial_dissolved_Ag_mol':amount(before,'Ag+'),'final_dissolved_Ag_mol':amount(v,'Ag+'),'final_total_Ag_mol':elem(v,'Ag')};notes=['Partialcoupledsolubility/redoxextentonly; notcompleteAgClreductionoraconversionratemodel. ExcessZnremainingnotmislabellednoreaction.']
   elif suffix=='ec-010':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');isolated=after[0];assert amount(isolated,'Zn','solid')==0;beforeAg=next(before[0] for row,before,after in frames if row['operator']['op']=='add' and row['operator']['species']=='AgNO3');cu_loss=amount(beforeAg,'Cu','solid')-amount(state[0],'Cu','solid');ag=amount(state[0],'Ag','solid');assert cu_loss>0 and abs(ag-2*cu_loss)<=D('1e-10');metrics={'isolated_solid_contents':isolated['contents'],'copper_loss_in_Ag_step_mol':cu_loss,'formed_Ag_mol':ag,'Ag_to_Cu_ratio':ag/cu_loss,'actual_temperature_K':[beforeAg['temperature'],state[0]['temperature']]};notes=['RecoveredCuismetallicdonor; residualZnexplicitlyabsent. Sulfate/mineralcontaminationretainedincompleteledger, notassumedpureCu.']
   elif suffix=='ec-012':
    vals=[]
    for row,before,after in frames:
     if row['operator']['op']!='electrolyse':continue
     op=row['operator'];v=after[0];b=before[0];q=op['amps']*op['seconds'];deltaCu=amount(v,'Cu','solid')-amount(b,'Cu','solid');deltaH2=amount(v,'H2','gas')-amount(b,'H2','gas');deltaO2=amount(v,'O2','gas')-amount(b,'O2','gas');electrons=q/D('96485.33212');assert abs(2*(deltaCu+deltaH2)-electrons)<=D('1e-10');assert abs(4*deltaO2-electrons)<=D('1e-10');assert amount(v,'Cu','solid')<=D('.00005')+TOL;vals.append({'charge_C':q,'Cu_increment_mol':deltaCu,'H2_increment_mol':deltaH2,'O2_increment_mol':deltaO2,'total_Cu_metal_mol':amount(v,'Cu','solid'),'electrons_mol':electrons,'bench_elapsed_seconds_before':b['elapsed_seconds'],'bench_elapsed_seconds_after':v['elapsed_seconds']})
    assert len(vals)==2 and vals[1]['H2_increment_mol']>0;metrics={'charge_intervals':vals};notes=['ExcesscathodicchargeproducesactualH2afterfiniteCuinventoryisdepleted. InitialairO2subtracted; electronbudgetsclosebothcathodeandanode, notjustreportedcurrentefficiency.']
   elif suffix=='qa-005':
    assert len(titrants)==2;p=titrants[0]['solute_mol'];t=p+titrants[1]['solute_mol'];oh=2*p-t;co3=t-p;criteria=[p>t/2,abs(oh-D('.0004'))<=D('.00002') and abs(co3-D('.0003'))<=D('.00002')];metrics={'actual_titrant_stages':titrants,'P_mol':p,'T_mol':t,'reconstructed_OH_mol':oh,'reconstructed_CO3_mol':co3};hold='original-open-carbonate-endpoint-reconstruction';notes=['CumulativeTdoseincludesbothstages. Original0.4/0.3mmolreconstructionfails; preserveopenCO2exportsratherthanforceidealendpointformula.']
   elif suffix=='qa-006':
    tit,=titrants;v=state[0];before=next(before[0] for row,before,after in frames if row['operator']['op']=='titrate');spent=amount(before,'H2C2O4')-amount(v,'H2C2O4');mn=amount(v,'Mn+2');remaining=amount(v,'MnO4-');assert spent>0 and abs(spent-D('.001'))<=TOL and remaining>0;assert abs(mn*D('2.5')-spent)<=TOL;assert abs(mn+remaining-tit['solute_mol'])<=TOL;assert tit['volume_L']-tit['step_L']-D('1e-12')<=D('.020')<=tit['volume_L']+D('1e-12');criteria=[True,True];metrics={'titrant':tit,'previous_increment_volume_L':tit['volume_L']-tit['step_L'],'oxalic_acid_consumed_mol':spent,'Mn2_product_mol':mn,'persistent_permanganate_mol':remaining,'exported_CO2_mol':sum(e['moles'] for e in exports),'actual_T_K':[before['temperature'],v['temperature']]};notes=['20.0–20.2mLendpointbracketfromactual0.2mLstep; finitepersistentoxidantandoxalatedepletionverifiedindependently. Actualnonisothermalteachingendpoint, notinductionkineticsorcalibratedtitre.']
   elif suffix=='se-003':
    vals=[]
    for row,before,after in frames:
     if row['operator']['op']=='filter':
      frm,to=row['operator']['from'],row['operator']['to'];vals.append({'source_before':before[frm]['contents'],'retained_after':after[frm]['contents'],'filtrate_after':after[to]['contents'],'actual_T_K':before[frm]['temperature'],'crystal_KNO3_mol':amount(after[frm],'KNO3','solid')})
    assert len(vals)==2 and all(v['crystal_KNO3_mol']==0 for v in vals);criteria=[True,False];hold='original-crystallisation-thermal-protocol';energies=[]
    for row,before,after in frames:
     if row['operator']['op'] not in ('heat','cool'):continue
     v=row['operator']['vessel'];q=sum(float(part['moles'])*sensible(gold[part['species']],part['phase'],float(before[v]['temperature']),float(after[v]['temperature'])) for part in before[v]['contents']);requested=float(row['operator']['energy'])*(1 if row['operator']['op']=='heat' else -1);assert abs(q-requested)<.001;energies.append({'operator':row['operator'],'independent_sensible_inventory_energy_J':q,'signed_requested_energy_J':requested})
    metrics={'filtrations':vals,'independent_thermal_controls':energies,'actual_energy_events':[e for e in events if e['event']=='energy_transferred']};notes=['Actualcoolingends316.408Kandcontainsnocrystrals. Originalrecoverydemonstrationnotestablished; noassumedKNO3crystalformationfromtemperaturechangealone. Insufficientcoolingversussourcecoverage requiresseparatesolubilitydomainreview.']
   output.append({'id':case['id'],'proposed_candidate':all(criteria),'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],criteria,strict=True)],'metrics':metrics,'limitations':notes,'shared_gap_or_protocol_hold':hold,'named_model_limits':case['named_model_limits'],'source':case['engine'],'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(stderr),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt'],'per_step_atom_charge':perstep,'maxima':maxima,'boundary_exports':exports,'titrant_carrier':titrants,'initial_air':boundaries})
 return {'schema':'kerotakis.batch2.fourth-saved-review.v1','cases':output,'raw_archive_sha256':ardigest,'registry_sha256':sha(regraw),'thermal_fixture_sha256':sha(goldraw),'source_blobs_sha256':{p:sha(show(p)) for p in ['crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/displacement.rs','crates/kerotakis-core/src/relations.rs','crates/kerotakis-core/src/states.rs','crates/kerotakis-core/src/heat_capacity.rs','crates/kerotakis-core/src/ops.rs']},'original_lock_qualification':binding['lock_qualification'],'new_engine_or_HTTP_requests':0,'catalog_selection_assigned':False,'material_tolerance_mol':TOL}
if __name__=='__main__':print(json.dumps(review(),indent=2,default=str))
