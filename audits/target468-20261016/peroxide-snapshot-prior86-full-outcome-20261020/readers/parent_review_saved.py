import base64
import os, pathlib, sys
memory=int(next(line.split()[1] for line in pathlib.Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemAvailable:')))//1024
if memory<3200 or os.getloadavg()[0]>3.5:
    print('Saved review resource gate closed; no acceptance assigned.')
    sys.exit(75)
import ast,datetime,hashlib,json,pathlib,subprocess,tarfile,yaml
import argparse
parser=argparse.ArgumentParser();parser.add_argument('--run-dir',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parent);parser.add_argument('--repo',type=pathlib.Path,required=True);args=parser.parse_args()
B=args.run_dir;R=args.repo;E=B/'output/full-fleet-evidence';P=json.loads((B/'frozen-plan.json').read_text());sha=lambda raw:hashlib.sha256(raw).hexdigest()
def expected_test_line(name):
 return 'test '+name+(' - should panic' if name=='an_unresolved_static_phrase_key_fails_closed' else '')+' ... ok'
def stream_digest(stream, expected_bytes=None):
 h=hashlib.sha256();length=0
 while chunk:=stream.read(262144):h.update(chunk);length+=len(chunk)
 if expected_bytes is not None:assert length==expected_bytes
 return h.hexdigest()
def file_digest(path):
 path=pathlib.Path(path)
 with path.open('rb') as stream:return stream_digest(stream,path.stat().st_size)

source=P['reviewed_head'];get=lambda path:subprocess.check_output(['git','show',source+':'+path],cwd=R)
assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=R,text=True).strip()==P['tree']
workflow=get('.github/workflows/chemistry-audit.yml');assert sha(workflow)==P['workflow_sha256'];wf=yaml.safe_load(workflow)
worker=(B/'upload/full_fleet.py').read_bytes();start=json.loads((E/'STARTED.json').read_text());assert sha(worker)==start['script_sha256']==json.loads((B/'PUSH-ISSUED.json').read_text())['worker_sha256']
def decode_plan_transport(value):
 import base64,hashlib,json,zlib
 maximum=16*1024*1024
 assert type(value) is dict and set(value)=={'encoding','decoded_bytes','decoded_sha256','payload'}
 assert value['encoding']=='base64-zlib-json-v1' and type(value['decoded_bytes']) is int and 0<value['decoded_bytes']<=maximum
 assert type(value['decoded_sha256']) is str and len(value['decoded_sha256'])==64 and all(c in '0123456789abcdef' for c in value['decoded_sha256'])
 assert type(value['payload']) is str and len(value['payload'])<=((maximum+2)//3)*4
 packed=base64.b64decode(value['payload'],validate=True)
 assert base64.b64encode(packed).decode('ascii')==value['payload'] and len(packed)<=maximum
 decoder=zlib.decompressobj()
 raw=decoder.decompress(packed,maximum+1)
 assert len(raw)==value['decoded_bytes'] and len(raw)<=maximum
 assert decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail,'Truncated, concatenated or excess zlib transport'
 assert hashlib.sha256(raw).hexdigest()==value['decoded_sha256']
 def unique_pairs(pairs):
  result={}
  for key,item in pairs:
   assert key not in result,'Duplicate JSON plan key'
   result[key]=item
  return result
 def reject_constant(text):raise ValueError('Nonfinite JSON plan value: '+text)
 result=json.loads(raw.decode('utf-8'),object_pairs_hook=unique_pairs,parse_constant=reject_constant)
 assert type(result) is dict
 return result
transport=ast.literal_eval(ast.parse(worker).body[0].value);embedded=decode_plan_transport(transport);assert embedded==json.loads((E/'frozen-plan.json').read_text());assert all(P[k]==v for k,v in embedded.items())
for path,digest in P['source_native_manifest_files_sha256'].items():assert sha(get(path))==digest,path
hashes=json.loads((E/'SHA256.json').read_text())
for path,digest in hashes.items():assert file_digest(E/path)==digest,path
col=json.loads((B/'COLLECTED.json').read_text())
for path,row in col['files'].items():assert file_digest(B/'output'/path)==row['sha256'] and (B/'output'/path).stat().st_size==row['bytes'],path
binding=json.loads((E/'engine-binding.json').read_text());assert binding['source']==source and binding['tree']==P['tree'];assert sha((E/'producer-Cargo.lock').read_bytes())==binding['actual_producer_lock_sha256']
assert len(P['jobs'])==33;step_count=0
for job in P['jobs']:
 original=wf['jobs'][job['key']];family=job['family'];expected=[]
 for i,step in enumerate(original['steps']):
  if 'run' not in step:continue
  command=step['run'].replace('${{ matrix.family }}',family or '').replace('${{ github.sha }}',source)
  expected.append((i,command))
 assert [(s['index'],s['run']) for s in job['steps']]==expected,job['name']
 name=job['key']+('-'+family if family else '')
 for step in job['steps']:
  root=E/name/('step-'+str(step['index']));d=json.loads(root.with_suffix('.json').read_text());assert d['argv']==['bash','-e','-o','pipefail','-c',step['run']] and d['exit_status']==0,(name,step['index'])
  for stream in ('stdout','stderr'):assert sha(root.with_suffix('.'+stream).read_bytes())==d[stream+'_sha256']
  step_count+=1
 assert json.loads((E/name/'source-unchanged.json').read_text())['exit_status']==0
finished=json.loads((E/'COMPLETED.json').read_text());assert finished['source']==source and finished['all33_original_jobs_pass'] and len(finished['logical_jobs'])==33
reports=[];artifact_hashes={};actual_cli_archive_found=False
with tarfile.open(E/'actual-artifacts.tar.gz') as archive:
 for m in archive.getmembers():
  if not m.isfile():continue
  with archive.extractfile(m) as stream:digest=stream_digest(stream,m.size)
  artifact_hashes[m.name]=digest
  if m.name.endswith('/target/release/kero'):assert digest==binding['binary_sha256'];actual_cli_archive_found=True
  if m.name.endswith('/Cargo.lock'):assert digest==binding['actual_producer_lock_sha256']
  if m.name.endswith('law-checks.json'):
   with archive.extractfile(m) as stream:d=json.load(stream)
   assert d['unmet']==0,m.name
   assert all(check.get('passed',False) for check in d['checks']),m.name
   if 'commit' in d:assert d['commit']==source
   if 'binary_sha256' in d:assert d['binary_sha256']==binding['binary_sha256']
   reports.append({'artifact_member':m.name,'sha256':digest,'checks':len(d['checks']),'unmet':d.get('unmet',[])})
assert actual_cli_archive_found
assert len(reports)==29,len(reports)
aggregates={}
for key in ('source-fleet-aggregate','source-fleet-v2-aggregate','source-fleet-v3-aggregate'):
 d=json.loads((E/key/'step-2.stdout').read_text());assert d['unmet_families']==[];aggregates[key]=d

import re,gzip
assert not (E/'STOPPED.json').exists()
assert step_count==142
native_results=[]
required_core=['refused_drain_creates_no_receiver_and_moves_no_material','refused_extraction_preserves_the_numerical_reason_and_inventory','globally_invalid_transfer_state_keeps_error_and_rollback_contract','ordinary_supported_drain_retains_both_solute_shares_and_total','checked_layer_detection_and_mixing_preserve_refused_state','invalid_column_coordinates_refuse_without_panicking_or_mutating_inventory','globally_invalid_temperatures_keep_the_bench_error_and_rollback_contract','invalid_extraction_coordinate_refuses_without_panicking_or_moving_matter','methanol_column_uses_the_dedicated_subgroup_reference_at_both_temperatures','a_partial_sample_keeps_its_supported_peak_and_missing_data_note','lessons_produce_stable_output']
required_core += ['blank_ph_caveat_is_in_the_actual_german_measurement', 'blank_ph_caveat_is_in_the_actual_french_measurement', 'english_blank_reading_and_legacy_caveat_are_preserved', 'localization_does_not_expand_the_admitted_blank_domain', 'dynamic_temperature_windows_are_localized_at_multiple_coordinates', 'missing_ph_and_catalyst_alternatives_keep_localized_reasons', 'medium_light_and_unknown_solvent_labels_use_localized_explanations', 'required_and_forbidden_group_explanations_are_localized', 'gate_order_english_legacy_and_machine_identity_are_preserved', 'shipped_family_terms_and_all_gate_labels_have_catalogue_entries', 'dynamic_english_temperature_and_catalyst_messages_remain_exact']
required_core += P['phase_owned_controls'] + P['borrowed_controls'] + P['peroxide_heat_controls']
required_thermo=['checked_binary_lle_preserves_invalid_coordinate_and_group_errors','phase_scan_stops_at_the_first_failed_evaluation','checked_refusals_keep_the_invalid_coordinate_typed','checked_infinite_dilution_propagates_a_missing_group_without_mutation','unifac::tests::missing_forward_interaction_is_a_named_refusal','unifac::tests::reverse_interaction_does_not_stand_in_for_missing_direction']
required_thermo += ['underflowed_directional_weights_refuse_even_if_final_gammas_look_finite','underflowed_weights_cannot_supply_a_phase_verdict','ordinary_directional_weights_keep_the_existing_two_phase_control','nonrepresentable_interaction_factor_keeps_its_direction']
for n,command in enumerate(P['focused_native_commands']):
 root=E/('combined-native-'+str(n));record=json.loads(root.with_suffix('.json').read_text());assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha(root.with_suffix('.'+stream).read_bytes())==record[stream+'_sha256']
 raw=root.with_suffix('.stdout').read_text();passed=0
 if n in [2,3,4,5]:
  counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw);assert counts and 'FAILED' not in raw and all(int(f)==0 and int(i)==0 for _,f,i in counts)
  passed=sum(int(a) for a,_,_ in counts);assert passed>={2:620,3:P['required_total_core_tests'],4:2,5:184}[n],(n,passed)
  for name in (required_core if n==3 else required_thermo if n==5 else []):assert 'test '+name+' ... ok' in raw,name
 native_results.append({'command':command,'exit_status':0,'tests_passed':passed,'stdout_sha256':record['stdout_sha256'],'stderr_sha256':record['stderr_sha256']})
for index,command in enumerate(P['additional_gate_commands']):
 root=E/('additional-gate-'+str(index));record=json.loads(root.with_suffix('.json').read_bytes());assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha(root.with_suffix('.'+stream).read_bytes())==record[stream+'_sha256']
report=json.loads((E/'additional-gate-1.stdout').read_bytes());assert len(report['prompts'])==500 and report['unmet_requirements']==0 and not report['failures'] and not report['baseline_drift']
assert P['primary_event_diagnostics']==[]
for name,digest in P['reused_liquid_baseline_and_first_failure_sha256'].items():assert sha((E/'prior-liquid-evidence'/name).read_bytes())==digest,name
prior=E/'prior-liquid-evidence';raw=(prior/'baseline.stdout').read_text();assert '1 passed; 3 failed; 0 ignored' in raw and 'error[E' not in (prior/'baseline.stderr').read_text()
for name in required_core[:3]:assert name+' ... FAILED' in raw,name
assert required_core[3]+' ... ok' in raw
raw=(prior/'fixed-2.stderr').read_text();assert 'UNIFAC calculation refused' in raw and 'error[E' not in raw
for ref in [source,P['reused_liquid_baseline_source']]:assert sha(subprocess.check_output(['git','show',ref+':'+P['baseline_fixture']],cwd=R))==P['baseline_fixture_sha256']
factor=json.loads((E/'factor-baseline.json').read_text());assert factor['exit_status']==101 and factor['argv']==P['factor_baseline_command']
for stream in ['stdout','stderr']:assert sha((E/('factor-baseline.'+stream)).read_bytes())==factor[stream+'_sha256']
raw=(E/'factor-baseline.stdout').read_text();assert '1 passed; 2 failed; 0 ignored' in raw and 'error[E' not in (E/'factor-baseline.stderr').read_text()
for name in P['factor_expected_failing_tests']:assert name+' ... FAILED' in raw
assert P['factor_positive_control']+' ... ok' in raw
for ref in [source,P['factor_baseline_head']]:assert sha(subprocess.check_output(['git','show',ref+':'+P['factor_fixture']],cwd=R))==P['factor_fixture_sha256']
mac=(B/'retained-first-macos.log').read_bytes();assert sha(mac)==P['first_actual_macos_failure']['log_sha256']
for name in P['first_actual_macos_failure']['unchanged_failed_tests']:assert (name+' ... FAILED').encode() in mac
# No parameter or numerical oracle values are introduced by this precursor.
for name in ['crates/kerotakis-thermo/tests/fixtures/thermo_oracle.tsv','crates/kerotakis-thermo/tests/thermo_oracle.rs']:
 assert get(name)==subprocess.check_output(['git','show','0b579699:'+name],cwd=R)
old=subprocess.check_output(['git','show','0b579699:crates/kerotakis-thermo/src/unifac.rs'],cwd=R);new=get('crates/kerotakis-thermo/src/unifac.rs');section=lambda data:data.split(b'const SOURCE_1975:',1)[1].split(b'pub type GroupDecomposition',1)[0];assert section(old)==section(new)
debug=json.loads((E/'debug-engine-binding.json').read_bytes());assert debug['source']==source and debug['actual_producer_lock_sha256']==P['expected_producer_lock_sha256']
with gzip.open(E/'actual-debug-kero.gz','rb') as f:assert stream_digest(f)==debug['binary_sha256']
assert binding['actual_producer_lock_sha256']==P['expected_producer_lock_sha256']
for label,bundle in P['reused_locale_evidence'].items():
 root=E/'prior-locale-evidence'/label
 for name,digest in bundle['files_sha256'].items():assert sha((root/name).read_bytes())==digest
 receipt=json.loads((root/'FOCUSED-ACCEPTED.json').read_text());assert receipt['source']==bundle['fixed_source'] and receipt['baseline_head']==bundle['baseline_head'] and receipt['full_source_acceptance'] is False
 for path,digest in bundle['fixture_sha256'].items():
  for ref in [source,bundle['fixed_source'],bundle['baseline_head']]:assert sha(subprocess.check_output(['git','show',ref+':'+path],cwd=R))==digest
 text=(root/'baseline.stdout').read_text();record=json.loads((root/'baseline.json').read_text());assert record['exit_status']==101 and 'error[E' not in (root/'baseline.stderr').read_text()
 for stream in ['stdout','stderr']:assert sha((root/('baseline.'+stream)).read_bytes())==record[stream+'_sha256']
 if label=='blank':assert '2 passed; 2 failed; 0 ignored' in text and receipt['four_fixed_controls_and_locale_composed_clippy_pass']
 else:
  counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert len(counts)==3 and sum(int(p) for p,_,_ in counts)==2 and sum(int(f) for _,f,_ in counts)==5 and all(int(i)==0 for _,_,i in counts)
  assert receipt['seven_fixed_controls_and_locale_composed_clippy_pass']
# Exact-source focused baseline acceptance is retained, not executed again.
focus=P['phase_owned_focused_evidence'];assert focus and focus['accepted']
froot=E/'prior-phase-owned-evidence';expected=P['phase_owned_focus_expected']
for path,digest in focus['files_sha256'].items():assert sha((froot/path).read_bytes())==digest,path
receipt=json.loads((froot/'metadata/FOCUSED-ACCEPTED.json').read_text())
assert sha((froot/'metadata/FOCUSED-ACCEPTED.json').read_bytes())==focus['receipt_sha256']
assert receipt['source']==expected['source'] and receipt['tree']==expected['tree'] and receipt['baseline_head']==expected['baseline_head'] and receipt['lock_sha256']==P['expected_producer_lock_sha256']
assert receipt['six_real_baseline_failures_and_two_positive_controls'] and receipt['eight_fixed_controls_and_locale_composed_clippy_pass'] and receipt['full_source_acceptance'] is False
fp=json.loads((froot/'metadata/frozen-plan.json').read_text());assert fp['source']==expected['source'] and fp['tree']==expected['tree'] and fp['baseline_head']==expected['baseline_head']
for ref in [expected['source'],expected['baseline_head']]:assert sha(subprocess.check_output(['git','show',ref+':'+expected['fixture']],cwd=R))==expected['fixture_sha256']
fi=json.loads((froot/'metadata/ISSUE-BINDING.json').read_text());fc=json.loads((froot/'metadata/COLLECTED.json').read_text());assert fi['source']==expected['source'] and fc['version']==fi['version']
assert sha((froot/'metadata/worker.py').read_bytes())==fi['worker_sha256']==receipt['worker_sha256']
for path,row in fc['files'].items():
 raw=(froot/'output'/path).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
fe=froot/'output/phase-owned-evidence';assert not (fe/'STOPPED.json').exists()
assert json.loads((fe/'frozen-plan.json').read_text())==fp
for path,row in json.loads((fe/'manifest.json').read_text()).items():
 raw=(fe/path).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
started=json.loads((fe/'STARTED.json').read_text());assert started['source']==expected['source'] and started['worker_sha256']==fi['worker_sha256']
for label,head,passed,failed in [('baseline',expected['baseline_head'],2,6),('fixed',expected['source'],8,0)]:
 rec=json.loads((fe/(label+'.json')).read_text());assert rec['argv']==['cargo','test','--locked','-p','kerotakis-core','--test','phase_owned_transfers','--','--nocapture'] and rec['exit_status']==(101 if label=='baseline' else 0)
 for stream in ['stdout','stderr']:assert sha((fe/(label+'.'+stream)).read_bytes())==rec[stream+'_sha256']
 text=(fe/(label+'.stdout')).read_text();assert 'error[E' not in (fe/(label+'.stderr')).read_text()
 counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert counts==[(str(passed),str(failed),'0')]
 for name in expected['expected_failing']:assert 'test '+name+' ... '+('FAILED' if label=='baseline' else 'ok') in text
 for name in expected['positive_controls']:assert 'test '+name+' ... ok' in text
 bins=json.loads((fe/(label+'-test-binaries.json')).read_text());assert bins['source']==head and bins['lock_sha256']==P['expected_producer_lock_sha256'] and len(bins['binaries'])==1
 for entry in bins['binaries'].values():assert len(entry['sha256'])==64 and entry['bytes']>0
for label,command in [('locale',['python3','tools/engine-locale-lint.py','--check']),('composed',['cargo','test','--locked','-p','kerotakis-core','--test','composed_locale','--','--nocapture']),('clippy',['cargo','clippy','--locked','-p','kerotakis-core','--all-targets','--','-D','warnings'])]:
 rec=json.loads((fe/(label+'.json')).read_text());assert rec['argv']==command and rec['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((fe/(label+'.'+stream)).read_bytes())==rec[stream+'_sha256']
complete=json.loads((fe/'COMPLETED.json').read_text());assert complete['source']==expected['source'] and complete['tree']==expected['tree'] and complete['lock_sha256']==P['expected_producer_lock_sha256'] and complete['full_source_acceptance'] is False
# Full current-source runs must preserve all prior34 and new8 named controls.
all_native='\n'.join((E/('combined-native-'+str(n)+'.stdout')).read_text() for n in [2,3,4,5])
platform_names=P['prior_platform_controls']+P['phase_owned_controls'];assert len(set(platform_names))==42
for name in platform_names:
 assert re.search(r'^test (?:[A-Za-z0-9_]+::)*'+re.escape(name)+r' \.\.\. ok$',all_native,re.M),name
# Separate current-source cache-only commands and all18 actual host units.
for name,command in P['cache_only_commands']:
 record=json.loads((E/(name+'.json')).read_text());assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((E/(name+'.'+stream)).read_bytes())==record[stream+'_sha256']
text=(E/'cache-only-host-tests.stdout').read_text();actual=re.findall(r'^test ([\w:]+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)$',text,re.M)
assert len(actual)==18 and {n for n,_ in actual}==set(P['cache_only_host_names']) and all(state=='ok' for _,state in actual)
assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)==[('18','0','0')]
all_owned=platform_names+P['browser_controls']+P['borrowed_controls']+P['peroxide_heat_controls'];assert set(all_owned)==set(P['required_platform_controls']) and len(set(all_owned))==P['expected_required_platform_controls']==59
for name in P['borrowed_controls']:assert 'test '+name+' ... ok' in (E/'combined-native-3.stdout').read_text()
# Real executables are archived on every cargo test stage, not inferred from exit0.
for record_path in E.glob('*-test-binaries.json'):
 report=json.loads(record_path.read_text());assert report['source']==source and report['tree']==P['tree'] and report['lock_sha256']==P['expected_producer_lock_sha256'] and report['binaries']
 for item in report['binaries']:
  path=pathlib.PurePosixPath(item['gzip']);assert not path.is_absolute() and '..' not in path.parts
  assert file_digest(E/path)==item['gzip_sha256']
  h=hashlib.sha256();length=0
  with gzip.open(E/path,'rb') as src:
   while chunk:=src.read(262144):length+=len(chunk);assert length<=item['bytes'];h.update(chunk)
  assert length==item['bytes'] and h.hexdigest()==item['sha256']
for name in ['cache-only-host-tests','combined-native-2','combined-native-3','combined-native-4','combined-native-5','additional-gate-0','additional-gate-3','additional-gate-4']:assert (E/(name+'-test-binaries.json')).is_file(),name
# Embedded prior101 receipt is explicitly prior-only; source binding belongs to this producer.
for key in ['phase_integration_evidence','combined_source_review','browser_focus_prior']:
 entry=P[key];assert entry;raw=base64.b64decode(entry['receipt_base64'],validate=True);assert sha(raw)==entry['sha256']
 target='prior-browser-focused-accepted.json' if key=='browser_focus_prior' else key+'.json';assert (E/target).read_bytes()==raw
phase=json.loads((E/'phase_integration_evidence.json').read_text())
assert phase['pr']==846 and phase['head']=='b4daecf742be080577e9fda57fc1ef55aefef3d3' and len(phase['parents'])==2 and phase['parents'][1]==phase['head']
assert phase['actual_tree']==phase['reconstructed_tree'] and phase['ci25_pass'] is True and phase['both42_name_platforms_accepted'] is True and phase['all33_original_jobs_accepted'] is True
actual_raw=subprocess.check_output(['git','cat-file','-p',phase['merge']],cwd=R,text=True);parents=[line.split()[1] for line in actual_raw.splitlines() if line.startswith('parent ')]
assert parents==phase['parents'] and subprocess.check_output(['git','rev-parse',phase['merge']+'^{tree}'],cwd=R,text=True).strip()==phase['actual_tree']
record=json.loads((E/'phase-integration-fetch.json').read_text());assert record['argv']==['git','fetch','--depth','1','origin',phase['merge']] and record['exit_status']==0
for stream in ['stdout','stderr']:assert sha((E/('phase-integration-fetch.'+stream)).read_bytes())==record[stream+'_sha256']
review=json.loads((E/'combined_source_review.json').read_text());assert review['source']==source and review['tree']==P['tree'] and review['phase_integration_is_ancestor'] is True and review['browser_workflow_head_is_ancestor'] is True
# Additional bundled source controls are genuine current-source execution, not prior-only reuse.
record=json.loads((E/'ci-classifier-tests.json').read_text());assert record['argv']==P['classifier_command'] and record['exit_status']==0
for stream in ['stdout','stderr']:assert file_digest(E/('ci-classifier-tests.'+stream))==record[stream+'_sha256']
classifier=(E/'ci-classifier-tests.stderr').read_text();assert re.search(r'Ran 11 tests in ',classifier) and classifier.rstrip().endswith('OK')
for name in P['classifier_expected_names']:assert re.search(r'^'+re.escape(name)+r' \(.+\) \.\.\. ok$',classifier,re.M),name
assert len(P['classifier_expected_names'])==11
for group,prefix in [('prior_full32_receipts','prior32-'),('peroxide_focus_receipts','priorheat-')]:
 for name,entry in P[group].items():
  raw=base64.b64decode(entry['receipt_base64'],validate=True);assert sha(raw)==entry['sha256'] and (E/(prefix+name)).read_bytes()==raw
for name,entry in P['peroxide_focused_raw_small'].items():assert file_digest(E/'prior-peroxide-focused-raw'/name)==entry['sha256']
for key,target in [('bundle_source_review','bundle-source-review.json'),('actual848_integration','prior848-integration.json')]:
 entry=P[key];raw=base64.b64decode(entry['receipt_base64'],validate=True);assert sha(raw)==entry['sha256'] and (E/target).read_bytes()==raw
assert sha(get('crates/kerotakis-core/src/clock.rs'))==P['peroxide_clock_sha256'] and sha(get('crates/kerotakis-core/tests/peroxide_heat_basis.rs'))==P['peroxide_fixture_sha256']
entry=P['successor_parent_source_review'];raw=base64.b64decode(entry['receipt_base64'],validate=True);assert sha(raw)==entry['sha256'];assert (E/'successor-parent-source-review.json').read_bytes()==raw
parent_review=json.loads(raw);assert parent_review['source']==source and parent_review['tree']==P['tree']
for key in ['golden_matches_both_authenticated_platform_artifacts','clock_and_four_control_fixture_match_accepted_focus','ci_runtime_commands_unchanged_from_independently_reviewed_first_head','eleven_classifier_tests_rechecked','combined_diff_reviewed','phase_debits_preserved','no_test_or_original_criterion_weakened']:assert parent_review[key] is True,key
result={'reviewed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':source,'tree':P['tree'],'engine':binding,'logical_jobs':33,'original_run_steps_independently_checked':step_count,'law_reports':reports,'aggregates':aggregates,'authenticated_evidence_files':len(hashes),'authenticated_artifact_files':len(artifact_hashes),'worker_sha256':sha(worker),'artifact_archive_sha256':file_digest(E/'actual-artifacts.tar.gz'),'all_original_steps_and_named_laws_pass':True,'all500_original_requirements_no_drift':True,'focused_native_results':native_results,'first_liquid_failure_and_baseline_preserved':True,'two_factor_failures_and_positive_preserved':True,'actual_first_macos_failure_preserved':True,'approved_parameters_and_independent_reference_values_unchanged':True,'protected_contexts_replaced':False,'integration_or_deployment_claim':False,'eleven_combined_locale_controls_pass':True,'real_locale_baseline_failures_reused_without_execution':True,'eight_phase_owned_controls_pass':True,'six_real_phase_baseline_failures_reused_without_execution':True,'all42_prior_and_new_native_names_present':True,'all55_prior_native_and_browser_controls_present':True,'all59_native_browser_and_peroxide_controls_present':True,'eleven_ci_classifier_tests_pass':True,'four_peroxide_heat_controls_pass':True,'real_peroxide_baseline_and_fixed_reused_without_execution':True,'all18_cache_only_host_controls_pass':True,'actual_executed_test_binaries_rehashed':True,'phase_focused_receipt_sha256':focus['receipt_sha256'],'catalog_additions':0}
with (B/'ROOT-PARENT-ACCEPTED.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result))
