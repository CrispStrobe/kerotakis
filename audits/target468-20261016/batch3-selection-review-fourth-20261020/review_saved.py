#!/usr/bin/env python3
"""Eight disjoint historical saved cases. No engine, network, catalog or source edits."""
import collections,hashlib,inspect,json,math,os,pathlib,re,runpy,zipfile
OUT=pathlib.Path(__file__).resolve().parent;ROOT=OUT.parents[2]
PRIOR=ROOT/'audits/target468-20261016/batch3-selection-review-third-20261020/review_saved.py'
a=runpy.run_path(str(PRIOR));sha=a['sha'];source=a['source'];REG=a['REG'];RECIPES=a['RECIPES'];MASSES=a['MASSES'];close=a['close'];dump=a['dump'];SOURCE=a['SOURCE'];BASE=a['BASE'];COMPOSITION=a['COMPOSITION'];CHARGE=a['CHARGE'];R=a['R'];vecadd=a['vecadd'];atoms=a['atoms'];events=a['events'];measures=a['measures']
SELECT={'en-006','en-008','po-007','co-004','fo-004','su-002','su-003','su-005'}
# Each of these events has an explicit ownership role reviewed below. No gas
# export/inlet event is silently admitted; only operator-owned adds/sealed air.
ALLOWED={'added','material_added','vessel_created','solution_routed','solution_characterized','observed','measured','neutralised','dissolved','temperature_changed','energy_transferred','enzyme_hydrolysed','stirred','material_layers_formed','vessel_sealed','precipitated','headspace_equilibrated','gas_contained','hazard_warning','not_yet_modeled','gravity_settled','reaction_occurred','inert','curdling_changed','corroded','reacted'}
ledger_source=inspect.getsource(a['ledger'])
ledger_source=ledger_source.replace('expected=collections.defaultdict', 'failed=[];expected=collections.defaultdict',1)
ledger_source=ledger_source.replace("assert close(actual[e],expected[vi][e]),(i,vi,e,actual[e],expected[vi][e])", "if not close(actual[e],expected[vi][e]):failed.append({'row_index':i,'raw_jsonl_line':row['raw_jsonl_line'],'vessel':vi,'coordinate':e,'actual':actual[e],'expected':expected[vi][e],'residual':actual[e]-expected[vi][e]})")
ledger_source=ledger_source.replace("return {'domain':","return {'strict_balance_failures':failed,'domain':",1)
exec(ledger_source,globals())
def file_digest(path):
 h=hashlib.sha256()
 with path.open('rb') as stream:
  while chunk:=stream.read(262144):h.update(chunk)
 return h.hexdigest()
