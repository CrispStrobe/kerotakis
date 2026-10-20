import argparse,hashlib,json,pathlib,re,datetime
p=argparse.ArgumentParser();p.add_argument('--run-dir',required=True,type=pathlib.Path);B=p.parse_args().run_dir
sha=lambda b:hashlib.sha256(b).hexdigest()
P=json.loads((B/'frozen-plan.json').read_text());I=json.loads((B/'ISSUE-BINDING.json').read_text());C=json.loads((B/'COLLECTED.json').read_text())
assert C['version']==I['version'] and I['source']==P['source']
prerequisite=P['phase_full_prerequisite'];assert prerequisite
import base64,gzip
prior=base64.b64decode(prerequisite['receipt_base64']);assert sha(prior)==prerequisite['sha256']==I['prerequisite_sha256']
assert (B/'full-prerequisite-acceptance.json').read_bytes()==prior
accepted=json.loads(prior);assert accepted['source']=='b4daecf742be080577e9fda57fc1ef55aefef3d3' and accepted['logical_jobs']==33 and accepted['original_run_steps_independently_checked']==142 and len(accepted['law_reports'])==29 and accepted['all500_original_requirements_no_drift'] and accepted['all42_prior_and_new_native_names_present']
for name,row in C['files'].items():
 b=(B/'output'/name).read_bytes();assert sha(b)==row['sha256'] and len(b)==row['bytes']
E=B/'output/browser-contract-evidence';assert json.loads((E/'STOPPED.json').read_text())['error']=='format'
manifest=json.loads((E/'manifest.json').read_text())
for name,row in manifest.items():
 b=(E/name).read_bytes();assert sha(b)==row['sha256'] and len(b)==row['bytes']
assert json.loads((E/'frozen-plan.json').read_text())==P
S=json.loads((E/'STARTED.json').read_text());assert S['source']==P['source'] and S['worker_sha256']==I['worker_sha256']==sha((B/'upload/worker.py').read_bytes())
command=['cargo','test','--locked','-p','kerotakis-wasm','--lib','solver_contract_tests','--','--nocapture']
for label,head in [('baseline',P['baseline_head']),('fixed',P['source'])]:
 row=json.loads((E/(label+'.json')).read_text());assert row['argv']==command and row['exit_status']==(101 if label=='baseline' else 0)
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==row[stream+'_sha256']
 stdout=(E/(label+'.stdout')).read_text();assert 'error[E' not in (E/(label+'.stderr')).read_text()
 counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',stdout)
 assert len(counts)==1 and all(int(i)==0 for _,_,i in counts)
 assert sum(int(n) for n,_,_ in counts)==(3 if label=='baseline' else 12)
 assert sum(int(n) for _,n,_ in counts)==(9 if label=='baseline' else 0)
 for name in P['expected_failing']:assert 'test '+name+' ... '+('FAILED' if label=='baseline' else 'ok') in stdout
 for name in P['positive_controls']:assert 'test '+name+' ... ok' in stdout
 bins=json.loads((E/(label+'-test-binaries.json')).read_text());assert bins['source']==head and bins['lock_sha256']==P['lock_sha256'] and len(bins['binaries'])==1
 for record in bins['binaries'].values():
  compressed=E/record['gzip'];assert compressed.name==record['gzip'] and sha(compressed.read_bytes())==record['gzip_sha256']
  h=hashlib.sha256();length=0
  with gzip.open(compressed,'rb') as src:
   while chunk:=src.read(262144):
    length+=len(chunk);assert length<=record['bytes'];h.update(chunk)
  assert h.hexdigest()==record['sha256'] and length==record['bytes'] and length>0
row=json.loads((E/'format.json').read_text());assert row['argv']==['cargo','fmt','--all','--','--check'] and row['exit_status']==1
paths=set(re.findall(r'^Diff in ([^:]+):',(E/'format.stdout').read_text(),re.M));assert paths and all('/sundials-kinetics-rs/' in p for p in paths)
assert not (E/'borrowed-and-existing-transactions.json').exists() and not (E/'COMPLETED.json').exists()
accepted={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':P['source'],'tree':P['tree'],'baseline_head':P['baseline_head'],'lock_sha256':P['lock_sha256'],'worker_sha256':S['worker_sha256'],'nine_real_baseline_failures_and_three_positive_controls':True,'twelve_fixed_controls_pass':True,'actual_test_executable_bytes_independently_rehashed':True,'format_all_stopped_only_unchanged_third_party':True,'remaining_fmt_39_transactions_locale_composed_clippy_wasm_gates_accepted':False,'full_source_acceptance':False,'production_or_original_experiment_acceptance':False,'scientific_selections':0,'test_binary_bindings':{label:json.loads((E/(label+'-test-binaries.json')).read_text()) for label in ['baseline','fixed']}}
with (B/'CONTROLS-ACCEPTED.json').open('x') as f:json.dump(accepted,f,indent=2);f.write('\n')
print(json.dumps(accepted,indent=2))
