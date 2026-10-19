"""Verify saved CPU outputs and actual test archive; never invoke an engine."""
import argparse, ast, hashlib, json, re, subprocess, tarfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--evidence',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--binary-archive',type=Path);a=p.parse_args()
b=Path(__file__).resolve().parent;r=b.parents[2];e=a.evidence
sha=lambda raw:hashlib.sha256(raw).hexdigest()
plan=json.loads((b/'preparation.json').read_text());worker=(b/'worker.py').read_bytes();assert sha(worker)==plan['uploaded_worker_sha256']
embedded=ast.literal_eval(ast.parse(worker).body[0].value);actual=json.loads((e/'frozen-plan.json').read_text());assert embedded==actual
for k,v in embedded.items():
 if k!='lock_base64':assert plan[k]==v,k
assert not (e/'STOPPED.json').exists()
for n,d in json.loads((e/'SHA256.json').read_text()).items():
 path=Path(n);assert not path.is_absolute() and '..' not in path.parts
 assert sha((a.binary_archive if n=='actual-test-binaries.tar.gz' and a.binary_archive else e/n).read_bytes())==d,n
start=json.loads((e/'STARTED.json').read_text());assert start['source']==plan['source'] and start['tree']==plan['tree'] and start['worker_sha256']==plan['uploaded_worker_sha256']
for n,d in plan['source_sha256'].items():assert sha(subprocess.check_output(['git','show',plan['source']+':'+n],cwd=r))==d,n
for ref in [plan['source'],plan['baseline_source']]:assert sha(subprocess.check_output(['git','show',ref+':'+plan['baseline_fixture']],cwd=r))==plan['baseline_fixture_sha256']
record=json.loads((e/'baseline.json').read_text());assert record['exit_status']==101 and record['argv']==plan['baseline_command']
raw=(e/'baseline.stdout').read_text();assert '1 passed; 3 failed; 0 ignored' in raw and 'error[E' not in (e/'baseline.stderr').read_text()
for n in plan['expected_failing_names']:assert n+' ... FAILED' in raw,n
assert plan['positive_control']+' ... ok' in raw
for i,cmd in enumerate(plan['fixed_commands']):
 rec=json.loads((e/f'fixed-{i}.json').read_text());assert rec['argv']==cmd and rec['exit_status']==0
 for stream in ['stdout','stderr']:assert sha((e/f'fixed-{i}.{stream}').read_bytes())==rec[stream+'_sha256']
raw=(e/'fixed-2.stdout').read_text();counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw)
assert counts and 'FAILED' not in raw and all(int(f)==0 and int(i)==0 for _,f,i in counts)
for n in plan['expected_failing_names']+[plan['positive_control'],'a_partial_sample_keeps_its_supported_peak_and_missing_data_note','globally_invalid_temperatures_keep_the_bench_error_and_rollback_contract','invalid_extraction_coordinate_refuses_without_panicking_or_moving_matter','methanol_column_uses_the_dedicated_subgroup_reference_at_both_temperatures','invalid_column_coordinates_refuse_without_panicking_or_mutating_inventory']:assert n+' ... ok' in raw,n
assert sha((e/'producer-Cargo.lock').read_bytes())==plan['lock_sha256'];binding=json.loads((e/'TEST-BINARY-BINDING.json').read_text());assert binding['source']==plan['source'] and binding['tree']==plan['tree'] and binding['actual_lock_sha256']==plan['lock_sha256']
archive=a.binary_archive or e/'actual-test-binaries.tar.gz';assert sha(archive.read_bytes())==binding['archive_sha256'];found={}
with tarfile.open(archive) as t:
 for m in t:
  assert not Path(m.name).is_absolute() and '..' not in Path(m.name).parts
  if m.isfile():found[m.name]=sha(t.extractfile(m).read())
assert found==binding['compiled_test_binary_sha256']
executables=[n for n in found if not n.endswith('.so')];assert len(executables)==len(counts)
for prefix in ['liquid_partition_refusals-','partition_consumer_admission-']:assert sum(n.startswith(prefix) for n in executables)==1,prefix
# Reuse focused numerical evidence only on the exact same implementation subtree.
subtree=subprocess.check_output(['git','rev-parse',plan['source']+':crates/kerotakis-thermo'],cwd=r,text=True).strip();assert subtree==plan['unchanged_thermo_subtree']
receipt=r/'audits/target468-20261016/unifac-input-focused-outcome-20261019/FOCUSED-ACCEPTED-v2.json';assert sha(receipt.read_bytes())==plan['reused_thermo_acceptance_sha256']
# Check the reviewed lesson change is precisely the two independent-reference corrections.
f='crates/kerotakis-core/tests/golden/lessons.json';old=subprocess.check_output(['git','show','4d87acf9a943b1db8fb824c1bc8362a976a5679d:'+f],cwd=r);new=subprocess.check_output(['git','show',plan['source']+':'+f],cwd=r);needle=b'methanol at 63 s, Rf 0.08 (50% area)';assert old.count(needle)==2 and old.replace(needle,b'methanol at 63 s, Rf 0.10 (50% area)')==new
result={'source':plan['source'],'tree':plan['tree'],'three_genuine_liquid_baseline_failures':True,'nine_focused_controls_pass':True,'formatter_clippy_focused_liquid_and_column_controls_pass':True,'focused_core_tests_passed':sum(int(n) for n,_,_ in counts),'actual_lock_sha256':plan['lock_sha256'],'test_executables_verified':len(executables),'compiled_shared_libraries_verified':len(found)-len(executables),'unchanged_thermo_focused_evidence_reused':True,'prior_two_reviewed_lesson_corrections_preserved':True,'combined_source_full_product_platform_or_deployment_acceptance':False,'scientific_catalog_additions':0,'original_experiment_requests':0}
assert not a.out.exists();a.out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
