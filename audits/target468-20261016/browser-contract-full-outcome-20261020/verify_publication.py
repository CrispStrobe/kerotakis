#!/usr/bin/env python3
"""Authenticate this small saved publication; never build, request or execute an engine.
Full artifact-member/executable rehashing needs retained private archives and saved readers.
"""
import argparse
import hashlib
import json
import pathlib
import re
import subprocess

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--repo',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parents[3]);args=parser.parse_args()
 root=pathlib.Path(__file__).resolve().parent;raw=root/'raw'
 def load(p):return json.loads(p.read_text())
 def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
 pub=load(root/'publication.json')
 for prefix,key in [(raw,'copied_small_records'),(root,'record_bindings')]:
  for name,row in pub[key].items():
   path=pathlib.PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
   p=prefix/name;assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],name
 first=load(root/'root-accepted.json');parent=load(root/'root-parent-accepted.json')
 assert {k:v for k,v in first.items() if k!='reviewed_utc'}=={k:v for k,v in parent.items() if k!='reviewed_utc'}
 plan=load(root/'control-plan.safe.json');source=plan['reviewed_head'];assert first['source']==source
 get=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=args.repo)
 assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=args.repo,text=True).strip()==plan['tree']==first['tree']
 assert hashlib.sha256(get('.github/workflows/chemistry-audit.yml')).hexdigest()==plan['workflow_sha256']
 for path,digest in plan['source_native_manifest_files_sha256'].items():assert hashlib.sha256(get(path)).hexdigest()==digest,path
 engine=load(raw/'engine-binding.json');assert engine==first['engine']
 assert sha(raw/'producer-Cargo.lock')==engine['actual_producer_lock_sha256']
 manifest=load(raw/'SHA256.json')
 for path,row in pub['copied_small_records'].items():
  if path in manifest:assert row['sha256']==manifest[path],path
 for category in ['retained_large_records','withheld_private_records']:
  for path,row in pub[category].items():assert row['sha256']==manifest[path],path
 steps=0
 for job in plan['jobs']:
  name=job['key']+('-'+job['family'] if job['family'] else '')
  for step in job['steps']:
   base=raw/name/('step-'+str(step['index']));rec=load(base.with_suffix('.json'))
   assert rec['argv']==['bash','-e','-o','pipefail','-c',step['run']] and rec['exit_status']==0
   for stream in ['stdout','stderr']:assert sha(base.with_suffix('.'+stream))==rec[stream+'_sha256']
   steps+=1
  assert load(raw/name/'source-unchanged.json')['exit_status']==0
 assert steps==142==first['original_run_steps_independently_checked'] and len(plan['jobs'])==33==first['logical_jobs']
 assert len(first['law_reports'])==29 and all(r['unmet']==0 for r in first['law_reports'])
 for report in first['focused_native_results']:
  assert report['exit_status']==0
 for n,command in enumerate(plan['focused_native_commands']):
  rec=load(raw/('combined-native-'+str(n)+'.json'));assert rec['argv']==command and rec['exit_status']==0
  for stream in ['stdout','stderr']:assert sha(raw/('combined-native-'+str(n)+'.'+stream))==rec[stream+'_sha256']
  if n in [2,3,4,5]:
   text=(raw/('combined-native-'+str(n)+'.stdout')).read_text();counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
   assert counts and all(f=='0' and i=='0' for _,f,i in counts)
   assert sum(int(a) for a,_,_ in counts)>= {2:620,3:2173,4:2,5:184}[n]
 native_text='\n'.join((raw/('combined-native-'+str(n)+'.stdout')).read_text() for n in [2,3,4,5])
 names=plan['prior_platform_controls']+plan['phase_owned_controls']+plan['browser_controls']+plan['borrowed_controls'];assert len(set(names))==55
 for name in names:assert re.search(r'^test (?:[A-Za-z0-9_]+::)*'+re.escape(name)+r' \.\.\. ok$',native_text+'\n'+(raw/'cache-only-host-tests.stdout').read_text(),re.M),name
 for name,command in plan['cache_only_commands']:
  rec=load(raw/(name+'.json'));assert rec['argv']==command and rec['exit_status']==0
  for stream in ['stdout','stderr']:assert sha(raw/(name+'.'+stream))==rec[stream+'_sha256']
 host=(raw/'cache-only-host-tests.stdout').read_text();actual=re.findall(r'^test ([\w:]+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)$',host,re.M)
 assert len(actual)==18 and {n for n,_ in actual}==set(plan['cache_only_host_names']) and all(state=='ok' for _,state in actual)
 for n,command in enumerate(plan['additional_gate_commands']):
  rec=load(raw/('additional-gate-'+str(n)+'.json'));assert rec['argv']==command and rec['exit_status']==0
  for stream in ['stdout','stderr']:assert sha(raw/('additional-gate-'+str(n)+'.'+stream))==rec[stream+'_sha256']
 coverage=load(raw/'additional-gate-1.stdout');assert len(coverage['prompts'])==500 and coverage['unmet_requirements']==0 and not coverage['failures'] and not coverage['baseline_drift']
 original=(root/'readers/first-reader.py').read_bytes();fixed=(root/'readers/review_saved_import_fixed.py').read_bytes();assert fixed==b'import base64\n'+original
 correction=load(root/'reader-import-correction.json');assert hashlib.sha256(original).hexdigest()==correction['first_reader_sha256'] and hashlib.sha256(fixed).hexdigest()==correction['successor_reader_sha256']
 for k in ['all_original_steps_and_named_laws_pass','all500_original_requirements_no_drift','all55_native_and_browser_controls_present','all18_cache_only_host_controls_pass','actual_executed_test_binaries_rehashed']:assert first[k] is True
 assert first['catalog_additions']==0 and first['protected_contexts_replaced'] is False and first['integration_or_deployment_claim'] is False
 print(json.dumps({'status':'PASS','source':source,'engine':engine,'logical_jobs':33,'original_steps':steps,'law_reports':29,'small_records_authenticated':len(pub['copied_small_records']),'large_records_metadata_only':len(pub['retained_large_records']),'receipt_equality_except_reviewed_utc':True,'engine_or_job_executions':0},indent=2))
if __name__=='__main__':main()
