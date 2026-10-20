"""Generic host pins/admission/resource guards. No requests at import."""
from pathlib import Path
import ast,base64,hashlib,json,os,re,shutil,subprocess,time
from release_binding import checked_binding,RECEIPTS,canonical
SOURCE='33e043ef4a4a4225e5ea8d49ba93e8df99716c5f'
def digest(path):
 with Path(path).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(path,value):
 with Path(path).open('x',encoding='utf-8')as f:json.dump(value,f,indent=2,allow_nan=False);f.write('\n')
def safe_relative(name):
 p=Path(name);assert name and not p.is_absolute()and all(x not in ['', '.', '..']for x in name.split('/'))and '\\'not in name and '\x00'not in name;return p
def validate_ready(bound,ready_path,ready_sha):
 assert re.fullmatch('[a-f0-9]{64}',ready_sha)and digest(ready_path)==ready_sha
 ready=json.loads(ready_path.read_text(encoding='utf-8'));assert ready['runtime_source']==SOURCE and ready['parent_reviewed_exact_producer_and_readers']is True
 here=Path(__file__).resolve().parent
 for name in ['host_common.py','release_binding.py','issue_once.py','collect_once.py','fast_spool.py']:assert digest(here/name)==ready['host_files_sha256'][name],name
 for name in ['CONFIG.json','grouped_browser.py','BOUND-NOT-ISSUED.json']:assert digest(bound/name)==ready['bound_files_sha256'][name],name
 raw=(bound/'CONFIG.json').read_bytes();config=json.loads(raw);assert raw==canonical(config)+b'\n'and hashlib.sha256(canonical(config)).hexdigest()==ready['config_sha256'];assert config['runtime_source']==SOURCE
 tree=ast.parse((bound/'grouped_browser.py').read_bytes());values={}
 for node in tree.body:
  if isinstance(node,ast.Assign):
   for t in node.targets:
    if isinstance(t,ast.Name)and t.id in ['CONFIG','CONFIG_SHA256']:assert t.id not in values;values[t.id]=ast.literal_eval(node.value)
 assert values['CONFIG']==config and values['CONFIG_SHA256']==ready['config_sha256']
 assert digest(bound/'grouped_browser.py')==ready['worker_sha256']
 assert (bound/'grouped_browser.py').stat().st_size<1024**2,'Provider code payload must stay below1MiB; require separate reviewed transport if larger'
 marker=json.loads((bound/'BOUND-NOT-ISSUED.json').read_text(encoding='utf-8'));assert marker['worker_sha256']==ready['worker_sha256']and marker['config_sha256']==ready['config_sha256']and marker['runtime_source']==SOURCE
 decoded={}
 for name,row in config['inputs'].items():
  safe_relative(name);data=base64.b64decode(row['base64'],validate=True);assert hashlib.sha256(data).hexdigest()==row['sha256'];decoded[name]=data
 raw_receipts={name:decoded['prior-acceptance/'+name]for name in RECEIPTS}
 binding,prior=checked_binding(config['binding']['binding'],raw_receipts,SOURCE,config['runtime_tree'],decoded['forecasts/original-api-protocols.json'])
 assert binding==config['binding']['binding']and prior==config['prior_release']
 assert ready['frozen_input_files_sha256']=={name:row['sha256']for name,row in config['inputs'].items()}
 assert ready['private_legacy_reader_closure_reviewed']is True and ready['no_owner_key_contents_in_upload']is True
 return ready,config

def resource_open(fast_root,storage_root,allowance=2*1024**3):
 memory=int(next(x.split()[1]for x in Path('/proc/meminfo').read_text(encoding='utf-8').splitlines()if x.startswith('MemAvailable:')))//1024
 return memory>=3200 and os.getloadavg()[0]<=3.5 and shutil.disk_usage(fast_root).free>=3*1024**3+allowance and shutil.disk_usage(storage_root).free>=25*1024**3

def require_ext4(root):assert subprocess.check_output(['findmnt','-T',str(root),'-n','-o','FSTYPE'],timeout=30).decode('utf-8').strip()=='ext4'
def wait_resource(fast,storage,allowance,deadline):
 while time.monotonic()<deadline:
  if resource_open(fast,storage,allowance):return
  time.sleep(min(30,max(.001,deadline-time.monotonic())))
 raise AssertionError('Bounded resource deadline; no threshold waiver')
