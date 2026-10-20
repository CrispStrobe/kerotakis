"""One source-bound producer observation/collection; never issues or retries it."""
from pathlib import Path
import argparse,datetime,hashlib,json,time,traceback
from host_common import validate_ready,digest,save,wait_resource,require_ext4,SOURCE
from fast_spool import safe_name,download,copy_verified,MAX_FILE

def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def unique_pairs(rows):
 out={}
 for key,value in rows:assert key not in out;out[key]=value
 return out

def versioned_request(request_type,owner,kernel,version,*,page_size=None,page_token=None):
 assert type(version)is int and version==1,'Only actual externally bound versionone is admitted'
 q=request_type();q.user_name=owner;q.kernel_slug=kernel;q.version_label=str(version)
 if page_size is not None:q.page_size=page_size
 if page_token:q.page_token=page_token
 return q

def main():
 p=argparse.ArgumentParser();p.add_argument('--execute',action='store_true');p.add_argument('--bound',type=Path);p.add_argument('--ready',type=Path);p.add_argument('--ready-sha256');p.add_argument('--state',type=Path);p.add_argument('--issue',type=Path);p.add_argument('--issue-sha256');p.add_argument('--fast-root',type=Path);p.add_argument('--storage-root',type=Path);a=p.parse_args()
 if not a.execute:print('PREPARED ONLY: no provider observation or download.');return
 assert all([a.bound,a.ready,a.ready_sha256,a.state,a.issue,a.issue_sha256,a.fast_root,a.storage_root]);ready,config=validate_ready(a.bound,a.ready,a.ready_sha256)
 assert digest(a.issue)==a.issue_sha256;issue=json.loads(a.issue.read_text(encoding='utf-8'));assert issue['worker_sha256']==ready['worker_sha256']and issue['config_sha256']==ready['config_sha256']and issue['runtime_source']==SOURCE and issue['one_issue_only']is True and issue['parent_reviewed_exact_producer_and_readers']is True and issue['version']==1 and issue['is_private']is True
 assert issue['kernel_id'].removeprefix('/code/')==ready['owned_kernel_id']
 fast_root=a.fast_root.resolve(strict=True);storage_root=a.storage_root.resolve(strict=True);require_ext4(fast_root);deadline=time.monotonic()+14400
 wait_resource(fast_root,storage_root,MAX_FILE,deadline);state=a.state.resolve(strict=True);assert digest(state/'ISSUE.json')==a.issue_sha256 and (state/'PUSH-ATTEMPT.json').is_file()and digest(state/'upload/grouped_browser.py')==ready['worker_sha256'];collection=state/'collection';assert state.is_relative_to(fast_root)and not collection.exists();collection.mkdir(mode=0o700)
 save(collection/'COLLECTOR-STARTED.json',{'utc':utc(),'program_sha256':digest(Path(__file__)),'issue_sha256':a.issue_sha256,'worker_sha256':ready['worker_sha256'],'config_sha256':ready['config_sha256'],'no_provider_mutation_or_retry':True})
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
    phase=status['status'].upper();errors=0;event({'status':phase,'requested_version_label':str(issue['version'])})
   except Exception as error:
    errors+=1;event({'poll_error':type(error).__name__,'consecutive':errors})
    if errors>=3:raise
    time.sleep(30);continue
   if phase in ['RUNNING','QUEUED','PENDING']:time.sleep(30);continue
   assert phase in ['COMPLETE','COMPLETED','ERROR','CANCELLED','CANCEL_ACKNOWLEDGED'];save(collection/'TERMINAL.json',{'utc':utc(),'version':1,'status':status});break
  else:raise AssertionError('Observation deadline; no reissue')
  gate();spool=collection/'spool';spool.mkdir();destination=storage_root/('grouped-browser-'+ready['config_sha256'][:16]);assert not destination.exists()and not destination.is_symlink();destination.mkdir(mode=0o700)
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
  save(collection/'OUTPUT-METADATA.json',{'files':list(indexed),'pages':page+1,'signed_URLs_not_logged':True,'issue_version':issue['version'],'requested_version_label':str(issue['version'])})
  session=requests.Session();session.mount('https://',requests.adapters.HTTPAdapter(max_retries=0));totals=[0];completed={}
  def fetch(name,want=None,maximum=MAX_FILE):
   gate(maximum);key=hashlib.sha256(name.encode('utf-8')).hexdigest();partial=spool/(key+'.download-partial');verified=spool/(key+'.verified');rec=download(session,indexed[name].url,partial,verified,want,gate,deadline,totals,maximum);return verified,rec
  manifest_name='browser-evidence/SHA256.json';mf,mr=fetch(manifest_name,maximum=16*1024**2);manifest=json.loads(mf.read_text(encoding='utf-8'),object_pairs_hook=unique_pairs);assert type(manifest)is dict
  expected={}
  for name,sha in manifest.items():safe_name(name);assert type(sha)is str and len(sha)==64 and all(c in '0123456789abcdef'for c in sha);expected['browser-evidence/'+name]=sha
  expected[manifest_name]=mr['sha256'];assert set(expected)==set(indexed),'Actual provider outputs do not equal producer manifest'
  staged={manifest_name:(mf,mr)}
  for name in ['browser-evidence/STARTED.json','browser-evidence/execution-admission.json','browser-evidence/design-binding.json']:
   staged[name]=fetch(name,expected[name],16*1024**2)
  started=json.loads(staged['browser-evidence/STARTED.json'][0].read_text(encoding='utf-8'));assert started['script_sha256']==ready['worker_sha256']and started['runtime_source']==SOURCE and started['harness_source']==config['harness_head']
  assert staged['browser-evidence/execution-admission.json'][0].read_bytes()==(a.bound/'CONFIG.json').read_bytes()
  observed=json.loads(staged['browser-evidence/design-binding.json'][0].read_text(encoding='utf-8'));assert observed['worker_sha256']==ready['worker_sha256']and observed['config_sha256']==ready['config_sha256']and observed['parent_acceptance_assigned']is False
  present_inputs=[]
  for name,row in config['inputs'].items():
   key='browser-evidence/frozen-inputs/'+name
   if key in expected:assert expected[key]==row['sha256'],'Archived frozen input differs';present_inputs.append(name)
  # A genuine early producer resource/setup failure can precede input materialization.
  # Collect its complete failed raw set; never describe that as source/product acceptance.
  if 'browser-evidence/COMPLETED.json'in expected:assert len(present_inputs)==len(config['inputs']),'Completed producer omitted frozen source inputs'
  for name in list(staged)+[key for key in expected if key not in staged]:
   gate();fast,rec=staged[name]if name in staged else fetch(name,expected[name]);assert rec['sha256']==expected[name];target=destination/name;record=copy_verified(fast,target,expected[name],deadline);completed[name]={'bytes':record['bytes'],'sha256':record['sha256']};event({'collected':name,'sha256':record['sha256'],'verified_fast_spool_retained':True})
  session.close();assert set(completed)==set(expected)
  if provider_log:
   raw=provider_log.encode('utf-8');assert len(raw)<=16*1024**2
   with(collection/'provider.log').open('xb')as f:f.write(raw)
  # This receipt authenticates bytes only, including genuine failed-group reports. No pass is assigned.
  save(collection/'COLLECTED.json',{'utc':utc(),'version':1,'runtime_source':SOURCE,'worker_sha256':ready['worker_sha256'],'config_sha256':ready['config_sha256'],'issue_sha256':a.issue_sha256,'terminal_status':status,'files':completed,'producer_manifest_sha256':mr['sha256'],'frozen_input_files_present':present_inputs,'all_frozen_inputs_materialized':len(present_inputs)==len(config['inputs']),'serial_ext4_before_storage_verified':True,'parent_acceptance_assigned':False,'scientific_selections':0,'total_downloaded_bytes':totals[0]})
 except BaseException as error:save(collection/'COLLECTION-STOPPED.json',{'utc':utc(),'error_type':type(error).__name__,'error':str(error),'traceback':traceback.format_exc(),'all_original_partials_and_verified_spools_retained':True,'no_retry_or_redispatch':True});raise
if __name__=='__main__':main()
