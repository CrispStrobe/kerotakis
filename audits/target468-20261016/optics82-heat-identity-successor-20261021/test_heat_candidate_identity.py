"""Source admission tests only: no browser, simulator or product requests."""
import sys,base64,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parent
from heat_candidate_identity import check_heat_candidate,CANDIDATE
class HeatCandidateIdentity(unittest.TestCase):
 def files(self):return {name:{'base64':base64.b64encode((ROOT/'modules'/name).read_bytes()).decode()}for name in ['heat-controls.mjs','run-heat.mjs']}
 def changed(self,name,change):
  files=self.files();raw=base64.b64decode(files[name]['base64']);files[name]['base64']=base64.b64encode(change(raw)).decode();return files
 def test_actual_current_modules_match_external_candidate(self):check_heat_candidate(self.files(),CANDIDATE)
 def test_stale_launcher_rejected(self):
  with self.assertRaises(ValueError):check_heat_candidate(self.changed('run-heat.mjs',lambda b:b.replace(CANDIDATE.encode(),b'18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a')),CANDIDATE)
 def test_stale_scientific_module_metadata_rejected(self):
  with self.assertRaises(ValueError):check_heat_candidate(self.changed('heat-controls.mjs',lambda b:b.replace(CANDIDATE.encode(),b'18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a')),CANDIDATE)
 def test_missing_guard_rejected(self):
  with self.assertRaises(ValueError):check_heat_candidate(self.changed('run-heat.mjs',lambda b:b.replace(b'assert.equal(binding.reviewed_candidate_source,',b'assert.notEqual(binding.reviewed_candidate_source,')),CANDIDATE)
 def test_duplicate_guard_rejected(self):
  with self.assertRaises(ValueError):check_heat_candidate(self.changed('heat-controls.mjs',lambda b:b+b"\nconst REVIEWED_CANDIDATE_SOURCE='"+CANDIDATE.encode()+b"';\n"),CANDIDATE)
 def test_wrong_external_candidate_rejected(self):
  with self.assertRaises(ValueError):check_heat_candidate(self.files(),'0'*40)
 def test_missing_external_authority_rejected(self):
  for candidate in [None,'',123,'bad',CANDIDATE+'x']:
   with self.subTest(candidate=candidate),self.assertRaises(ValueError):check_heat_candidate(self.files(),candidate)
if __name__=='__main__':unittest.main(verbosity=2)
