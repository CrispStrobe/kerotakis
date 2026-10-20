#!/usr/bin/env python3
"""Verify saved originals, unchanged predecessors and exact fixed-target membership."""
import collections,hashlib,json,subprocess,sys,tempfile,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent; A=HERE.parent; R=A.parents[1]; P=A/'PQ002-PQ003-saved-original-outcome-20261020'
load=lambda p:json.loads(p.read_bytes()); sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=load(HERE/'selection.json');old=load(R/s['previous_catalog']);new=load(R/s['new_catalog'])
assert sha(R/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(R/s['new_catalog'])==s['new_catalog_sha256']
assert old['count']==len(old['experiments'])==334 and new['count']==len(new['experiments'])==336 and new['experiments'][:334]==old['experiments']
assert new['counts_by_group']==dict(collections.Counter(e['group'] for e in new['experiments']))
for f,h in load(P/'evidence-manifest.json')['files_sha256'].items():assert sha(P/f)==h,f
adj=load(P/'parent-adjudication.json');a=load(P/'assessment.json');assert load(P/'parent-assessment.json')==a
assert sha(P/'parent-adjudication.json')==s['parent_adjudication_sha256']
assert adj['reader_sha256']==sha(P/'review_saved.py') and adj['assessment_sha256']==sha(P/'assessment.json') and adj['parent_assessment_sha256']==sha(P/'parent-assessment.json')
assert adj['accepted_originals']==s['new_selected_originals']==[c['original_id'] for c in a['cases']]==['batch2:pq-002','batch2:pq-003']
assert adj['old_engine']==s['historical_source']==a['source_engine']
assert a['catalog_selection_assigned'] is False and a['new_engine_API_provider_build_calls']==0
assert [len(c['per_command_per_vessel']) for c in a['cases']]==[41,22] and a['owned_atom_charge_tolerance_mol']=='1E-10'
b=load(P/'raw-bindings.json');archive=R/b['archive_reference']
with archive.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==b['archive_sha256']==a['archive_sha256']
with zipfile.ZipFile(archive) as z:
 for f,member in b['exact_copied_members'].items():assert (P/f).read_bytes()==z.read(member)
 assert not any(n.endswith(('/kero','/Cargo.lock','.exe')) for n in z.namelist())
intake=load(R/'audits/new-experiments-201-20261016/intake.json')
for e,c in zip(new['experiments'][334:],a['cases'],strict=True):
 o=next(i for i in intake if i['original_id']==c['original_id']);assert e['original_id']==c['original_id']
 assert e['script']==c['original_script']==(R/'audits/new-experiments-201-20261016'/o['original_script']).read_text(encoding='utf-8')
 assert e['expected_result']==c['original_forecast']==o['original_expected_result'] and c['original_criteria']==o['original_acceptance_checks']
 assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations']==c['original_limitations']
 assert e['validated_on']==a['source_engine'] and all(c['criteria'].values())
 assert len(e['acceptance_checks'])==len(o['original_acceptance_checks'])==2
 for x,y in zip(e['acceptance_checks'],o['original_acceptance_checks'],strict=True):assert {k:v for k,v in x.items() if k!='result'}=={k:v for k,v in y.items() if k!='result'} and x['result']=='met_in_original_supported_scope'
 assert c['domain']['UVvis_success'] is False and c['domain']['unregistered_or_unoffered_oxide_precipitation_not_modelled'] is True
for e in new['experiments']:assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
assert len({e['id'] for e in new['experiments']})==336
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};assert len(target)==468
mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==332 and len(outside)==4
assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(R/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':332}
r=load(HERE/'remaining136.json');prior=load(A/'selection-MO001-20261020/remaining138.json');assert r['cases']==[c for c in prior['cases'] if c['original_id'] not in s['new_selected_originals']]
assert r['remaining']==len(r['cases'])==136 and r['additional_needed_to401']==401-332==69
assert {c['original_id'] for c in r['cases']}==target-{m['original_target_id'] for m in mapping}
score=load(A/'scoreboard-20261020-reviewed-PQ002-PQ003.json');assert score['catalog_sha256']==s['new_catalog_sha256'] and score['current_working_catalog_entries']==336 and score['source_qualified_selected_in_target']==332 and score['additional_selections_needed_for_401']==69
with tempfile.TemporaryDirectory(prefix='kero-pq-saved-') as td:
 out=Path(td)/'assessment.json';p=subprocess.run([sys.executable,str(P/'review_saved.py'),'--repo',str(R),'--out',str(out)],text=True,capture_output=True);assert p.returncode==0,p.stderr;assert load(out)==a
print(json.dumps({'all334_predecessors_exact':True,'saved_reader_reproduced_exactly':True,'all63_owned_ledgers':True,'original_scripts_forecasts_criteria_limits_preserved':True,'historical_binary_lock_producer_identification_only':True,'complete_physical_equilibrium_or_UVvis_claim':False,'catalog_entries':336,'unique_fixed468':332,'remaining':136,'additional_to401':69,'new_engine_API_executions':0}))
