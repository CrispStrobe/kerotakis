#!/usr/bin/env python3
"""Authenticate published small raw evidence; never execute a test or hosted job."""
import ast,hashlib,json,pathlib,re,subprocess
O=pathlib.Path(__file__).resolve().parent;R=O.parents[2];load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
pub=load(O/'publication.json');raw=O/'raw';P=load(O/'frozen-plan.json');manifest=load(raw/'manifest.json')
for name,row in pub['copied_raw'].items():
 assert pathlib.PurePosixPath(name).name==name and sha(raw/name)==row['sha256'] and (raw/name).stat().st_size==row['bytes']
 if name!='manifest.json':assert row==manifest[name]
for name,row in pub['retained_large_binaries_metadata_only'].items():assert row==manifest[name]
assert load(raw/'frozen-plan.json')==P and load(O/'FOCUSED-ACCEPTED.json')==load(O/'PARENT-FOCUSED-ACCEPTED.json')
receipt=load(O/'FOCUSED-ACCEPTED.json');assert receipt['source']==P['source_heads']['fixed'] and receipt['baseline']==P['source_heads']['baseline'];assert receipt['all_four_fixed_controls'] and receipt['three_observed_baseline_basis_failures_one_positive'] and receipt['format_and_core_clippy_pass'];assert receipt['full_source_acceptance']is False and receipt['scientific_selections']==0
for label,expected in [('baseline',(1,3)),('fixed',(4,0))]:
 outcome=load(raw/(label+'-actual-outcome.json'));assert (outcome['passed'],outcome['failed'])==expected and outcome['ignored']==0 and outcome['head']==P['source_heads'][label]
 text=(raw/(label+'.stdout')).read_text();actual=dict(re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',text,re.M));assert actual==outcome['tests'] and set(actual)==set(P['tests'])
 assert sum(s=='ok'for s in actual.values())==expected[0] and sum(s=='FAILED'for s in actual.values())==expected[1]
 if label=='baseline':assert (raw/'baseline.stderr').read_text().count('reference heat basis mismatch:')==3
 command=load(raw/(label+'.json'));assert command['exit_status']==outcome['exit_status'] and command['stdout_sha256']==sha(raw/(label+'.stdout')) and command['stderr_sha256']==sha(raw/(label+'.stderr'))
 binary=load(raw/(label+'-test-binary.json'));assert binary==receipt['actual_test_binaries'][label] and binary['gzip_sha256']==pub['retained_large_binaries_metadata_only'][binary['gzip']]['sha256']
 assert sha(raw/(label+'-Cargo.lock'))==P['lock_sha256']==receipt['actual_lock_sha256']
 for path,h in P['source_files'][label].items():assert hashlib.sha256(subprocess.check_output(['git','show',P['source_heads'][label]+':'+path],cwd=R)).hexdigest()==h
 assert load(raw/(label+'-source-after.json'))==P['source_files'][label]
for name in ['format','core-clippy']:assert load(raw/(name+'.json'))['exit_status']==0
assert load(raw/'COMPLETED.json')['full_source_acceptance']is False and not (raw/'STOPPED.json').exists()
print(json.dumps({'status':'PASS','observed_baseline_passed':1,'observed_baseline_failed':3,'observed_candidate_passed':4,'actual_parent_receipt_equal':True,'published_raw_hashes_verified':len(pub['copied_raw']),'large_binaries_metadata_only':True,'new_test_or_job_executions':0}))
