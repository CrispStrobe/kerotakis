import argparse,base64,datetime,gzip,hashlib,json,pathlib,re
def validate_prerequisites(plan):
 receipts={}
 for key in ['phase_full_prerequisite','focused_controls_prerequisite']:
  raw=base64.b64decode(plan[key]['receipt_base64'],validate=True);assert sha(raw)==plan[key]['sha256'];receipts[key]=(raw,json.loads(raw))
 full=receipts['phase_full_prerequisite'][1];controls=receipts['focused_controls_prerequisite'][1]
 assert full['source']=='b4daecf742be080577e9fda57fc1ef55aefef3d3' and full['logical_jobs']==33 and full['original_run_steps_independently_checked']==142 and len(full['law_reports'])==29
 for key in ['all_original_steps_and_named_laws_pass','all500_original_requirements_no_drift','all42_prior_and_new_native_names_present','eight_phase_owned_controls_pass','six_real_phase_baseline_failures_reused_without_execution']:assert full[key] is True,key
 assert full['protected_contexts_replaced'] is False and full['engine']['actual_producer_lock_sha256']==plan['lock_sha256']
 assert controls['source']==plan['preceding_controls_source'] and controls['tree']==plan['preceding_controls_tree'] and controls['lock_sha256']==plan['lock_sha256']
 for key in ['nine_real_baseline_failures_and_three_positive_controls','twelve_fixed_controls_pass','actual_test_executable_bytes_independently_rehashed','format_all_stopped_only_unchanged_third_party']:assert controls[key] is True,key
 assert controls['remaining_fmt_39_transactions_locale_composed_clippy_wasm_gates_accepted'] is False and controls['scientific_selections']==0 and controls['full_source_acceptance'] is False
 return receipts

def test_results(name,names,counts):
 text=(O/(name+'.stdout')).read_text();stderr=(O/(name+'.stderr')).read_text();assert 'error[E' not in stderr
 actual=re.findall(r'^test ([\w:]+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)$',text,re.M)
 assert len(actual)==len(names) and {n for n,_ in actual}==set(names) and all(status=='ok' for _,status in actual),(name,actual)
 summaries=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
 assert sorted(int(n) for n,_,_ in summaries)==sorted(counts) and all(int(f)==0 and int(i)==0 for _,f,i in summaries)
parser=argparse.ArgumentParser();parser.add_argument('--run-dir',required=True,type=pathlib.Path);B=parser.parse_args().run_dir
sha=lambda b:hashlib.sha256(b).hexdigest()
P=json.loads((B/'frozen-plan.json').read_text());I=json.loads((B/'ISSUE-BINDING.json').read_text());C=json.loads((B/'COLLECTED.json').read_text())
assert C['version']==I['version']==1 and I['source']==P['source']
assert I['reader_sha256']==sha(pathlib.Path(__file__).read_bytes())
for key,(raw,_) in validate_prerequisites(P).items():
 assert (B/(key+'.json')).read_bytes()==raw and I['prerequisites_sha256'][key]==sha(raw)
for name,row in C['files'].items():
 path=pathlib.PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
 data=(B/'output'/name).read_bytes();assert sha(data)==row['sha256'] and len(data)==row['bytes']
O=B/'output/browser-contract-gates-evidence';assert not (O/'STOPPED.json').exists()
manifest=json.loads((O/'manifest.json').read_text())
for name,row in manifest.items():
 path=pathlib.PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
 data=(O/name).read_bytes();assert sha(data)==row['sha256'] and len(data)==row['bytes']
assert json.loads((O/'frozen-plan.json').read_text())==P
for key,(raw,_) in validate_prerequisites(P).items():assert (O/(key+'.json')).read_bytes()==raw
S=json.loads((O/'STARTED.json').read_text());assert S['source']==P['source'] and S['worker_sha256']==I['worker_sha256']==sha((B/'upload/worker.py').read_bytes()) and S['no_prior_baseline_reexecution'] is True and S['twelve_fixed_controls_rerun_on_changed_cfg_source'] is True
binding=json.loads((O/'source-binding.json').read_text());assert binding=={'source':P['source'],'tree':P['tree'],'lock_sha256':P['lock_sha256'],'blobs_sha256':P['fixtures_sha256']|P['source_runtime_blobs_sha256']}
for name,argv in P['commands']:
 row=json.loads((O/(name+'.json')).read_text());assert row['argv']==argv and row['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((O/(name+'.'+stream)).read_bytes())==row[stream+'_sha256']
 if name=='transactions':test_results(name,P['existing_transaction_controls']+P['borrowed_control_names'],[1,38])
 if name=='composed':test_results(name,P['composed_control_names'],[18])
 if name=='current-browser-controls':test_results(name,P['current_browser_control_names'],[12])
binary_report={'source':P['source'],'lock_sha256':P['lock_sha256'],'binaries':{}}
for label in ['kerotakis_wasm','borrowed_solver_routes','composed_locale']:
 report=json.loads((O/('test-binaries-'+label+'.json')).read_text());assert report['source']==P['source'] and report['lock_sha256']==P['lock_sha256'];assert not set(report['binaries'])&set(binary_report['binaries']);binary_report['binaries'].update(report['binaries'])
assert set(binary_report['binaries'])=={'kerotakis_wasm','borrowed_solver_routes','solver_transactions','composed_locale'}
for name,record in binary_report['binaries'].items():
 assert record['gzip']==name+'.gz' and sha((O/record['gzip']).read_bytes())==record['gzip_sha256'] and record['bytes']>0
 h=hashlib.sha256();length=0
 with gzip.open(O/record['gzip'],'rb') as stream:
  while chunk:=stream.read(262144):
   length+=len(chunk);assert length<=record['bytes'];h.update(chunk)
 assert length==record['bytes'] and h.hexdigest()==record['sha256']
result=json.loads((O/'COMPLETED.json').read_text());assert result['source']==P['source'] and result['tree']==P['tree'] and result['lock_sha256']==P['lock_sha256'] and result['all_remaining_gates_pass'] is True and result['no_prior_baseline_reexecution'] is True and result['twelve_fixed_controls_rerun_on_changed_cfg_source'] is True and result['full_source_acceptance'] is False
accepted={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':P['source'],'tree':P['tree'],'lock_sha256':P['lock_sha256'],'worker_sha256':S['worker_sha256'],'prerequisites_sha256':I['prerequisites_sha256'],'reused_prior_nine_baseline_failures_three_positives_twelve_fixed_controls':True,'canonical_format_and_39_transactions_18_composed_locale_clippy_wasm_gates_pass':True,'new_test_executable_bytes_independently_rehashed':True,'new_baseline_executions':0,'twelve_current_source_fixed_controls_pass':True,'full_source_acceptance':False,'production_or_original_experiment_acceptance':False,'scientific_selections':0,'test_binaries':binary_report}
with (B/'FOCUSED-ACCEPTED.json').open('x') as f:json.dump(accepted,f,indent=2);f.write('\n')
print(json.dumps(accepted,indent=2))
