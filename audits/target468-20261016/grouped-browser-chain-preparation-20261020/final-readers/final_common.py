"""Saved grouped evidence/parent chain. No network/browser/provider/engine calls."""
import json,pathlib,hashlib
from saved_envelope import authenticate,load,digest,ExplicitAliases
from read_group import ast_proof
LOCALE={'review-locale18-848.py':'84fe50ca53fb67a27bdda1d40869df70a56948b31c01daa1ad79b8c1597cec6a','locale18_saved_predicates.py':'4b66938c88db0da8863d44a10a7e4a34e72aa578891133e72ac1818d7dd4e063','locale-controls.mjs':'7290327a0d59d981bcc8bc0e35b5d0941c3559d14067ecdf488ae4f24c0f8819','run-locale18.mjs':'8456b089e8389c2995fd1c58bcd316c64fffcfcad4db002927f336fa58a8bd21'}
HEAT={'review_saved.py':'817d9fdf9106d5581919a5b7696658074033e6737154c1336b5cbc0f3d516252','heat_saved_predicates.py':'705915e2fdb218c754c69c7408d135f1432bd521343336383f9aeadee0ae9264','heat-controls.mjs':'cf19825bd6a273115eb686feab0b3242dfebb5533e571fed4765b8f49c371d6f','run-heat.mjs':'316980a04ba85c94a54665f1ad5418a310f8485892afe97b85c9ba97daa6d592'}

def add_inputs(parser):
 for n in ['evidence','design','worker','issue','api','verified','native-reference','prior37','prior44']:parser.add_argument('--'+n,type=pathlib.Path,required=True)
 parser.add_argument('--prior37-sha256',required=True);parser.add_argument('--prior44-sha256',required=True)
def environment(a,group):
 return authenticate(evidence=a.evidence,design_path=a.design,worker_path=a.worker,issue_path=a.issue,api_path=a.api,verified_path=a.verified,native_reference_path=a.native_reference,group=group)
def frozen_bytes(env,want):
 paths=[p for p in env['input_paths'] if digest(p)==want];assert paths,'Required exact original/helper/reader absent from frozen inputs: '+want
 return paths[0].read_bytes()
def check_receipt_meta(value,*,group,source,engine,design_sha,top_sha,worker_sha,issue_sha,report_sha,manifest_sha):
 assert value['source']==source and value['engine']==engine and value['group']==group and value['execution_design_sha256']==design_sha and value['top_manifest_sha256']==top_sha and value['combined_worker_sha256']==worker_sha and value['external_issue_sha256']==issue_sha
 assert value['postcollection_saved_review'] is True and value['original_assertion_AST_verified'] is True and value['owner_credentials_supplied'] is False and value['new_browser_engine_provider_calls']==0 and value['scientific_catalog_additions']==0
 if group=='browser37':assert value['unchanged_checks']==37 and value['browser_report_sha256']==report_sha and value['manifest_sha256']==manifest_sha
 else:assert value['unchanged_API_controls']==11 and value['unchanged_browser_controls']==37 and value['fresh_targeted_browser_controls']==7 and value['targeted_report_sha256']==report_sha and value['targeted_manifest_sha256']==manifest_sha and value['actual_five_asset_hashes_verified'] is True

def parent_chain(a):
 # Explicit expected hashes are supplied from genuine parent review, never
 # producer flags. Parent files must be outside the immutable producer output.
 for path,want in [(a.prior37,a.prior37_sha256),(a.prior44,a.prior44_sha256)]:
  assert digest(path)==want and a.evidence.resolve() not in path.resolve().parents
 envs={group:environment(a,group) for group in ['browser37','targeted7']};p37=load(a.prior37);p44=load(a.prior44)
 for group,parent in [('browser37',p37),('targeted7',p44)]:
  env=envs[group];proof=ast_proof(env,group);check_receipt_meta(parent,group=group,source=env['S'],engine=env['binding']['engine'],design_sha=env['execution_design_sha256'],top_sha=env['top_manifest_sha256'],worker_sha=env['worker_sha256'],issue_sha=env['external_issue_sha256'],report_sha=digest(env['e']/'actual-browser/report.json'),manifest_sha=env['group_manifest_sha256']);assert parent['original_assertion_AST_sha256']==proof['assertion_block_AST_sha256'];assert parent['verified_artifact_receipt_sha256']==env['verified_sha256'] and parent['api_acceptance_sha256']==env['api_sha256'] and parent['native_reference_sha256']==env['native_reference_sha256']
 assert p44['unchanged_browser_acceptance_sha256']==digest(a.prior37)
 # Reapply exact original raw predicates too: matching PASS metadata alone is
 # insufficient. This is saved review only, never another browser execution.
 B=ExplicitAliases({'partition-factor-targeted-before-binding.json':a.native_reference,'browser-contract-848-production-delivery-20261020/API-ACCEPTED.json':a.api});O=ExplicitAliases({'BROWSER37-ACCEPTED.json':a.prior37})
 from group37_predicates import review as original37
 from group7_predicates import review as original7
 for group,review in [('browser37',original37),('targeted7',original7)]:
  env=envs[group];legacy=dict(env['group_design'],script_sha256=env['worker_sha256']);review(env['e'],legacy,env['binding'],env['S'],B,O)
 return envs

def save_new(path,value):
 with pathlib.Path(path).open('x',encoding='utf-8') as f:json.dump(value,f,indent=2,allow_nan=False);f.write('\n')
def bind_final_sources(env):
 # Current final helper code must itself have been frozen BEFORE execution.
 P=pathlib.Path(__file__).resolve().parent;hashes={digest(p) for p in env['input_paths']}
 for name in ['final_common.py','build_locale_context.py','review_heat_group.py']:
  assert digest(P/name) in hashes,('Final saved reader missing from preissue frozen source closure',name)

def check_prior62(value,*,source,engine,locale_report_sha,locale_manifest_sha,prior44_sha):
 assert value['source']==source and value['engine']==engine and value['API_controls']==11 and value['unchanged_browser_controls']==37 and value['prior_targeted_browser_controls']==7 and value['supplemental_locale_browser_controls']==18 and value['all62_browser_checks_accepted'] is True
 assert value['locale_report_sha256']==locale_report_sha and value['manifest_sha256']==locale_manifest_sha and value['prior44_receipt_sha256']==prior44_sha and value['catalog_additions']==0 and value['all_i18n_complete_claim'] is False and value['scientific_yield_claim'] is False
