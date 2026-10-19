import os, pathlib, sys, argparse
parser=argparse.ArgumentParser();parser.add_argument("--run-dir",type=pathlib.Path,required=True);args=parser.parse_args()
REPO=pathlib.Path(__file__).resolve().parents[3]
memory=int(next(line.split()[1] for line in pathlib.Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemAvailable:')))//1024
if memory<3200 or os.getloadavg()[0]>3.5:
    print('Saved review resource gate closed; no acceptance assigned.')
    sys.exit(75)
import ast,datetime,hashlib,json,pathlib,subprocess,tarfile,yaml
B=args.run_dir;E=B/'output/full-fleet-evidence';P=json.loads((B/'frozen-plan.json').read_text());sha=lambda raw:hashlib.sha256(raw).hexdigest()
def expected_test_line(name):
 return 'test '+name+(' - should panic' if name=='an_unresolved_static_phrase_key_fails_closed' else '')+' ... ok'
source=P['reviewed_head'];get=lambda path:subprocess.check_output(['git','show',source+':'+path],cwd=REPO)
assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=REPO,text=True).strip()==P['tree']
workflow=get('.github/workflows/chemistry-audit.yml');assert sha(workflow)==P['workflow_sha256'];wf=yaml.safe_load(workflow)
worker=(B/'upload/full_fleet.py').read_bytes();start=json.loads((E/'STARTED.json').read_text());assert sha(worker)==start['script_sha256']==json.loads((B/'PUSH-ISSUED.json').read_text())['worker_sha256']
embedded=ast.literal_eval(ast.parse(worker).body[0].value);assert embedded==json.loads((E/'frozen-plan.json').read_text());assert all(P[k]==v for k,v in embedded.items())
for path,digest in P['source_native_manifest_files_sha256'].items():assert sha(get(path))==digest,path
hashes=json.loads((E/'SHA256.json').read_text())
for path,digest in hashes.items():assert sha((E/path).read_bytes())==digest,path
col=json.loads((B/'COLLECTED.json').read_text())
for path,row in col['files'].items():assert sha((B/'output'/path).read_bytes())==row['sha256'] and (B/'output'/path).stat().st_size==row['bytes'],path
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
reports=[];artifact_hashes={}
with tarfile.open(E/'actual-artifacts.tar.gz') as archive:
 for m in archive.getmembers():
  if not m.isfile():continue
  raw=archive.extractfile(m).read();artifact_hashes[m.name]=sha(raw)
  if m.name.endswith('/target/release/kero'):assert sha(raw)==binding['binary_sha256']
  if m.name.endswith('/Cargo.lock'):assert sha(raw)==binding['actual_producer_lock_sha256']
  if m.name.endswith('law-checks.json'):
   d=json.loads(raw);assert d['unmet']==0,m.name
   assert all(check.get('passed',False) for check in d['checks']),m.name
   if 'commit' in d:assert d['commit']==source
   if 'binary_sha256' in d:assert d['binary_sha256']==binding['binary_sha256']
   reports.append({'artifact_member':m.name,'sha256':sha(raw),'checks':len(d['checks']),'unmet':d.get('unmet',[])})
assert len(reports)==29,len(reports)
aggregates={}
for key in ('source-fleet-aggregate','source-fleet-v2-aggregate','source-fleet-v3-aggregate'):
 d=json.loads((E/key/'step-2.stdout').read_text());assert d['unmet_families']==[];aggregates[key]=d

import re,gzip
assert not (E/'STOPPED.json').exists()
assert step_count==142
native_results=[]
required_core=['refused_drain_creates_no_receiver_and_moves_no_material','refused_extraction_preserves_the_numerical_reason_and_inventory','globally_invalid_transfer_state_keeps_error_and_rollback_contract','ordinary_supported_drain_retains_both_solute_shares_and_total','checked_layer_detection_and_mixing_preserve_refused_state','invalid_column_coordinates_refuse_without_panicking_or_mutating_inventory','globally_invalid_temperatures_keep_the_bench_error_and_rollback_contract','invalid_extraction_coordinate_refuses_without_panicking_or_moving_matter','methanol_column_uses_the_dedicated_subgroup_reference_at_both_temperatures','a_partial_sample_keeps_its_supported_peak_and_missing_data_note','lessons_produce_stable_output']
required_thermo=['checked_binary_lle_preserves_invalid_coordinate_and_group_errors','phase_scan_stops_at_the_first_failed_evaluation','checked_refusals_keep_the_invalid_coordinate_typed','checked_infinite_dilution_propagates_a_missing_group_without_mutation','missing_forward_interaction_is_a_named_refusal','reverse_interaction_does_not_stand_in_for_missing_direction']
required_thermo += ['underflowed_directional_weights_refuse_even_if_final_gammas_look_finite','underflowed_weights_cannot_supply_a_phase_verdict','ordinary_directional_weights_keep_the_existing_two_phase_control','nonrepresentable_interaction_factor_keeps_its_direction']
for n,command in enumerate(P['focused_native_commands']):
 root=E/('combined-native-'+str(n));record=json.loads(root.with_suffix('.json').read_text());assert record['argv']==command and record['exit_status']==0
 for stream in ['stdout','stderr']:assert sha(root.with_suffix('.'+stream).read_bytes())==record[stream+'_sha256']
 raw=root.with_suffix('.stdout').read_text();passed=0
 if n in [2,3,4,5]:
  counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw);assert counts and 'FAILED' not in raw and all(int(f)==0 and int(i)==0 for _,f,i in counts)
  passed=sum(int(a) for a,_,_ in counts);assert passed>={2:620,3:2153,4:2,5:184}[n],(n,passed)
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
for ref in [source,P['reused_liquid_baseline_source']]:assert sha(subprocess.check_output(['git','show',ref+':'+P['baseline_fixture']],cwd=REPO))==P['baseline_fixture_sha256']
factor=json.loads((E/'factor-baseline.json').read_text());assert factor['exit_status']==101 and factor['argv']==P['factor_baseline_command']
for stream in ['stdout','stderr']:assert sha((E/('factor-baseline.'+stream)).read_bytes())==factor[stream+'_sha256']
raw=(E/'factor-baseline.stdout').read_text();assert '1 passed; 2 failed; 0 ignored' in raw and 'error[E' not in (E/'factor-baseline.stderr').read_text()
for name in P['factor_expected_failing_tests']:assert name+' ... FAILED' in raw
assert P['factor_positive_control']+' ... ok' in raw
for ref in [source,P['factor_baseline_head']]:assert sha(subprocess.check_output(['git','show',ref+':'+P['factor_fixture']],cwd=REPO))==P['factor_fixture_sha256']
mac=(pathlib.Path(__file__).resolve().parent/'first-macos-failure.log').read_bytes();assert sha(mac)==P['first_actual_macos_failure']['log_sha256']
for name in P['first_actual_macos_failure']['unchanged_failed_tests']:assert (name+' ... FAILED').encode() in mac
# No parameter or numerical oracle values are introduced by this precursor.
for name in ['crates/kerotakis-thermo/tests/fixtures/thermo_oracle.tsv','crates/kerotakis-thermo/tests/thermo_oracle.rs']:
 assert get(name)==subprocess.check_output(['git','show','0b579699:'+name],cwd=REPO)
