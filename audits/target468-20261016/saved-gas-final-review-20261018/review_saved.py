"""Read original saved responses; never executes Kerotakis or changes forecasts."""
import hashlib,json,pathlib,subprocess,zipfile
HERE=pathlib.Path(__file__).resolve().parent;ROOT=HERE.parents[2];AUDIT=HERE.parent
sha=lambda x:hashlib.sha256(x).hexdigest()
screen=json.loads((AUDIT/'saved64-first-screen/first-screen.json').read_text());prepared={c['id']:c for c in screen['cases']};archive=AUDIT/'common-engine-completion/results/raw-artifact.zip'
with archive.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==screen['raw_artifact_sha256']
source=screen['engine']['source_head'];vessel_source=subprocess.check_output(['git','show',source+':crates/kerotakis-core/src/vessel.rs'],cwd=ROOT);assert b'const R_LITRE_PASCAL: f64 = 8_314.462_618;' in vessel_source;R=8314.462618
carbon_names={'CO2','CO2(aq)','HCO3-','CO3-2'}
def ctotal(v):return sum(p['moles'] for p in v['contents'] if p['species'] in carbon_names)
def gas(v,name=None):return sum(p['moles'] for p in v['contents'] if p['phase']=='gas' and (name is None or p['species']==name))
def state(rows,row,vid):return next(v for v in rows[row]['bench']['vessels'] if v['id']==vid)
def partial(v):return gas(v,'CO2')*R*v['temperature']/v['headspace']['volume']
def aqc(v):return sum(p['moles'] for p in v['contents'] if p['species'] in carbon_names and p['phase']!='gas')
results=[]
with zipfile.ZipFile(archive) as z:
 for cid in ['new120:gs-001','new120:gs-003','new120:gs-009']:
  original=prepared[cid];prefix=cid.replace(':','_')+'/';script=z.read(prefix+'experiment.lab');raw=z.read(prefix+'stdout.jsonl');assert sha(script)==original['original_script_sha256'];assert sha(raw)==original['saved_execution_receipt']['stdout_sha256'];rows=[json.loads(l) for l in raw.splitlines()]
  assert original['material_projection']['max_per_step_residual']<1e-8
  assert not any(e['event'] in ['solver_failed','not_yet_modeled'] for r in rows for e in r['events'])
  pressure_residuals=[]
  for row in rows:
   for v in row['bench']['vessels']:
    vol=v['headspace'].get('volume')
    if vol:pressure_residuals.append(v['pressure']-gas(v)*R*v['temperature']/vol)
  reasons=[]
  if cid.endswith('001'):
   a,b,c=[state(rows,i,0) for i in [2,4,6]];deltas=[b['pressure']-a['pressure'],c['pressure']-b['pressure']];predicted=[(gas(a)+rows[3]['operator']['moles'])*R*b['temperature']/b['headspace']['volume'],(gas(b)+rows[5]['operator']['moles'])*R*c['temperature']/c['headspace']['volume']]
   assert a['pressure']>1e5 and all(x>0 for x in deltas)
   molar_residuals=[(actual['pressure']-expected)*actual['headspace']['volume']/(R*actual['temperature']) for actual,expected in zip([b,c],predicted)]
   assert all(abs(residual)<1e-8 for residual in molar_residuals)
   metrics={'sealed_air_baseline_Pa':a['pressure'],'added_pressure_increments_Pa':deltas,'dose_and_actual_temperature_predicted_pressures_Pa':predicted,'actual_pressures_Pa':[b['pressure'],c['pressure']],'inferred_dose_molar_residuals':molar_residuals}
   reasons=['The saved sealed-air baseline is101324.630104Pa; the two positive increments are2478.957599Pa each rather than pressures measured from vacuum.','Independent dose-plus-baseline nRT/V calculations use each actual temperature and1L headspace and agree within the existing molar inventory resolution; both exact pressure residuals are retained. This selects ideal finite-carrier additivity, without a quantitative N2/O2 dissolution claim.']
  elif cid.endswith('003'):
   neutral=state(rows,4,0);alkaline=state(rows,11,1);base_neutral=state(rows,1,0);base_alkaline=state(rows,8,1)
   arms=[]
   for before,after in [(base_neutral,neutral),(base_alkaline,alkaline)]:
    residual=ctotal(after)-ctotal(before)-.001;assert abs(residual)<1e-8
    arms.append({'temperature_K':after['temperature'],'aqueous_carbon_mol':aqc(after),'gas_CO2_mol':gas(after,'CO2'),'aqueous_carbon_fraction':aqc(after)/ctotal(after),'CO2_partial_pressure_Pa':partial(after),'closed_CO2_addition_residual_mol':residual})
   assert arms[1]['aqueous_carbon_fraction']>arms[0]['aqueous_carbon_fraction'] and arms[1]['CO2_partial_pressure_Pa']<arms[0]['CO2_partial_pressure_Pa']
   metrics={'neutral_and_alkaline_arms':arms}
   reasons=['Owned aqueous/gas carbon fractions and CO2 partial pressures are independently calculated at both recorded temperatures; the warmer alkaline arm retains more carbon in solution and has lower CO2 partial pressure.','Both finite CO2 additions close against their own trapped-air baseline. Whole-bench atoms/charge also close at the existing1e-8mol/eq audit resolution; decreased pressure is not carbon deletion.']
  else:
   a=state(rows,3,0);b=state(rows,9,0);assert a['pressure']==b['pressure']==100000 and b['headspace']['volume']>a['headspace']['volume'];assert abs(ctotal(b)-ctotal(a))<1e-8;assert aqc(b)<aqc(a) and partial(b)<partial(a)
   metrics={'before_after_temperature_K':[a['temperature'],b['temperature']],'before_after_pressure_Pa':[a['pressure'],b['pressure']],'before_after_headspace_L':[a['headspace']['volume'],b['headspace']['volume']],'before_after_CO2_partial_pressure_Pa':[partial(a),partial(b)],'before_after_aqueous_carbon_mol':[aqc(a),aqc(b)],'carbon_change_mol':ctotal(b)-ctotal(a)}
   reasons=['The actual setpoint remains100000Pa while finite headspace expands from0.521314636L to0.645852822L after the nitrogen dose.','Owned total carbon remains unchanged while dissolved carbon and actual-temperature CO2 partial pressure decrease. Regulated expansion is treated as materially closed; no vent is inferred.']
  criteria=[{'id':r['id'],'criterion':r['criterion'],'result':'PASS','reason':reason} for r,reason in zip(original['original_criteria'],reasons)];assert len(criteria)==len(original['original_criteria'])==2
  results.append({'id':cid,'status':'PASS_IN_ORIGINAL_BOUNDED_DOMAIN','original_forecast':original['original_forecast'],'original_criteria_review':criteria,'script_sha256':sha(script),'stdout_sha256':sha(raw),'metrics':metrics,'maximum_absolute_pressure_identity_residual_Pa':max(map(abs,pressure_residuals)),'material_projection':original['material_projection'],'source_binding':original['saved_execution_receipt']['engine'],'scope':'Original finite ideal-carrier/aqueous carbonate teaching domain. No quantitative N2/O2 solubility, real-gas/high-pressure, empirical partition calibration, reaction heat or latest-production claim.'})
report={'schema':'kerotakis-saved-gas-final-review-v1','source':source,'archive_sha256':screen['raw_artifact_sha256'],'source_vessel_sha256':sha(vessel_source),'cases':results,'selected':3,'new_engine_or_API_executions':0,'GS008':'Not selected by this review: its explicit nonideal-correction criterion needs its own final domain review; no criterion is silently dropped.','GS010':'Remains excluded: original and current brine conservation failure; PR833 requires separate combined-source and deployed acceptance.'}
expected=HERE/'assessment.json'
if expected.exists():assert json.loads(expected.read_text())==report
else:expected.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'original_cases_reviewed':len(results),'original_criteria_pass':sum(len(c['original_criteria_review']) for c in results),'new_executions':0}))
