"""Verify immutable predecessor, original criteria/scripts and unique target membership."""
from pathlib import Path
import hashlib,json
H=Path(__file__).resolve().parent;R=H.parents[2];A=H.parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(H/'selection.json');old=load(R/s['previous_catalog']);new=load(R/s['new_catalog'])
assert sha(R/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(R/s['new_catalog'])==s['new_catalog_sha256']
assert new['experiments'][:248]==old['experiments'] and new['count']==len(new['experiments'])==268
inventory=load(A/'common-engine-all668.json');all_ids={c['id'] for c in inventory['cases']};target={id for id in all_ids if not id.startswith('curriculum200:')}
originals={e['original_id']:e for e in load(R/'audits/new-experiments-201-20261016/intake.json')};first={e['id']:e for e in load(A/'common-engine-completion/originals/new-experiments-20261016.json')['experiments']}
ids=set();mapping=[];outside=[]
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256'];oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&all_ids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
for e in new['experiments'][248:]:
 oid=e['original_id']
 if oid in originals:
  original=originals[oid];assert e['script']==(R/'audits/new-experiments-201-20261016'/original['original_script']).read_text();assert e['expected_result']==original['original_expected_result'];checks=original['original_acceptance_checks']
 else:
  original=first[oid];assert e['script']==original['script'];assert e['expected_result']==original['expected_result'];checks=original['acceptance_checks']
 assert len(checks)==len(e['acceptance_checks'])
 for a,b in zip(checks,e['acceptance_checks'],strict=True):assert {k:v for k,v in a.items() if k!='result'}=={k:v for k,v in b.items() if k!='result'}
 assert not oid.startswith('batch3-diverse:bd-')
assert len(mapping)==len({m['original_target_id'] for m in mapping})==264
assert load(H/'catalog-membership.json')=={'catalog_sha256':sha(R/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':264}
assert set(s['new_selected_originals'])=={e['original_id'] for e in new['experiments'][248:]}
print(json.dumps({'all248_prior_entries_exact':True,'all20_original_scripts_forecasts_and_criteria_exact':True,'unique_fixed468':264,'catalog_entries':268,'boundary_demonstrations_excluded':3,'new_engine_executions':0}))
