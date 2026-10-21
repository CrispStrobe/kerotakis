#!/usr/bin/env python3
"""Verify original AD005 selection, all predecessors and saved-only provenance."""
import collections,hashlib,json,subprocess,sys,tempfile,zipfile
from pathlib import Path
H=Path(__file__).resolve().parent;A=H.parent;R=A.parents[1];P=A/'AD005-saved-original-review-20261021'
load=lambda p:json.loads(p.read_bytes())
def sha(p):
 with p.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
s=load(H/'selection.json');old=load(R/s['previous_catalog']);new=load(R/s['new_catalog']);adj=load(P/'parent-adjudication.json');a=load(P/'assessment.json');parent=load(P/'parent-assessment.json')
assert sha(R/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(R/s['new_catalog'])==s['new_catalog_sha256']
assert old['count']==len(old['experiments'])==336 and new['count']==len(new['experiments'])==337
assert new['experiments'][:336]==old['experiments'] and new['counts_by_group']==dict(collections.Counter(x['group']for x in new['experiments']))
for name,value in load(P/'evidence-manifest.json')['files_sha256'].items():assert sha(P/name)==value,name
assert a==parent and adj['reproduction_semantically_identical'] is True
assert adj['reader_sha256']==sha(P/'review_saved.py') and adj['assessment_sha256']==sha(P/'assessment.json') and adj['parent_reproduction_sha256']==sha(P/'parent-assessment.json')
assert sha(P/'parent-adjudication.json')==s['parent_adjudication_sha256']
assert adj['accepted_originals']==s['new_selected_originals']==['batch3-diverse:ad-005']
assert len(adj['parent_independent_global_component_checks'])==13 and all(x['component_ownership_holds']for x in adj['parent_independent_global_component_checks'])
c=next(x for x in a['cases']if x['original_id']=='batch3-diverse:ad-005');e=new['experiments'][-1]
o=next(x for x in load(R/'audits/new-experiments-201-20261016/intake.json')if x['original_id']==c['original_id'])
assert e['original_id']==c['original_id'] and e['script']==(R/'audits/new-experiments-201-20261016'/o['original_script']).read_text(encoding='utf-8')
assert e['script_sha256']==o['script_sha256']==c['script_sha256'] and e['expected_result']==o['original_expected_result']==c['original_forecast']
assert c['original_criteria']==o['original_acceptance_checks'] and len(e['acceptance_checks'])==4
for x,y in zip(e['acceptance_checks'],o['original_acceptance_checks'],strict=True):assert {k:v for k,v in x.items()if k!='result'}=={k:v for k,v in y.items()if k!='result'} and x['result']=='met_in_original_supported_scope'
assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations'] and e['validated_on']==a['engine']
assert set(c['criterion_assessment'])=={'question','inventory','coverage','controls'} and len(c['exact_coverage_warnings'])==4
assert c['metrics']['peak_nm']==465 and c['metrics']['counterfactual_is_NOT_an_additional_observation'] is True
assert a['actual_binary_or_lock_bytes_rehashed'] is False and a['product_engine_API_provider_build_calls']==0
b=load(P/'raw-bindings.json');archive=R/b['archive_reference'];assert sha(archive)==b['archive_sha256']==a['archive_sha256']
with zipfile.ZipFile(archive)as z:
 for name,member in b['exact_copied_members'].items():assert (P/name).read_bytes()==z.read(member)
 assert not any(n.endswith(('/kero','/Cargo.lock','.exe'))for n in z.namelist())
for x in new['experiments']:assert hashlib.sha256(x['script'].encode()).hexdigest()==x['script_sha256']
assert len({x['id']for x in new['experiments']})==337
allids={x['id']for x in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};assert len(target)==468
mapping=[];outside=[]
for x in new['experiments']:
 oid=x.get('original_id',x['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(x['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':x['id'],'original_target_id':match.pop()})
assert len(mapping)==len({x['original_target_id']for x in mapping})==333 and len(outside)==4
assert load(H/'catalog-membership.json')=={'catalog_sha256':sha(R/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':333}
rem=load(H/'remaining135.json');prior=load(A/'selection-PQ002-PQ003-20261020/remaining136.json');assert rem['cases']==[x for x in prior['cases']if x['original_id']!='batch3-diverse:ad-005']
assert rem['remaining']==len(rem['cases'])==135 and rem['current_unique_fixed468']==333 and rem['additional_needed_to401']==68
assert {x['original_id']for x in rem['cases']}==target-{x['original_target_id']for x in mapping}
score=load(A/'scoreboard-20261021-reviewed-AD005.json');assert score['catalog_sha256']==s['new_catalog_sha256'] and score['current_working_catalog_entries']==337 and score['source_qualified_selected_in_target']==333
with tempfile.TemporaryDirectory(prefix='kero-ad005-saved-')as td:
 out=Path(td)/'review';p=subprocess.run([sys.executable,str(P/'review_saved.py'),'--repo',str(R),'--out-dir',str(out)],capture_output=True,text=True,timeout=120);assert p.returncode==0,p.stderr;assert load(out/'assessment.json')==a
print(json.dumps({'all336_predecessors_exact':True,'original_four_criteria_preserved':True,'saved_reader_semantically_reproduced':True,'all13_command_owned_ledgers':True,'catalog_entries':337,'unique_fixed468':333,'remaining':135,'additional_to401':68,'new_product_engine_API_executions':0,'empirical_calibration_or_current_release_claim':False}))
