"""Single CPU producer preparation. CONFIG remains unbound; import performs no work."""
import base64,datetime,hashlib,json,os,re,shutil,signal,subprocess,time,traceback
from pathlib import Path,PurePosixPath
CONFIG=None
CONFIG_SHA256=None
WORK_DEADLINE=None
GROUPS=[('browser37','live-client.mjs',37,240),('targeted7','targeted.mjs',7,240),('locale18','run-locale18.mjs',18,300),('heat2','run-heat.mjs',2,300)]
FROZEN_MODULES={'live-client.mjs': '1f0ddea0b8a9e16d78cb15cfaaa319abbe2e6cd817999d7075c4e9c235a903a0', 'targeted.mjs': 'f17c5c7fe143d9930efeeafa8cf9c14c1a873ba10ea4927ca5fc30fa91cba4f9', 'locale-controls.mjs': '7290327a0d59d981bcc8bc0e35b5d0941c3559d14067ecdf488ae4f24c0f8819', 'run-locale18.mjs': '8456b089e8389c2995fd1c58bcd316c64fffcfcad4db002927f336fa58a8bd21', 'heat-controls.mjs': 'cf19825bd6a273115eb686feab0b3242dfebb5533e571fed4765b8f49c371d6f', 'run-heat.mjs': '316980a04ba85c94a54665f1ad5418a310f8485892afe97b85c9ba97daa6d592'}
def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def digest(path):
 with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode('utf-8')
def save(path,value):
 with Path(path).open('x',encoding='utf-8')as stream:json.dump(value,stream,indent=2,allow_nan=False);stream.write('\n')
def write_admission(path,config):
 with Path(path).open('xb')as stream:stream.write(canonical(config)+b'\n')
def safe_name(name):
 assert type(name)is str and name and not PurePosixPath(name).is_absolute()and all(x not in ['', '.', '..']for x in name.split('/'))and '\\'not in name and '\x00'not in name
 return Path(name)
def raw_commit_parents(repo,head):
 raw=subprocess.check_output(['git','cat-file','-p',head],cwd=repo,timeout=60).decode('utf-8',errors='strict');header=raw.split('\n\n',1)[0].splitlines()
 return [line.removeprefix('parent ')for line in header if line.startswith('parent ')]
def manifest(root):return {str(p.relative_to(root)):digest(p)for p in sorted(root.rglob('*'))if p.is_file()and p!=root/'SHA256.json'}
def command(root,name,argv,cwd,env,timeout):
 """One exact argv, fresh owned process group; preserve streams on failure/timeout."""
 assert type(argv)is list and argv and all(type(s)is str for s in argv)
 if WORK_DEADLINE is not None:timeout=min(timeout,max(.001,WORK_DEADLINE-time.monotonic()));assert time.monotonic()<WORK_DEADLINE,'Overall3300s producer deadline'
 started=time.monotonic();process=None;status=None;failure=None
 try:
  with(root/(name+'.stdout')).open('xb')as out,(root/(name+'.stderr')).open('xb')as err:
   process=subprocess.Popen(argv,cwd=cwd,env=env,stdout=out,stderr=err,start_new_session=True)
   try:status=process.wait(timeout=timeout)
   except BaseException:
    if process.poll()is None:
     os.killpg(process.pid,signal.SIGTERM)
     try:process.wait(timeout=5)
     except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=10)
    status=process.returncode;raise
 except BaseException as error:failure=type(error).__name__;raise
 finally:
  save(root/(name+'.json'),{'argv':argv,'exit_status':status,'pid':process.pid if process else None,'fresh_owned_session':True,'cwd':str(cwd),'elapsed_seconds':time.monotonic()-started,'error_type':failure,'stdout_sha256':digest(root/(name+'.stdout'))if(root/(name+'.stdout')).is_file()else None,'stderr_sha256':digest(root/(name+'.stderr'))if(root/(name+'.stderr')).is_file()else None})
 return status

