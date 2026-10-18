PLAN={'frozen_utc': '2026-10-18T20:22:21.350764+00:00', 'accepted_runtime_base': 'd3883757ef12433561cb7831a9c9ce3f1b999eef', 'baseline': '2157e5fa5891abc5fea03334d3779a8a883bfbb3', 'baseline_tree': 'eee022a5efb9c69dc4d277ca336008d2dcdc4fff', 'fix': 'bf03c11ac1d427e371071897543e2e1b2d81d67b', 'fix_tree': '9e47bc312d60b55b7cfb162982db696b7871fe84', 'frozen_control_sha256': {'crates/kerotakis-core/tests/named_alkaline.rs': '2e7e7d79523870abc46d7b75c5b2378a3d15444d90aa7d5e4c568a2e30f298c1', 'crates/kerotakis-phreeqc/tests/named_hydrolysis.rs': 'c3c44e13733d90a9ce975f88217cab771361f85085dc96a5ca9fdfbc83dc6325'}, 'fix_source_sha256': {'crates/kerotakis-core/tests/named_alkaline.rs': '2e7e7d79523870abc46d7b75c5b2378a3d15444d90aa7d5e4c568a2e30f298c1', 'crates/kerotakis-phreeqc/tests/named_hydrolysis.rs': 'c3c44e13733d90a9ce975f88217cab771361f85085dc96a5ca9fdfbc83dc6325', 'crates/kerotakis-core/src/bench.rs': '8f765b0c7b70c510531d0a4d56e9d3e6f13b6490ab4c93afb788263630767cb9', 'crates/kerotakis-core/src/lib.rs': 'cc0022bb26e09b9edb86f0ff8809f7238831059f1fca6c177c6b34c692ab1334', 'crates/kerotakis-core/src/named_alkaline.rs': '5d0260e78827f5dc6d47693db31624e3a78666a51e06f8ef62de1c11c6005099', 'docs/named-alkaline-capacity.md': 'c20c373f1185313e32bbff0a1737653787bbe0f9bd97d2c3112e28ed31c7632c', 'crates/kerotakis-core/i18n/de.toml': 'fe9c4b6bb10c79e8a40f23a508b1bdcf8209ccbcab909b0bee8d3bcb26643710', 'crates/kerotakis-core/i18n/fr.toml': 'a7162e6634a8b06ce4230b563a9f502b78c15f17a5178f45987646bcc5129932'}, 'baseline_commands': [['cargo', 'test', '-p', 'kerotakis-core', '--test', 'named_alkaline', '--', '--nocapture'], ['cargo', 'test', '-p', 'kerotakis-phreeqc', '--test', 'named_hydrolysis', '--', '--nocapture']], 'fix_commands': [['cargo', 'clippy', '-p', 'kerotakis-core', '-p', 'kerotakis-phreeqc', '-p', 'kerotakis-cea', '--all-targets', '--', '-D', 'warnings'], ['cargo', 'test', '-p', 'kerotakis-phreeqc', '-p', 'kerotakis-cea', '--lib', '--tests', '--no-fail-fast', '--', '--nocapture'], ['cargo', 'test', '-p', 'kerotakis-core', '--lib', '--tests', '--no-fail-fast', '--', '--nocapture']], 'cpu_jobs': 2, 'worker_seconds': 6800, 'provider_timeout_seconds': 7200, 'original_corpus_replays': 0, 'first_outcomes_immutable': True, 'remaining_acceptance': 'Root independent chemical/charge/proton/counterion review, applicable full33/protected/platform integration and original OR006 delivery replay. No scientific selection.', 'previous_fix': '03aab10c020a7020ee366d58d928ca8219181b5a', 'baseline_reexecuted': False, 'previous_authenticated_failure_sha256': '514e5fec4bbf4ad8cd4d4552ef5f4ff7fc193864231a815eba1ef5352a1344fd', 'baseline_manifest_sha256': '79ab6165bde4c3f5dd7709f0f548e1ec2b51cdda57722763b826264990068a5a', 'expected_producer_lock_sha256': '668868136fa9ee1ebd6433d59eb68d5fa46224d874fd96c4036f7614b3c4cd85'}
"""Private, explicitly authorized CPU-only native controls; no production access."""
import datetime,hashlib,json,os,pathlib,platform,shutil,subprocess,time,traceback
OUT=pathlib.Path('/kaggle/working/kerotakis-evidence');OUT.mkdir(exist_ok=False)
WORK=pathlib.Path('/kaggle/temp/kerotakis-native-controls');WORK.mkdir(parents=True,exist_ok=False)
START=time.monotonic();STEPS=[]
def sha(raw):return hashlib.sha256(raw).hexdigest()
def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def save(name,data):
 p=OUT/name
 with p.open('x') as f:json.dump(data,f,indent=2);f.write('\n')
