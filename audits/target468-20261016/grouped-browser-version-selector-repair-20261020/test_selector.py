"""Actual local SDK object serialization and synthetic paths; NO provider calls."""
import json
from pathlib import Path
import tempfile
import unittest
from collect_once import versioned_request, collection_path
from kagglesdk.kernels.types.kernels_api_service import ApiListKernelSessionOutputRequest, ApiGetKernelSessionStatusRequest

class SelectorAndName(unittest.TestCase):
 def test_actual_status_type_serializes_v1(self):
  q=versioned_request(ApiGetKernelSessionStatusRequest,'synthetic','synthetic-worker',1)
  self.assertEqual(q.version_label,'v1');self.assertEqual(q.to_dict()['versionLabel'],'v1');self.assertEqual(json.loads(q.to_json())['versionLabel'],'v1')
 def test_actual_output_type_serializes_v1_on_all_pages(self):
  for token in [None,'synthetic-page-two']:
   q=versioned_request(ApiListKernelSessionOutputRequest,'synthetic','synthetic-worker',1,page_size=100,page_token=token)
   self.assertEqual(q.to_dict()['versionLabel'],'v1');self.assertEqual(json.loads(q.to_json())['versionLabel'],'v1');self.assertEqual(q.page_size,100)
   if token:self.assertEqual(q.page_token,token)
 def test_only_actual_numeric_version_one(self):
  for value in [None,True,0,2,'1','v1']:
   with self.subTest(value=value):
    with self.assertRaisesRegex(AssertionError,'externally bound versionone'):versioned_request(ApiGetKernelSessionStatusRequest,'synthetic','worker',value)
 def test_fresh_reviewed_collection_path_no_creation(self):
  with tempfile.TemporaryDirectory()as raw:
   root=Path(raw);name='collection-selector-v1';self.assertEqual(collection_path(root,{'collection_name':name},name),root/name);self.assertFalse((root/name).exists())
 def test_strict_single_component_name(self):
  with tempfile.TemporaryDirectory()as raw:
   for name in [None,'collection','../collection-v1','collection-A','collection-x/y','/collection-x','collection-x\\y','collection-x\n','collection-']:
    with self.subTest(name=name):
     with self.assertRaisesRegex(AssertionError,'strict collection-name'):collection_path(Path(raw),{'collection_name':name},name)
 def test_parent_ready_name_mismatch(self):
  with tempfile.TemporaryDirectory()as raw:
   with self.assertRaisesRegex(AssertionError,'externally reviewed'):collection_path(Path(raw),{'collection_name':'collection-different'},'collection-selector-v1')
 def test_existing_and_dangling_collection_refused(self):
  with tempfile.TemporaryDirectory()as raw:
   root=Path(raw);name='collection-selector-v1';(root/name).mkdir()
   with self.assertRaisesRegex(AssertionError,'fresh'):collection_path(root,{'collection_name':name},name)
   (root/name).rmdir();(root/name).symlink_to(root/'does-not-exist')
   with self.assertRaisesRegex(AssertionError,'fresh'):collection_path(root,{'collection_name':name},name)

if __name__=='__main__':unittest.main()
