"""Saved raw predicates only; explicit reviewed input SHA arguments, no browser/engine."""
from pathlib import Path
import argparse,hashlib,json
from heat_saved_predicates import validate_report,REVIEWED_CANDIDATE_SOURCE
p=argparse.ArgumentParser();p.add_argument('--report',type=Path,required=True);p.add_argument('--report-sha256',required=True);p.add_argument('--binding',type=Path,required=True);p.add_argument('--binding-sha256',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
def digest(path):
 with path.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert digest(a.report)==a.report_sha256 and digest(a.binding)==a.binding_sha256
r=json.loads(a.report.read_text(encoding='utf-8'));binding=json.loads(a.binding.read_text(encoding='utf-8'));details=validate_report(r,binding)
result={'state':'SAVED_HEAT_PREDICATES_ACCEPTED','reviewed_candidate_source':REVIEWED_CANDIDATE_SOURCE,'source':binding['expected_engine_source'],'report_sha256':a.report_sha256,'binding_sha256':a.binding_sha256,'details':details,'scientific_selections':0,'prior62_acceptance_replaced':False,'full_thermal_ledger_or_rate_calibration_claim':False,'producer_archive_asset_and_source_authentication_still_requires_parent_delivery_review':True}
with a.out.open('x',encoding='utf-8')as f:json.dump(result,f,indent=2);f.write('\n')
