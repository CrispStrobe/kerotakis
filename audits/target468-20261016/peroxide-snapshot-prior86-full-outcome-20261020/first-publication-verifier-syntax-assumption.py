#!/usr/bin/env python3
"""Verify the published small subset only; no builds, engine calls or network."""
from pathlib import Path
import hashlib,json,re,subprocess
P=Path(__file__).resolve().parent;R=P.parents[2];E=P/'raw';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
pub=load(P/'publication.json');plan=load(P/'control-plan.safe.json');source='86df84aa702b049ace43cd576793cd62bbc8f582';assert pub['source']==plan['reviewed_head']==source
manifest=load(E/'SHA256.json');assert sha(E/'SHA256.json')==pub['producer_manifest_sha256']
assert set(pub['copied_current_small_records'])|set(pub['omitted_retained_records'])==set(manifest)
assert not(set(pub['copied_current_small_records'])&set(pub['omitted_retained_records']))
for name,row in pub['copied_current_small_records'].items():assert sha(E/name)==row['sha256']==manifest[name]and(E/name).stat().st_size==row['bytes'],name
for name,row in pub['omitted_retained_records'].items():assert row['sha256']==manifest[name],name
first=load(P/'root-accepted.json');parent=load(P/'root-parent-accepted.json');assert {k:v for k,v in first.items()if k!='reviewed_utc'}=={k:v for k,v in parent.items()if k!='reviewed_utc'}
assert first['source']==source and first['tree']==plan['tree']==subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=R,text=True).strip()
for path,h in plan['source_native_manifest_files_sha256'].items():assert hashlib.sha256(subprocess.check_output(['git','show',source+':'+path],cwd=R)).hexdigest()==h,path
assert first['engine']==load(E/'engine-binding.json') and sha(E/'producer-Cargo.lock')==first['engine']['actual_producer_lock_sha256']
steps=0
for job in plan['jobs']:
 name=job['key']+('-'+job['family']if job['family']else'')
 for step in job['steps']:
  base=E/name/('step-'+str(step['index']));record=load(base.with_suffix('.json'));assert record['argv']==['bash','-e','-o','pipefail','-c',step['run']]and record['exit_status']==0
  for stream in ['stdout','stderr']:assert sha(base.with_suffix('.'+stream))==record[stream+'_sha256']
  steps+=1
 assert load(E/name/'source-unchanged.json')['exit_status']==0
assert len(plan['jobs'])==first['logical_jobs']==33 and steps==first['original_run_steps_independently_checked']==142 and len(first['law_reports'])==29 and all(row['unmet']==0 for row in first['law_reports'])
for i,command in enumerate(plan['focused_native_commands']):
 record=load(E/('combined-native-'+str(i)+'.json'));assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha(E/('combined-native-'+str(i)+'.'+stream))==record[stream+'_sha256']
 if i in [2,3,4,5]:
  text=(E/('combined-native-'+str(i)+'.stdout')).read_text(encoding='utf-8');counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert counts and all(f==g=='0'for _,f,g in counts);assert sum(int(n)for n,_,_in counts)>={2:620,3:2177,4:2,5:184}[i]
text='\n'.join((E/('combined-native-'+str(i)+'.stdout')).read_text(encoding='utf-8')for i in [2,3,4,5]);assert len(set(plan['required_platform_controls']))==59
for name in plan['required_platform_controls']:assert re.search(r'^test (?:[A-Za-z0-9_]+::)*'+re.escape(name)+r' \.\.\. ok$',text,re.M),name
for name,command in plan['cache_only_commands']:
 record=load(E/(name+'.json'));assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha(E/(name+'.'+stream))==record[stream+'_sha256']
host=(E/'cache-only-host-tests.stdout').read_text(encoding='utf-8');actual=re.findall(r'^test ([\w:]+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)$',host,re.M);assert len(actual)==18 and{n for n,_in actual}==set(plan['cache_only_host_names'])and all(s=='ok'for _,s in actual)
record=load(E/'ci-classifier-tests.json');assert record['argv']==plan['classifier_command']and record['exit_status']==0
for stream in ['stdout','stderr']:assert sha(E/('ci-classifier-tests.'+stream))==record[stream+'_sha256']
assert 'Ran 11 tests in 'in(E/'ci-classifier-tests.stderr').read_text(encoding='utf-8')
coverage=load(E/'additional-gate-1.stdout');assert len(coverage['prompts'])==500 and coverage['unmet_requirements']==0 and not coverage['failures']and not coverage['baseline_drift']
assert first['catalog_additions']==0 and first['integration_or_deployment_claim']is False
original=(P/'readers/review_saved.py').read_text(encoding='utf-8');assert(P/'readers/parent_review_saved.py').read_text(encoding='utf-8')==original.replace("B/'ROOT-ACCEPTED.json'","B/'ROOT-PARENT-ACCEPTED.json'")
print(json.dumps({'public_small_records_verified':len(pub['copied_current_small_records']),'source':source,'jobs':33,'steps':142,'law_reports':29,'current_PR18_or_integration_acceptance':False,'new_engine_calls':0}))
