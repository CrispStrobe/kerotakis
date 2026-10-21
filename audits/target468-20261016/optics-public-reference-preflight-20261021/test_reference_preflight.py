"""Actual public-reference validation and synthetic bad metadata only; no chemistry."""
import copy,json,subprocess,unittest
from pathlib import Path
from validate_optics_references import validate_optics_references
ROOT=Path(__file__).parent
class ReferenceAdmission(unittest.TestCase):
 def setUp(self):self.plan=json.loads((ROOT/'references.json').read_bytes())
 def rejects(self,plan,callback=None):
  with self.assertRaises((AssertionError,KeyError)):validate_optics_references(plan,callback)
 def test_eight_actual_public_git_blobs(self):
  rows=validate_optics_references(self.plan,lambda commit,path:subprocess.check_output(['git','show',commit+':'+path]));self.assertEqual(len(rows),8)
 def test_legacy_missing_coordinates_refused_early(self):
  e=self.plan['optics_frozen_inputs']['label-forecast.json'];del e['public_commit'];del e['public_path'];self.rejects(self.plan)
 def test_missing_reference(self):
  del self.plan['optics_frozen_inputs']['forecast.json'];self.rejects(self.plan)
 def test_unknown_reference(self):
  self.plan['optics_frozen_inputs']['unknown.json']=copy.deepcopy(self.plan['optics_frozen_inputs']['forecast.json']);self.rejects(self.plan)
 def test_content_hash_mismatch(self):
  self.plan['optics_frozen_inputs']['forecast.json']['sha256']='0'*64;self.rejects(self.plan)
 def test_label_authority_substitution(self):
  self.plan['optics_frozen_inputs']['label-forecast.json']['public_commit']='0'*40;self.rejects(self.plan)
 def test_lost_origin(self):
  del self.plan['optics_frozen_inputs']['label-forecast.json']['origin_kind'];self.rejects(self.plan)
 def test_traversal_path(self):
  self.plan['optics_frozen_inputs']['forecast.json']['public_path']='../forecast.json';self.rejects(self.plan)
 def test_wrong_public_bytes(self):self.rejects(self.plan,lambda commit,path:b'SYNTHETIC_WRONG_PUBLIC_METADATA_BYTES')
if __name__=='__main__':unittest.main(verbosity=2)