def amount(v,sp):return sum(p['moles'] for p in v['contents'] if p['species']==sp)
def pool(v,key):return next(p for p in v.get('unresolved_materials',[]) if p['recipe_id']==key)
def assess(k,rows,scenes,raw_rows):
 final=rows[-1]['bench']['vessels']; scene=scenes[-1]['scene']['vessels']; calculations={};hold=None
 assert all(math.isfinite(p['moles']) and p['moles']>=0 for r in rows for v in r['bench']['vessels'] for p in v['contents'])
 if k in {'en-006','en-008'}:
  enzyme_source=source('crates/kerotakis-core/src/enzyme.rs').decode();activity_source=source('crates/kerotakis-core/src/enzyme_activity.rs').decode()
  assert '(-0.5 * z * z).exp()' in activity_source and '1.0 - (1.0 - before) * (-exponent).exp()' in activity_source
  predictions=[]
  for line,e in events(rows,'enzyme_hydrolysed'):
   i=next(i for i,r in enumerate(rows) if r['raw_jsonl_line']==line);before=rows[i-1]['bench']['vessels'][e['vessel']];after=rows[i]['bench']['vessels'][e['vessel']]
   family=e['family'];profile=re.search(r'family: EnzymeFamily::'+family.title()+r',.*?ph_width: ([0-9.]+),',enzyme_source,re.S).group(0)
   fields={key:float(re.search(key+r': ([0-9.]+)',profile).group(1)) for key in ['optimum_temperature_k','temperature_width_k','optimum_ph','ph_width']}
   recipe={'lipase':'household/vegetable-oil-surrogate','lactase':'food/whole-milk-surrogate','protease':'food/gelatin-dry-sheets'}[family]
   # Use the actual owned material name to identify its immutable recipe; table names vary across source eras.
   old=next(p for p in before['unresolved_materials'] if (family=='lipase' and 'vegetable-oil' in p['recipe_id']) or (family=='lactase' and 'milk' in p['recipe_id']) or (family=='protease' and 'gelatin' in p['recipe_id']))
   new=pool(after,old['recipe_id']);share=0.048/0.1220149 if family=='lactase' else 1.;substrate=old['amount']*share
   dose=amount(before,family)*{'lipase':33000.,'lactase':120000.,'protease':30000.}[family]
   t=before['temperature'];ph=before.get('solution',{}).get('ph');tf=math.exp(-.5*((t-fields['optimum_temperature_k'])/fields['temperature_width_k'])**2);pf=1 if ph is None else math.exp(-.5*((ph-fields['optimum_ph'])/fields['ph_width'])**2)
   oldf=old.get('enzyme_hydrolysis',{}).get('converted_fraction',0);prediction=1-(1-oldf)*math.exp(-.01*dose/substrate*tf*pf*e['seconds'])
   assert close(prediction,e['converted_fraction']) and close((prediction-oldf)*substrate,e['hydrolysed_mass_g']);assert close(new['enzyme_hydrolysis']['converted_fraction'],prediction)
   predictions.append({'raw_jsonl_line':line,'vessel':e['vessel'],'family':family,'actual_temperature_k':t,'actual_prior_ph':ph,'substrate_g':substrate,'enzyme_g':dose,'seconds':e['seconds'],'independent_fraction':prediction,'independent_annotation_mass_g':(prediction-oldf)*substrate})
  calculations['editorial_enzyme_model']=predictions
  if k=='en-006':
   assert len(predictions)==2 and all(p['vessel']==0 for p in predictions);assert all(close(v['elapsed_seconds'],2720) for v in final)
   assert scene[0]['enzyme_hydrolysis'][0]['family']=='lipase' and not scene[1].get('enzyme_hydrolysis') and not scene[2].get('enzyme_hydrolysis')
   assert close(amount(final[1],'cellulose'),1/MASSES['cellulose']);assert not any(p.get('enzyme_hydrolysis') for v in final[1:] for p in v.get('unresolved_materials',[]))
   note='Lipase changes only the retained oil aggregate annotation, with wrong-substrate cellulose and oil-without-enzyme controls. Independent prior-temperature/pH/dose/time reconstruction agrees with each state and persistent scene. No glycerol, fatty-acid products, acidity yield, interfacial kinetics or empirical enzyme calibration is inferred.'
  else:
   assert len(predictions)==4 and all(close(v['elapsed_seconds'],2700) for v in final)
   assert [len(v.get('enzyme_hydrolysis',[])) for v in scene]==[1,1,2]
   assert {x['family'] for x in scene[2]['enzyme_hydrolysis']}=={'lactase','protease'}
   hold='Unexplained strict charge residual in combined arm; valid separate aggregate annotations do not waive the all-step inventory criterion.'
   note='Separate milk lactose share (1.44 g) and gelatin protein share (0.45 g), not total milk mass, own the distinct annotations. Single-family controls and combined state retain identities; no exact molecular digestion products, competition or complete milk proteome model.'
 elif k=='co-004':
  src=source('crates/kerotakis-core/src/curdling.rs').decode();assert 'acid_moles' in src
  e=events(rows,'curdling_changed');assert len(e)==1;line,event=e[0];milk=next(p for p in final[0]['unresolved_materials'] if 'milk'in p['recipe_id']);role=next(r for r in RECIPES[(milk['recipe_id'],milk['recipe_version'])]['roles'] if 'curd' in r['kind']);calculations['pinned_curdling_role']=role
  progress=max(0,min(1,(.003/milk['amount']-role['onset_moles_per_gram'])/(role['full_moles_per_gram']-role['onset_moles_per_gram'])));fraction=progress*role['max_curdled_fraction'];assert close(event['to_formed_fraction'],fraction) and close(event['curd_solids_mass_g'],milk['amount']*fraction)
  curds=[v.get('curds') for v in scene];assert curds[0] and not curds[1] and not curds[2];calculations['actual_persistent_curd_state']=curds
  ph=measures(rows,'ph_meter');assert ph[0][1]['value']<ph[1][1]['value'];assert all(v['elapsed_seconds']==0 for v in final)
  calculations['opacity']=[v['liquid']['cloudiness'] for v in scene];calculations['curd_dry_aggregate_g']=event['curd_solids_mass_g'];calculations['acid_mol_per_unresolved_milk_g']=.003/milk['amount']
  note='Only acid arm owns persistent aggregate separation; diluted and untouched controls have no curd state. Source acid-family dose per unresolved milk gram predicts saturated 28% aggregate (not wet-curd yield or casein isoelectric calculation). pH and balance are separate actual measurements, not the curd trigger; opacity alone does not classify coagulation.'
 elif k=='po-007':
  before=next(r for r in rows if r['operator'].get('species')=='starch')['operator']['moles'];converted=amount(final[0],'maltose');assert close(converted,before/2) and amount(final[0],'starch')==0;assert close(amount(final[1],'cellulose'),next(r for r in rows if r['operator'].get('species')=='cellulose')['operator']['moles'])
  assert all(close(v['elapsed_seconds'],1800) for v in final);assert len(events(rows,'reaction_occurred'))==1
  calculations={'starch_repeat_unit_input_mol':before,'retained_maltose_mol':converted,'stoichiometric_water_consumed_mol':converted,'control_cellulose_mol':amount(final[1],'cellulose'),'reaction_raw_line':events(rows,'reaction_occurred')[0][0]}
  assert all(e['value']==0 for _,e in measures(rows,'spectrophotometer'));assert close(amount(final[1],'water')-amount(final[0],'water'),converted)
  note='Actual starch depletion/maltose formation and stoichiometric water debit distinguish cellulose, despite identical repeat-unit elemental composition. Reaction occurs at enzyme addition, not by a calibrated 30-minute kinetic law. Both iodine measurements read zero at405nm; unsupported KI mapping and particle glyph/speciation displays are not molecular inventory or a universal cellulose assay.'
 elif k=='fo-004':
  dye_added=sum(c['moles'] for _,e in events(rows,'material_added') if e['vessel']==0 for c in e['components'] if c['species']=='indigo_carmine');assert dye_added>0 and close(amount(final[0],'indigo_carmine'),dye_added);assert any('acrylic'in p['recipe_id'] and close(p['amount'],.275) for p in final[1]['unresolved_materials']);assert not final[0].get('unresolved_materials')
  readings=measures(rows,'spectrophotometer');assert readings[0][1]['value']>0 and readings[0][1]['unit']=='absorbance at 605 nm' and readings[1][1]['value']==0
  assert scene[0]['liquid']['srgb']!=scene[1]['liquid']['srgb'];assert all(close(v['elapsed_seconds'],20) for v in final)
  calculations={'dye_retained_mol':amount(final[0],'indigo_carmine'),'opaque_binder_pigment_retained_g':.275,'actual_rgb_after_dilution':[v['liquid']['srgb'] for v in scene],'actual_spectrophotometry':readings}
  note='Equal formulation doses retain dissolved indigo versus an unresolved pigment/binder pool through equal dilution. Persistent colour states differ. The opaque formulation zero at405nm is only absence of an installed molecular-spectrum contribution, not measured transparency or Beer-Lambert behaviour; no pigment sedimentation or binder film law is inferred.'
 elif k in {'su-002','su-003'}:
  before=next(r for r in reversed(rows) if r['operator']['op']=='add' and r['operator'].get('species')=='O2')['bench']['vessels'];reaction=events(rows,'reacted');assert len(reaction)==1;line,e=reaction[0];assert e['reaction']=='iron-corrosion' and e['vessel']==1
  x=e['moles'];assert close(amount(before[0],'Fe'),amount(before[1],'Fe'));assert close(amount(final[0],'Fe'),amount(before[0],'Fe'));assert close(amount(before[1],'Fe')-amount(final[1],'Fe'),4*x);assert close(amount(final[1],'Fe2O3'),2*x);assert close(amount(before[1],'O2')-amount(final[1],'O2'),3*x)
  assert not amount(final[0],'Fe2O3');assert all(close(v['elapsed_seconds'],172800 if k=='su-002' else 259200) for v in final)
  material=final[0]['unresolved_materials'][0];barrier='metal/painted-iron' if k=='su-002' else 'metal/stainless-steel';assert material['recipe_id']==barrier;assert ('"material recipe '+barrier+'"') in source('crates/kerotakis-core/src/corrosion.rs').decode()
  calculations={'matched_initial_Fe_mol':amount(before[0],'Fe'),'actual_iron_consumed_mol':4*x,'actual_oxide_formed_mol':2*x,'actual_contained_O2_consumed_mol':3*x,'retained_coating_or_alloy_pool_g':material['amount'],'reaction_raw_line':line,'actual_temperatures_k':[v['temperature'] for v in final]}
  if k=='su-003':
   readings=measures(rows,'conductivity_meter');assert close(readings[0][1]['value'],readings[1][1]['value']);calculations['equal_electrolyte_conductivity']=readings
  note='Matched Fe/water/salt/air/O2 and clocks isolate a declared provenance-owned accessibility barrier from bare-iron oxidation. Independent4Fe:3O2:2Fe2O3 state changes agree with actual extent; unresolved coating/alloy remains, air and retained gas are fully counted. This is the original bounded barrier question, not computed chromium passivation, coating defects, pitting immunity, surface-area matching or calibrated corrosion rate.'
 else:
  readings=measures(rows,'conductivity_meter');assert [e['vessel'] for _,e in readings]==[0,2] and all(e['unit']=='S/m' and e['value']>0 for _,e in readings)
  refusals=[(l,e) for l,e in events(rows,'not_yet_modeled') if e.get('reason',{}).get('key')=='not-modeled.conductivity-on-a-dry-solid'];assert [e['vessel'] for _,e in refusals]==[1,3]
  assert close(amount(final[0],'graphite'),amount(final[1],'diamond'));assert all(v['elapsed_seconds']==0 for v in final)
  calculations={'actual_solid_conductivity':readings,'actual_missing_resistivity_refusals':refusals,'actual_density':measures(rows,'densitometer')};hold='Diamond and polyethylene have no installed reviewed solid resistivity; missing readings cannot complete original graphite/diamond/metal/polymer electrical comparison.'
  note='Shared carbon inventory and distinct density are established, but do not supply the missing diamond electrical descriptor. Particle glyphs encode elemental inventory, not bonding topology or electronic structure. Unsupported measurements are not zero conductivity.'
 warnings=[{'raw_jsonl_line':l,'event':e} for l,e in events(rows,'not_yet_modeled')]
 return {'status':'hold' if hold else 'proposal_pending_parent_review','hold_reason':hold,'model_scope':note,'independent_calculations':calculations,'coverage_warnings':warnings,'energy_scope':'All actual temperatures and clocks retained per step. No calorimetric or empirical kinetic forecast is present; no unmeasured energy or yield asserted.','actual_auxiliary_reservoirs':[{'raw_jsonl_line':r['raw_jsonl_line'],'vessels':[{'id':v['id'],'pending_co2_transfer_mol':v.get('pending_co2_transfer_mol'),'free_proton':v.get('free_proton'),'free_hydroxide':v.get('free_hydroxide'),'solute_charge':v.get('solute_charge'),'excess_enthalpy_j':v.get('excess_enthalpy_j'),'thermal_mode':v.get('thermal_mode'),'headspace':v.get('headspace')}for v in r['bench']['vessels']]}for r in rows],'raw_nonbench_observations':[{'raw_jsonl_line':i+1,'record':r} for i,r in enumerate(raw_rows) if 'bench' not in r and 'scene' not in r]}
