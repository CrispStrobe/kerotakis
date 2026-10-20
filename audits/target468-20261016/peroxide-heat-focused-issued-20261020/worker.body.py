import base64,datetime,gzip,hashlib,json,os,pathlib,re,shutil,subprocess,time,traceback
O=pathlib.Path('/kaggle/working/peroxide-heat-evidence');O.mkdir(exist_ok=False)
W=pathlib.Path('/kaggle/temp/peroxide-heat-focus');W.mkdir(parents=True,exist_ok=False)
R=W/'source';started=time.monotonic();env=dict(os.environ)
def sha(b):return hashlib.sha256(b).hexdigest()
def digest(path):
 h=hashlib.sha256()
 with path.open('rb')as f:
  while chunk:=f.read(262144):h.update(chunk)
 return h.hexdigest()
def save(name,obj):
 with (O/name).open('x')as f:json.dump(obj,f,indent=2);f.write('\n')
def run(name,args,cwd=W,required=True):
 remaining=P['maximum_seconds']-(time.monotonic()-started)
 assert remaining>1,'Wall budget exhausted before '+name
 with (O/(name+'.stdout')).open('wb')as out,(O/(name+'.stderr')).open('wb')as err:
  result=subprocess.run(args,cwd=cwd,env=env,stdout=out,stderr=err,timeout=remaining)
 save(name+'.json',{'argv':args,'exit_status':result.returncode,'stdout_sha256':digest(O/(name+'.stdout')),'stderr_sha256':digest(O/(name+'.stderr'))})
 if (R/'Cargo.lock').exists():assert digest(R/'Cargo.lock')==P['lock_sha256'],'Dependency lock changed'
 print(name,result.returncode,flush=True)
 if required:assert result.returncode==0,name
 return result.returncode
