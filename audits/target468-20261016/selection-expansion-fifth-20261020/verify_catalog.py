#!/usr/bin/env python3
"""Verify seven earned fifth-unit selections and reproduce the saved mathematical review."""
import hashlib,json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];A=HERE.parent;F=A/'batch2-selection-review-fifth-20261020';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(HERE/'selection.json');old=load(ROOT/s['previous_catalog']);new=load(ROOT/s['new_catalog']);assert sha(ROOT/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(ROOT/s['new_catalog'])==s['new_catalog_sha256'];assert old['count']==len(old['experiments'])==310 and new['experiments'][:310]==old['experiments'] and new['count']==len(new['experiments'])==317
assessment=load(F/'assessment.json');adj=load(F/'parent-adjudication.json');assert adj['assessment_sha256']==sha(F/'assessment.json') and adj['reader_sha256']==sha(F/'review_saved.py');assert adj['parent_independently_reproduced_actual_CLI_JSON'] and adj['parent_inspected_all_seven_original_forecasts_and_criteria'];assessed={c['id']:c for c in assessment['cases']};expected={'batch2:ec-005','batch2:ec-006','batch2:ab-028','batch2:pq-011','batch2:pq-012','batch2:mo-002','batch2:tc-003'};assert {c['id'] for c in assessment['cases'] if c['proposed_candidate']}==expected==set(adj['accepted_originals'])==set(s['new_selected_originals']);assert set(adj['held_originals'])=={'batch2:pq-009','batch2:mo-005','batch2:mo-010','batch2:qa-003','batch2:tc-008'}
originals={e['original_id']:e for e in load(ROOT/'audits/new-experiments-201-20261016/intake.json')};ids=set()
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
for e in new['experiments'][310:]:
 oid=e['original_id'];c=assessed[oid];o=originals[oid];assert oid in expected and c['proposed_candidate'] and c['material_budget_pass'] and all(x['result']=='met' for x in c['criteria']);assert e['script']==(ROOT/'audits/new-experiments-201-20261016'/o['original_script']).read_text();assert e['expected_result']==o['original_expected_result'];assert e['validated_on']==c['source'];assert e['observed_result']['independent_calculations']==c['metrics'];assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations'];assert len(o['original_acceptance_checks'])==len(e['acceptance_checks'])
 for x,y in zip(o['original_acceptance_checks'],e['acceptance_checks'],strict=True):assert {k:v for k,v in x.items() if k!='result'}=={k:v for k,v in y.items() if k!='result'}
 assert all((ROOT/path).exists() for path in e['evidence'].values());assert 'producer-reported' in e['limitations'][-1] and 'raw Cargo.lock bytes are absent' in e['limitations'][-1]
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==313 and len(outside)==4;assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(ROOT/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':313};remaining=load(HERE/'remaining155.json');assert remaining['remaining']==len(remaining['cases'])==155;assert {r['original_id'] for r in remaining['cases']}==target-{m['original_target_id'] for m in mapping};assert remaining['additional_needed_to401']==401-313==88
result=subprocess.run([sys.executable,str(F/'review_saved.py')],cwd=ROOT,text=True,capture_output=True);assert result.returncode==0,result.stderr;assert json.loads(result.stdout)==assessment
print(json.dumps({'all310_predecessor_entries_exact':True,'all7_original_scripts_forecasts_criteria_exact':True,'saved_mathematical_review_reproduced':True,'catalog_entries':317,'unique_fixed468':313,'remaining':155,'additional_to401':88,'five_original_holds_preserved':True,'new_engine_or_API_executions':0}))
