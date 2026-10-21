"""Saved-log and synthetic identity controls only; never runs a Rust test or guard."""
import json, os, hashlib
from pathlib import Path
import unittest
from exact_native_identity import native_test_rows,require_exact_native_test,QUALIFIED_NAMES
B=Path(__file__).resolve().parent
LOGS=Path(os.environ['KEROTAKIS_PLATFORM_LOG_DIR']) if 'KEROTAKIS_PLATFORM_LOG_DIR' in os.environ else None
DECLARED='missing_forward_interaction_is_a_named_refusal'
CANONICAL=QUALIFIED_NAMES[DECLARED]
def raw(name,outcome='ok'):
 return ('2026-10-21T16:15:02.2852126Z test '+name+' ... '+outcome+'\n').encode()
class ExactNativeIdentity(unittest.TestCase):
 @unittest.skipIf(LOGS is None, 'Supply authenticated actual platform logs through KEROTAKIS_PLATFORM_LOG_DIR')
 def test_both_actual_platform_logs_all108_exact_identities(self):
  required=json.loads((B/'DECLARED-108-NATIVE-CONTROLS.json').read_bytes())
  logs=sorted(LOGS.glob('*.log'));self.assertEqual(len(logs),2)
  authority=json.loads((B/'ACTUAL-PLATFORM-LOG-IDENTITY-REVIEW.json').read_bytes())['logs']
  expected={name.replace('.parent-partial',''):row for name,row in authority.items()}
  self.assertEqual({f.name for f in logs},set(expected))
  for file in logs:
   raw_bytes=file.read_bytes();self.assertEqual(len(raw_bytes),expected[file.name]['bytes']);self.assertEqual(hashlib.sha256(raw_bytes).hexdigest(),expected[file.name]['sha256'])
   rows=native_test_rows(raw_bytes)
   for name in required:
    with self.subTest(platform=file.name,name=name):self.assertEqual(require_exact_native_test(rows,name)['actual_outcome'],'ok')
 def test_exact_canonical_alias_and_unchanged_bare_identity_pass(self):
  self.assertEqual(require_exact_native_test(native_test_rows(raw(CANONICAL)),DECLARED)['canonical_test'],CANONICAL)
  self.assertEqual(require_exact_native_test(native_test_rows(raw('unchanged_bare_control')),'unchanged_bare_control')['canonical_test'],'unchanged_bare_control')
 def test_missing_and_case_mismatch_reject(self):
  for data in [b'',raw(CANONICAL.upper())]:
   with self.subTest(data=data),self.assertRaises(AssertionError):require_exact_native_test(native_test_rows(data),DECLARED)
 def test_wrong_module_rejects(self):
  with self.assertRaises(AssertionError):require_exact_native_test(native_test_rows(raw('different::tests::'+DECLARED)),DECLARED)
 def test_bare_alias_cannot_replace_qualified_identity(self):
  for data in [raw(DECLARED),raw(DECLARED)+raw(CANONICAL)]:
   with self.subTest(data=data),self.assertRaises(AssertionError):require_exact_native_test(native_test_rows(data),DECLARED)
 def test_duplicate_exact_row_rejects(self):
  with self.assertRaises(AssertionError):require_exact_native_test(native_test_rows(raw(CANONICAL)*2),DECLARED)
 def test_failed_or_conflicting_result_rejects(self):
  for data in [raw(CANONICAL,'FAILED'),raw(CANONICAL)+raw(CANONICAL,'FAILED')]:
   with self.subTest(data=data),self.assertRaises(AssertionError):require_exact_native_test(native_test_rows(data),DECLARED)
 def test_ignored_is_not_pass(self):
  with self.assertRaises(AssertionError):require_exact_native_test(native_test_rows(raw(CANONICAL,'ignored')),DECLARED)
if __name__=='__main__':unittest.main(verbosity=2)
