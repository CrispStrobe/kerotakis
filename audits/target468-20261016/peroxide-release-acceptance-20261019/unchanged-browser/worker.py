HEAD='b890ba176bf34a5818d3ceac275f53cde3c22376'
TREE='a4a81fccc2a72b9f6d44f1457335ca9ccf0b746e'
RUNTIME='0b57969911f96bcb97fb159eef725d7db7048801'
HELPER_SHA='1f0ddea0b8a9e16d78cb15cfaaa319abbe2e6cd817999d7075c4e9c235a903a0'
BINDING_SHA='d30504e39c80b0f7f477d7f0632104aea7a013fab49217102d3246ef684b2a24'
import datetime,hashlib,json,os,pathlib,re,shutil,subprocess,time,traceback
O=pathlib.Path('/kaggle/working/browser-evidence');O.mkdir(exist_ok=False)
W=pathlib.Path('/kaggle/temp/kerotakis-live-browser');W.mkdir(parents=True,exist_ok=False)
def sha(raw):return hashlib.sha256(raw).hexdigest()
def save(name,v):
 with (O/name).open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def run(name,args,cwd=W,env=None,required=True,timeout=300):
 with (O/(name+'.stdout')).open('wb') as out,(O/(name+'.stderr')).open('wb') as err:
  p=subprocess.run(args,cwd=cwd,env=env,stdout=out,stderr=err,timeout=timeout)
 save(name+'.json',{'argv':args,'exit_status':p.returncode,'stdout_sha256':sha((O/(name+'.stdout')).read_bytes()),'stderr_sha256':sha((O/(name+'.stderr')).read_bytes())});print(name,p.returncode,flush=True)
 if required:assert p.returncode==0,name
 return p.returncode
save('STARTED.json',{'script_sha256':sha(pathlib.Path(__file__).read_bytes()),'cpu_only':True,'owner_credentials_supplied':False,'runtime_source':RUNTIME,'harness_source':HEAD})
try:
 available=int(next(x.split()[1] for x in pathlib.Path('/proc/meminfo').read_text().splitlines() if x.startswith('MemAvailable:')))//1024
 assert available>=6000 and shutil.disk_usage(W).free>=10*1024**3
 if hasattr(os,'sched_getaffinity'):os.sched_setaffinity(0,set(sorted(os.sched_getaffinity(0))[:2]))
 env=dict(os.environ);env['PLAYWRIGHT_BROWSERS_PATH']=str(W/'browsers');repo=W/'source'
 run('git-init',['git','init',str(repo)]);run('git-remote',['git','remote','add','origin','https://github.com/CrispStrobe/kerotakis.git'],cwd=repo)
 run('git-fetch',['git','fetch','--depth','1','origin',HEAD],cwd=repo);run('git-checkout',['git','checkout','--detach','FETCH_HEAD'],cwd=repo)
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==HEAD
 assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip()==TREE
 helper=repo/'audits/live-rest-browser-20261018/live-client.mjs';bind=repo/'audits/live-rest-browser-20261018/release-binding.json'
 assert sha(helper.read_bytes())==HELPER_SHA and sha(bind.read_bytes())==BINDING_SHA
 (O/'release-binding.json').write_bytes(bind.read_bytes());save('source.json',{'harness_head':HEAD,'harness_tree':TREE,'runtime_source':RUNTIME,'harness_script_sha256':HELPER_SHA,'binding_sha256':BINDING_SHA})
 run('node-checksums',['curl','-fsSL','--retry','0','https://nodejs.org/dist/latest-v22.x/SHASUMS256.txt','-o',str(W/'node-shasums.txt')])
 candidates=[line.split() for line in (W/'node-shasums.txt').read_text().splitlines() if re.fullmatch(r'[0-9a-f]{64}  node-v22\.[0-9.]+-linux-x64\.tar\.xz',line)]
 assert len(candidates)==1;csha,cname=candidates[0]
 run('node-download',['curl','-fsSL','--retry','0','https://nodejs.org/dist/latest-v22.x/'+cname,'-o',str(W/cname)])
 assert sha((W/cname).read_bytes())==csha;node=W/'node';node.mkdir();run('node-extract',['tar','-xJf',str(W/cname),'--strip-components=1','-C',str(node)])
 save('node-distribution.json',{'filename':cname,'sha256':csha,'official_checksum_verified':True});env['PATH']=str(node/'bin')+os.pathsep+env['PATH'];run('node-version',['node','--version'],env=env)
 package=W/'browser';package.mkdir();(package/'package.json').write_text('{"private":true,"type":"module"}\n')
 run('install-playwright',['npm','install','--ignore-scripts','--no-audit','--no-fund','--save-exact','@playwright/test@1.62.1'],cwd=package,env=env,timeout=300)
 run('install-chromium',['npx','playwright','install','--with-deps','chromium'],cwd=package,env=env,timeout=600)
 (O/'package-lock.json').write_bytes((package/'package-lock.json').read_bytes());(package/'live-client.mjs').write_bytes(helper.read_bytes());env['GITHUB_WORKSPACE']=str(repo);env['BROWSER_REPORT_DIR']=str(O/'actual-browser')
 exit_status=run('actual-live-browser',['node',str(package/'live-client.mjs')],cwd=package,env=env,required=False,timeout=240)
 run('source-unchanged',['git','diff','--exit-code','HEAD'],cwd=repo,env=env)
 save('COMPLETED.json',{'harness_head':HEAD,'runtime_source':RUNTIME,'browser_exit_status':exit_status,'raw_report_exists':(O/'actual-browser/report.json').exists(),'root_review_required':True,'owner_credentials_supplied':False,'scientific_catalog_selections':0})
except BaseException as exc:
 save('STOPPED.json',{'error_type':type(exc).__name__,'error':str(exc),'traceback':traceback.format_exc(),'browser_acceptance':False});print(traceback.format_exc(),flush=True);raise
finally:
 save('SHA256.json',{str(p.relative_to(O)):sha(p.read_bytes()) for p in O.rglob('*') if p.is_file()})
