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
E=B/'output/browser-contract-evidence';assert not (E/'STOPPED.json').exists()
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
for label,argv in [('locale',['python3','tools/engine-locale-lint.py','--check']),('composed',['cargo','test','--locked','-p','kerotakis-core','--test','composed_locale','--','--nocapture']),('clippy',['cargo','clippy','--locked','-p','kerotakis-core','-p','kerotakis-wasm','--all-targets','--','-D','warnings'])]:
 row=json.loads((E/(label+'.json')).read_text());assert row['argv']==argv and row['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==row[stream+'_sha256']
 if label=='composed':assert '0 failed; 0 ignored' in (E/(label+'.stdout')).read_text()
for label,command in [('format',['cargo','fmt','--all','--','--check']),('borrowed-and-existing-transactions',['cargo','test','--locked','-p','kerotakis-core','--test','borrowed_solver_routes','--test','solver_transactions','--','--nocapture']),('wasm-target',['rustup','target','add','wasm32-unknown-unknown']),('wasm-check',['cargo','check','--locked','-p','kerotakis-wasm','--target','wasm32-unknown-unknown'])]:
 row=json.loads((E/(label+'.json')).read_text());assert row['argv']==command and row['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==row[stream+'_sha256']
 if label=='borrowed-and-existing-transactions':
  text=(E/(label+'.stdout')).read_text();assert 'test borrowed_adapter_cannot_bypass_required_material_conservation ... ok' in text
  counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert sorted(int(p) for p,_,_ in counts)==[1,38] and all(int(f)==0 and int(i)==0 for _,f,i in counts)
  for name in P['existing_transaction_controls']:assert 'test '+name+' ... ok' in text
assert (E/'prior-full-acceptance.json').read_bytes()==prior
result=json.loads((E/'COMPLETED.json').read_text());assert result['source']==P['source'] and result['tree']==P['tree'] and result['lock_sha256']==P['lock_sha256'] and result['full_source_acceptance'] is False
accepted={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':P['source'],'tree':P['tree'],'baseline_head':P['baseline_head'],'lock_sha256':P['lock_sha256'],'worker_sha256':S['worker_sha256'],'nine_real_baseline_failures_and_three_positive_controls':True,'twelve_fixed_controls_and_locale_composed_clippy_pass':True,'full_source_acceptance':False,'production_or_original_experiment_acceptance':False,'scientific_selections':0,'test_binary_binding_hashes':{n:sha((E/(n+'-test-binaries.json')).read_bytes()) for n in ['baseline','fixed']}}
with (B/'FOCUSED-ACCEPTED.json').open('x') as f:json.dump(accepted,f,indent=2);f.write('\n')
print(json.dumps(accepted,indent=2))
