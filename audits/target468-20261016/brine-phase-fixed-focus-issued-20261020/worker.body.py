"""One bounded actual TC008/brine observation; no energy verdict is assigned."""
import base64,datetime,gzip,hashlib,json,os,pathlib,re,shutil,signal,subprocess,time,traceback
P=None

def sha(raw):return hashlib.sha256(raw).hexdigest()
def digest(path):
 with pathlib.Path(path).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 assert P is not None and P['issue_authorization']is not None,'Parent exact-source issue review remains unbound'
 O=pathlib.Path('/kaggle/working/brine-fixed-evidence');O.mkdir(exist_ok=False)
 W=pathlib.Path('/kaggle/temp/brine-fixed-focus');W.mkdir(parents=True,exist_ok=False)
 R=W/'source';started=time.monotonic();env=dict(os.environ);runtime=None
 def save(name,value):
  with(O/name).open('x',encoding='utf-8')as f:json.dump(value,f,indent=2,allow_nan=False);f.write('\n')
 def remaining():
  value=P['maximum_seconds']-(time.monotonic()-started);assert value>0,'Overall1800s deadline';return value
 def run(name,argv,cwd=W,required=True,maximum=None):
  process=None;status=None;error=None
  try:
   with(O/(name+'.stdout')).open('xb')as out,(O/(name+'.stderr')).open('xb')as err:
    process=subprocess.Popen(argv,cwd=cwd,env=env,stdout=out,stderr=err,start_new_session=True)
    try:status=process.wait(timeout=min(remaining(),maximum or P['maximum_seconds']))
    except BaseException:
     deadline=time.monotonic()+5
     try:os.killpg(process.pid,signal.SIGTERM)
     except ProcessLookupError:pass
     try:process.wait(timeout=max(.001,deadline-time.monotonic()))
     except subprocess.TimeoutExpired:pass
     left=max(0,deadline-time.monotonic())
     if left:time.sleep(left)
     try:os.killpg(process.pid,signal.SIGKILL)
     except ProcessLookupError:pass
     process.wait(timeout=10);status=process.returncode;raise
   if required:assert status==0,name+' failed; preserve first compiler/setup failure'
   return status
  except BaseException as exc:error=type(exc).__name__;raise
  finally:
   save(name+'.json',{'argv':argv,'exit_status':status,'error_type':error,'owned_process_group':process.pid if process else None,'stdout_sha256':digest(O/(name+'.stdout'))if(O/(name+'.stdout')).is_file()else None,'stderr_sha256':digest(O/(name+'.stderr'))if(O/(name+'.stderr')).is_file()else None})
 def source_proof(label):
  assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R).decode().strip()==P['source']
  assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=R).decode().strip()==P['tree']
  observed={name:digest(R/name)for name in P['source_blobs_sha256']};assert observed==P['source_blobs_sha256']
  assert subprocess.check_output(['git','diff','--name-only','HEAD'],cwd=R)==b'','Tracked runtime source changed'
  actual_overlays={target:digest(R/target)for name,target in P['overlays'].items()};assert actual_overlays=={target:P['files'][name]['sha256']for name,target in P['overlays'].items()}
  assert digest(R/'Cargo.lock')==P['dependency_lock_sha256'];save(label+'.json',{'runtime_source':P['source'],'runtime_tree':P['tree'],'unchanged_runtime_files':observed,'tests_only_overlays':actual_overlays,'actual_producer_lock_sha256':digest(R/'Cargo.lock')})
 def build(target,package):
  run(target+'-build',['cargo','test','--locked','-p',package,'--test',target,'--no-run','--message-format=json'],cwd=R)
  messages=[]
  for line in(O/(target+'-build.stdout')).read_text(encoding='utf-8').splitlines():
   try:messages.append(json.loads(line))
   except json.JSONDecodeError:pass
  matches=[m['executable']for m in messages if m.get('reason')=='compiler-artifact'and m.get('target',{}).get('name')==target and m.get('profile',{}).get('test')and m.get('executable')]
  assert len(matches)==1,matches;exe=pathlib.Path(matches[0]);exe.relative_to(R/'target');assert exe.is_file()and os.access(exe,os.X_OK)
  with exe.open('rb')as src,gzip.open(O/(target+'.gz'),'wb',compresslevel=1)as dst:shutil.copyfileobj(src,dst,262144)
  save(target+'-binary.json',{'runtime_source':P['source'],'runtime_tree':P['tree'],'tests_overlay_qualification':'Exact frozen observers over unchanged runtime; not the deployed release executable','actual_producer_lock_sha256':digest(R/'Cargo.lock'),'executable':str(exe.relative_to(R)),'sha256':digest(exe),'bytes':exe.stat().st_size,'gzip_sha256':digest(O/(target+'.gz')),'gzip_bytes':(O/(target+'.gz')).stat().st_size})
  return exe
 def outcome(target,expected,code):
  raw=(O/(target+'.stdout')).read_text(encoding='utf-8');actual=re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',raw,re.M);totals=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)
  passed=failed=ignored=None
  complete=len(actual)==len(expected)and{n for n,_ in actual}==set(expected)and len(totals)==1
  if len(totals)==1:passed,failed,ignored=map(int,totals[0])
  if complete:complete=ignored==0 and passed+failed==len(expected)and passed==sum(status=='ok'for _,status in actual)and failed==sum(status=='FAILED'for _,status in actual)and((code==0 and failed==0)or(code==101 and failed>0))
  # Even an incomplete/crashed test target must not suppress other independent
  # targets. Preserve raw counts/status without inventing missing outcomes.
  value={'exit_status':code,'passed':passed,'failed':failed,'ignored':ignored,'observed_tests':actual,'expected_tests':expected,'complete_expected_test_set':complete,'energy_acceptance_assigned':False};save(target+'-actual-outcome.json',value);return value
 try:
  save('frozen-plan.json',P);save('STARTED.json',{'worker_sha256':digest(pathlib.Path(__file__)),'runtime_source':P['source'],'runtime_tree':P['tree'],'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'purpose':'Necessary fixed-source followup:8 frozen brine +1 original TC008 trace +12 unchanged phase regressions; no verdict assigned'})
  auth=P['issue_authorization'];raw=base64.b64decode(auth['base64'],validate=True);assert sha(raw)==auth['sha256'];approved=json.loads(raw);assert approved['source']==P['source']and approved['tree']==P['tree']and approved['reviewed_for_one_bounded_focus']is True and approved['actual_release_api11_browser64_accepted']is True and approved['actual_accepted_release_source']==P['baseline_actual_source']
  (O/'parent-issue-review.json').write_bytes(raw)
  memory=int(next(x.split()[1]for x in pathlib.Path('/proc/meminfo').read_text(encoding='utf-8').splitlines()if x.startswith('MemAvailable:')))//1024
  assert memory>=P['minimum_available_memory_mib']and shutil.disk_usage(W).free>=P['minimum_disk_bytes'],'Hosted resource gate'
  if hasattr(os,'sched_getaffinity'):os.sched_setaffinity(0,set(sorted(os.sched_getaffinity(0))[:2]))
  env.update(CARGO_HOME=str(W/'cargo'),RUSTUP_HOME=str(W/'rustup'),CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',CARGO_TERM_COLOR='never',CMAKE_BUILD_PARALLEL_LEVEL='2',GITHUB_SHA=P['source'])
  run('fetch-rustup',['curl','-fsSL','--retry','0','https://sh.rustup.rs','-o',str(W/'rustup-init.sh')],maximum=90);save('rustup-bootstrap.json',{'actual_download_sha256':digest(W/'rustup-init.sh'),'qualification':'Observed bootstrap bytes; mutable upstream bootstrap URL, pinned installed toolchain separately'})
  run('install-rust',['sh',str(W/'rustup-init.sh'),'-y','--profile','minimal','--default-toolchain',P['toolchain'],'--no-modify-path'],maximum=420);env['PATH']=str(W/'cargo/bin')+os.pathsep+env['PATH'];run('compiler',['rustc','-Vv']);run('cargo-version',['cargo','--version'])
  run('git-init',['git','init',str(R)]);run('git-remote',['git','remote','add','origin',P['repository']],cwd=R);run('git-fetch',['git','fetch','--depth','1','origin',P['source']],cwd=R);run('git-checkout',['git','checkout','--detach',P['source']],cwd=R)
  run('fixture-source-fetch',['git','fetch','--depth','1','origin',P['fixture_commit']],cwd=R)
  tree=subprocess.check_output(['git','ls-tree','-r','--full-tree',P['source']],cwd=R);(O/'runtime-git-tree.txt').write_bytes(tree)
  frozen=O/'frozen-inputs';frozen.mkdir()
  for name,row in P['files'].items():
   raw=base64.b64decode(row['base64'],validate=True);assert sha(raw)==row['sha256'];assert subprocess.check_output(['git','show',P['fixture_commit']+':'+P['fixture_public_prefix']+name],cwd=R)==raw,'Frozen public evidence differs'
   path=frozen/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
  for name,target in P['overlays'].items():
   path=R/target;raw=(frozen/name).read_bytes()
   if path.exists():
    assert path.read_bytes()==raw and subprocess.check_output(['git','show',P['source']+':'+target],cwd=R)==raw,'Existing overlay differs from byte-exact committed fixture'
   else:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
  external=O/'external-saved-reader';external.mkdir();row=P['external_reader'];raw=base64.b64decode(row['base64'],validate=True);assert sha(raw)==row['sha256'];(external/'review_trace.py').write_bytes(raw)
  (external/'fixtures').mkdir();(external/'fixtures/tc008-original.lab').write_bytes((frozen/'fixtures/tc008-original.lab').read_bytes());raw=base64.b64decode(row['baseline_saved_result_base64'],validate=True);assert sha(raw)==row['baseline_saved_result_sha256'];(external/'baseline-saved-reader-result.json').write_bytes(raw)
  lock=base64.b64decode(P['dependency_lock_base64'],validate=True);assert sha(lock)==P['dependency_lock_sha256'];(R/'Cargo.lock').write_bytes(lock);(O/'seed-Cargo.lock').write_bytes(lock)
  run('native-submodules',['git','submodule','update','--init','--recursive','--depth','1'],cwd=R,maximum=240)
  source_proof('source-before')
  # Compile every selected target first using one shared build cache. A genuine
  # compiler failure preserves raw evidence and cannot masquerade as execution.
  executables={row['target']:build(row['target'],row['package'])for row in P['targets']}
  outcomes={}
  for row in P['targets']:
   target=row['target']
   if row['trace']:
    env['KERO_TC008_TRACE_DIR']=str(O/'tc008-trace');assert not(O/'tc008-trace').exists()
   code=run(target,[str(executables[target]),'--nocapture','--test-threads=1'],cwd=R,required=False)
   outcomes[target]=outcome(target,row['tests'],code)
  source_proof('source-after');(O/'actual-producer-Cargo.lock').write_bytes((R/'Cargo.lock').read_bytes())
  if outcomes['tc008_energy_trace']['exit_status']==0:assert(O/'tc008-trace/tc008-trace.jsonl').is_file(),'Trace-collection test omitted actual trace'
  save('COMPLETED.json',{'runtime_source':P['source'],'runtime_tree':P['tree'],'actual_target_outcomes':outcomes,'actual_producer_lock_sha256':digest(R/'Cargo.lock'),'collection_completed_not_energy_acceptance':True,'independent_saved_trace_reader_required':True,'scientific_selections':0,'protected_checks_replaced':False,'baseline_not_reexecuted':True})
 except BaseException as exc:
  save('STOPPED.json',{'error_type':type(exc).__name__,'error':str(exc),'traceback':traceback.format_exc(),'first_partial_evidence_preserved':True,'automatic_retry':False});raise
 finally:
  if(R/'Cargo.lock').is_file()and not(O/'actual-producer-Cargo.lock').exists():(O/'actual-producer-Cargo.lock').write_bytes((R/'Cargo.lock').read_bytes())
  save('manifest.json',{str(p.relative_to(O)):{'sha256':digest(p),'bytes':p.stat().st_size}for p in sorted(O.rglob('*'))if p.is_file()and p!=O/'manifest.json'})

if __name__=='__main__':main()
