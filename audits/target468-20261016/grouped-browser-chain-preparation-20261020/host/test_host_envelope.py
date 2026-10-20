"""SYNTHETIC local metadata/files/byte-stream checks; no issuer, collector or provider calls."""
from pathlib import Path
import hashlib,json,tempfile,time,unittest
from issue_once import metadata
from host_common import save,safe_relative
import fast_spool as spool
from collect_once import versioned_request
from kagglesdk.kernels.types.kernels_api_service import ApiListKernelSessionOutputRequest,ApiGetKernelSessionStatusRequest
class Response:
 def __init__(self,data,length=None):self.data=data;self.status_code=200;self.headers={'Content-Length':str(len(data)if length is None else length)}
 def __enter__(self):return self
 def __exit__(self,*a):pass
 @property
 def content(self):raise AssertionError('whole-response buffering prohibited')
 def iter_content(self,chunk_size):
  assert chunk_size==262144
  for i in range(0,len(self.data),chunk_size):yield self.data[i:i+chunk_size]
class Session:
 def __init__(self,response):self.response=response;self.calls=0
 def get(self,*args,**kw):assert kw['stream']is True and kw['allow_redirects']is False;self.calls+=1;return self.response
class HostEnvelope(unittest.TestCase):
 def setUp(self):self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name);self.data=b'synthetic-stream-only'*30000;self.sha=hashlib.sha256(self.data).hexdigest()
 def download(self,want=None,length=None):
  session=Session(Response(self.data,length));rec=spool.download(session,'https://example.invalid/owned',self.root/'partial',self.root/'verified',self.sha if want is None else want,lambda amount:None,time.monotonic()+30,[0]);return rec,session
 def test_private_CPU_metadata_only(self):
  m=metadata({'kernel_title':'Synthetic grouped source 20261020','owned_kernel_id':'synthetic/synthetic-grouped-source-20261020','owned_account_confirmed':True});self.assertEqual(m['is_private'],'true');self.assertEqual(m['enable_gpu'],'false');self.assertEqual(m['dataset_sources'],[])
 def test_mismatched_owned_ID_metadata_rejected(self):
  with self.assertRaises(AssertionError):metadata({'kernel_title':'Synthetic source','owned_kernel_id':'synthetic/unrelated','owned_account_confirmed':True})
 def test_exclusive_marker_cannot_be_reused(self):
  save(self.root/'attempt.json',{'synthetic_fixture':True})
  with self.assertRaises(FileExistsError):save(self.root/'attempt.json',{})
 def test_stream_spool_then_verified_storage(self):
  rec,s=self.download();self.assertEqual(s.calls,1);self.assertEqual(rec['sha256'],self.sha);target=self.root/'storage/owned/file';spool.copy_verified(self.root/'verified',target,self.sha,time.monotonic()+30);self.assertEqual(target.read_bytes(),self.data);self.assertTrue((self.root/'verified').exists())
 def test_bad_hash_preserves_first_partial(self):
  with self.assertRaises(AssertionError):self.download('0'*64)
  self.assertTrue((self.root/'partial').exists());self.assertFalse((self.root/'verified').exists())
 def test_length_failure_preserves_first_partial(self):
  with self.assertRaises(AssertionError):self.download(length=len(self.data)+1)
  self.assertTrue((self.root/'partial').exists())
 def test_storage_failure_keeps_verified_fast_bytes(self):
  self.download();(self.root/'blocked').write_bytes(b'owned')
  with self.assertRaises(OSError):spool.copy_verified(self.root/'verified',self.root/'blocked/file',self.sha,time.monotonic()+30)
  self.assertEqual(spool.digest(self.root/'verified'),self.sha)
 def test_owned_descendant_symlink_refused(self):
  self.download();(self.root/'outside').mkdir();(self.root/'alias').symlink_to(self.root/'outside',target_is_directory=True)
  with self.assertRaises(AssertionError):spool.copy_verified(self.root/'verified',self.root/'alias/file',self.sha,time.monotonic()+30)
 def test_trusted_root_resolves_once(self):
  self.download();(self.root/'storage').mkdir();(self.root/'trusted').symlink_to(self.root/'storage',target_is_directory=True);target=(self.root/'trusted').resolve(strict=True)/'owned/file';spool.copy_verified(self.root/'verified',target,self.sha,time.monotonic()+30);self.assertEqual(target.read_bytes(),self.data)
 def test_actual_output_request_serializes_exact_version_on_each_page(self):
  for token in [None,'synthetic-page2']:
   q=versioned_request(ApiListKernelSessionOutputRequest,'synthetic','synthetic-versioned',1,page_size=100,page_token=token);raw=q.to_dict();self.assertEqual(raw['versionLabel'],'1');self.assertEqual(json.loads(q.to_json())['versionLabel'],'1');self.assertEqual(raw['pageSize'],100)
   if token:self.assertEqual(raw['pageToken'],token)
 def test_actual_status_request_serializes_exact_version(self):
  q=versioned_request(ApiGetKernelSessionStatusRequest,'synthetic','synthetic-versioned',1);self.assertEqual(q.to_dict()['versionLabel'],'1');self.assertEqual(json.loads(q.to_json())['versionLabel'],'1')
 def test_nonexact_version_request_rejected(self):
  for version in [None,True,0,2,'1']:
   with self.subTest(version=version):
    with self.assertRaises(AssertionError):versioned_request(ApiGetKernelSessionStatusRequest,'synthetic','synthetic-versioned',version)
 def test_unsafe_paths_rejected(self):
  for name in ['/absolute','../escape','a/../b','a//b','a\\b','a\x00b','./file']:
   with self.subTest(name=name):
    with self.assertRaises(AssertionError):safe_relative(name)
if __name__=='__main__':unittest.main()