def execute_groups(groups,package,out,env,check_closure,source_fields,worker_sha,argv_for=None,source_command=None):
 """Execution metadata only. argv_for is a test seam; production always uses exact Node argv."""
 observations=[]
 for index,g in enumerate(groups):
  root=out/'groups'/g['id'];root.mkdir(parents=True,exist_ok=False);status=None;report_exists=False
  groupenv=dict(env);groupenv['BROWSER_REPORT_DIR']=str(root/'actual-browser')
  groupenv['HEAT_RELEASE_BINDING']=str(package/'heat-release-binding.json')
  try:
   before=check_closure();save(root/'source-before.json',before)
   save(root/'STARTED.json',{'script_sha256':worker_sha,'runtime_source':source_fields['runtime_source'],'harness_source':source_fields['harness_head'],'group':g['id'],'execution_only_not_parent_acceptance':True})
   save(root/'source.json',dict(source_fields,group=g['id'],entrypoint_path=g['entry'],entrypoint_sha256=digest(package/g['entry'])))
   save(root/'group-design.json',{'source':source_fields['runtime_source'],'worker_sha256':worker_sha,'binding':source_fields['binding'],'entrypoint_path':g['entry'],'entrypoint_sha256':digest(package/g['entry']),'modules_sha256':source_fields['modules_sha256'],'expected_checks':g['expected_checks'],'input_files_sha256':before['input_files_sha256'],'execution_only_not_parent_acceptance':True})
   argv=argv_for(g)if argv_for else ['node',str(package/g['entry'])]
   child_error=None
   try:status=command(root,'actual-live-browser',argv,package,groupenv,g['timeout_seconds'])
   except BaseException as error:
    child_error=error;record=root/'actual-live-browser.json'
    if record.is_file():status=json.loads(record.read_text(encoding='utf-8'))['exit_status']
   # Check closure even after timeout/child transport failure; preserve both failures if present.
   try:
    after=check_closure();save(root/'source-after.json',after);assert after==before,'Frozen source/input closure changed during group'
    if source_command is not None:source_command(root)
   except BaseException as closure_error:
    save(root/'source-after-error.json',{'error_type':type(closure_error).__name__,'error':str(closure_error)})
    if child_error is not None:raise child_error from closure_error
    raise
   if child_error is not None:raise child_error
   report=root/'actual-browser/report.json';report_exists=report.is_file()
   save(root/'COMPLETED.json',{'harness_head':source_fields['harness_head'],'runtime_source':source_fields['runtime_source'],'browser_exit_status':status,'raw_report_exists':report_exists,'root_review_required':True,'owner_credentials_supplied':False,'scientific_catalog_selections':0,'group':g['id'],'parent_acceptance_assigned':False})
   assert status==0,'Group child failed: '+g['id'];assert report_exists,'Missing raw report: '+g['id']
   raw=json.loads(report.read_text(encoding='utf-8'));assert type(raw)is dict and raw.get('all_checks_passed')is True,'Original child assertions did not report completion'
   assert type(raw.get('checks'))is list and len(raw['checks'])==g['expected_checks']and all(type(c)is dict and c.get('passed')is True for c in raw['checks']),'Original child check list incomplete/failed'
   observations.append({'group':g['id'],'exit_status':status,'report_sha256':digest(report),'expected_checks':g['expected_checks'],'parent_acceptance_assigned':False})
  except BaseException as error:
   save(root/'STOPPED.json',{'group':g['id'],'error_type':type(error).__name__,'error':str(error),'exit_status':status,'parent_acceptance_assigned':False,'later_groups_not_executed':[x['id']for x in groups[index+1:]],'all_first_partial_outputs_preserved':True});raise
  finally:save(root/'SHA256.json',manifest(root))
 return observations

