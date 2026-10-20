#!/usr/bin/env python3
"""Saved-only37/7 browser review. No dispatch, browser, engine or network code."""
import argparse,ast,datetime,hashlib,json,pathlib
from saved_envelope import authenticate,ExplicitAliases,digest,load

def ast_proof(envelope,group):
 P=pathlib.Path(__file__).resolve().parent
 module='group37_predicates.py' if group=='browser37' else 'group7_predicates.py';index=0 if group=='browser37' else 1;proof=load(P/'AST-EQUALITY-PROOF.json')['blocks'][index];assert proof['new_module']==module and digest(P/module)==proof['new_module_sha256']
 tree=ast.parse((P/module).read_bytes());function=next(n for n in tree.body if isinstance(n,ast.FunctionDef));block=ast.Module(body=function.body[:-1],type_ignores=[]);assert hashlib.sha256(ast.dump(block,include_attributes=False).encode()).hexdigest()==proof['assertion_block_AST_sha256']
 # Full old-reader source is authenticated frozen input, parsed only, never imported.
 inputs=envelope['input_paths'];original=[p for p in inputs if digest(p)==proof['old_reader_sha256']];assert len(original)>=1
 old=next(n for n in ast.parse(original[0].read_bytes()).body if isinstance(n,ast.Try));lo,hi=proof['original_try_statement_slice'];assert ast.dump(ast.Module(body=old.body[lo:hi],type_ignores=[]),include_attributes=False)==ast.dump(block,include_attributes=False)
 for name in ['read_group.py','saved_envelope.py',module,'AST-EQUALITY-PROOF.json']:
  assert digest(P/name) in {digest(p) for p in inputs},('reader not frozen in execution source closure',name)
 return proof

def main():
 p=argparse.ArgumentParser();p.add_argument('--group',choices=['browser37','targeted7'],required=True)
 for n in ['evidence','design','worker','issue','api','verified','native-reference','out']:p.add_argument('--'+n,type=pathlib.Path,required=True)
 p.add_argument('--prior37',type=pathlib.Path);a=p.parse_args();assert not a.out.exists();env=authenticate(evidence=a.evidence,design_path=a.design,worker_path=a.worker,issue_path=a.issue,api_path=a.api,verified_path=a.verified,native_reference_path=a.native_reference,group=a.group);proof=ast_proof(env,a.group);binding=env['binding'];S=env['S'];e=env['e'];legacy_design=env['group_design'].copy();legacy_design['script_sha256']=legacy_design['worker_sha256'];B=ExplicitAliases({'partition-factor-targeted-before-binding.json':a.native_reference,'browser-contract-848-production-delivery-20261020/API-ACCEPTED.json':a.api});O=ExplicitAliases({'BROWSER37-ACCEPTED.json':a.prior37}) if a.prior37 else ExplicitAliases({})
 if a.group=='browser37':
  from group37_predicates import review
  report=review(e,legacy_design,binding,S,B,O)
 else:
  assert a.prior37 and a.prior37.is_file(),'Genuine parent37 saved review required before7 acceptance';prior=load(a.prior37);g37=env['E']/'groups/browser37';assert prior['source']==S and prior['engine']==binding['engine'];assert prior['execution_design_sha256']==env['execution_design_sha256'] and prior['top_manifest_sha256']==env['top_manifest_sha256'];assert prior['browser_report_sha256']==digest(g37/'actual-browser/report.json') and prior['manifest_sha256']==digest(g37/'SHA256.json');assert prior['unchanged_checks']==37 and prior['original_assertion_AST_verified'] and prior['owner_credentials_supplied'] is False
  from group7_predicates import review
  report=review(e,legacy_design,binding,S,B,O)
 receipt={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':S,'engine':binding['engine'],'group':a.group,'execution_design_sha256':env['execution_design_sha256'],'top_manifest_sha256':env['top_manifest_sha256'],'combined_worker_sha256':env['worker_sha256'],'external_issue_sha256':env['external_issue_sha256'],'config_canonical_sha256':env['config_canonical_sha256'],'verified_artifact_receipt_sha256':env['verified_sha256'],'api_acceptance_sha256':env['api_sha256'],'native_reference_sha256':env['native_reference_sha256'],'original_assertion_AST_verified':True,'original_assertion_AST_sha256':proof['assertion_block_AST_sha256'],'owner_credentials_supplied':False,'scientific_catalog_additions':0,'new_browser_engine_provider_calls':0,'postcollection_saved_review':True,'all64_final_acceptance_still_pending':True}
 if a.group=='browser37':receipt.update(unchanged_checks=37,browser_report_sha256=digest(e/'actual-browser/report.json'),manifest_sha256=env['group_manifest_sha256'],targeted_partition_delivery_still_pending=True,catalog_additions=0)
 else:receipt.update(unchanged_API_controls=11,unchanged_browser_controls=37,fresh_targeted_browser_controls=7,actual_five_asset_hashes_verified=True,targeted_report_sha256=digest(e/'actual-browser/report.json'),targeted_manifest_sha256=env['group_manifest_sha256'],unchanged_browser_acceptance_sha256=digest(a.prior37),original_experiment_requests=0,supplemental18_still_required=True,qualification='Confirms checked methanol/phase consumer implementation, owned drain inventories and explicit peroxide medium admission; independent reference agreement is not empirical retention/activity/kinetics calibration.')
 with a.out.open('x') as f:json.dump(receipt,f,indent=2);f.write('\n')
 print(json.dumps({'group':a.group,'raw_checks_passed':len(report['checks']),'receipt_written':True,'all64_accepted':False}))
if __name__=='__main__':main()
