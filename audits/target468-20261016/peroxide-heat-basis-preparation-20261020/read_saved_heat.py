#!/usr/bin/env python3
"""Authenticate existing peroxide heat observations; no new engine execution."""
import hashlib,json,subprocess,zipfile
from decimal import Decimal as D
from pathlib import Path
R=Path(__file__).resolve().parents[3];B=R/'audits/new-experiments-201-20261016/native-screen-results'
sha=lambda raw:hashlib.sha256(raw).hexdigest()
case=json.loads((B/'cases/batch2_ki-008.json').read_text());meta=json.loads((B/'artifact-metadata.json').read_text())
archive=B/'raw-artifact.zip'
with archive.open('rb') as f:archive_hash=hashlib.file_digest(f,'sha256').hexdigest()
assert archive_hash==meta['digest'].removeprefix('sha256:')
with zipfile.ZipFile(archive) as z:
 raw=z.read('batch2_ki-008/stdout.jsonl');script=z.read('batch2_ki-008/experiment.lab');err=z.read('batch2_ki-008/stderr.txt')
 assert sha(raw)==case['execution']['stdout_sha256'] and sha(script)==case['selected_script_sha256'] and sha(err)==case['execution']['stderr_sha256']
 rows=[json.loads(l,parse_float=D) for l in raw.splitlines() if l.strip()]
observations=[];suppressed=[];state={}
def amount(v,s):return sum((p['moles'] for p in v['contents'] if p['species']==s),D(0))
for i,row in enumerate(rows):
 before=state.copy();state.update({v['id']:v for v in row['bench']['vessels']})
 if row['operator']['op']!='wait':continue
 for vid,v in state.items():
  used=amount(before[vid],'H2O2')-amount(v,'H2O2')
  q=sum((e['energy_j'] for e in row['events'] if e['event']=='reaction_heat_released' and e['reaction']=='peroxide-decomposition' and e['vessel']==vid),D(0))
  assert used>0
  if q==0:
   suppressed.append({'row':i,'vessel':vid,'actual_H2O2_consumed_mol':str(used),'actual_heat_event_present':False,'absence_is_not_zero_heat':True});continue
  assert abs(q/used-D(49100))<D('.001')
  observations.append({'row':i,'vessel':vid,'actual_H2O2_consumed_mol':str(used),'actual_reaction_heat_J':str(q),'actual_heat_per_consumed_mole_J':str(q/used),'NIST_reference_magnitude_J_per_consumed_mole':98480,'NIST_reference_uncertainty_J_per_mol':790,'heat_basis_matches_reference':abs(q/used-D(98480))<=790})
assert len(observations)==2 and len(suppressed)==1 and all(not r['heat_basis_matches_reference'] for r in observations)
current='32c5c96b2685b08ddea048861006ec486357c7a8';source=case['engine']['source_head'];paths=['crates/kerotakis-core/src/clock.rs','crates/kerotakis-core/src/kinetics.rs'];bindings={}
for h in [source,current]:
 blobs={p:subprocess.check_output(['git','show',h+':'+p],cwd=R) for p in paths};assert b'let energy_j = 98_200.0 * moles.0;' in blobs[paths[0]];bindings[h]={p:sha(raw) for p,raw in blobs.items()}
print(json.dumps({'kind':'authenticated_historical_runtime_heat_basis_counterexample','archive_sha256':archive_hash,'original_script_sha256':sha(script),'stdout_sha256':sha(raw),'engine':case['engine'],'lock_qualification':'Inherited original producer-reported lock; no new actual lock bytes asserted.','source_bindings':bindings,'observations':observations,'below_heat_event_observability_threshold':suppressed,'new_engine_or_API_executions':0,'prepared_Rust_controls_executed':False,'current_candidate_validated':False},indent=2))
