import argparse,hashlib,json,pathlib,re,datetime
p=argparse.ArgumentParser();p.add_argument('--run-dir',required=True,type=pathlib.Path);B=p.parse_args().run_dir
sha=lambda b:hashlib.sha256(b).hexdigest()
P=json.loads((B/'raw/frozen-plan.json').read_text());I=json.loads((B/'issue-binding.json').read_text());C=json.loads((B/'collected.json').read_text())
assert C['version']==I['version'] and I['source']==P['source']
for name,row in C['files'].items():
 b=((B/'raw'/name.removeprefix('phase-owned-evidence/')) if name.startswith('phase-owned-evidence/') else (B/'producer.log')).read_bytes();assert sha(b)==row['sha256'] and len(b)==row['bytes']
E=B/'raw';assert not (E/'STOPPED.json').exists()
manifest=json.loads((E/'manifest.json').read_text())
for name,row in manifest.items():
 b=(E/name).read_bytes();assert sha(b)==row['sha256'] and len(b)==row['bytes']
assert json.loads((E/'frozen-plan.json').read_text())==P
S=json.loads((E/'STARTED.json').read_text());assert S['source']==P['source'] and S['worker_sha256']==I['worker_sha256']==sha((B.parent/'phase-owned-transfer-preparation-20261020/worker.py').read_bytes())
command=['cargo','test','--locked','-p','kerotakis-core','--test','phase_owned_transfers','--','--nocapture']
for label,head in [('baseline',P['baseline_head']),('fixed',P['source'])]:
 row=json.loads((E/(label+'.json')).read_text());assert row['argv']==command and row['exit_status']==(101 if label=='baseline' else 0)
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==row[stream+'_sha256']
 stdout=(E/(label+'.stdout')).read_text();assert 'error[E' not in (E/(label+'.stderr')).read_text()
 counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',stdout)
 assert len(counts)==1 and all(int(i)==0 for _,_,i in counts)
 assert sum(int(n) for n,_,_ in counts)==(2 if label=='baseline' else 8)
 assert sum(int(n) for _,n,_ in counts)==(6 if label=='baseline' else 0)
 for name in P['expected_failing']:assert 'test '+name+' ... '+('FAILED' if label=='baseline' else 'ok') in stdout
 for name in P['positive_controls']:assert 'test '+name+' ... ok' in stdout
 bins=json.loads((E/(label+'-test-binaries.json')).read_text());assert bins['source']==head and bins['lock_sha256']==P['lock_sha256'] and len(bins['binaries'])==1
 for record in bins['binaries'].values():assert len(record['sha256'])==64 and record['bytes']>0
for label,argv in [('locale',['python3','tools/engine-locale-lint.py','--check']),('composed',['cargo','test','--locked','-p','kerotakis-core','--test','composed_locale','--','--nocapture']),('clippy',['cargo','clippy','--locked','-p','kerotakis-core','--all-targets','--','-D','warnings'])]:
 row=json.loads((E/(label+'.json')).read_text());assert row['argv']==argv and row['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==row[stream+'_sha256']
 if label=='composed':assert '0 failed; 0 ignored' in (E/(label+'.stdout')).read_text()
result=json.loads((E/'COMPLETED.json').read_text());assert result['source']==P['source'] and result['tree']==P['tree'] and result['lock_sha256']==P['lock_sha256'] and result['full_source_acceptance'] is False
accepted={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':P['source'],'tree':P['tree'],'baseline_head':P['baseline_head'],'lock_sha256':P['lock_sha256'],'worker_sha256':S['worker_sha256'],'six_real_baseline_failures_and_two_positive_controls':True,'eight_fixed_controls_and_locale_composed_clippy_pass':True,'full_source_acceptance':False,'production_or_original_experiment_acceptance':False,'scientific_selections':0,'test_binary_binding_hashes':{n:sha((E/(n+'-test-binaries.json')).read_bytes()) for n in ['baseline','fixed']}}
with (B/'reproduced-acceptance.json').open('x') as f:json.dump(accepted,f,indent=2);f.write('\n')
print(json.dumps(accepted,indent=2))
