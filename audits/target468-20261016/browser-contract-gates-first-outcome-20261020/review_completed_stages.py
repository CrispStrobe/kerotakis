import argparse,ast,base64,datetime,hashlib,json,pathlib,re
def validate_prerequisites(plan):
 receipts={}
 for key in ['phase_full_prerequisite','focused_controls_prerequisite']:
  raw=base64.b64decode(plan[key]['receipt_base64'],validate=True);assert sha(raw)==plan[key]['sha256'];receipts[key]=(raw,json.loads(raw))
 full=receipts['phase_full_prerequisite'][1];controls=receipts['focused_controls_prerequisite'][1]
 assert full['source']=='b4daecf742be080577e9fda57fc1ef55aefef3d3' and full['logical_jobs']==33 and full['original_run_steps_independently_checked']==142 and len(full['law_reports'])==29
 for key in ['all_original_steps_and_named_laws_pass','all500_original_requirements_no_drift','all42_prior_and_new_native_names_present','eight_phase_owned_controls_pass','six_real_phase_baseline_failures_reused_without_execution']:assert full[key] is True,key
 assert full['protected_contexts_replaced'] is False and full['engine']['actual_producer_lock_sha256']==plan['lock_sha256']
 assert controls['source']==plan['source'] and controls['tree']==plan['tree'] and controls['lock_sha256']==plan['lock_sha256']
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
sha=lambda data:hashlib.sha256(data).hexdigest()
original=(B/'output').is_dir()
raw_root=B/('output' if original else 'raw')
O=raw_root/'browser-contract-gates-evidence'
P=json.loads((O/'frozen-plan.json').read_text())
I=json.loads((B/('ISSUE-BINDING.json' if original else 'issue-binding.json')).read_text())
C=json.loads((B/('COLLECTED.json' if original else 'collected.json')).read_text())
assert P['source']=='6b637e4d7a00847865990b259171a8f4c6c11c1e' and P['tree']=='48a68818a1aadabf54caa6eb4b5d815331f0c8de' and P['lock_sha256']=='668868136fa9ee1ebd6433d59eb68d5fa46224d874fd96c4036f7614b3c4cd85'
assert C['version']==I['version']==1 and I['source']==P['source']
assert sha(base64.b64decode(P['lock_base64'],validate=True))==P['lock_sha256']
for key,(raw,_) in validate_prerequisites(P).items():
 assert (O/(key+'.json')).read_bytes()==raw and I['prerequisites_sha256'][key]==sha(raw)
for name,row in C['files'].items():
 path=pathlib.PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
 data=(raw_root/name).read_bytes();assert sha(data)==row['sha256'] and len(data)==row['bytes']
manifest=json.loads((O/'manifest.json').read_text())
for name,row in manifest.items():
 path=pathlib.PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
 data=(O/name).read_bytes();assert sha(data)==row['sha256'] and len(data)==row['bytes']
assert json.loads((O/'frozen-plan.json').read_text())==P
for key,(raw,_) in validate_prerequisites(P).items():assert (O/(key+'.json')).read_bytes()==raw
worker=(B/('upload/worker.py' if original else 'producer.py')).read_bytes();assert ast.literal_eval(ast.parse(worker.decode()).body[0].value)==P
S=json.loads((O/'STARTED.json').read_text());assert S['source']==P['source'] and S['worker_sha256']==I['worker_sha256']==sha(worker) and S['no_focused_control_reexecution'] is True
binding=json.loads((O/'source-binding.json').read_text());assert binding=={'source':P['source'],'tree':P['tree'],'lock_sha256':P['lock_sha256'],'blobs_sha256':P['fixtures_sha256']|P['source_runtime_blobs_sha256']}
expected=[['format',['cargo','fmt','--check']],['transactions',['cargo','test','--locked','-p','kerotakis-core','--test','borrowed_solver_routes','--test','solver_transactions','--','--nocapture']],['wasm-target',['rustup','target','add','wasm32-unknown-unknown']],['wasm-check',['cargo','check','--locked','-p','kerotakis-wasm','--target','wasm32-unknown-unknown']],['locale',['python3','tools/engine-locale-lint.py','--check']],['composed',['cargo','test','--locked','-p','kerotakis-core','--test','composed_locale','--','--nocapture']],['clippy',['cargo','clippy','--locked','-p','kerotakis-core','-p','kerotakis-wasm','--all-targets','--','-D','warnings']]]
assert P['commands']==expected and len(P['existing_transaction_controls'])==38 and P['borrowed_control_names']==['borrowed_adapter_cannot_bypass_required_material_conservation'] and len(P['composed_control_names'])==18
for name,argv in expected:
 row=json.loads((O/(name+'.json')).read_text());assert row['argv']==argv and row['exit_status']==(101 if name=='clippy' else 0)
 for stream in ['stdout','stderr']:assert sha((O/(name+'.'+stream)).read_bytes())==row[stream+'_sha256']
 if name=='transactions':test_results(name,P['existing_transaction_controls']+P['borrowed_control_names'],[1,38])
 if name=='composed':test_results(name,P['composed_control_names'],[18])
stop=json.loads((O/'STOPPED.json').read_text());assert stop['error']=='clippy' and stop['error_type']=='AssertionError' and stop['no_auto_retry'] is True
stderr=(O/'clippy.stderr').read_text();assert 'error: associated function `new` is never used' in stderr and 'crates/kerotakis-phreeqc/src/aqueous.rs:412:8' in stderr and 'impl PathResult' in stderr and '-D dead-code' in stderr
errors=re.findall(r'^error: (.+)$',stderr,re.M);assert len(errors)==2 and errors[1]=='could not compile `kerotakis-phreeqc` (lib) due to 1 previous error'
assert not (O/'COMPLETED.json').exists() and not (O/'test-binaries.json').exists() and not list(O.glob('*.gz'))
accepted={'reviewed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':P['source'],'tree':P['tree'],'lock_sha256':P['lock_sha256'],'worker_sha256':sha(worker),'prerequisites_sha256':I['prerequisites_sha256'],'canonical_format_pass':True,'exact_one_borrowed_and_38_existing_transactions_pass':True,'exact18_composed_locale_controls_pass':True,'locale_and_wasm_target_compile_pass':True,'clippy_pass':False,'clippy_stop':'PathResult::new unused without native engine feature','completed_test_executables_retained':False,'completed_test_executables_independently_rehashed':False,'original_baseline_or_12_controls_reexecuted':False,'full_focused_acceptance':False,'full_source_acceptance':False,'production_or_original_experiment_acceptance':False,'scientific_selections':0}
print(json.dumps(accepted,indent=2))
