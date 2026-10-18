"""Independent Decimal material ledger and original GS010 domain review; no engine calls."""
import decimal,hashlib,json,pathlib,subprocess
B=pathlib.Path(__file__).resolve().parent;R=B.parents[2];G=B/'gas';D=lambda n:decimal.Decimal(str(n));decimal.getcontext().prec=45
sha=lambda b:hashlib.sha256(b).hexdigest()
design=json.loads((G/'design.json').read_bytes());props=json.loads((G/'properties.json').read_bytes());raw=(G/'responses/new120-gs-010.json').read_bytes();response=json.loads(raw);S=design['accepted_engine']['source_head'];assert S=='d3883757ef12433561cb7831a9c9ce3f1b999eef'
assert response['engine']==design['accepted_engine'] and response['script_sha256']==design['cases'][0]['script_sha256']==sha(design['cases'][0]['script'].encode());assert response['exit_status']==0
for path,h in props['property_source_sha256'].items():assert sha(subprocess.check_output(['git','show',S+':'+path],cwd=R))==h,path
vendor=subprocess.check_output(['git','ls-tree',S,'vendor/iphreeqc'],cwd=R,text=True).split()[2];database=subprocess.check_output(['git','show',vendor+':database/pitzer.dat'],cwd=R/'vendor/iphreeqc');db=database.decode('latin1')
assert '-LAMBDA' in db and 'CO2       Na+       0.085' in db and 'Cl-       CO2      -0.005' in db
species=props['species'];expected={};state={};all_residuals=[];additions=[]
def book(ledger,key,n):
 p=species[key]
 for element,count in {**p['elements'],'charge':p['charge']}.items():ledger[element]=ledger.get(element,D(0))+D(n)*D(count)
def ledger(vessels):
 out={}
 for v in vessels:
  for p in v['contents']:
   assert D(p['moles']).is_finite() and D(p['moles'])>=0;book(out,p['species'],p['moles'])
 return out
def residual(after,before,delta):return {k:after.get(k,D(0))-before.get(k,D(0))-delta.get(k,D(0)) for k in set(after)|set(before)|set(delta)}
def amount(v,key,phase=None):return sum((D(p['moles']) for p in v['contents'] if p['species']==key and (phase is None or p['phase']==phase)),D(0))
for index,row in enumerate(response['observations']):
 before=state.copy();op=row['operator'];state.update({v['id']:v for v in row['bench']['vessels']});assert not any(e['event']=='solver_failed' for e in row['events'])
 if op['op']=='add':book(expected,op['species'],op['moles'])
 for e in row['events']:
  if e['event']=='vessel_sealed':
   for key,fraction in [('N2','0.7901'),('O2','0.2095'),('CO2','0.0004')]:book(expected,key,D(e['trapped_air'])*D(fraction))
  assert e['event'] not in ['vented','gas_exchanged','vessel_burst'],'No unaccounted external material boundary admitted'
 actual=ledger(state.values());whole=residual(actual,{},expected);assert all(abs(n)<D('1e-8') for n in whole.values()),(index,whole);all_residuals.append({'row':index,'operator':op['op'],'residual_mol_or_eq':{k:str(v) for k,v in whole.items()}})
 if op['op']=='add' and op['species']=='CO2':
  vid=op['vessel'];v=state[vid];delta={};book(delta,'CO2',op['moles']);r=residual(ledger([v]),ledger([before[vid]]),delta);assert all(abs(n)<D('1e-8') for n in r.values())
  solution=v['solution'];provenance=solution['provenance'];additions.append({'vessel':vid,'temperature_K':v['temperature'],'ionic_strength_mol_kgw':solution['ionic_strength'],'dataset':provenance['dataset'],'activity_model':provenance['model'],'neutral_CO2_aq_mol':str(amount(v,'CO2(aq)','aqueous')),'bicarbonate_mol':str(amount(v,'HCO3-')),'carbonate_mol':str(amount(v,'CO3-2')),'gas_CO2_mol':str(amount(v,'CO2','gas')),'carbon_baseline_mol':str(ledger([before[vid]])['C']),'addition_residual_mol_or_eq':{k:str(v) for k,v in r.items()}})
assert len(additions)==2 and D(additions[1]['neutral_CO2_aq_mol'])<D(additions[0]['neutral_CO2_aq_mol']);assert 'pitzer.dat' in additions[1]['dataset'] and 'Pitzer' in additions[1]['activity_model']
assert all(290<a['temperature_K']<305 for a in additions)
maximum=max(abs(D(n)) for row in all_residuals for n in row['residual_mol_or_eq'].values());maximum_add=max(abs(D(n)) for a in additions for n in a['addition_residual_mol_or_eq'].values())
original=design['cases'][0];reviews=[dict(original['original_criteria'][0],result='PASS',reason='Actual owned neutral dissolved CO2, bicarbonate, carbonate and gas carbon are separately counted. Each addition and every whole-bench timepoint closes the represented atom/charge ledger including trapped air.'),dict(original['original_criteria'][1],result='PASS',reason='The actual brine arm uses Pitzer, whose pinned native database includes neutral CO2/Na+ and CO2/Cl- LAMBDA interactions. The supported model shows less neutral CO2 in brine; no missing salinity correction is invented. Actual temperatures and distinct activity routes are retained, so this is a qualitative model demonstration rather than an isothermal or empirical salinity calibration.')]
result={'source':S,'engine':response['engine'],'response_sha256':sha(raw),'original_id':original['id'],'original_script_sha256':original['script_sha256'],'original_forecast':original['original_forecast'],'original_criteria_review':reviews,'decimal_precision':45,'all_timepoint_material_residuals':all_residuals,'max_whole_material_charge_residual_mol_or_eq':str(maximum),'max_CO2_addition_material_charge_residual_mol_or_eq':str(maximum_add),'arms':additions,'native_database_submodule_commit':vendor,'native_pitzer_database_sha256':sha(database),'native_salinity_interactions':{'CO2_Na_LAMBDA':'0.085','CO2_Cl_LAMBDA':'-0.005'},'new_engine_or_API_requests':0,'selection':'Original bounded qualitative neutral-CO2 salinity comparison and finite inventory closure. The0.931K arm temperature difference and different dilute/concentrated routes preclude an isolated quantitative salinity coefficient or empirical calibration claim.'}
p=B/'ROOT-INDEPENDENT-REVIEW.json'
if p.exists():assert json.loads(p.read_bytes())==result
else:p.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'original_criteria_pass':2,'maximum_whole_residual':str(maximum),'maximum_CO2_addition_residual':str(maximum_add),'new_requests':0}))