def validate_config(config,expected_sha):
 assert type(config)is dict and re.fullmatch('[a-f0-9]{64}',expected_sha or '')and hashlib.sha256(canonical(config)).hexdigest()==expected_sha,'Actual immutable execution admission missing/unbound'
 assert config['reviewed_candidate_source']=='18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a'
 for k in ['runtime_source','runtime_tree','harness_head','harness_tree']:assert re.fullmatch('[a-f0-9]{40}',config[k])
 assert config['runtime_source']=='33e043ef4a4a4225e5ea8d49ba93e8df99716c5f'and config['runtime_tree']=='f77a9e82941f30a6433583e1bd417939118d13c6'
 assert [(g['id'],g['entry'],g['expected_checks'],g['timeout_seconds'])for g in config['groups']]==GROUPS
 inputs=config['inputs'];assert type(inputs)is dict and set(FROZEN_MODULES)<=set(inputs)and len(inputs)<=256;total_input_bytes=0
 for name,row in inputs.items():
  safe_name(name);assert name in FROZEN_MODULES or name in ['release-binding.json','heat-release-binding.json']or name.startswith(('saved-readers/','forecasts/','prior-acceptance/'))
  assert set(row)=={'base64','sha256'}and re.fullmatch('[a-f0-9]{64}',row['sha256']);raw=base64.b64decode(row['base64'],validate=True);assert base64.b64encode(raw).decode('ascii')==row['base64']and hashlib.sha256(raw).hexdigest()==row['sha256']
  total_input_bytes+=len(raw);assert len(raw)<=2*1024**2 and total_input_bytes<=8*1024**2,'Bounded frozen source admission exceeded'
 for name,want in FROZEN_MODULES.items():assert inputs[name]['sha256']==want,name
 assert inputs['release-binding.json']['sha256']==config['binding_sha256']and inputs['heat-release-binding.json']['sha256']==config['heat_binding_sha256']
 bind=json.loads(base64.b64decode(inputs['release-binding.json']['base64']));heat=json.loads(base64.b64decode(inputs['heat-release-binding.json']['base64']));envelope=config['binding'];prior=config['prior_release']
 assert envelope['binding']==bind and envelope['runtime_source']==config['runtime_source']and envelope['harness_head']==config['harness_head']and envelope['harness_tree']==config['harness_tree']and envelope['binding_sha256']==config['binding_sha256']and envelope['harness_script_sha256']==FROZEN_MODULES['live-client.mjs']
 assert set(prior['frontend_assets'])=={'iphreeqc.mjs','iphreeqc.wasm','kerotakis.mjs','kerotakis_wasm.js','kerotakis_wasm_bg.wasm'}
 for k in ['API_receipt_sha256','verified_artifact_receipt_sha256','native_binary_sha256','native_actual_producer_lock_sha256','client_sha256']:assert re.fullmatch('[a-f0-9]{64}',prior[k])
 for value in prior['frontend_assets'].values():assert re.fullmatch('[a-f0-9]{64}',value)
 assert bind['expected_engine_source']==heat['expected_engine_source']==config['runtime_source']and heat['reviewed_candidate_source']==config['reviewed_candidate_source']
 assert bind['frontend_assets']==heat['frontend_assets']==prior['frontend_assets']and bind['client_sha256']==heat['client_sha256']==prior['client_sha256']
 assert bind['expected_binary_sha256']==prior['native_binary_sha256']and envelope['engine']=={'source_head':config['runtime_source'],'binary_sha256':prior['native_binary_sha256'],'cargo_lock_sha256':prior['native_actual_producer_lock_sha256']}
 assert bind['native_bridge_reference']['engine']['source_head']==config['runtime_source']and bind['native_bridge_reference']['engine']['binary_sha256']==prior['native_binary_sha256']
 assert bind['native_bridge_reference']['engine']==envelope['engine']
 for name,key in [('API-ACCEPTED.json','API_receipt_sha256'),('verified-release.json','verified_artifact_receipt_sha256')]:assert inputs['prior-acceptance/'+name]['sha256']==prior[key]
 api=json.loads(base64.b64decode(inputs['prior-acceptance/API-ACCEPTED.json']['base64']));verified=json.loads(base64.b64decode(inputs['prior-acceptance/verified-release.json']['base64']));assessment=json.loads(base64.b64decode(inputs['prior-acceptance/script-assessment.json']['base64']))
 assert api['source']==config['runtime_source']and api['engine']==envelope['engine']and api['eleven_original_protocols_pass']is True and api['concurrency']==[200,429]
 assert all(verified['engine'][key]==value for key,value in envelope['engine'].items())and {name:row['sha256']for name,row in verified['frontend_assets'].items()if name!='rest.html'}==prior['frontend_assets']and verified['frontend_assets']['rest.html']['sha256']==prior['client_sha256']
 assert assessment['all_script_controls_pass']is True and assessment['native_browser_reference']==bind['native_bridge_reference']
 return inputs

