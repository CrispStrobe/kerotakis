#!/usr/bin/env python3
"""Verify five earned changed-source selections and reproduce saved response review."""
import hashlib,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];A=HERE.parent;O=A/'partition-beneficiaries-outcome-20261020';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(HERE/'selection.json');old=load(ROOT/s['previous_catalog']);new=load(ROOT/s['new_catalog']);assert sha(ROOT/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(ROOT/s['new_catalog'])==s['new_catalog_sha256'];assert new['experiments'][:296]==old['experiments'] and new['count']==len(new['experiments'])==301
independent=load(O/'independent-assessment-bound.json');parent=load(O/'parent-independent-assessment.json');assert independent==parent;agreement=load(O/'independent-review-agreement.json');assert agreement['parsed_assessments_equal'];assert agreement['independent_assessment_sha256']==sha(O/'independent-assessment-bound.json') and agreement['parent_assessment_sha256']==sha(O/'parent-independent-assessment.json')
expected={'batch2:gs-014','batch2:gs-018','batch2:ki-002','batch2:tc-002','batch2:tc-009'};assert set(s['new_selected_originals'])==expected;assessed={c['id']:c for c in independent['cases']};assert {k for k,v in assessed.items() if v['status']=='PASS'}==expected;assert assessed['batch2:se-009']['status']=='HOLD'
originals={e['original_id']:e for e in load(ROOT/'audits/new-experiments-201-20261016/intake.json')};ids=set()
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
for e in new['experiments'][296:]:
 oid=e['original_id'];o=originals[oid];c=assessed[oid];assert oid in expected and c['status']=='PASS' and all(c['checks'].values()) and all(c['criteria'].values());assert e['script']==(ROOT/'audits/new-experiments-201-20261016'/o['original_script']).read_text();assert e['expected_result']==o['original_expected_result'];assert e['validated_on']=={**independent['engine'],'source_tree':'76820b9c397038cd8fb6e1f267708d34c153cbe1'}
 assert len(o['original_acceptance_checks'])==len(e['acceptance_checks'])
 for x,y in zip(o['original_acceptance_checks'],e['acceptance_checks'],strict=True):assert {k:v for k,v in x.items() if k!='result'}=={k:v for k,v in y.items() if k!='result'}
 assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations'];assert e['observed_result']['independent_calculations']==c['metrics'];assert all((ROOT/path).exists() for path in e['evidence'].values())
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==297 and len(outside)==4;assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(ROOT/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':297}
remaining=load(HERE/'remaining171.json');assert remaining['remaining']==len(remaining['cases'])==171;assert {c['original_id'] for c in remaining['cases']}==target-{m['original_target_id'] for m in mapping};assert remaining['additional_needed_to401']==401-297==104
manifest=load(O/'evidence-sha256.json')
for path,digest in manifest['files'].items():assert sha(O/path)==digest,path
for path,digest in manifest['external_bindings'].items():assert sha(ROOT/path)==digest,path
# Preserve every original saved response; reproduce only the independent mathematical reader.
reader=ROOT/'audits/target468-20261016/partition-beneficiaries-preparation-20261020/review_responses.py'
with tempfile.TemporaryDirectory(prefix='kero-saved-selection-review-') as temp:
 out=Path(temp)/'assessment.json';result=subprocess.run([sys.executable,str(reader),'--repo',str(ROOT),'--design',str(O/'design.json'),'--results-dir',str(O/'results'),'--artifact-receipt',str(A/'partition-release-acceptance-20261020/artifact-acceptance.json'),'--production-receipt',str(A/'partition-release-acceptance-20261020/production-acceptance.json'),'--out',str(out)],capture_output=True,text=True);assert result.returncode==0,result.stderr;assert load(out)==independent
print(json.dumps({'all296_predecessor_entries_exact':True,'all5_original_scripts_forecasts_criteria_exact':True,'independent_and_parent_saved_reviews_reproduced':True,'catalog_entries':301,'unique_fixed468':297,'remaining':171,'additional_to401':104,'SE009_held':True,'new_engine_or_API_executions':0}))
