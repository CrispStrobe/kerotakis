import argparse,datetime,hashlib,json,pathlib
parser=argparse.ArgumentParser();parser.add_argument('--run-dir',type=pathlib.Path,required=True);args=parser.parse_args();B=args.run_dir
sha=lambda b:hashlib.sha256(b).hexdigest()
P=json.loads((B/'frozen-plan.json').read_text());C=json.loads((B/'COLLECTED.json').read_text())
for path,record in C['files'].items():
 raw=(B/'output'/path).read_bytes();assert len(raw)==record['bytes'] and sha(raw)==record['sha256'],path
candidates=list((B/'output').rglob('COMPLETED.json'));assert len(candidates)==1
E=candidates[0].parent;assert not (E/'STOPPED.json').exists()
manifest=json.loads((E/'manifest.json').read_text())
for path,record in manifest.items():
 raw=(E/path).read_bytes();assert len(raw)==record['bytes'] and sha(raw)==record['sha256'],path
assert json.loads((E/'frozen-plan.json').read_text())==P
started=json.loads((E/'STARTED.json').read_text());assert started['worker_sha256']==sha((B/'upload/worker.py').read_bytes()) and started['source']==P['source']
result=json.loads(candidates[0].read_text());assert result['source']==P['source'] and result['tree']==P['tree'] and result['lock_sha256']==P['lock_sha256']
for label,expected in [('baseline','2 passed; 2 failed; 0 ignored'),('fixed','4 passed; 0 failed; 0 ignored')]:
 record=json.loads((E/(label+'.json')).read_text())
 assert record['argv']==['cargo','test','--locked','-p','kerotakis-core','--test','neutral_blank_note_locale','--','--nocapture']
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==record[stream+'_sha256']
 text=(E/(label+'.stdout')).read_text();assert expected in text
 assert 'error[E' not in (E/(label+'.stderr')).read_text()
 assert record['exit_status']==(101 if label=='baseline' else 0)
 for name in ['blank_ph_caveat_is_in_the_actual_german_measurement','blank_ph_caveat_is_in_the_actual_french_measurement']:
  assert 'test '+name+' ... '+('FAILED' if label=='baseline' else 'ok') in text
 for name in ['english_blank_reading_and_legacy_caveat_are_preserved','localization_does_not_expand_the_admitted_blank_domain']:
  assert 'test '+name+' ... ok' in text
for label,command in [
 ('locale',['python3','tools/engine-locale-lint.py','--check']),
 ('composed',['cargo','test','--locked','-p','kerotakis-core','--test','composed_locale','--','--nocapture']),
 ('clippy',['cargo','clippy','--locked','-p','kerotakis-core','--all-targets','--','-D','warnings'])]:
 record=json.loads((E/(label+'.json')).read_text());assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((E/(label+'.'+stream)).read_bytes())==record[stream+'_sha256']
 if label=='composed':assert '0 failed; 0 ignored' in (E/(label+'.stdout')).read_text()
assert result['full_source_acceptance'] is False and result['scientific_selections']==0
accepted={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':P['source'],'baseline_head':P['baseline_head'],'tree':P['tree'],'lock_sha256':P['lock_sha256'],'worker_sha256':started['worker_sha256'],'two_real_locale_baseline_failures_and_two_positive_passes':True,'four_fixed_controls_and_locale_composed_clippy_pass':True,'authenticated_output_files':len(C['files']),'full_source_acceptance':False,'integration_or_production_acceptance':False,'scientific_selections':0}
with (B/'FOCUSED-ACCEPTED.json').open('x') as f:json.dump(accepted,f,indent=2);f.write('\n')
print(json.dumps(accepted,indent=2))
