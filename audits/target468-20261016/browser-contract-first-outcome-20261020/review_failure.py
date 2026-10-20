#!/usr/bin/env python3
"""Authenticate the stopped run; never promote it to source acceptance."""
import hashlib,json,re
from pathlib import Path
D=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_bytes())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
collected=load(D/'collected.json')
for name,record in collected['files'].items():
 p=D/'raw'/name
 assert p.stat().st_size==record['bytes'] and sha(p)==record['sha256'],name
E=D/'raw/browser-contract-evidence';manifest=load(E/'manifest.json')
for name,record in manifest.items():
 p=E/name
 assert p.stat().st_size==record['bytes'] and sha(p)==record['sha256'],name
plan=load(E/'frozen-plan.json');start=load(E/'STARTED.json');issue=load(D/'issue-binding.json')
assert plan['baseline_head']=='cc2fe02adac818b6af6c8834fa19ca36ab1540c4'
assert start['source']==plan['source']==issue['source']=='d7f7b130d207f918033a2c0bf7a1aaeb5f78c5d6'
assert start['worker_sha256']==issue['worker_sha256']=='70fff8b7b905117ae55168409771915ab997d16e7f87b2a0a8cf8ee94e2ec02c'
assert sha(E/'prior-full-acceptance.json')==issue['prerequisite_sha256']
text=(E/'baseline.stdout').read_text();stderr=(E/'baseline.stderr').read_text()
assert 'error[E' not in stderr and load(E/'baseline.json')['exit_status']==101
assert 'test result: FAILED. 2 passed; 10 failed; 0 ignored' in text
failed=re.findall(r'^test (\S+) \.\.\. FAILED$',text,re.M);passed=re.findall(r'^test (\S+) \.\.\. ok$',text,re.M)
extra='solver_contract_tests::untrusted_wrapper_cannot_get_receipt_from_existing_solution'
assert set(failed)==set(plan['expected_failing'])|{extra}
assert set(passed)==set(plan['positive_controls'])-{extra}
assert len(failed)==10 and len(passed)==2
assert 'real analytic adapter supplies its own receipt: NotConverged' in stderr
assert not (E/'fixed.json').exists() and not (E/'COMPLETED.json').exists()
assert load(E/'STOPPED.json')['error_type']=='AssertionError'
print(json.dumps({'compiled_baseline_controls':12,'actual_failed':failed,'actual_passed':passed,'forecast_mismatch_preserved':True,'candidate_executed':False,'actual_executable_retained':False,'source_acceptance':False,'scientific_selections':0},indent=2))
