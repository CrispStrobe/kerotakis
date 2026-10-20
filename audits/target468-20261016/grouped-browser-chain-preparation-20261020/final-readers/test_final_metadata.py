"""Synthetic parent-chain schema probes ONLY. No genuine acceptance/chemistry claim."""
import copy,unittest
from final_common import check_receipt_meta,check_prior62
class Metadata(unittest.TestCase):
 def setUp(self):
  self.expected={'group':'browser37','source':'SYNTHETIC_SOURCE_NOT_A_RUN','engine':{'binary_sha256':'SYNTHETIC_ENGINE'},'design_sha':'D','top_sha':'T','worker_sha':'W','issue_sha':'I','report_sha':'R','manifest_sha':'M'}
  self.receipt={'source':self.expected['source'],'engine':self.expected['engine'],'group':'browser37','execution_design_sha256':'D','top_manifest_sha256':'T','combined_worker_sha256':'W','external_issue_sha256':'I','postcollection_saved_review':True,'original_assertion_AST_verified':True,'owner_credentials_supplied':False,'new_browser_engine_provider_calls':0,'scientific_catalog_additions':0,'unchanged_checks':37,'browser_report_sha256':'R','manifest_sha256':'M'}
 def test_schema_only_positive(self):check_receipt_meta(self.receipt,**self.expected)
 def test_fake_flags_without_matching_report_refused(self):
  x=copy.deepcopy(self.receipt);x['browser_report_sha256']='fake-pass-label'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_execution_design_refused(self):
  x=copy.deepcopy(self.receipt);x['execution_design_sha256']='other'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_top_manifest_refused(self):
  x=copy.deepcopy(self.receipt);x['top_manifest_sha256']='other'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_group_manifest_refused(self):
  x=copy.deepcopy(self.receipt);x['manifest_sha256']='other'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_source_refused(self):
  x=copy.deepcopy(self.receipt);x['source']='another-source'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_engine_refused(self):
  x=copy.deepcopy(self.receipt);x['engine']={}
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_external_issue_refused(self):
  x=copy.deepcopy(self.receipt);x['external_issue_sha256']='producer-self-asserted'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_different_worker_refused(self):
  x=copy.deepcopy(self.receipt);x['combined_worker_sha256']='other'
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_missing_raw_proof_refused(self):
  x=copy.deepcopy(self.receipt);x['original_assertion_AST_verified']=False
  with self.assertRaises(AssertionError):check_receipt_meta(x,**self.expected)
 def test_producer_receipt_not_parent_metadata(self):
  with self.assertRaises(KeyError):check_receipt_meta({'browser_exit_status':0,'all_checks_passed':True},**self.expected)
 def test_prior44_schema(self):
  x=copy.deepcopy(self.receipt);x.update(group='targeted7',unchanged_API_controls=11,unchanged_browser_controls=37,fresh_targeted_browser_controls=7,targeted_report_sha256='R',targeted_manifest_sha256='M',actual_five_asset_hashes_verified=True);expected=dict(self.expected,group='targeted7');check_receipt_meta(x,**expected);x['fresh_targeted_browser_controls']=6
  with self.assertRaises(AssertionError):check_receipt_meta(x,**expected)
 def test_prior62_identity_and_raw_hashes(self):
  x={'source':'S','engine':{},'API_controls':11,'unchanged_browser_controls':37,'prior_targeted_browser_controls':7,'supplemental_locale_browser_controls':18,'all62_browser_checks_accepted':True,'locale_report_sha256':'R','manifest_sha256':'M','prior44_receipt_sha256':'P','catalog_additions':0,'all_i18n_complete_claim':False,'scientific_yield_claim':False};expected={'source':'S','engine':{},'locale_report_sha':'R','locale_manifest_sha':'M','prior44_sha':'P'};check_prior62(x,**expected)
  for key in ['locale_report_sha256','manifest_sha256','prior44_receipt_sha256']:
   y=copy.deepcopy(x);y[key]='other'
   with self.assertRaises(AssertionError):check_prior62(y,**expected)
if __name__=='__main__':unittest.main()
