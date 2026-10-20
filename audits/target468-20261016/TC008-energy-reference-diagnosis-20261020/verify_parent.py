#!/usr/bin/env python3
"""Reproduce a reference diagnostic only; never request or execute the engine."""
from pathlib import Path
import hashlib,json,subprocess,sys,tempfile
P=Path(__file__).resolve().parent;R=P.parents[2];load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
adj=load(P/'parent-adjudication.json');a=load(P/'assessment.json');assert load(P/'parent-assessment.json')==a
assert sha(P/'assessment.json')==adj['assessment_sha256'] and sha(P/'quantify_saved.py')==adj['reader_sha256']
for q in load(P/'source-quotes.json'):
 raw=subprocess.check_output(['git','show',q['source']+':'+q['path']],cwd=R);assert hashlib.sha256(raw).hexdigest()==q['source_blob_sha256'];assert '\n'.join(raw.decode().splitlines()[q['first_line']-1:q['last_line']])+'\n'==q['text']
f=load(P/'existing-fixture-binding.json');assert sha(R/f['repository_path'])==f['sha256']
E=R/'audits/target468-20261016/respiration-brine-beneficiaries-review-20261020/evidence';request=load(E/'results/batch2-tc-008/request.json');response=load(E/'results/batch2-tc-008/response.body');original=next(c for c in load(R/'audits/new-experiments-201-20261016/intake.json')if c['original_id']=='batch2:tc-008')
assert request['script']==(R/'audits/new-experiments-201-20261016'/original['original_script']).read_text(encoding='utf-8');assert hashlib.sha256(request['script'].encode()).hexdigest()==original['script_sha256']==response['script_sha256']==adj['script_sha256'];assert sha(E/'results/batch2-tc-008/response.body')==adj['historical_response_sha256']==a['actual_response_sha256']
assert a['historical_oracle_unchanged'] and a['new_engine_API_provider_build_calls']==0 and a['scientific_selection_assigned'] is False
with tempfile.TemporaryDirectory(prefix='kero-TC008-reference-')as td:
 out=Path(td)/'assessment.json';p=subprocess.run([sys.executable,str(P/'quantify_saved.py'),'--repo',str(R),'--out',str(out)],text=True,capture_output=True);assert p.returncode==0,p.stderr;assert load(out)==a
for name,h in load(P/'public-manifest.json')['files_sha256'].items():assert sha(P/name)==h,name
print(json.dumps({'saved_analytic_calculation_reproduced_exactly':True,'all14_source_quotes_verified':True,'original_protocol_preserved':True,'historical_oracle_unchanged':True,'consistent_reference_residual_J':a['terms']['consistent298_source_reference_residual_J'],'new_executions':0,'scientific_selection_assigned':False}))
