"""Authenticate one original saved comparison and independently audit its ownership."""
from pathlib import Path
from decimal import Decimal, getcontext
import hashlib, json, math, re, subprocess, zipfile
getcontext().prec = 45
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]; A=HERE.parent
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
CID='new120:kn-012';SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83'
screen=load(A/'saved64-first-screen/first-screen.json')
originals=A/'common-engine-completion/originals/new-experiments-20261016.json'
assert sha(originals.read_bytes())==screen['originals_sha256']
c=next(c for c in load(originals)['experiments'] if c['id']==CID)
receipt=next(c for c in load(A/'common-engine-completion/results/verified-results.json')['cases'] if c['id']==CID)
assert sha(c['script'].encode())==c['script_sha256']==receipt['script_sha256']
archive=A/'common-engine-completion/results/raw-artifact.zip';assert sha(archive.read_bytes())==screen['raw_artifact_sha256']
with zipfile.ZipFile(archive) as z:
 raw=z.read('new120_kn-012/stdout.jsonl');err=z.read('new120_kn-012/stderr.txt')
assert sha(raw)==receipt['stdout_sha256'] and sha(err)==receipt['stderr_sha256'] and not err
rows=[json.loads(l) for l in raw.splitlines()]
D=lambda n:Decimal(str(n))
form={'water':({'H':2,'O':1},0),'HCl':({'H':1,'Cl':1},0),'Na2S2O3':({'Na':2,'S':2,'O':3},0),'Na+':({'Na':1},1),'Cl-':({'Cl':1},-1),'H+':({'H':1},1),'S2O3-2':({'S':2,'O':3},-2),'S':({'S':1},0),'SO2':({'S':1,'O':2},0)}
keys=['H','O','Cl','Na','S','charge']
def zero():return {k:Decimal(0) for k in keys}
def book(d,s,n):
 f,q=form[s]
 for e,k in f.items():d[e]+=D(n)*k
 d['charge']+=D(n)*q
incoming={};maxima={};per_step=[];prior={};initial=None;final=None
for i,r in enumerate(rows):
 op=r['operator'];assert op['op'] in ['add','new_vessel','wait','inspect']
 if op['op']=='add':book(incoming.setdefault(op['vessel'],zero()),op['species'],op['moles'])
 assert not any(e['event'] in ['gas_evolved','gas_vented','gas_absorbed','evaporated','diluted','solver_failed'] for e in r['events'])
 for v in r['bench']['vessels']:
  assert not v.get('unresolved_materials') and not v.get('surfaces')
  actual=zero()
  for p in v['contents']:assert p['moles']>=0;book(actual,p['species'],p['moles'])
  expected=incoming.get(v['id'],zero());res={k:actual[k]-expected[k] for k in keys};mx=max(abs(x) for x in res.values());assert mx<Decimal('1e-8'),(i,v['id'],res)
  maxima[v['id']]=max(maxima.get(v['id'],Decimal(0)),mx)
  per_step.append({'row':i,'vessel':v['id'],'maximum_absolute_mol_or_eq_residual':str(mx)})
 if op['op']=='wait':
  assert initial is None and op['seconds']==10
  initial=prior.copy();final={v['id']:v for v in r['bench']['vessels']}
 prior.update({v['id']:v for v in r['bench']['vessels']})
def amount(v,s):return sum(p['moles'] for p in v['contents'] if p['species']==s)
metrics=[]
for vid in [0,1]:
 b=initial[vid];f=final[vid];extent=amount(b,'S2O3-2')-amount(f,'S2O3-2')
 assert amount(b,'S2O3-2')==.001 and amount(b,'H+')==.004
 assert b['elapsed_seconds']==0 and f['elapsed_seconds']==10 and extent>0
 for actual,expected in [(amount(f,'S'),extent),(amount(f,'SO2'),extent),(amount(b,'H+')-amount(f,'H+'),2*extent),(amount(f,'water')-amount(b,'water'),extent)]:assert abs(actual-expected)<1e-12
 assert amount(f,'H+')>0
 metrics.append({'vessel':vid,'initial_temperature_K':b['temperature'],'final_temperature_K':f['temperature'],'initial_pH':b['solution']['ph'],'final_pH':f['solution']['ph'],'extent_mol':extent,'remaining_thiosulfate_mol':amount(f,'S2O3-2'),'generated_all_phase_sulfur_mol':amount(f,'S'),'generated_retained_SO2_mol':amount(f,'SO2'),'acid_fraction_consumed':2*extent/.004,'remaining_acid_equivalents_mol':amount(f,'H+')})
assert metrics[1]['extent_mol']>metrics[0]['extent_mol']
assert metrics[1]['remaining_thiosulfate_mol']<metrics[0]['remaining_thiosulfate_mol']
assert initial[1]['temperature']>initial[0]['temperature'] and final[1]['temperature']>final[0]['temperature']
source=subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-core/src/kinetics.rs'],cwd=ROOT)
block=source.decode().split('id: "thiosulfate-acid"',1)[1].split('reverse: None',1)[0]
assert 'pre_exponential: 3.0e8' in block and 'activation_energy: 51_000.0' in block
assert 'computed pH' in block and 'consumed' in block
limits=[e for r in rows for e in r['events'] if e['event']=='not_yet_modeled'];assert len(limits)==2 and all(e['reason']['key']=='not-modeled.finite-acid-inventory' for e in limits)
activity_ratio=10**(initial[0]['solution']['ph']-initial[1]['solution']['ph'])
thermal_factor=math.exp(51000/8.31446261815324*(1/initial[0]['temperature']-1/initial[1]['temperature']))
report={'schema':'kerotakis.saved-thiosulfate-bounded-review.v1','original_id':CID,'original_script_sha256':c['script_sha256'],'stdout_sha256':sha(raw),'archive_sha256':sha(archive.read_bytes()),'engine':screen['engine'],'kinetics_source_sha256':sha(source),'original_forecast':c['expected_result'],'original_criteria':c['acceptance_checks'],'metrics':metrics,'initial_warm_over_cool_proton_activity_ratio':activity_ratio,'source_Ea_only_Arrhenius_direction_factor':thermal_factor,'initial_activity_times_thermal_direction':activity_ratio*thermal_factor,'per_vessel_maximum_atom_charge_residual':{str(k):str(v) for k,v in maxima.items()},'per_step_ledger':per_step,'criterion_review':{'KN-012-C1':'At the same actual10s, warm substrate loss and retained all-phase sulfur/SO2 exceed the cool arm; every step closes per-vessel material and charge. No turbidity-only inference.','KN-012-C2':'Actual initial/final temperatures and pH are retained; initial proton activities differ by under1%, while positive source Ea gives a substantially larger temperature-direction factor. Acid depletion is explicit, bounded and stoichiometric, not held constant.'},'qualified_scope':'Qualitative direction within the declared teaching rate law; finite-acid interval starts with frozen activity coefficient and re-solves equilibrium between intervals. No measured/calibrated rate, isothermal exposure or continuously coupled acid-equilibrium prediction.','proposed_bounded_verdict':'pass','catalog_selection_assigned':False,'new_engine_or_API_executions':0}
p=HERE/'assessment.json'
if p.exists():assert load(p)==report
else:p.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'original_id':CID,'proposed_bounded_verdict':'pass','new_executions':0,'metrics':metrics,'ledger':report['per_vessel_maximum_atom_charge_residual']}))
