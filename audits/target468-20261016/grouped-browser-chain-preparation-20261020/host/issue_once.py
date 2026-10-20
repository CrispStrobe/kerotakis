"""Explicitly reviewed once-only private CPU issue. Default is no action."""
from pathlib import Path
import argparse,datetime,json,re,shutil
from host_common import validate_ready,digest,save,resource_open,require_ext4,SOURCE

def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def metadata(ready):
 title=ready['kernel_title'];ref=ready['owned_kernel_id'];assert type(title)is str and len(title)<=50 and re.fullmatch('[a-zA-Z0-9][a-zA-Z0-9 -]+',title)
 assert re.fullmatch('[a-zA-Z0-9_-]+/[a-z0-9-]+',ref)and ref.split('/')[1]==title.lower().replace(' ','-')and ready['owned_account_confirmed']is True
 return {'id':ref,'title':title,'code_file':'grouped_browser.py','language':'python','kernel_type':'script','is_private':'true','enable_gpu':'false','enable_internet':'true','competition_sources':[],'dataset_sources':[],'kernel_sources':[],'model_sources':[]}
def main():
 p=argparse.ArgumentParser();p.add_argument('--execute',action='store_true');p.add_argument('--bound',type=Path);p.add_argument('--ready',type=Path);p.add_argument('--ready-sha256');p.add_argument('--state',type=Path);p.add_argument('--fast-root',type=Path);p.add_argument('--storage-root',type=Path);a=p.parse_args()
 if not a.execute:print('PREPARED ONLY: no provider call or issue.');return
 assert all([a.bound,a.ready,a.ready_sha256,a.state,a.fast_root,a.storage_root]);ready,config=validate_ready(a.bound,a.ready,a.ready_sha256);meta=metadata(ready)
 fast=a.fast_root.resolve(strict=True);storage=a.storage_root.resolve(strict=True);require_ext4(fast);assert resource_open(fast,storage)
 state=a.state.absolute();assert state.parent.resolve(strict=True).is_relative_to(fast)and not state.exists()and not state.is_symlink();state.mkdir(mode=0o700)
 save(state/'PREISSUE-STARTED.json',{'utc':utc(),'runtime_source':SOURCE,'ready_sha256':a.ready_sha256,'bound_worker_sha256':ready['worker_sha256'],'config_sha256':ready['config_sha256'],'provider_mutations':0})
 try:
  upload=state/'upload';upload.mkdir();shutil.copyfile(a.bound/'grouped_browser.py',upload/'grouped_browser.py');shutil.copyfile(a.bound/'CONFIG.json',state/'CONFIG.json');save(upload/'kernel-metadata.json',meta)
  assert digest(upload/'grouped_browser.py')==ready['worker_sha256']and meta['is_private']=='true'and meta['enable_gpu']=='false'
  from kaggle.api.kaggle_api_extended import KaggleApi
  api=KaggleApi();api.authenticate();rows=api.kernels_list(mine=True,search=meta['id'].split('/')[1],page_size=100)
  assert rows is not None and len(rows)<100,'Remote owned-ID result incomplete: no issue'
  assert not any(row is not None and row.ref==meta['id']for row in rows),'Exact owned ID already exists: never redispatch'
  save(state/'OWNED-ID-UNUSED.json',{'utc':utc(),'exact_owned_id':meta['id'],'matching_count':0,'complete_fuzzy_count':len(rows)})
  validate_ready(a.bound,a.ready,a.ready_sha256);assert resource_open(fast,storage);assert digest(upload/'grouped_browser.py')==ready['worker_sha256']
  save(state/'PUSH-ATTEMPT.json',{'utc':utc(),'owned_kernel_id':meta['id'],'attempt':1,'worker_sha256':ready['worker_sha256'],'config_sha256':ready['config_sha256'],'is_private':True,'CPU_only':True,'no_automatic_retry':True})
  raw=str(api.kernels_push(str(upload),timeout='3600'))
  with(state/'push-response.private.txt').open('x',encoding='utf-8')as f:f.write(raw+'\n')
  response=json.loads(raw);assert response['error']is None and response['versionNumber']==1 and response['ref'].removeprefix('/code/')==meta['id']
  save(state/'ISSUE.json',{'utc':utc(),'worker_sha256':ready['worker_sha256'],'config_sha256':ready['config_sha256'],'runtime_source':SOURCE,'one_issue_only':True,'parent_reviewed_exact_producer_and_readers':True,'kernel_id':response['ref'],'version':1,'provider_kernel_id':response['kernelId'],'is_private':True})
 except BaseException as error:save(state/'ISSUE-STOPPED.json',{'utc':utc(),'error_type':type(error).__name__,'error':str(error),'no_automatic_retry_or_redispatch':True,'inspect_exclusive_attempt_and_provider_response_before_continuation':True});raise
if __name__=='__main__':main()
