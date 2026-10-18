"""Independently reproduce saved kinetics metrics; no simulations or selections."""
from pathlib import Path
import hashlib,json,re,subprocess,zipfile
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83'
def sha(b):return hashlib.sha256(b).hexdigest()
def amount(v,s,phase=None):return sum(p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase))
raw_source=subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-core/src/kinetics.rs'],cwd=ROOT);source=raw_source.decode()
formula='1.0 + max_mixing_gain * tip_speed / (mixing_half_speed_m_s + tip_speed)';assert formula in source and 'context.mixing_tip_speed_m_s.max(0.0)' in source
m=re.search(r'activity: CatalystActivity::Surface\s*\{[^}]*?max_mixing_gain:\s*([0-9.]+),\s*mixing_half_speed_m_s:\s*([0-9.]+)',source,re.S);assert m;gain,half=map(float,m.groups());assert gain>=0 and half>0
material=json.loads((HERE.parent/'saved64-first-screen/material-projection.json').read_text());by={r['id']:r for r in material['records']}
archive=ROOT/'audits/target468-20261016/common-engine-completion/results/raw-artifact.zip';assert sha(archive.read_bytes())==material['raw_artifact_sha256'];records=[]
originals=ROOT/'audits/target468-20261016/common-engine-completion/originals/new-experiments-20261016.json'
assert sha(originals.read_bytes())=='8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd'
cases={c['id']:c for c in json.loads(originals.read_bytes())['experiments']}
verified=json.loads((originals.parent.parent/'results/verified-results.json').read_bytes());assert verified['engine']['source_head']==SOURCE
receipts={c['id']:c for c in verified['cases']}
with zipfile.ZipFile(archive) as z:
 for n in range(1,15):
  cid=f'new120:kn-{n:03}';c={**cases[cid],'receipt':receipts[cid]};assert sha(c['script'].encode())==c['script_sha256']==c['receipt']['script_sha256'];raw=z.read(cid.replace(':','_')+'/stdout.jsonl');assert sha(raw)==c['receipt']['stdout_sha256'];rows=[json.loads(l) for l in raw.splitlines()];state={};timeline=[]
  for i,r in enumerate(rows):
   state.update({v['id']:v for v in r['bench']['vessels']})
   if r['operator']['op'] in ['wait','stir','filter','inspect','measure']:
    observations=[]
    for v in state.values():
     observations.append({'vessel':v['id'],'elapsed_seconds':v['elapsed_seconds'],'temperature_K':v['temperature'],'reported_ph':(v.get('solution') or {}).get('ph'),'H2O2_mol':amount(v,'H2O2'),'MnO2_mol':amount(v,'MnO2'),'catalase_mol':amount(v,'catalase'),'O2_gas_mol':amount(v,'O2','gas'),'thiosulfate_mol':amount(v,'S2O3-2'),'sulfur_all_phases_mol':amount(v,'S'),'I2_all_phases_mol':amount(v,'I2'),'I3_mol':amount(v,'I3-'),'owned_H_basis_mol':amount(v,'H+'),'qualification':'Owned H basis is not a universal weak-acid titration reserve; reported pH/species are not extra matter.'})
    timeline.append({'row':i,'operator':r['operator'],'vessels':observations})
  limits=[{'row':i,**e} for i,r in enumerate(rows) for e in r['events'] if e['event'] in ['not_yet_modeled','solver_failed']]
  record={'id':cid,'script_sha256':c['script_sha256'],'stdout_sha256':sha(raw),'original_criteria':c['acceptance_checks'],'timeline':timeline,'explicit_limit_events':limits,'represented_whole_bench_final_residual':by[cid]['final_residual'],'physical_verdict':'not_assigned'}
  if n in [1,2,3,4,5,6,9]:
   ids=[0,1] if n!=5 else [0,1,2]
   if n==6:ids=[1,3]
   record['final_peroxide_by_comparison_vessel']=[{'vessel':vid,'elapsed_seconds':state[vid]['elapsed_seconds'],'temperature_K':state[vid]['temperature'],'remaining_mol':amount(state[vid],'H2O2'),'MnO2_mol':amount(state[vid],'MnO2'),'catalase_mol':amount(state[vid],'catalase')} for vid in ids]
  if n==8:
   samples=[r['bench']['vessels'][0] for r in rows if r['operator']['op']=='inspect'];q=[amount(v,'H2O2') for v in samples];record['equal_interval_actual_times']=[v['elapsed_seconds'] for v in samples];record['peroxide_mol']=q;record['successive_positive_mole_losses']=[a-b for a,b in zip(q,q[1:])]
  if n==10:
   samples=[r for r in rows if r['operator']['op']=='inspect'];record['sampled_iodine_and_scavenger']=[{'vessel':v['id'],'elapsed_seconds':v['elapsed_seconds'],'I2_mol':amount(v,'I2'),'I3_mol':amount(v,'I3-'),'thiosulfate_mol':amount(v,'S2O3-2')} for r in samples for v in r['bench']['vessels']]
  if n==13:
   samples=[r['bench']['vessels'][0] for r in rows if r['operator']['op']=='inspect'];record['acid_replenishment_snapshots']=[{'elapsed_seconds':v['elapsed_seconds'],'H_basis_mol':amount(v,'H+'),'thiosulfate_mol':amount(v,'S2O3-2'),'sulfur_mol':amount(v,'S')} for v in samples]
  records.append(record)
report={'source':SOURCE,'kinetics_source_sha256':sha(raw_source),'archive_sha256':material['raw_artifact_sha256'],'source_bound_mixing_law':{'actual_expression':formula,'nonnegative_tip_speed':True,'max_mixing_gain':gain,'half_speed_m_per_s':half,'derived_factor_range':[1,1+gain],'qualification':'Algebraic bound follows nonnegative speed and positive denominator; upper endpoint is approached, not a new fitted rate. Source declares the correction a teaching correlation, not empirical mixing calibration.'},'supplemental_volume_metrics':'supplement-metrics-before-assessment.json','cases':records,'new_engine_or_api_executions':0,'catalog_selections':0,'physical_verdicts_assigned':0,'qualification':'Detailed original-criterion observations for subsequent independent selection. No unsupported branch credited for its zero rate; no clock threshold inferred from lower final iodine alone; catalyst tracer conservation is not an elemental protein formula.'}
expected=json.loads((HERE/'metrics.json').read_text());assert report==expected
print(json.dumps({'cases_reproduced':len(records),'exact_numeric_and_source_match':True,'new_executions':0,'selections':0}))
