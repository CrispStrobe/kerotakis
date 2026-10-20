"""Collect one issued TC008 focus version; bytes only, no scientific verdict."""
from pathlib import Path
import argparse,ast,datetime,hashlib,json,os,shutil,subprocess,time,traceback
SOURCE='33e043ef4a4a4225e5ea8d49ba93e8df99716c5f'
def digest(path):
 with Path(path).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(path,value):
 with path.open('x',encoding='utf-8')as f:json.dump(value,f,indent=2,allow_nan=False);f.write('\n')
def require_ext4(root):assert subprocess.check_output(['findmnt','-T',str(root),'-n','-o','FSTYPE'],timeout=30).decode().strip()=='ext4'
def wait_resource(fast,storage,allowance,deadline):
 while time.monotonic()<deadline:
  mem=int(next(x.split()[1]for x in Path('/proc/meminfo').read_text().splitlines()if x.startswith('MemAvailable:')))//1024
  if mem>=3200 and os.getloadavg()[0]<=3.5 and shutil.disk_usage(fast).free>=3*1024**3+allowance and shutil.disk_usage(storage).free>=25*1024**3:return
  time.sleep(min(30,max(.001,deadline-time.monotonic())))
 raise AssertionError('Bounded resource deadline; no threshold waiver')
from fast_spool import safe_name,download,copy_verified,MAX_FILE

def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def unique_pairs(rows):
 out={}
 for key,value in rows:assert key not in out;out[key]=value
 return out

def versioned_request(request_type,owner,kernel,version,*,page_size=None,page_token=None):
 assert type(version)is int and version==1,'Only actual externally bound versionone is admitted'
 q=request_type();q.user_name=owner;q.kernel_slug=kernel;q.version_label="v1"
 if page_size is not None:q.page_size=page_size
 if page_token:q.page_token=page_token
 return q

