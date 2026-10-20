#!/usr/bin/env python3
"""Portable source/lock/raw/executable review. Reads evidence; never executes a test."""
import argparse,base64,gzip,hashlib,json,pathlib,re
HERE=pathlib.Path(__file__).resolve().parent
P=json.loads((HERE/'frozen-plan.json').read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def digest(path):
 h=hashlib.sha256()
 with path.open('rb')as f:
  while chunk:=f.read(262144):h.update(chunk)
 return h.hexdigest()
def main():
 parser=argparse.ArgumentParser();parser.add_argument('evidence',type=pathlib.Path);parser.add_argument('--receipt',type=pathlib.Path,required=True);args=parser.parse_args();o=args.evidence
 assert P['issue_authorization'] is not None,'Unissued preparation cannot accept execution'
 def load(name):return json.loads((o/name).read_text())
 manifest=load('manifest.json')
 actual_files={str(p.relative_to(o))for p in o.rglob('*')if p.is_file()}-{'manifest.json'};assert actual_files==set(manifest),'Missing or unmanifested raw evidence'
 for name,entry in manifest.items():
  path=pathlib.PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts
  assert digest(o/name)==entry['sha256'] and (o/name).stat().st_size==entry['bytes'],name
 assert load('frozen-plan.json')==P
 worker_sha=digest(HERE/'upload/worker.py');assert load('STARTED.json')['worker_sha256']==worker_sha
 authorization=base64.b64decode(P['issue_authorization']['receipt_base64']);assert sha(authorization)==P['issue_authorization']['sha256'] and (o/'issue-authorization.json').read_bytes()==authorization
 lock=base64.b64decode(P['lock_base64']);assert sha(lock)==P['lock_sha256'] and (o/'Cargo.lock').read_bytes()==lock
 assert (o/'COMPLETED.json').exists() and not (o/'STOPPED.json').exists(),'Stopped producer is not focused acceptance'
 summaries={};binaries={}
 def command(name,expected=None):
  r=load(name+'.json');assert digest(o/(name+'.stdout'))==r['stdout_sha256'] and digest(o/(name+'.stderr'))==r['stderr_sha256']
  if expected is not None:assert r['argv']==expected,(name,r['argv'],expected)
  return r
 for label in ['baseline','fixed']:
  src=load(label+'-source.json');assert src=={'head':P['source_heads'][label],'tree':P['source_trees'][label],'files_sha256':P['source_files'][label]}
  assert digest(o/(label+'-git-tree.txt'))==P['tree_manifest_sha256'][label]
  assert (o/(label+'-Cargo.lock')).read_bytes()==lock;assert load(label+'-source-after.json')==P['source_files'][label]
  build=command(label+'-build',['cargo','test','--locked','-p','kerotakis-core','--test','peroxide_heat_basis','--no-run','--message-format=json']);assert build['exit_status']==0
  compiler=[]
  for line in (o/(label+'-build.stdout')).read_text().splitlines():
   try:compiler.append(json.loads(line))
   except json.JSONDecodeError:pass
  executable=[m['executable']for m in compiler if m.get('reason')=='compiler-artifact' and m.get('target',{}).get('name')=='peroxide_heat_basis' and m.get('profile',{}).get('test') and m.get('executable')];assert len(executable)==1
  run=command(label,[executable[0],'--nocapture','--test-threads=1']);text=(o/(label+'.stdout')).read_text();cases=re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',text,re.M);totals=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
  assert len(cases)==4 and {n for n,_ in cases}==set(P['tests']) and len(totals)==1
  passed,failed,ignored=map(int,totals[0]);assert ignored==0 and passed+failed==4 and passed==sum(s=='ok'for _,s in cases) and failed==sum(s=='FAILED'for _,s in cases)
  outcome=load(label+'-actual-outcome.json');assert outcome=={'head':P['source_heads'][label],'exit_status':run['exit_status'],'passed':passed,'failed':failed,'ignored':ignored,'tests':dict(cases),'forecast_not_observation':False}
  if label=='baseline':
   assert run['exit_status']==101 and passed==1 and failed==3
   assert all(dict(cases)[n]=='FAILED'for n in P['baseline_forecast_failures']) and all(dict(cases)[n]=='ok'for n in P['baseline_forecast_positive'])
   assert (o/'baseline.stderr').read_text().count('reference heat basis mismatch:')==3
  else:assert run['exit_status']==0 and passed==4 and failed==0
  b=load(label+'-test-binary.json');assert b['head']==P['source_heads'][label] and b['tree']==P['source_trees'][label] and b['lock_sha256']==P['lock_sha256']
  assert pathlib.PurePosixPath(executable[0]).as_posix().endswith('/'+b['executable']);assert b['executable'].startswith('target/debug/deps/peroxide_heat_basis-')
  gz=o/b['gzip'];assert b['gzip']==label+'-peroxide_heat_basis.gz' and digest(gz)==b['gzip_sha256'] and gz.stat().st_size==b['gzip_bytes']
  h=hashlib.sha256();size=0
  with gzip.open(gz,'rb')as stream:
   while chunk:=stream.read(262144):h.update(chunk);size+=len(chunk)
  assert size==b['bytes'] and h.hexdigest()==b['sha256'];summaries[label]=outcome;binaries[label]=b
 assert command('format',['cargo','fmt','--check'])['exit_status']==0
 assert command('core-clippy',['cargo','clippy','--locked','-p','kerotakis-core','--all-targets','--','-D','warnings'])['exit_status']==0
 complete=load('COMPLETED.json');assert complete['source']==P['source_heads']['fixed'] and complete['tree']==P['source_trees']['fixed'] and complete['lock_sha256']==P['lock_sha256'] and complete['scientific_selections']==0 and complete['protected_contexts_replaced'] is False and complete['full_source_acceptance'] is False
 result={'schema':'kerotakis-peroxide-heat-focused-acceptance-v1','source':P['source_heads']['fixed'],'tree':P['source_trees']['fixed'],'baseline':P['source_heads']['baseline'],'worker_sha256':worker_sha,'producer_manifest_sha256':digest(o/'manifest.json'),'actual_lock_sha256':digest(o/'Cargo.lock'),'actual_results':summaries,'actual_test_binaries':binaries,'reference':P['reference'],'all_four_fixed_controls':True,'three_observed_baseline_basis_failures_one_positive':True,'format_and_core_clippy_pass':True,'protected_contexts_replaced':False,'full_source_acceptance':False,'scientific_selections':0}
 with args.receipt.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({'status':'FOCUSED-ACCEPTED','actual_baseline_passed':summaries['baseline']['passed'],'actual_baseline_failed':summaries['baseline']['failed'],'actual_fixed_passed':summaries['fixed']['passed']}))
if __name__=='__main__':main()
