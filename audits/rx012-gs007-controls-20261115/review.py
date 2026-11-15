"""Fail-closed reader for the one prepared run. This file does not run chemistry."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
SOURCE='c637d2b2f1d3e431cfa41e742214256d19cbbbc4'
TREE='27f4470de6b3b1ee1183df2fba3b2235cda5212a'
EXPECTED={
 'rx012-four-core':[
  'reverse_leads_reverse_signed_readout_without_reversing_chemistry_or_changing_stock',
  'unresolved_or_equal_activity_does_not_gain_polarity_authority',
  'legacy_event_has_no_invented_lead_order_and_roundtrips_unchanged',
  'ordered_negative_reading_is_visible_in_each_locale'],
 'rx012-cli-scalar':['native_startup_tests::semantic_event_adapter_exposes_only_documented_scalar_fields'],
 'gs007-two-projection':[
  'swept_reservoir_warns_without_changing_carbon_export_or_inventory',
  'finite_dose_does_not_acquire_reservoir_warning']}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def test_names(stdout,expected):
 actual=re.findall(r'^test (\S+) \.\.\. ok\s*$',stdout,re.M)
 assert len(actual)==len(expected),(actual,expected)
 assert len(set(actual))==len(actual)
 for name in expected:assert sum(x==name or x.endswith('::'+name)for x in actual)==1,name
 assert re.search(r'test result: ok\. '+str(len(expected))+r' passed; 0 failed; 0 ignored;',stdout)
 return actual

def review(e,repo):
 # Missing/partial output must refuse, rather than becoming a zero-test pass.
 assert (e/'COMPLETED.json').is_file()and not(e/'FAILED.json').exists()
 manifest=json.loads((e/'SHA256.json').read_text());verified=[]
 for name,h in manifest.items():
  path=Path(name);assert not path.is_absolute()and '..'not in path.parts
  p=e/path
  if path.name.startswith('native-')and path.suffix=='':continue
  assert p.is_file()and sha(p)==h,name;verified.append(name)
 completed=json.loads((e/'COMPLETED.json').read_text())
 assert completed['source']==SOURCE and completed['tree']==TREE
 assert completed['originals_replayed']is False and completed['qualification_delta']==0
 assert completed['all_commands_passed']and completed['command_exits']==dict.fromkeys(EXPECTED,0)
 assert subprocess.check_output(['git','-C',str(repo),'rev-parse',SOURCE+'^{tree}'],text=True).strip()==TREE
 source=json.loads((e/'source-and-lock.json').read_text())
 assert source['source']==SOURCE and source['tree']==TREE
 assert source['cargo_lock_sha256']==sha(e/'actual-producer-Cargo.lock')=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert sha(e/'compiler.stdout')=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6'
 for name,h in json.loads((e/'composed-source-files.json').read_text()).items():
  raw=subprocess.check_output(['git','-C',str(repo),'show',SOURCE+':'+name]);assert hashlib.sha256(raw).hexdigest()==h,name
 counts={}
 for name,expected in EXPECTED.items():
  step=json.loads((e/(name+'.json')).read_text());assert step['exit']==0
  assert step['stdout_sha256']==sha(e/(name+'.stdout'))and step['stderr_sha256']==sha(e/(name+'.stderr'))
  resource=json.loads((e/(name+'-resource-admission.json')).read_text())
  assert resource['MemAvailable_MiB']>=4608 and resource['load1']<3.5 and resource['free_bytes']>12*1024**3 and resource['elapsed_seconds']<1650
  counts[name]=test_names((e/(name+'.stdout')).read_text(),expected)
 binaries=json.loads((e/'native-executables.json').read_text());assert len(binaries)==3
 streams=json.loads((e/'streamed-executables.json').read_text())
 assert {b['stage']for b in binaries}==set(EXPECTED)
 for b in binaries:
  assert b['source']==SOURCE and b['tree']==TREE and b['profile']['test']
  assert streams[b['artifact']]['verified']and streams[b['artifact']]['actual_sha256']==b['sha256']
 return dict(source=SOURCE,tree=TREE,verified_files=len(verified),actual_tests=counts,streamed_binary_hashes={b['artifact']:b['sha256']for b in binaries},qualification_delta=0,originals_replayed=False,full_source_platform_fleet_acceptance=False)

def self_test():
 # Refusal tests only; none is represented as an executed Rust/chemistry pass.
 import tempfile
 refused=[]
 for label,stdout in [('zero_match','running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;'),('wrong_suite','test old_native_test ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;'),('ignored_instead_of_executed','test expected ... ignored\ntest result: ok. 0 passed; 0 failed; 1 ignored;')]:
  try:test_names(stdout,['expected'])
  except AssertionError:refused.append(label)
  else:raise AssertionError(label)
 with tempfile.TemporaryDirectory()as d:
  try:review(Path(d),Path(d))
  except AssertionError:refused.append('missing_producer')
  else:raise AssertionError('missing_producer')
 print(json.dumps(dict(reader_refusals=refused,Rust_controls_executed=False,scientific_qualification=False)))

def main():
 p=argparse.ArgumentParser();p.add_argument('--evidence',type=Path);p.add_argument('--repo',type=Path);p.add_argument('--out',type=Path);p.add_argument('--self-test',action='store_true');a=p.parse_args()
 if a.self_test:self_test();return
 assert a.evidence and a.repo and a.out
 result=review(a.evidence,a.repo)
 with a.out.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result))
if __name__=='__main__':main()
