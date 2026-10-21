"""Synthetic orchestration metadata only; no actual release receipt or product result."""
import hashlib,json,os,unittest
from pathlib import Path
from worker_slots import render_worker,check_bound_source,normalized,canonical
TEMPLATE=b'CONFIG=None\nEXPECTED_RELEASE=None\nCONFIG_SHA256=None\ndef unchanged():\n return 7\n'
CONFIG={'runtime_source':'synthetic-source','runtime_tree':'synthetic-tree','test_only':True}
EXPECTED={'source':'synthetic-source','tree':'synthetic-tree','test_only':True}
SHA=hashlib.sha256(canonical(CONFIG)).hexdigest()
class ThreeSlots(unittest.TestCase):
 def render(self,template=TEMPLATE,config=CONFIG,expected=EXPECTED,sha=SHA):return render_worker(template,config,sha,expected)
 def test_three_slot_roundtrip_and_complete_frozen_template(self):
  result=self.render();check_bound_source(TEMPLATE,result,CONFIG,SHA,EXPECTED)
 def test_actual_external_template_only_if_explicitly_hash_pinned(self):
  name=os.environ.get('KERO_OPTICS82_WORKER_TEMPLATE');want=os.environ.get('KERO_OPTICS82_WORKER_TEMPLATE_SHA256')
  if not name and not want:self.skipTest('Private actual worker check requires explicit template path and SHA256')
  self.assertTrue(name and want,'Both explicit actual template path and SHA256 are required')
  actual=Path(name).read_bytes();self.assertEqual(hashlib.sha256(actual).hexdigest(),want);render_worker(actual,CONFIG,SHA,EXPECTED)
 def test_missing_third_slot_rejects(self):
  with self.assertRaises(ValueError):self.render(TEMPLATE.replace(b'EXPECTED_RELEASE=None\n',b''))
 def test_duplicate_slot_rejects(self):
  with self.assertRaises(ValueError):self.render(TEMPLATE+b'EXPECTED_RELEASE=None\n')
 def test_unknown_null_slot_rejects(self):
  with self.assertRaises(ValueError):self.render(TEMPLATE+b'UNREVIEWED_RELEASE=None\n')
 def test_nested_shadowed_slot_rejects(self):
  with self.assertRaises(ValueError):self.render(TEMPLATE+b'def hidden():\n CONFIG={}\n')
 def test_unbound_release_rejects(self):
  with self.assertRaises(ValueError):self.render(expected=None)
 def test_unbound_config_rejects(self):
  with self.assertRaises(ValueError):self.render(config=None)
 def test_wrong_config_digest_rejects(self):
  with self.assertRaises(ValueError):self.render(sha='0'*64)
 def test_non_slot_behavior_change_rejects(self):
  actual=self.render().replace(b'return 7',b'return 8')
  with self.assertRaises(ValueError):check_bound_source(TEMPLATE,actual,CONFIG,SHA,EXPECTED)
 def test_changed_release_or_missing_bound_third_slot_rejects(self):
  result=self.render()
  for value in [result.replace(b"'synthetic-tree'",b"'other-tree'",1),result.replace(b"EXPECTED_RELEASE="+repr(EXPECTED).encode(),b'EXPECTED_RELEASE=None')]:
   with self.subTest(value=value),self.assertRaises(ValueError):check_bound_source(TEMPLATE,value,CONFIG,SHA,EXPECTED)
 def test_bound_unknown_slot_rejects(self):
  with self.assertRaises(ValueError):check_bound_source(TEMPLATE,self.render()+b'UNREVIEWED_RELEASE=None\n',CONFIG,SHA,EXPECTED)
 def test_final_serialized_byte_limit_rejects(self):
  huge=dict(CONFIG,padding='x'*(1024**2));digest=hashlib.sha256(canonical(huge)).hexdigest()
  with self.assertRaises(ValueError):self.render(config=huge,sha=digest)
if __name__=='__main__':unittest.main(verbosity=2)
