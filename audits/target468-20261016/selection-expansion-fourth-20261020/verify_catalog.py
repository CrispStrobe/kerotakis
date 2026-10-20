#!/usr/bin/env python3
"""Verify nine bounded saved selections, exact predecessor and both independent readers."""
import hashlib,json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];A=HERE.parent;F=A/'batch2-selection-review-fourth-20261020';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(HERE/'selection.json');old=load(ROOT/s['previous_catalog']);new=load(ROOT/s['new_catalog']);assert sha(ROOT/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(ROOT/s['new_catalog'])==s['new_catalog_sha256'];assert new['experiments'][:301]==old['experiments'] and new['count']==len(new['experiments'])==310
assessment=load(F/'assessment.json');addendum=load(F/'parent-questions-addendum.json');adjudication=load(F/'parent-adjudication.json');assert adjudication['assessment_sha256']==sha(F/'assessment.json') and adjudication['addendum_sha256']==sha(F/'parent-questions-addendum.json');assert adjudication['saved_reader_sha256']==sha(F/'review_saved.py') and adjudication['addendum_reader_sha256']==sha(F/'review_addendum.py');assert adjudication['parent_independently_reproduced_saved_reader_JSON'] and adjudication['parent_independently_reproduced_addendum_CLI_JSON'];assessed={c['id']:c for c in assessment['cases']};expected={c['id'] for c in assessment['cases'] if c['proposed_candidate']};assert len(expected)==9 and set(adjudication['accepted_originals'])==expected==set(s['new_selected_originals']);assert set(adjudication['held_originals'])=={'batch2:ec-004','batch2:qa-005','batch2:se-003'}
originals={e['original_id']:e for e in load(ROOT/'audits/new-experiments-201-20261016/intake.json')};ids=set()
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
for e in new['experiments'][301:]:
 oid=e['original_id'];c=assessed[oid];o=originals[oid];assert oid in expected and c['proposed_candidate'] and all(x['result']=='met' for x in c['criteria']);assert e['script']==(ROOT/'audits/new-experiments-201-20261016'/o['original_script']).read_text();assert e['expected_result']==o['original_expected_result'];assert e['validated_on']==c['source'];assert e['observed_result']['independent_calculations']==c['metrics'];assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations'];assert len(o['original_acceptance_checks'])==len(e['acceptance_checks'])
 for x,y in zip(o['original_acceptance_checks'],e['acceptance_checks'],strict=True):assert {k:v for k,v in x.items() if k!='result'}=={k:v for k,v in y.items() if k!='result'}
 assert all((ROOT/path).exists() for path in e['evidence'].values())
 if oid=='batch2:ec-007':assert 'threshold-limited' in e['observed_result']['interpretation'] and 'no coupled equilibrium endpoint' in e['observed_result']['interpretation']
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==306 and len(outside)==4;assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(ROOT/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':306};remaining=load(HERE/'remaining162.json');assert remaining['remaining']==len(remaining['cases'])==162;assert {r['original_id'] for r in remaining['cases']}==target-{m['original_target_id'] for m in mapping};assert remaining['additional_needed_to401']==401-306==95
# main() prints JSON and returns None. Compare actual CLI stdout, not a nonexistent return object.
for reader,saved in [('review_saved.py','assessment.json'),('review_addendum.py','parent-questions-addendum.json')]:
 result=subprocess.run([sys.executable,str(F/reader)],cwd=ROOT,text=True,capture_output=True);assert result.returncode==0,result.stderr;assert json.loads(result.stdout)==load(F/saved)
print(json.dumps({'all301_predecessor_entries_exact':True,'all9_original_scripts_forecasts_criteria_exact':True,'saved_assessment_and_addendum_CLI_reproduced':True,'catalog_entries':310,'unique_fixed468':306,'remaining':162,'additional_to401':95,'EC007_partial_threshold_limited_only':True,'held_EC004_QA005_SE003':True,'new_engine_or_API_executions':0}))