def run(label,args,cwd=WORK,env=None,required=True,seconds=1200):
 assert time.monotonic()-START<6600,'Overall worker budget reached'
 begin=time.monotonic()
 with (OUT/(label+'.stdout')).open('wb') as o,(OUT/(label+'.stderr')).open('wb') as e:
  try:p=subprocess.run(args,cwd=cwd,env=env,stdout=o,stderr=e,timeout=min(seconds,max(1,6700-(time.monotonic()-START))))
  except subprocess.TimeoutExpired:
   save(label+'.timeout.json',{'utc':utc(),'argv':args,'timeout':True});raise
 record={'label':label,'utc':utc(),'argv':args,'exit_status':p.returncode,'elapsed_seconds':time.monotonic()-begin,'stdout_sha256':sha((OUT/(label+'.stdout')).read_bytes()),'stderr_sha256':sha((OUT/(label+'.stderr')).read_bytes())}
 save(label+'.json',record);STEPS.append(record);print(json.dumps(record),flush=True)
 if required:assert p.returncode==0,label
 return p.returncode
save('STARTED.json',{'utc':utc(),'script_sha256':sha(pathlib.Path(__file__).read_bytes()),'cpu_only':True,'gpu_requested':False,'platform':platform.platform(),'cpu_count':os.cpu_count(),'memory_info':pathlib.Path('/proc/meminfo').read_text(),'disk_free_bytes':shutil.disk_usage(WORK).free,'source_builds_not_empirical_calibration':True})
try:
 mem=int(next(x.split()[1] for x in pathlib.Path('/proc/meminfo').read_text().splitlines() if x.startswith('MemAvailable:')))//1024
 assert mem>=6000 and shutil.disk_usage(WORK).free>=10*1024**3,'Insufficient isolated worker resources'
 env=dict(os.environ);env.update(CARGO_HOME=str(WORK/'cargo-home'),RUSTUP_HOME=str(WORK/'rustup-home'),CARGO_TARGET_DIR=str(WORK/'target'),CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',CARGO_TERM_COLOR='never')
 run('fetch-rustup',['curl','--proto','=https','--tlsv1.2','-fsSL','--retry','0','https://sh.rustup.rs','-o',str(WORK/'rustup-init.sh')],env=env,seconds=90)
 save('rustup-bootstrap.json',{'sha256':sha((WORK/'rustup-init.sh').read_bytes()),'toolchain':'1.99.1','toolchain_matches_first_hosted_native_compiler':True})
 run('install-rust',['sh',str(WORK/'rustup-init.sh'),'-y','--profile','minimal','--default-toolchain','1.99.1','--no-modify-path'],env=env,seconds=420)
 os.sched_setaffinity(0,set(sorted(os.sched_getaffinity(0))[:2]));env['CMAKE_BUILD_PARALLEL_LEVEL']='2';env['PATH']=str(WORK/'cargo-home/bin')+os.pathsep+env['PATH'];run('rustc-version',['rustc','--version','--verbose'],env=env);run('cargo-version',['cargo','--version'],env=env)
 run('rustup-clippy',['rustup','component','add','clippy'],env=env,seconds=240)
 repo=WORK/'source';run('git-init',['git','init',str(repo)],env=env)
 run('git-remote',['git','remote','add','origin','https://github.com/CrispStrobe/kerotakis.git'],cwd=repo,env=env)
 def checkout(label,source,tree):
  run(label+'-fetch',['git','fetch','--depth','1','origin',source],cwd=repo,env=env,seconds=180)
  run(label+'-checkout',['git','checkout','--detach','FETCH_HEAD'],cwd=repo,env=env)
  assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==source
  assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip()==tree
 checkout('fix',PLAN['fix'],PLAN['fix_tree'])
 run('init-submodules',['git','submodule','update','--init','--recursive','--depth','1'],cwd=repo,env=env,seconds=360)
 run('fetch-sundials',['bash','tools/fetch-sundials.sh'],cwd=repo,env=env,seconds=180)
 save('frozen-plan.json',PLAN)
 for path,digest in PLAN['fix_source_sha256'].items():assert sha((repo/path).read_bytes())==digest
 for index,command in enumerate(PLAN['fix_commands']):
  run('fix-'+str(index),command,cwd=repo,env=env,seconds=3000)
  lock=(repo/'Cargo.lock').read_bytes();assert sha(lock)==PLAN['expected_producer_lock_sha256'],'Changed producer dependency resolution'
  if index==0:(OUT/'fix-Cargo.lock').write_bytes(lock)
 run('fix-source-unchanged',['git','diff','--exit-code','HEAD'],cwd=repo,env=env)
 save('FIX-CONTROLS-PASSED.json',{'source':PLAN['fix'],'tree':PLAN['fix_tree'],'baseline':PLAN['baseline'],'baseline_not_reexecuted':True,'actual_producer_lock_sha256':sha(lock),'same_frozen_control_files':True,'root_independent_actual_log_and_chemical_review_required':True,'scientific_selections':0})
 save('COMPLETED.json',{'utc':utc(),'elapsed_seconds':time.monotonic()-START,'steps':STEPS,'root_log_and_physical_review_required':True,'github_protected_contexts_unchanged':True,'scientific_catalog_selections':0})
except BaseException as exc:
 save('STOPPED.json',{'utc':utc(),'error_type':type(exc).__name__,'error':str(exc),'traceback':traceback.format_exc(),'steps':STEPS,'no_retry':True,'test_acceptance':False});print(traceback.format_exc(),flush=True);raise
finally:
 manifest={p.name:sha(p.read_bytes()) for p in OUT.iterdir() if p.is_file()};save('SHA256.json',manifest)