def main():
 p=argparse.ArgumentParser();p.add_argument('--execute',action='store_true')
 for name in ['state','fast-root','storage-root']:p.add_argument('--'+name,type=Path,required=True)
 for name in ['issue-sha256','plan-sha256']:p.add_argument('--'+name,required=True)
 a=p.parse_args()
 if not a.execute:print('PREPARED ONLY: no provider observation or download.');return
 state=a.state.resolve(strict=True);issue_path=state/'ISSUE.json';plan_path=state/'actual-plan.private.json'
 assert digest(issue_path)==a.issue_sha256 and digest(plan_path)==a.plan_sha256
 issue=json.loads(issue_path.read_text());plan=json.loads(plan_path.read_text())
 assert issue['version']==1 and type(issue['version'])is int and issue['version_label']=='v1' and issue['one_issue_only']is True and issue['runtime_source']==plan['source']==SOURCE and issue['actual_plan_sha256']==a.plan_sha256
 workers=list((state/'upload').glob('*.py'));assert len(workers)==1 and digest(workers[0])==issue['worker_sha256']
 tree=ast.parse(workers[0].read_bytes());embedded=ast.literal_eval(next(n.value for n in tree.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='P'for t in n.targets)));assert embedded==plan
 assert (state/'PUSH-ATTEMPT.json').is_file()
 fast_root=a.fast_root.resolve(strict=True);storage_root=a.storage_root.resolve(strict=True);require_ext4(fast_root);assert state.is_relative_to(fast_root)
 deadline=time.monotonic()+14400;wait_resource(fast_root,storage_root,MAX_FILE,deadline)
 collection=state/'collection';assert not collection.exists()and not collection.is_symlink();collection.mkdir(mode=0o700)
 save(collection/'COLLECTOR-STARTED.json',{'utc':utc(),'program_sha256':digest(Path(__file__)),'issue_sha256':a.issue_sha256,'worker_sha256':issue['worker_sha256'],'actual_plan_sha256':a.plan_sha256,'no_provider_mutation_or_retry':True})
 def gate(allowance=MAX_FILE):wait_resource(fast_root,storage_root,allowance,deadline)
 def event(value):
  with(collection/'events.jsonl').open('a',encoding='utf-8')as f:f.write(json.dumps(dict(utc=utc(),**value))+'\n')
 try:
  from kaggle.api.kaggle_api_extended import KaggleApi
  from kagglesdk.kernels.types.kernels_api_service import ApiListKernelSessionOutputRequest,ApiGetKernelSessionStatusRequest
  import requests
  api=KaggleApi();api.authenticate();slug=issue['kernel_id'].removeprefix('/code/');owner,kernel,parsed_version=api.parse_kernel_string(slug);assert parsed_version is None;errors=0
  while time.monotonic()<deadline:
   gate()
   try:
    with api.build_kaggle_client()as client:
     request=versioned_request(ApiGetKernelSessionStatusRequest,owner,kernel,issue['version']);status=json.loads(str(client.kernels.kernels_api_client.get_kernel_session_status(request)))
    phase=status['status'].upper();errors=0;event({'status':phase,'requested_version_label':issue['version_label']})
   except Exception as error:
    errors+=1;event({'poll_error':type(error).__name__,'consecutive':errors})
    if errors>=3:raise
    time.sleep(30);continue
   if phase in ['RUNNING','QUEUED','PENDING']:time.sleep(30);continue
   assert phase in ['COMPLETE','COMPLETED','ERROR'];save(collection/'TERMINAL.json',{'utc':utc(),'version':1,'status':status});break
  else:raise AssertionError('Observation deadline; no reissue')
  gate();spool=collection/'spool';spool.mkdir();destination=storage_root/('tc008-focus-'+a.plan_sha256[:16]);assert not destination.exists()and not destination.is_symlink();destination.mkdir(mode=0o700)
  assert destination.resolve(strict=True).parent==storage_root;output=collection/'output';output.symlink_to(destination,target_is_directory=True)
  save(collection/'COLLECTION-ATTEMPT.json',{'utc':utc(),'attempt':1,'verified_ext4_before_storage':True,'serial_downloads':1,'chunk_bytes':262144})
  rows=[];token=None;tokens=set();provider_log=None
  with api.build_kaggle_client()as client:
   for page in range(100):
    gate();q=versioned_request(ApiListKernelSessionOutputRequest,owner,kernel,issue['version'],page_size=100,page_token=token)
    result=client.kernels.kernels_api_client.list_kernel_session_output(q);rows.extend(result.files or []);assert len(rows)<=10000;provider_log=provider_log or result.log;token=result.next_page_token
    if not token:break
    assert token not in tokens;tokens.add(token)
   else:raise AssertionError('Metadata pagination bound')
  indexed={}
  for row in rows:safe_name(row.file_name);assert row.file_name not in indexed;indexed[row.file_name]=row
  save(collection/'OUTPUT-METADATA.json',{'files':list(indexed),'pages':page+1,'signed_URLs_not_logged':True,'issue_version':issue['version'],'requested_version_label':issue['version_label']})
  session=requests.Session();session.mount('https://',requests.adapters.HTTPAdapter(max_retries=0));totals=[0];completed={}
  def fetch(name,want=None,maximum=MAX_FILE):
   gate(maximum);key=hashlib.sha256(name.encode('utf-8')).hexdigest();partial=spool/(key+'.download-partial');verified=spool/(key+'.verified');rec=download(session,indexed[name].url,partial,verified,want,gate,deadline,totals,maximum);return verified,rec
  prefix='tc008-actual-evidence/'
  manifest_name=prefix+'manifest.json';mf,mr=fetch(manifest_name,maximum=16*1024**2);manifest=json.loads(mf.read_text(encoding='utf-8'),object_pairs_hook=unique_pairs);assert type(manifest)is dict
  expected={};sizes={}
  for name,row in manifest.items():
   safe_name(name);assert type(row)is dict and set(row)=={'sha256','bytes'}
   sha=row['sha256'];size=row['bytes'];assert type(sha)is str and len(sha)==64 and all(c in '0123456789abcdef'for c in sha);assert type(size)is int and 0<=size<=MAX_FILE
   expected[prefix+name]=sha;sizes[prefix+name]=size
  assert manifest_name not in expected and sum(sizes.values())+mr['bytes']<=20*1024**3
  expected[manifest_name]=mr['sha256'];sizes[manifest_name]=mr['bytes'];assert set(expected)==set(indexed),'Actual provider outputs do not equal producer manifest'
  staged={manifest_name:(mf,mr)}
  for name in [prefix+'STARTED.json',prefix+'frozen-plan.json']:
   staged[name]=fetch(name,expected[name],16*1024**2);assert staged[name][1]['bytes']==sizes[name]
  started=json.loads(staged[prefix+'STARTED.json'][0].read_text());assert started['worker_sha256']==issue['worker_sha256']and started['runtime_source']==plan['source']and started['runtime_tree']==plan['tree']
  assert staged[prefix+'frozen-plan.json'][0].read_bytes()==plan_path.read_bytes(),'Actual archived plan bytes differ from issued plan'
  for name in list(staged)+[key for key in expected if key not in staged]:
   gate();fast,rec=staged[name]if name in staged else fetch(name,expected[name]);assert rec['sha256']==expected[name]and rec['bytes']==sizes[name];target=destination/name;record=copy_verified(fast,target,expected[name],deadline);completed[name]={'bytes':record['bytes'],'sha256':record['sha256']};event({'collected':name,'sha256':record['sha256'],'verified_fast_spool_retained':True})
  session.close();assert set(completed)==set(expected)
  if provider_log:
   raw=provider_log.encode('utf-8');assert len(raw)<=16*1024**2
   with(collection/'provider.log').open('xb')as f:f.write(raw)
  # This receipt authenticates bytes only, including genuine failed-group reports. No pass is assigned.
  assert digest(issue_path)==a.issue_sha256 and digest(plan_path)==a.plan_sha256 and digest(workers[0])==issue['worker_sha256']
  save(collection/'COLLECTED.json',{'utc':utc(),'version':1,'version_label':'v1','runtime_source':SOURCE,'runtime_tree':plan['tree'],'worker_sha256':issue['worker_sha256'],'actual_plan_sha256':a.plan_sha256,'issue_sha256':a.issue_sha256,'terminal_status':status,'files':completed,'producer_manifest_sha256':mr['sha256'],'serial_ext4_before_storage_verified':True,'parent_acceptance_assigned':False,'scientific_selections':0,'energy_acceptance_assigned':False,'total_downloaded_bytes':totals[0]})
 except BaseException as error:save(collection/'COLLECTION-STOPPED.json',{'utc':utc(),'error_type':type(error).__name__,'error':str(error),'traceback':traceback.format_exc(),'all_original_partials_and_verified_spools_retained':True,'no_retry_or_redispatch':True});raise
if __name__=='__main__':main()