def main():
 global WORK_DEADLINE
 inputs=validate_config(CONFIG,CONFIG_SHA256) # Unbound preparation stops before filesystem/network/product work.
 WORK_DEADLINE=time.monotonic()+3300
 out=Path('/kaggle/working/browser-evidence');out.mkdir(exist_ok=False);work=Path('/kaggle/temp/kerotakis-grouped-browser850');work.mkdir(parents=True,exist_ok=False);worker_sha=digest(Path(__file__));env=dict(os.environ)
 def run(name,argv,cwd=work,timeout=300):
  status=command(out,name,argv,cwd,env,timeout);assert status==0,name
 def git(*argv):return subprocess.check_output(['git',*argv],cwd=repo,timeout=60).decode('utf-8',errors='strict').strip()
 try:
  save(out/'STARTED.json',{'script_sha256':worker_sha,'cpu_only':True,'owner_credentials_supplied':False,'runtime_source':CONFIG['runtime_source'],'harness_source':CONFIG['harness_head']});write_admission(out/'execution-admission.json',CONFIG);save(out/'design-binding.json',{'config_sha256':CONFIG_SHA256,'worker_sha256':worker_sha,'parent_acceptance_assigned':False})
  memory=int(next(x.split()[1]for x in Path('/proc/meminfo').read_text().splitlines()if x.startswith('MemAvailable:')))//1024;assert memory>=6000 and shutil.disk_usage(work).free>=10*1024**3
  if hasattr(os,'sched_getaffinity'):os.sched_setaffinity(0,set(sorted(os.sched_getaffinity(0))[:2]))
  env['PLAYWRIGHT_BROWSERS_PATH']=str(work/'browsers');repo=work/'source';run('git-init',['git','init',str(repo)]);run('git-remote',['git','remote','add','origin','https://github.com/CrispStrobe/kerotakis.git'],cwd=repo);run('git-fetch',['git','fetch','--depth','1','origin',CONFIG['harness_head']],cwd=repo);run('git-checkout',['git','checkout','--detach','FETCH_HEAD'],cwd=repo);run('git-runtime-fetch',['git','fetch','--depth','1','origin',CONFIG['runtime_source']],cwd=repo)
  assert git('rev-parse','HEAD')==CONFIG['harness_head']and git('rev-parse','HEAD^{tree}')==CONFIG['harness_tree']and git('rev-parse',CONFIG['runtime_source']+'^{tree}')==CONFIG['runtime_tree']
  assert raw_commit_parents(repo,'HEAD')==[CONFIG['runtime_source']]
  assert set(git('diff','--name-only',CONFIG['runtime_source'],'HEAD').splitlines())=={'audits/live-rest-browser-20261018/live-client.mjs','audits/live-rest-browser-20261018/release-binding.json'}
  package=work/'browser';package.mkdir();frozen=out/'frozen-inputs';frozen.mkdir()
  for name,row in inputs.items():
   raw=base64.b64decode(row['base64'],validate=True)
   for root in [package,frozen]:path=root/safe_name(name);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
  binding_path=repo/'audits/live-rest-browser-20261018/release-binding.json';helper=repo/'audits/live-rest-browser-20261018/live-client.mjs';assert digest(binding_path)==CONFIG['binding_sha256']and digest(helper)==FROZEN_MODULES['live-client.mjs']
  (out/'release-binding.json').write_bytes(binding_path.read_bytes())
  run('node-checksums',['curl','-fsSL','--retry','0','https://nodejs.org/dist/latest-v22.x/SHASUMS256.txt','-o',str(work/'node-shasums.txt')]);candidates=[line.split()for line in(work/'node-shasums.txt').read_text(encoding='utf-8').splitlines()if re.fullmatch(r'[0-9a-f]{64}  node-v22\.[0-9.]+-linux-x64\.tar\.xz',line)];assert len(candidates)==1;node_sha,node_name=candidates[0]
  run('node-download',['curl','-fsSL','--retry','0','https://nodejs.org/dist/latest-v22.x/'+node_name,'-o',str(work/node_name)]);assert digest(work/node_name)==node_sha;node=work/'node';node.mkdir();run('node-extract',['tar','-xJf',str(work/node_name),'--strip-components=1','-C',str(node)]);save(out/'node-distribution.json',{'filename':node_name,'sha256':node_sha,'official_checksum_verified':True});env['PATH']=str(node/'bin')+os.pathsep+env['PATH'];run('node-version',['node','--version'])
  (package/'package.json').write_text('{"private":true,"type":"module"}\n',encoding='utf-8');run('install-playwright',['npm','install','--ignore-scripts','--no-audit','--no-fund','--save-exact','@playwright/test@1.62.1'],cwd=package);run('install-chromium',['npx','playwright','install','--with-deps','chromium'],cwd=package,timeout=600);(out/'package-lock.json').write_bytes((package/'package-lock.json').read_bytes());env['GITHUB_WORKSPACE']=str(repo)
  def closure():
   assert time.monotonic()<WORK_DEADLINE,'Overall3300s producer deadline'
   assert git('rev-parse','HEAD')==CONFIG['harness_head']and git('rev-parse','HEAD^{tree}')==CONFIG['harness_tree'];assert digest(binding_path)==CONFIG['binding_sha256']and digest(helper)==FROZEN_MODULES['live-client.mjs']
   for name,row in inputs.items():assert digest(package/name)==row['sha256']and digest(frozen/name)==row['sha256'],name
   assert subprocess.run(['git','diff','--exit-code','HEAD'],cwd=repo,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=60).returncode==0
   return {'harness_head':CONFIG['harness_head'],'harness_tree':CONFIG['harness_tree'],'runtime_source':CONFIG['runtime_source'],'binding_sha256':CONFIG['binding_sha256'],'input_files_sha256':{name:row['sha256']for name,row in inputs.items()}}
  fields={'harness_head':CONFIG['harness_head'],'harness_tree':CONFIG['harness_tree'],'runtime_source':CONFIG['runtime_source'],'harness_script_sha256':FROZEN_MODULES['live-client.mjs'],'binding_sha256':CONFIG['binding_sha256'],'binding':CONFIG['binding'],'modules_sha256':FROZEN_MODULES}
  def source_command(root):
   status=command(root,'source-unchanged',['git','diff','--exit-code','HEAD'],repo,env,60);assert status==0
  observed=execute_groups(CONFIG['groups'],package,out,env,closure,fields,worker_sha,source_command=source_command)
  save(out/'COMPLETED.json',{'runtime_source':CONFIG['runtime_source'],'harness_head':CONFIG['harness_head'],'groups':observed,'all_four_children_executed_with_zero_status':True,'parent_acceptance_assigned':False,'scientific_catalog_selections':0})
 except BaseException as error:save(out/'STOPPED.json',{'error_type':type(error).__name__,'error':str(error),'traceback':traceback.format_exc(),'all_partial_outputs_preserved':True,'parent_acceptance_assigned':False});raise
 finally:save(out/'SHA256.json',manifest(out))
if __name__=='__main__':main()
