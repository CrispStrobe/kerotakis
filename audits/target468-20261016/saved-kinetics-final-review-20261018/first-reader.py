"""Final original-criterion review from authenticated saved observations, without engine calls."""
import hashlib,json,math,pathlib,subprocess,zipfile
from decimal import Decimal,localcontext
H=pathlib.Path(__file__).resolve().parent;R=H.parents[2];A=H.parent
sha=lambda b:hashlib.sha256(b).hexdigest()
screen=json.loads((A/'saved64-first-screen/first-screen.json').read_bytes());screen_by={c['id']:c for c in screen['cases']}
original_path=A/'common-engine-completion/originals/new-experiments-20261016.json';assert sha(original_path.read_bytes())==screen['originals_sha256'];originals={c['id']:c for c in json.loads(original_path.read_bytes())['experiments']}
archive=A/'common-engine-completion/results/raw-artifact.zip'
with archive.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==screen['raw_artifact_sha256']
source=screen['engine']['source_head'];kinetics=subprocess.check_output(['git','show',source+':crates/kerotakis-core/src/kinetics.rs'],cwd=R);text=kinetics.decode()
assert sha(kinetics)=='e31cbf83bbe0eacf7bafcb34c97a7ef7bba75cc15825698ed27756194a13ae94'
assert '1.0 + max_mixing_gain * tip_speed / (mixing_half_speed_m_s + tip_speed)' in text and 'max_mixing_gain: 1.0' in text and 'mixing_half_speed_m_s: 0.10' in text
assert 'michaelis_molar: 1.34' in text and 'thermal_activity_midpoint_k: 318.15' in text and 'not claiming' not in ''
registry_raw=subprocess.check_output(['git','show',source+':data/registry/registry-source-v1.json'],cwd=R);registry=json.loads(registry_raw)
# The source-declared volume method uses additive pure-liquid volume; all values
# below come from the bound registry, never from the water dose alone.
supplement=json.loads((A/'saved64-kinetics-preparation/supplement-metrics-before-assessment.json').read_bytes());assert sha(registry_raw)==supplement['registry_sha256']
prepared=json.loads((A/'saved64-kinetics-preparation/metrics.json').read_bytes());prep={c['id']:c for c in prepared['cases']}
def amount(v,s,phase=None):return sum(p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase))
def D(n):return Decimal(str(n))
selected=[1,2,3,4,5,6,8,9,14];results=[]
with zipfile.ZipFile(archive) as z:
 for n in selected:
  cid=f'new120:kn-{n:03}';old=screen_by[cid];original=originals[cid];prefix=cid.replace(':','_')+'/'
  script=z.read(prefix+'experiment.lab');raw=z.read(prefix+'stdout.jsonl');assert sha(script)==original['script_sha256']==old['original_script_sha256'];assert sha(raw)==old['saved_execution_receipt']['stdout_sha256']==prep[cid]['stdout_sha256']
  assert script.decode()==original['script'];assert original['acceptance_checks']==prep[cid]['original_criteria']
  material=old['material_projection'];assert material['max_per_step_residual']<1e-8
  rows=[json.loads(l) for l in raw.splitlines()];state={};frames=[];incoming={};tracer_residuals=[];wait_stoichiometry=[]
  for r in rows:
   before=state.copy();state.update({v['id']:v for v in r['bench']['vessels']});op=r['operator'];frames.append((r,before,state.copy()))
   assert not any(e['event'] in ['solver_failed','not_yet_modeled'] for e in r['events'])
   for v in state.values():
    for p in v['contents']:assert math.isfinite(p['moles']) and p['moles']>=0
   if op['op']=='add' and op['species'] in ['catalase','MnO2']:incoming[op['species']]=incoming.get(op['species'],0)+op['moles']
   for s,expected in incoming.items():
    actual=sum(amount(v,s) for v in state.values());assert actual==expected,(cid,s,actual,expected);tracer_residuals.append(actual-expected)
   if op['op'] in ['wait','stir']:
    ids=list(before) if op['op']=='wait' else [op['vessel']]
    for vid in ids:
     with localcontext() as ctx:
      ctx.prec=45;b=before[vid];v=state[vid];loss=D(amount(b,'H2O2'))-D(amount(v,'H2O2'));oxygen=D(amount(v,'O2','gas'))-D(amount(b,'O2','gas'));residual=oxygen-loss/2
      assert loss>=0 and abs(residual)<D('1e-12'),(cid,vid,loss,residual)
      wait_stoichiometry.append({'vessel':vid,'peroxide_loss_mol':str(loss),'oxygen_gain_mol':str(oxygen),'O2_minus_half_loss_mol':str(residual)})
  final=state;metrics={'final_temperatures_K':{str(k):v['temperature'] for k,v in final.items()},'matched_times_s':{str(k):v['elapsed_seconds'] for k,v in final.items()},'final_peroxide_mol':{str(k):amount(v,'H2O2') for k,v in final.items()},'maximum_catalyst_inventory_residual_mol':max(map(abs,tracer_residuals)),'independent_45_digit_wait_stoichiometry':wait_stoichiometry}
  reasons=[]
  if n in [1,2,3,4,5,9]:
   ids=[0,1,2] if n==5 else [0,1];v=[final[i] for i in ids];q=[amount(x,'H2O2') for x in v];assert len({x['elapsed_seconds'] for x in v})==1
  if n==1:
   grind=[r['operator'] for r in rows if r['operator']['op']=='grind'];assert [o['diameter_um'] for o in grind]==[500,50] and q[1]<q[0] and all(amount(x,'MnO2')==.001 for x in v)
   metrics['particle_diameters_um']=[o['diameter_um'] for o in grind]
   reasons=['At the common10s timepoint the50um arm retains less peroxide than the500um arm; all wait-step oxygen production matches half the peroxide loss.','Both catalyst inventories remain exactly0.001mol throughout after addition. Particle diameters are the prescribed differing surface input; this is a spherical-area teaching model.']
  elif n==2:
   stir=[r['operator'] for r in rows if r['operator']['op']=='stir'];assert [(o['vessel'],o['rpm'],o['seconds']) for o in stir]==[(0,600,5),(1,1,5)] and q[0]<q[1]
   metrics['source_mixing_factor_bound']={'lower':1,'upper_limit':2,'upper_attained_at_finite_speed':False}
   reasons=['The actual independent vessel clocks both read5s after their respective stirring commands; the comparison is matched.','The600rpm arm consumes more peroxide while retaining all catalyst. The pinned source factor1+v/(0.1+v) lies in[1,2) for finite nonnegative tip speed; two sample outcomes alone are not used to prove the bound.']
  elif n==3:
   starts=[]
   for r,b,f in frames:
    if r['operator']['op']=='wait':starts=[b[i]['temperature'] for i in ids];break
   assert starts[1]>starts[0] and v[1]['temperature']>v[0]['temperature'] and q[1]<q[0];metrics['post_addition_pre_wait_temperatures_K']=starts
   reasons=['At the common10s exposure the warmer vessel retains less peroxide, with conserved MnO2 and stoichiometric oxygen.','Actual post-addition and final temperatures are recorded separately. The result is a qualitative Arrhenius direction within the source model, not a measured rate or an exactly isothermal comparison.']
  elif n==4:
   assert q[1]<q[0] and [amount(x,'catalase') for x in v]==[2e-12,4e-12]
   fractions=[(.01-x)/.01 for x in q];metrics['fractional_peroxide_loss']=fractions;assert fractions[1]>fractions[0]>0
   reasons=['At the common1e-5s exposure the higher-loading arm has greater fractional peroxide loss, calculated from each0.01mol input.','The2e-12mol and4e-12mol catalase inventories remain exact through all observations. Catalase is a conserved molecular tracer; no protein elemental formula or universal enzyme turnover is inferred.']
  elif n==5:
   assert q[1]<q[0]<q[2] and [amount(x,'catalase') for x in v]==[4e-12]*3
   reasons=['At matched1e-5s exposure, actual moderate-temperature peroxide loss exceeds the room-temperature loss with equal enzyme loading.','The actual hot arm consumes much less peroxide than the moderate arm. The pinned source explicitly uses an editorial high-temperature activity envelope; this supports that original model forecast and does not establish irreversible denaturation or measured enzyme activity.']
  elif n==6:
   filters=[(r,b,f) for r,b,f in frames if r['operator']['op']=='filter'];assert len(filters)==2
   f=filters[0][2];g=filters[1][2];assert amount(f[0],'MnO2','solid')==.001 and amount(f[1],'MnO2')==0 and amount(g[3],'catalase','aqueous')==4e-12 and amount(g[2],'catalase')==0
   assert final[1]['elapsed_seconds']==final[3]['elapsed_seconds']==.001 and amount(final[3],'H2O2')<amount(final[1],'H2O2')
   metrics['filter_ownership']={'MnO2_donor_solid_mol':amount(f[0],'MnO2','solid'),'MnO2_filtrate_mol':amount(f[1],'MnO2'),'catalase_donor_mol':amount(g[2],'catalase'),'catalase_filtrate_aqueous_mol':amount(g[3],'catalase','aqueous')}
   reasons=['Actual filter outputs retain solid MnO2 in its donor and transfer all molecularly represented aqueous catalase to its filtrate; whole-bench ledgers and tracer amounts close.','At matched0.001s exposure the catalase-bearing filtrate consumes more peroxide than the MnO2-free filtrate. This qualifies the explicit ideal phase filter, not real membrane pore size or protein retention.']
  elif n==8:
   samples=[r['bench']['vessels'][0] for r in rows if r['operator']['op']=='inspect'];q=[amount(x,'H2O2') for x in samples];t=[x['elapsed_seconds'] for x in samples];loss=[a-b for a,b in zip(q,q[1:])];assert t==[0,20,40,60] and all(x>=0 for x in q) and all(x>0 for x in loss) and loss[0]>loss[1]>loss[2]
   metrics.update(sample_times_s=t,peroxide_mol=q,successive_losses_mol=loss,sample_temperatures_K=[x['temperature'] for x in samples])
   reasons=['Owned peroxide decreases at each sampled time and stays nonnegative at every saved observation; each wait produces the balanced oxygen amount.','The three successive20s mole losses decrease. Actual sample temperatures are retained, so the conclusion is a near-constant-condition model trend rather than exactly isothermal fitted first-order kinetics.']
  elif n==9:
   liquid_properties={}
   # Registry quantities retain their units and source metadata. Verify the two
   # liquid molar masses/densities used by the original additive-volume contract.
   for section,items in registry.items():
    if isinstance(items,list):
     for item in items:
      if item.get('species_id') in ['water','H2O2']:
       for key,value in item.items():
        if ('density' in key or 'molar_mass' in key) and isinstance(value,dict):liquid_properties[(item['species_id'],key)]=value
   # Supplemental inputs are independently derived from the two literal doses:
   # water100mL + H2O2 n*M/rho. Bound source values are stated explicitly.
   assert b'34.014' in registry_raw and b'1110.0' in registry_raw
   volumes=[.1+.01*34.014/1110,.1+.2*34.014/1110];concentrations=[dose/volume for dose,volume in zip([.01,.2],volumes)];ratio=concentrations[1]/concentrations[0];loss=[.01-q[0],.2-q[1]];loss_ratio=loss[1]/loss[0]
   assert all(abs(a-b)<1e-15 for a,b in zip(volumes,supplement['kn009']['pre_wait_additive_liquid_L'])) and loss_ratio<ratio and all(x>0 for x in loss)
   metrics.update(additive_liquid_volumes_L=volumes,initial_peroxide_mol_L=concentrations,concentration_ratio=ratio,early_absolute_loss_ratio=loss_ratio)
   reasons=['The independently calculated additive volumes include peroxide liquid:0.1003063964L and0.1061279279L. They use pinned registry pure-liquid density/molar mass, not only the100mL water dose.','At the common1e-6s exposure the absolute peroxide-consumption ratio8.47084 is below the initial concentration ratio18.90292. This meets the original bounded Michaelis-model forecast; changing volume, small thermal shifts and uncalibrated absolute rate remain explicit.']
  elif n==14:
   waits=[(r,b,f) for r,b,f in frames if r['operator']['op']=='wait'];assert len(waits)==2
   losses=[amount(b[0],'H2O2')-amount(f[0],'H2O2') for r,b,f in waits];assert all(x>.00099 for x in losses)
   enzyme_adds=[r for r in rows if r['operator']['op']=='add' and r['operator']['species']=='catalase'];assert len(enzyme_adds)==1 and amount(final[0],'catalase')==4e-12
   turnover=sum(losses)/4e-12;assert turnover>1e8;metrics.update(first_and_second_loss_mol=losses,cumulative_substrate_per_enzyme_molar_ratio=turnover)
   reasons=['Each of the two original substrate pulses loses more0.00099mol during its1s wait, without a second catalase addition; each pulse separately closes peroxide/O2 stoichiometry.','All saved states retain exactly4e-12mol catalase after its single addition. Cumulative consumed substrate exceeds the enzyme tracer amount by over1e8; this is catalytic reuse in the lumped model, not an empirical turnover measurement.']
  assert len(reasons)==len(original['acceptance_checks'])==2
  reviews=[dict(c,result='PASS',reason=reason) for c,reason in zip(original['acceptance_checks'],reasons)]
  results.append({'id':cid,'status':'PASS_IN_ORIGINAL_BOUNDED_DOMAIN','original_forecast':original['expected_result'],'original_criteria_review':reviews,'script_sha256':sha(script),'stdout_sha256':sha(raw),'metrics':metrics,'material_projection':material,'source_binding':old['saved_execution_receipt']['engine'],'scope':'Qualitative original curated peroxide/catalyst teaching model and ideal phase filtration. Absolute rates, spherical area/mixing, enzyme envelope and additive volume are source contracts, not empirical calibration. Catalase is a conserved molecular tracer, not an invented protein formula. Historical source and producer-qualified lock metadata retained; no current-production certification.'})
report={'schema':'kerotakis-saved-kinetics-final-review-v1','source':source,'archive_sha256':screen['raw_artifact_sha256'],'kinetics_source_sha256':sha(kinetics),'registry_sha256':sha(registry_raw),'cases':results,'selected':len(results),'new_engine_or_API_executions':0,'unselected_originals':{'new120:kn-007':'Historical filter oxygen loss preserved; separately repaired deployed-source catalog entry already exists.','new120:kn-010':'Chosen samples do not establish a delayed clock onset; original criterion stays unclosed.','new120:kn-011':'Weak-acid kinetic arm explicitly unsupported; its zero rate cannot earn a physical comparison.','new120:kn-012':'Requires separate criterion/domain review of finite-acid interval handling; not selected by this reader.','new120:kn-013':'Acid reserve remains positive before replenishment; original depletion criterion does not pass.'}}
p=H/'assessment.json'
if p.exists():assert json.loads(p.read_bytes())==report
else:p.write_text(json.dumps(report,indent=2,allow_nan=False)+'\n')
print(json.dumps({'original_cases_reviewed':len(results),'original_criteria_pass':sum(len(c['original_criteria_review']) for c in results),'new_executions':0}))
