#!/usr/bin/env python3
"""Twelve unchanged saved batch2 criteria; no engine invocations or selections."""
import hashlib,importlib.util,json,math,subprocess,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
SPEC=importlib.util.spec_from_file_location('second',HERE.parent/'batch2-selection-review-second-20261020/review_saved.py');S=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(S)
H=S.H;ROOT=H.ROOT;BASE=H.BASE;sha=H.sha;amount=H.amounts;snapshot=S.snapshot;carbon=S.carbon
H.A.update({'NH4Cl':{'N':1,'H':4,'Cl':1},'Ca(OH)2':{'Ca':1,'O':2,'H':2},'glucose':{'C':6,'H':12,'O':6},'ethanol':{'C':2,'H':6,'O':1},'KMnO4':{'K':1,'Mn':1,'O':4},'K+':{'K':1},'MnO4-':{'Mn':1,'O':4},'hexane':{'C':6,'H':14}});H.Q.update({'K+':1,'MnO4-':-1});H.KEYS[:]=sorted({k for d in H.A.values() for k in d})+['charge']
IDS=('gs-012','gs-014','gs-015','gs-016','gs-017','gs-018','ki-001','ki-002','ki-006','ki-012','se-009','se-011')
def gas(v,sp=None):return sum(p['moles'] for p in v['contents'] if p['phase']=='gas' and (sp is None or p['species']==sp))
def readback(v):return {'vessel':v['id'],'T_K':v['temperature'],'pH':v['solution']['ph'] if v.get('solution') else None,'C_mol':carbon(v),'CO2_gas_mol':gas(v,'CO2'),'CO2_aqueous_mol':amount(v,'CO2(aq)'),'bicarbonate_mol':amount(v,'HCO3-'),'carbonate_mol':amount(v,'CO3-2')}
def review():
 ar=BASE/'raw-artifact.zip';digest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert digest==json.loads((BASE/'artifact-metadata.json').read_text())['digest'].removeprefix('sha256:');binding=json.loads((BASE/'binding.json').read_text());source=binding['engine']['source_head'];assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip()==binding['engine']['source_tree']
 reg_raw=subprocess.check_output(['git','show',source+':crates/kerotakis-core/tests/golden/registry.json'],cwd=ROOT);reg={x['key']:x for x in json.loads(reg_raw)}
 intake={r['original_id']:r for r in json.loads((BASE.parent/'intake.json').read_text())};selected={r['original_target_id'] for r in json.loads((ROOT/'audits/target468-20261016/selection-expansion-second-20261020/catalog-membership.json').read_text())['mapping']};out=[]
 with zipfile.ZipFile(ar) as z:
  for suffix in IDS:
   key='batch2_'+suffix;case=json.loads((BASE/'cases'/f'{key}.json').read_text());original=intake[case['id']];assert case['id'] not in selected;assert case['engine']==binding['engine'];assert case['original_acceptance_checks']==original['original_acceptance_checks'] and case['original_physical_forecast']==original['original_expected_result']
   script=z.read(key+'/experiment.lab');raw=z.read(key+'/stdout.jsonl');stderr=z.read(key+'/stderr.txt');assert script==(BASE.parent/original['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(stderr)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0
   rows=[json.loads(x) for x in raw.splitlines() if x.strip()];expected={};prior={};maxima=H.zero();exports=[];seals=[];ledger=[];clocks=[];missing=[];pressures=[]
   for ix,row in enumerate(rows):
    op=row['operator'];typ=op['op'];assert typ in ('add','new_vessel','seal','inspect','heat','cool','measure','filter','drain','grind','centrifuge','wait','react'),typ
    if typ=='add':H.book(expected.setdefault(op['vessel'],H.zero()),op['species'],op['moles'])
    elif typ=='seal':
     v=prior[op['vessel']];closed=v['headspace']['boundary'] in ('sealed','pressure_controlled')
     n=0 if closed else 101325*op['headspace_volume']/(8314.462618*v['temperature']);target=expected[op['vessel']]
     for sp,f in [('N2',.7901),('O2',.2095),('CO2',.0004)]:H.book(target,sp,n*f)
     seals.append({'row':ix,'vessel':op['vessel'],'already_closed':closed,'new_air_mol':n,'new_air_C_mol':n*.0004})
    elif typ in ('filter','drain'):
     frm,to=op['from'],op['to'];expected.setdefault(to,H.zero())
     for p in prior[frm]['contents']:
      transfer=p['phase'] in ('liquid','aqueous') if typ=='filter' else p['species']=='water' or p['phase']=='aqueous'
      if transfer:H.book(expected[frm],p['species'],-p['moles']);H.book(expected[to],p['species'],p['moles'])
     if typ=='filter':
      lost=[p for p in prior[frm]['contents'] if p['phase']=='gas']
      if lost:missing.append({'row':ix,'from_vessel':frm,'unreported_removed_gases':lost})
    for e in row['events']:
     assert e['event'] not in ('solver_failed','not_yet_modeled','gas_absorbed')
     if e['event']=='gas_evolved':H.book(expected[e['vessel']],e['species'],-e['moles']);exports.append({'row':ix,**e})
    current=snapshot(row)
    if set(prior)-set(current):assert typ=='inspect' and not row['events'];assert all(v==prior[k] for k,v in current.items())
    if typ=='wait':
     for vid,v in current.items():assert abs(v['elapsed_seconds']-prior[vid]['elapsed_seconds']-op['seconds'])<1e-12;clocks.append({'row':ix,'vessel':vid,'start':prior[vid]['elapsed_seconds'],'end':v['elapsed_seconds']})
    prior.update(current)
    for vid,v in prior.items():
     assert not v.get('unresolved_materials') and not v.get('surfaces');actual=H.vectors(v);exp=expected.setdefault(vid,H.zero());res={k:actual[k]-exp[k] for k in H.KEYS}
     for k,d in res.items():maxima[k]=max(maxima[k],abs(d))
     ledger.append({'row':ix,'vessel':vid,'max_abs_residual':max(map(abs,res.values())),'nonzero_over_tolerance':{k:d for k,d in res.items() if abs(d)>H.TOL}})
     if v['headspace']['boundary']=='sealed':
      ideal=gas(v)*8314.462618*v['temperature']/v['headspace']['volume'];assert abs(ideal-v['pressure'])<.002 or typ in ('add','new_vessel','filter'),(suffix,ix,ideal,v['pressure']);pressures.append({'row':ix,'vessel':vid,'ideal_gas_pressure_Pa':ideal,'recorded_pressure_Pa':v['pressure']})
   material_ok=max(maxima.values())<=H.TOL
   assert material_ok or suffix in ('gs-018','ki-002'),(suffix,maxima)
   ev=[e for r in rows for e in r['events']];met=[True,True];observed={};notes=[];gap=None
   if suffix=='gs-012':
    states=[snapshot(r)[0] for r in rows if r['operator']['op']=='add' and r['operator']['species']=='CO2'];vals=[readback(v) for v in states];assert len(vals)==3 and vals[0]['carbonate_mol']>vals[0]['CO2_gas_mol']*100 and vals[-1]['CO2_gas_mol']>vals[0]['CO2_gas_mol']*100 and vals[-1]['pH']<vals[0]['pH'];observed={'pulse_states':vals};notes=['Finitecarbon doses and carbonate/bicarbonate speciation retained; not a sharp2OH-perCO2 cutoff.']
   elif suffix=='gs-014':
    states=[snapshot(r)[0] for r in rows if r['operator']['op']=='inspect'];vals=[{'T_K':v['temperature'],'pH':v['solution']['ph'],'NH3_gas_mol':gas(v,'NH3'),'NH3_aqueous_mol':amount(v,'NH3','aqueous'),'NH4_mol':amount(v,'NH4+')} for v in states];assert vals[1]['NH3_gas_mol']>vals[0]['NH3_gas_mol'];met=[False,True];gap='historical-finite-ammonia-gas-partition';observed={'base_then_acid':vals};notes=['Acid correctly increasesammonium but actualNH3gas rises, contradicting originalgas-shareforecast. Laterammoniarepair maybenefit; no current-source failureclaimedwithoutfollowup.']
   elif suffix=='gs-015':
    states=[snapshot(r)[0] for r in rows if r['operator']['op']=='inspect'];vals=[readback(v) for v in states];assert len(vals)==3 and vals[1]['T_K']>vals[0]['T_K']+4 and vals[1]['CO2_gas_mol']>vals[0]['CO2_gas_mol'];assert abs(vals[2]['T_K']-vals[0]['T_K'])<1e-6 and abs(vals[2]['CO2_gas_mol']-vals[0]['CO2_gas_mol'])<1e-10;assert max(v['C_mol'] for v in vals)-min(v['C_mol'] for v in vals)<1e-10;observed={'cycle':vals};notes=['Actualreturnedtemperature and retainedcarbon checked. Approximatepartitioncycle, not empiricallycalibratedHenrylaw or reactionenthalpy.']
   elif suffix=='gs-016':
    vals=[readback(prior[v]) for v in (0,1)];assert vals[1]['CO2_gas_mol']<vals[0]['CO2_gas_mol'] and vals[1]['CO2_aqueous_mol']>vals[0]['CO2_aqueous_mol'];assert abs(vals[0]['C_mol']-vals[1]['C_mol'])<1e-9;assert all(s['new_air_mol']>0 for s in seals);observed={'arms':vals,'water_mol':[amount(prior[v],'water') for v in (0,1)],'headspace_L':[prior[v]['headspace']['volume'] for v in (0,1)]};notes=['Actualtemperature difference retained; compare gasCO2 and aqueous carbon, not onlytotalpressure. Both1mmol introduceddose and identicalheadspaceair separatelyaccounted.']
   elif suffix=='gs-017':
    i,r=next((i,r) for i,r in enumerate(rows) if r['operator']['op']=='react');before=snapshot(rows[i-1])[0];after=snapshot(r)[0];extent=amount(before,'glucose')-amount(after,'glucose');o=amount(before,'O2')-amount(after,'O2');c=carbon(after)-6*amount(after,'glucose')-(carbon(before)-6*amount(before,'glucose'));assert extent>0 and abs(o-6*extent)<1e-10 and abs(c-6*extent)<1e-10;assert abs((gas(after)-gas(before))-(gas(after,'CO2')-gas(before,'CO2')-o))<1e-10;observed={'glucose_consumed_mol':extent,'O2_consumed_mol':o,'inorganic_C_gain_mol':c,'gas_total_change_mol':gas(after)-gas(before),'gas_CO2_gain_mol':gas(after,'CO2')-gas(before,'CO2'),'before':readback(before),'after':readback(after)};notes=['Gas-onlychange differsfromformal6:6 becauseinorganiccarbonpartitionsintoliquid. CompleteglucoseC/H/O budgetchecked; no respirationkinetics/thermalcalibrationclaim.']
   elif suffix=='gs-018':
    i,r=next((i,r) for i,r in enumerate(rows) if r['operator']['op']=='filter');before=snapshot(rows[i-1])[0];after=snapshot(r)[0];captured=amount(before,'CaCO3','solid');assert captured>0 and abs(amount(after,'CaCO3','solid')-captured)<1e-12;assert abs(carbon(prior[0])-captured)<1e-8;met=[True,True];gap='historical-filter-dropped-gas-ownership';observed={'isolated_CaCO3_mol':captured,'acidified_retained_C_mol':carbon(prior[0]),'filter_silent_gas_removal':missing};notes=['Recoveredsolidcarbon isreal, butfilterdropssealedheadspacegaseswithoutboundaryexport andfullatomsfail. LaterKN007filterrepair mayalreadyaddress; requireschangedsourcequalification, nothistoricalpass.']
   elif suffix in ('ki-001','ki-002','ki-006','ki-012'):
    rates=[]
    for i,r in enumerate(rows):
     if r['operator']['op']!='wait':continue
     before=snapshot(rows[i-1])
     for vid,v in snapshot(r).items():
      initial=amount(before[vid],'H2O2');final=amount(v,'H2O2')
      if initial<=0:continue
      loss=initial-final;assert loss>0;assert abs((amount(v,'water')-amount(before[vid],'water'))-loss)<1e-9;assert abs((amount(v,'O2')-amount(before[vid],'O2'))-loss/2)<1e-9;rates.append({'vessel':vid,'row':i,'initial_H2O2_mol':initial,'final_H2O2_mol':final,'fraction_remaining':final/initial,'loss_mol':loss,'apparent_k_s-1':-math.log(final/initial)/r['operator']['seconds'],'start_T_K':before[vid]['temperature'],'end_T_K':v['temperature'],'MnO2_mol':amount(v,'MnO2')})
    observed={'intervals':rates}
    if suffix=='ki-001':assert rates[1]['apparent_k_s-1']>3*rates[0]['apparent_k_s-1'] and rates[2]['apparent_k_s-1']>3*rates[0]['apparent_k_s-1'];assert all(abs(r['MnO2_mol']-1.1502185415228894e-5)<1e-15 for r in rates[1:]);notes=['Compareactual30sintervalfractions andremainingperoxide; eventabsencebelowreportthreshold doesnotmeanzerodecomposition. Catalystretained, indicativeabsoluterateonly.']
    elif suffix=='ki-002':assert rates[-1]['vessel']==2 and rates[-1]['loss_mol']<rates[-2]['loss_mol'];met=[True,True];gap='historical-filter-dropped-gas-ownership';notes=['Filtrateinitialsubstrate andnonzerouncatalysedrate arecorrectlydistinguished, butunreportedfilterO2loss breaksfullatomaccounting; do notqualifybeforeownedboundaryrepair.']
    elif suffix=='ki-006':
     waits=sorted({r['row'] for r in rates})
     for row in waits:
      a,b=[r for r in rates if r['row']==row];assert abs(a['fraction_remaining']-b['fraction_remaining'])<1e-6 and abs(a['apparent_k_s-1']/b['apparent_k_s-1']-1)<1e-3 and abs(a['end_T_K']-b['end_T_K'])<.001
     notes=['Normaliseeachowninitialinventory andactualinterval; temperaturesremainwithin0.001K andcatalystmass/activitymatched. Logdecaytestedonlythisnearconstantteachingdomain.']
    else:
     i,r=next((i,r) for i,r in enumerate(rows) if r['operator']['op']=='filter');isolated=snapshot(r)[0];assert abs(amount(isolated,'MnO2','solid')-4e-5)<1e-15;assert amount(isolated,'MnO4-')==0;assert rates[0]['apparent_k_s-1']>10*rates[1]['apparent_k_s-1'];observed['isolated_MnO2_mol']=amount(isolated,'MnO2');observed['isolated_remaining_permanganate_mol']=amount(isolated,'MnO4-');notes=['ActualMnO2formation/isolation precedesreaction. Permanganateisabsentfromcatalystsolid; remainingacetate/K pools stayseparate, nocarryoverstoichiometricoxidantclaim.']
   elif suffix=='se-009':
    dry=[{'row':i,'vessel':v['id'],'water_mol':amount(v,'water'),'hexane_mol':amount(v,'hexane'),'aqueous_silica_mol':amount(v,'SiO2','aqueous')} for i,r in enumerate(rows) for v in r['bench']['vessels'] if amount(v,'water')==0 and amount(v,'hexane')>0 and amount(v,'SiO2','aqueous')>0];assert dry;met=[False,True];gap='historical-waterless-organic-aqueous-silica';observed={'final_fractions':[{'vessel':v['id'],'water_mol':amount(v,'water'),'hexane_mol':amount(v,'hexane'),'solid_SiO2_mol':amount(v,'SiO2','solid'),'aqueous_SiO2_mol':amount(v,'SiO2','aqueous'),'Na_mol':amount(v,'Na+'),'Cl_mol':amount(v,'Cl-')} for v in prior.values()],'dry_organic_unsupported_aqueous_silica':dry};notes=['GlobalSiclosure andcoarsefractionlabels do notvalidatewater-solubilitydissolutioninto nowdryhexane. Latermediumadmission mayrepair; nocurrentbugclaimuntilnewsourcecontrol.']
   elif suffix=='se-011':
    silica=prior[0];salt=prior[1];solid=amount(silica,'SiO2','solid');aq=amount(silica,'SiO2','aqueous');assert solid>0 and aq>0 and abs(solid+aq-.0016643365954330605)<1e-12;assert amount(salt,'NaCl','solid')==0 and abs(amount(salt,'Na+')-.0017110689047447943)<1e-10;lots=[l for l in silica['lots'] if l['species']=='SiO2' and l['phase']=='solid'];assert lots and all(l['suspended_fraction']==0 for l in lots);obs=[e for e in ev if e['event']=='observed'];assert all(e['appearance']['cloudiness']==0 for e in obs);observed={'solid_silica_mol':solid,'dissolved_silica_mol':aq,'suspended_solid_fraction':[l['suspended_fraction'] for l in lots],'dissolved_Na_mol':amount(salt,'Na+'),'dissolved_Cl_mol':amount(salt,'Cl-'),'actual_appearance':obs};notes=['Clearliquidabove retainedsediment versusdissolvedsalt; neitherclaritynorcloudinessreduction establishchemicaldisappearance.']
   out.append({'id':case['id'],'proposed_candidate':all(met) and material_ok and gap!='historical-waterless-organic-aqueous-silica','original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],met,strict=True)],'observed':observed,'material_budget_pass':material_ok,'max_per_vessel_per_step_atom_charge_residual_mol':maxima,'per_step_ledger':ledger,'clock_checks':clocks,'ideal_pressure_checks':pressures,'trapped_air':seals,'exports':exports,'limitations':notes,'shared_gap':gap,'source':case['engine'],'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(stderr),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt']})
 return {'schema':'kerotakis.batch2.third-saved-review.v1','cases':out,'raw_artifact_sha256':digest,'new_engine_executions':0,'selection_performed':False,'tolerance_mol':H.TOL,'lock_qualification':binding['lock_qualification'],'source_blobs_sha256':{p:sha(subprocess.check_output(['git','show',source+':'+p],cwd=ROOT)) for p in ['crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/clock.rs','crates/kerotakis-core/src/kinetics.rs','crates/kerotakis-core/src/displacement.rs','crates/kerotakis-core/tests/golden/registry.json']},'helper_sha256':{str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in [HERE.parent/'batch2-selection-review-20261020/review_saved.py',HERE.parent/'batch2-selection-review-second-20261020/review_saved.py']}}
if __name__=='__main__':print(json.dumps(review(),indent=2,allow_nan=False))
