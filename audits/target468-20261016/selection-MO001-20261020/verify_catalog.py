#!/usr/bin/env python3
"""Reproduce one parent-accepted historical original; saved data/source only."""
import collections,hashlib,json,subprocess,sys,tempfile,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];A=HERE.parent;P=A/'MO001-saved-original-outcome-20261020';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(HERE/'selection.json');oldpath=ROOT/s['previous_catalog'];newpath=ROOT/s['new_catalog'];old=load(oldpath);new=load(newpath)
assert sha(oldpath)==s['previous_catalog_sha256'] and sha(newpath)==s['new_catalog_sha256'];assert old['count']==len(old['experiments'])==333 and new['count']==len(new['experiments'])==334 and new['experiments'][:333]==old['experiments'];assert new['counts_by_group']==dict(collections.Counter(e['group'] for e in new['experiments'])) and sum(new['counts_by_group'].values())==334
manifest=load(P/'evidence-manifest.json')
for name,digest in manifest['files_sha256'].items():assert sha(P/name)==digest,name
adj=load(P/'parent-adjudication.json');assessment=load(P/'assessment.json');assert adj['accepted_originals']==s['new_selected_originals']==['batch2:mo-001'];assert sha(P/'parent-adjudication.json')==s['parent_adjudication_sha256'];assert adj['parent_reproduced_full_saved_reader_exactly'] and adj['parent_inspected_both_original_criteria_and_named_model_domain'] and adj['complete_13_commands_19_owned_vessel_ledgers'] and adj['binary_and_lock_only_producer_identified_not_independently_rehashed'];assert adj['reader_sha256']==sha(P/'review_saved.py') and adj['assessment_sha256']==sha(P/'assessment.json') and adj['parent_assessment_sha256']==sha(P/'parent-assessment.json');assert load(P/'parent-assessment.json')==assessment
assert assessment['status']=='PASS_PROPOSAL_PENDING_PARENT_REVIEW' and all(assessment['criteria'].values()) and assessment['scientific_catalog_selection_assigned'] is False and assessment['new_engine_API_provider_build_calls']==0
assert len(assessment['per_command_per_vessel'])==19 and assessment['raw_binding']['captured_rows']==13
rawbind=load(P/'raw-bindings.json');archive=ROOT/rawbind['archive_reference']
with archive.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==rawbind['archive_sha256']==assessment['raw_binding']['archive_sha256']
with zipfile.ZipFile(archive) as z:
 for pub,member in (rawbind['exact_copied_members']|rawbind['exact_common_archive_bindings']).items():assert (P/pub).read_bytes()==z.read(member),pub
 assert not any(n.endswith(('/kero','/Cargo.lock','.exe')) for n in z.namelist())
engine=assessment['raw_binding']['source'];assert engine==s['historical_source']==adj['old_engine'];assert engine['source_head']=='c44692053a94c59dcf62e7fb67549f84c7d18f83' and engine['binary_sha256']=='670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f' and engine['cargo_lock_sha256']=='82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229';assert rawbind['raw_binary_supplied'] is False and rawbind['raw_Cargo_lock_supplied'] is False
for q in load(P/'source-quotes.json')['quotes']:
 raw=subprocess.check_output(['git','show',q['source']+':'+q['repository_path']],cwd=ROOT);assert hashlib.sha256(raw).hexdigest()==q['whole_source_sha256'];assert '\n'.join(raw.decode().splitlines()[q['first_line']-1:q['last_line']])+'\n'==q['quote']
e=new['experiments'][-1];o=next(x for x in load(ROOT/'audits/new-experiments-201-20261016/intake.json') if x['original_id']=='batch2:mo-001');assert e['original_id']==assessment['original_id']=='batch2:mo-001';assert e['script']==(P/'raw/experiment.lab').read_text()==(ROOT/'audits/new-experiments-201-20261016'/o['original_script']).read_text();assert e['expected_result']==o['original_expected_result']==assessment['original_forecast'];assert e['observed_result']['independent_calculations']==assessment['metrics'] and e['validated_on']==engine;assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations']==assessment['original_limitations'];assert assessment['original_criteria']==o['original_acceptance_checks'] and len(e['acceptance_checks'])==len(o['original_acceptance_checks'])==2
for c,original in zip(e['acceptance_checks'],o['original_acceptance_checks'],strict=True):assert {k:v for k,v in c.items() if k!='result'}=={k:v for k,v in original.items() if k!='result'} and c['result']=='met_in_original_supported_scope'
assert len({e['id'] for e in new['experiments']})==334
for e in new['experiments']:assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};assert len(target)==468
mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==330 and len(outside)==4;assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(newpath),'mapping':mapping,'outside_target':outside,'unique_fixed468':330};remaining=load(HERE/'remaining138.json');assert remaining['remaining']==len(remaining['cases'])==138 and remaining['additional_needed_to401']==401-330==71;assert {c['original_id'] for c in remaining['cases']}==target-{m['original_target_id'] for m in mapping};prior_remaining=load(A/'selection-phase-owned-SE009-20261020/remaining139.json');assert remaining['cases']==[x for x in prior_remaining['cases'] if x['original_id']!='batch2:mo-001']
score=load(A/'scoreboard-20261020-reviewed-MO001.json');assert score['current_working_catalog_entries']==334 and score['source_qualified_selected_in_target']==330 and score['additional_selections_needed_for_401']==71 and score['catalog_sha256']==sha(newpath)
first=load(P/'first-reader-source-span-failure.json');assert sha(P/'first-reader-source-span-assumption.py')==first['first_reader_sha256'];assert (P/'first-reader-source-span-assumption.py').read_bytes()!=(P/'review_saved.py').read_bytes()
with tempfile.TemporaryDirectory(prefix='kero-MO001-saved-') as td:
 out=Path(td)/'assessment.json';r=subprocess.run([sys.executable,str(P/'review_saved.py'),'--repo',str(ROOT),'--out',str(out)],cwd=ROOT,text=True,capture_output=True);assert r.returncode==0,r.stderr;assert load(out)==assessment
print(json.dumps({'all333_predecessor_entries_exact':True,'Counter_metadata_exact':True,'original_script_forecast_two_criteria_limits_exact':True,'saved_math_and_parent_output_reproduced_exactly':True,'all13commands19vesselledgers':True,'historical_binary_and_lock_producer_identification_only':True,'first_reader_span_failure_preserved':True,'catalog_entries':334,'unique_fixed468':330,'remaining':138,'additional_to401':71,'new_engine_API_provider_build_calls':0}))