try:
 save('frozen-plan.json',P);save('STARTED.json',{'worker_sha256':digest(pathlib.Path(__file__)),'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'no_original_experiments':True})
 authorization=P['issue_authorization'];assert authorization is not None,'PREPARATION ONLY: actual parent review and priority prerequisites remain unbound'
 raw=base64.b64decode(authorization['receipt_base64']);assert sha(raw)==authorization['sha256'];a=json.loads(raw)
 assert a['baseline']==P['source_heads']['baseline'] and a['fixed']==P['source_heads']['fixed'] and a['reviewed_for_one_bounded_focus'] is True and a['active_full32_priority_released'] is True
 (O/'issue-authorization.json').write_bytes(raw)
 memory=int(next(x.split()[1]for x in pathlib.Path('/proc/meminfo').read_text().splitlines()if x.startswith('MemAvailable:')))//1024
 assert memory>=P['minimum_available_memory_mib'] and shutil.disk_usage(W).free>=P['minimum_disk_bytes']
 if hasattr(os,'sched_getaffinity'):os.sched_setaffinity(0,set(sorted(os.sched_getaffinity(0))[:2]))
 env.update(CARGO_HOME=str(W/'cargo'),RUSTUP_HOME=str(W/'rustup'),CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',CARGO_TERM_COLOR='never')
 run('fetch-rustup',['curl','-fsSL','--retry','0','https://sh.rustup.rs','-o',str(W/'rustup-init.sh')])
 run('rustup',['sh',str(W/'rustup-init.sh'),'-y','--profile','minimal','--default-toolchain',P['toolchain'],'--no-modify-path'])
 env['PATH']=str(W/'cargo/bin')+os.pathsep+env['PATH'];run('rustc-version',['rustc','-Vv']);run('cargo-version',['cargo','--version']);run('components',['rustup','component','add','rustfmt','clippy'])
 run('git-init',['git','init',str(R)]);run('git-remote',['git','remote','add','origin',P['repository']],cwd=R)
 lock=base64.b64decode(P['lock_base64']);assert sha(lock)==P['lock_sha256'];(O/'Cargo.lock').write_bytes(lock)
 for label in ['baseline','fixed']:
  head=P['source_heads'][label];run(label+'-fetch',['git','fetch','--depth','1','origin',head],cwd=R);run(label+'-checkout',['git','checkout','--detach',head],cwd=R)
  assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()==head
  assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=R,text=True).strip()==P['source_trees'][label]
  tree=subprocess.check_output(['git','ls-tree','-r','--full-tree',head],cwd=R);assert sha(tree)==P['tree_manifest_sha256'][label];(O/(label+'-git-tree.txt')).write_bytes(tree)
  observed={f:digest(R/f)for f in P['source_files'][label]};assert observed==P['source_files'][label];save(label+'-source.json',{'head':head,'tree':P['source_trees'][label],'files_sha256':observed})
  (R/'Cargo.lock').write_bytes(lock);run(label+'-submodules',['git','submodule','update','--init','--recursive','--depth','1'],cwd=R)
  run(label+'-build',['cargo','test','--locked','-p','kerotakis-core','--test','peroxide_heat_basis','--no-run','--message-format=json'],cwd=R)
  messages=[]
  for line in (O/(label+'-build.stdout')).read_text().splitlines():
   try:messages.append(json.loads(line))
   except json.JSONDecodeError:pass
  exes=[m['executable']for m in messages if m.get('reason')=='compiler-artifact' and m.get('target',{}).get('name')=='peroxide_heat_basis' and m.get('profile',{}).get('test') and m.get('executable')]
  assert len(exes)==1,exes;exe=pathlib.Path(exes[0]);exe.relative_to(R/'target');assert exe.is_file() and os.access(exe,os.X_OK)
  compressed=O/(label+'-peroxide_heat_basis.gz')
  with exe.open('rb')as src,gzip.open(compressed,'wb',compresslevel=1)as dest:shutil.copyfileobj(src,dest,262144)
  save(label+'-test-binary.json',{'head':head,'tree':P['source_trees'][label],'lock_sha256':digest(R/'Cargo.lock'),'executable':str(exe.relative_to(R)),'sha256':digest(exe),'bytes':exe.stat().st_size,'gzip':compressed.name,'gzip_sha256':digest(compressed),'gzip_bytes':compressed.stat().st_size})
  result=run(label,[str(exe),'--nocapture','--test-threads=1'],cwd=R,required=False)
  text=(O/(label+'.stdout')).read_text();observations=re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',text,re.M);counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
  assert len(observations)==4 and {n for n,_ in observations}==set(P['tests']) and len(counts)==1
  passed,failed,ignored=map(int,counts[0]);assert ignored==0 and passed+failed==4 and passed==sum(s=='ok'for _,s in observations) and failed==sum(s=='FAILED'for _,s in observations)
  outcome={'head':head,'exit_status':result,'passed':passed,'failed':failed,'ignored':ignored,'tests':dict(observations),'forecast_not_observation':False};save(label+'-actual-outcome.json',outcome)
  if label=='baseline':
   assert result==101 and failed==3 and passed==1,'Actual baseline differs from frozen forecast; preserve then stop'
   assert all(outcome['tests'][n]=='FAILED'for n in P['baseline_forecast_failures']) and all(outcome['tests'][n]=='ok'for n in P['baseline_forecast_positive'])
   errors=(O/'baseline.stderr').read_text();assert errors.count('reference heat basis mismatch:')==3,'Expected molecular-basis failures, not unrelated panics'
  else:assert result==0 and passed==4 and failed==0
  assert digest(R/'Cargo.lock')==P['lock_sha256'];(O/(label+'-Cargo.lock')).write_bytes((R/'Cargo.lock').read_bytes());post={f:digest(R/f)for f in P['source_files'][label]};assert post==P['source_files'][label];save(label+'-source-after.json',post)
 run('format',['cargo','fmt','--check'],cwd=R)
 run('core-clippy',['cargo','clippy','--locked','-p','kerotakis-core','--all-targets','--','-D','warnings'],cwd=R)
 save('COMPLETED.json',{'source':P['source_heads']['fixed'],'tree':P['source_trees']['fixed'],'baseline_actual_outcome':'baseline-actual-outcome.json','fixed_actual_outcome':'fixed-actual-outcome.json','protected_contexts_replaced':False,'full_source_acceptance':False,'scientific_selections':0,'reference_heat_basis_only':True,'lock_sha256':digest(R/'Cargo.lock')})
except Exception as error:
 save('STOPPED.json',{'error_type':type(error).__name__,'error':str(error),'traceback':traceback.format_exc(),'no_automatic_retry':True});raise
finally:
 files={str(p.relative_to(O)):{'sha256':digest(p),'bytes':p.stat().st_size}for p in O.rglob('*')if p.is_file()};save('manifest.json',files)