old=subprocess.check_output(['git','show','0b579699:crates/kerotakis-thermo/src/unifac.rs'],cwd=REPO);new=get('crates/kerotakis-thermo/src/unifac.rs');section=lambda data:data.split(b'const SOURCE_1975:',1)[1].split(b'pub type GroupDecomposition',1)[0];assert section(old)==section(new)
debug=json.loads((E/'debug-engine-binding.json').read_bytes());assert debug['source']==source and debug['actual_producer_lock_sha256']==P['expected_producer_lock_sha256']
with gzip.open(E/'actual-debug-kero.gz','rb') as f:assert sha(f.read())==debug['binary_sha256']
assert binding['actual_producer_lock_sha256']==P['expected_producer_lock_sha256']
result={'reviewed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':source,'tree':P['tree'],'engine':binding,'logical_jobs':33,'original_run_steps_independently_checked':step_count,'law_reports':reports,'aggregates':aggregates,'authenticated_evidence_files':len(hashes),'authenticated_artifact_files':len(artifact_hashes),'worker_sha256':sha(worker),'artifact_archive_sha256':sha((E/'actual-artifacts.tar.gz').read_bytes()),'all_original_steps_and_named_laws_pass':True,'all500_original_requirements_no_drift':True,'focused_native_results':native_results,'first_liquid_failure_and_baseline_preserved':True,'two_factor_failures_and_positive_preserved':True,'actual_first_macos_failure_preserved':True,'approved_parameters_and_independent_reference_values_unchanged':True,'protected_contexts_replaced':False,'integration_or_deployment_claim':False,'catalog_additions':0}
with (B/'ROOT-ACCEPTED.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result))
