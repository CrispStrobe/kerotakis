#!/usr/bin/env python3
"""Verify three earned seventh-unit selections and reproduce the saved mathematical review."""
import collections,hashlib,json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];A=HERE.parent;F=A/'batch2-selection-review-seventh-20261020';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(HERE/'selection.json');old=load(ROOT/s['previous_catalog']);new=load(ROOT/s['new_catalog']);assert sha(ROOT/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(ROOT/s['new_catalog'])==s['new_catalog_sha256'];assert old['count']==len(old['experiments'])==322 and new['experiments'][:322]==old['experiments'] and new['count']==len(new['experiments'])==325
assessment=load(F/'assessment.json');adj=load(F/'parent-adjudication.json');assert adj['assessment_sha256']==sha(F/'assessment.json') and adj['reader_sha256']==sha(F/'review_saved.py');assert adj['parent_reproduced_full_saved_reader_output_exactly'] and adj['parent_inspected_original_C1_C2_and_forecast'];assessed={c['id']:c for c in assessment['cases']};expected={'batch2:ab-032','batch2:pq-005','batch2:mo-009'};assert {c['id'] for c in assessment['cases'] if c['proposed_candidate']}==expected==set(adj['accepted_originals'])==set(s['new_selected_originals']);assert set(adj['held_originals'])=={'batch2:ec-008','batch2:se-002','batch2:se-007','batch2:mo-007','batch2:mo-008'}
originals={e['original_id']:e for e in load(ROOT/'audits/new-experiments-201-20261016/intake.json')};ids=set()
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
for e in new['experiments'][322:]:
 oid=e['original_id'];c=assessed[oid];o=originals[oid];assert oid in expected and c['proposed_candidate'] and c['material_budget_pass'] and all(x['result']=='met' for x in c['criteria']);assert e['script']==(ROOT/'audits/new-experiments-201-20261016'/o['original_script']).read_text();assert e['expected_result']==o['original_expected_result'];assert e['validated_on']==c['source'];assert e['observed_result']['independent_calculations']==c['metrics'];assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations'];assert len(o['original_acceptance_checks'])==len(e['acceptance_checks'])
 for x,y in zip(o['original_acceptance_checks'],e['acceptance_checks'],strict=True):assert {k:v for k,v in x.items() if k!='result'}=={k:v for k,v in y.items() if k!='result'}
 assert all((ROOT/path).exists() for path in e['evidence'].values());assert 'producer-reported' in e['limitations'][-1] and 'raw Cargo.lock bytes are absent' in e['limitations'][-1]
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==321 and len(outside)==4;assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(ROOT/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':321};remaining=load(HERE/'remaining147.json');assert remaining['remaining']==len(remaining['cases'])==147;assert {r['original_id'] for r in remaining['cases']}==target-{m['original_target_id'] for m in mapping};assert remaining['additional_needed_to401']==401-321==80
result=subprocess.run([sys.executable,str(F/'review_saved.py')],cwd=ROOT,text=True,capture_output=True);assert result.returncode==0,result.stderr;assert json.loads(result.stdout)==assessment
assert new['counts_by_group']==dict(collections.Counter(e['group'] for e in new['experiments'])) and sum(new['counts_by_group'].values())==325
assert adj['independent_source_indicator_pKa_verified']==[3.47,9.4] and abs(adj['independent_interval_antiderivative_whole_heat_residual_J'])<adj['existing_energy_tolerance_J']==.1
sourceview=load(F/'source-capability-current-ref.json')
for path,digest in sourceview['source_blobs_sha256'].items():assert hashlib.sha256(subprocess.check_output(['git','show',sourceview['source_head']+':'+path],cwd=ROOT)).hexdigest()==digest
assert (F/'parent-quadrature-crosscheck-first-assumption.json').exists() and (F/'exploratory-data-shape-failures.json').exists()
print(json.dumps({'all322_predecessor_entries_exact':True,'all3_original_scripts_forecasts_criteria_exact':True,'actual_group_Counter_matches_metadata':True,'saved_mathematical_review_reproduced':True,'parent_independent_antiderivative_within_original_tolerance':True,'catalog_entries':325,'unique_fixed468':321,'remaining':147,'additional_to401':80,'five_original_holds_preserved':True,'new_engine_or_API_executions':0}))
