"""Verify exported raw evidence only; makes no requests or product executions."""
from pathlib import Path
import hashlib,json
P=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda p:json.loads(p.read_bytes())
for name,want in load(P/'SHA256.json').items():
 assert sha(P/name)==want,name
accepted=load(P/'PRODUCTION-ACCEPTED.json')
api_dir=P.parent/'production850-api-acceptance-20261020'
assert sha(api_dir/'API-ACCEPTED.json')==accepted['API_receipt_sha256']
assert sha(api_dir/'verified-release.json')==accepted['verified_artifact_receipt_sha256']
api=load(api_dir/'API-ACCEPTED.json');assert api['engine']==accepted['engine']
assessment=load(api_dir/'script-assessment.json');assert len(assessment['cases'])==11
for c in assessment['cases']:
 d=api_dir/'cases'/c['id']
 assert sha(d/'response.body')==c['response_sha256']
 assert hashlib.sha256(load(d/'request.json')['script'].encode()).hexdigest()==c['script_sha256']
 assert c['status']=='PASS' and all(c['criteria'].values())
assert accepted['unchanged_API_controls']==11 and accepted['actual_concurrency']==[200,429]
assert accepted['retained_browser_controls']==62 and accepted['supplemental_heat_controls']==2
for group,count in [('browser37',37),('targeted7',7),('locale18',18),('heat2',2)]:
 f=P/'groups'/group/'actual-browser/report.json';r=load(f)
 assert sha(f)==accepted['raw_report_sha256'][group]
 assert r['all_checks_passed'] and len(r['checks'])==count and all(c['passed']for c in r['checks'])
 assert load(P/'groups'/group/'COMPLETED.json')['browser_exit_status']==0
heat=load(P/'groups/heat2/actual-browser/report.json')
for c in heat['checks']:
 o=c['observed'];assert o['consumed_mol']>0
 measured=o['heat_j']/o['consumed_mol']
 assert abs(measured-98480)<=790 and abs(measured-o['j_per_consumed_mol'])<1e-8
r=load(P/'ALL64-SAVED-INVOCATIONS-VERIFIED.json')
assert sha(P/'ALL64-SAVED-INVOCATIONS-VERIFIED.json')==accepted['all64_invocations_receipt_sha256']
assert sha(P/'all64-agreement.json')==accepted['agreement_sha256']==r['agreement_sha256']
assert len(r['actual_invocations_sha256'])==9 and r['saved_predicates_and_actual_invocations_verified']
projection=load(P/'actual-invocations-public-projection.json')['records']
for name,want in r['actual_invocations_sha256'].items():
 assert projection[name]['private_original_record_sha256']==want
 assert projection[name]['exit_status']==0 and projection[name]['error_type']is None
assert accepted['scientific_selections']==0 and accepted['fixed468_qualified']==332
print(json.dumps({'public_export_integrity':True,'API_controls':11,'browser_controls':64,'private_full_provenance_qualified_in_PUBLIC_SCOPE':True,'new_product_calls':0,'scientific_selections':0}))
