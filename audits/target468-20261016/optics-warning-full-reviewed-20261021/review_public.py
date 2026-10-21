"""Saved public receipt checks only; no engine/network or scientific selection."""
from pathlib import Path
import hashlib,json
P=Path(__file__).parent
for n,h in json.loads((P/'SHA256.json').read_bytes()).items():assert hashlib.sha256((P/n).read_bytes()).hexdigest()==h,n
a=json.loads((P/'first.json').read_bytes());b=json.loads((P/'independent.json').read_bytes());assert{k:v for k,v in a.items()if k!='reviewed_utc'}=={k:v for k,v in b.items()if k!='reviewed_utc'}
assert a['source']=='4d87db686ab923415ce01af1cca78696572a030d'and a['logical_jobs']==33 and a['original_run_steps_independently_checked']==142 and len(a['law_reports'])==29
assert a['all500_original_requirements_no_drift']is True and a['all_original_steps_and_named_laws_pass']is True
for i,wanted in [(2,627),(3,2216),(4,2),(5,184)]:assert a['focused_native_results'][i]['tests_passed']==wanted
assert a['reviewer_program_sha256']=='7f74ef0c3eb90ad6407c3211c25826e2a0cea46fdb06f21dc718d769c79c133b'
assert a['engine']['binary_sha256']=='b2a823e42a47ff490571626c3aa57b2687b6a01e2eb1c6a5621bec627b09abc5'and a['engine']['actual_producer_lock_sha256']=='668868136fa9ee1ebd6433d59eb68d5fa46224d874fd96c4036f7614b3c4cd85'
for n in ['output-corrected-invocation.json','output-independent-invocation.json']:assert json.loads((P/n).read_bytes())['actual_exit_status']==0
assert a['cap_domain_energy_closure_claimed']is False and a['current_TC008_trace_replay_claimed']is False and a['catalog_additions']==0
current=json.loads((P/'current62f-full-issued.json').read_bytes());assert current['source']!=a['source']and current['current_full_acceptance_claimed']is False
print('Public saved receipts agree; preceding4d full authenticated, current62f result not assigned.')