def main():
 memory=int(next(line.split()[1] for line in pathlib.Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemAvailable:')))//1024
 assert memory>=3200 and os.getloadavg()[0]<=3.5,'Saved archive review resource gate closed'
 archive=BASE/'native-screen-results/raw-artifact.zip';verification=json.loads((BASE/'native-screen-results/verification.json').read_text());assert file_digest(archive)==verification['artifact_sha256']
 binding=json.loads((BASE/'native-screen-results/binding.json').read_text());intake=json.loads((BASE/'intake.json').read_text());index=[]
 with zipfile.ZipFile(archive) as z:
  for k in sorted(SELECT):
   oid='batch3-diverse:'+k;original=next(x for x in intake if x['original_id']==oid);prefix=oid.replace(':','_');summary=json.loads((BASE/'native-screen-results/cases'/f'{prefix}.json').read_text());execution=json.loads(z.read(prefix+'/execution.json'));assert summary['engine']==binding['engine'];hashes={}
   for name,key in [('experiment.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
    raw=z.read(prefix+'/'+name);digest=sha(raw);assert digest==execution[key]==summary['execution'][key];hashes[name]=digest
   assert z.read(prefix+'/experiment.lab')==(BASE/original['original_script']).read_bytes();assert hashes['experiment.lab']==original['script_sha256'];assert execution['exit_code']==0 and not summary['solver_failures']
   raw_rows=[json.loads(line) for line in z.read(prefix+'/stdout.jsonl').splitlines()];rows=[dict(row,raw_jsonl_line=i+1) for i,row in enumerate(raw_rows) if 'bench'in row];scenes=[{'raw_jsonl_line':i+1,'scene':row['scene']} for i,row in enumerate(raw_rows) if 'scene'in row]
   inventory=ledger(k,rows);assessment=assess(k,rows,scenes,raw_rows)
   if inventory['strict_balance_failures']:assert k=='en-008' and assessment['status']=='hold'
   inventory['domain']='Every operator and every vessel/step: pinned recipe additions, all resolved atoms/charge, exact formula-free enzyme carriers, unresolved material pools, sealed dry air, retained gases, actual clocks and temperatures. Unknown/export events rejected; no duplicate free-ion counting or invented aggregate formulas.'
   report={'original_id':oid,'title':original['title'],'original_forecast':original['original_expected_result'],'original_limitations':original['original_limitations'],'original_script_sha256':original['script_sha256'],'raw_member_prefix':prefix,'raw_sha256':hashes,'engine':binding['engine'],'lock_qualification':binding['lock_qualification'],'status':assessment['status'],'assessment':assessment,'per_criterion':[dict(c,assessment=('hold' if (c['id']=='inventory' and inventory['strict_balance_failures']) or (k=='su-005' and c['id'] in {'question','controls'}) else 'supported_with_stated_model_scope'),evidence={'inventory':'inventory.trace','question':'assessment.independent_calculations','coverage':'assessment.model_scope and assessment.coverage_warnings','controls':'assessment.independent_calculations and final_scene'}[c['id']]) for c in original['original_acceptance_checks']],'no_catalog_selection':True,'original_criteria':original['original_acceptance_checks'],'inventory':inventory,'actual_events':[{'raw_jsonl_line':row['raw_jsonl_line'],'operator':row['operator'],'events':row['events']} for row in rows],'final_scene':scenes[-1] if scenes else None}
   dump(OUT/(k+'.json'),report);index.append({'original_id':oid,'assessment':k+'.json','status':assessment['status']})
 dump(OUT/'source-binding.json',{'source_commit':SOURCE,'source_files_sha256':{path:sha(source(path)) for path in ['data/registry/registry-source-v1.json','crates/kerotakis-core/src/enzyme.rs','crates/kerotakis-core/src/enzyme_activity.rs','crates/kerotakis-core/src/curdling.rs','crates/kerotakis-core/src/corrosion.rs','crates/kerotakis-core/src/kinetics.rs']},'raw_archive_sha256':file_digest(archive),'actual_lock_bytes_available':False,'lock_qualification':binding['lock_qualification'],'reader_chain_sha256':{str(path.relative_to(ROOT)):file_digest(path) for path in [pathlib.Path(__file__),PRIOR,ROOT/'audits/target468-20261016/batch3-selection-review-second-20261020/review_saved.py',ROOT/'audits/target468-20261016/batch3-selection-review-20261020/review_saved.py']}})
 dump(OUT/'candidate-index.json',index);print(json.dumps({'saved_inventory_reports':len(index),'no_engine_execution':True}))
if __name__=='__main__':main()
