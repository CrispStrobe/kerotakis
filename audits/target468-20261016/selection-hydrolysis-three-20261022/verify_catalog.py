#!/usr/bin/env python3
"""Reproduce original hydrolysis selection from saved public bytes; no engine/HTTP."""
import argparse,ast,base64,collections,hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path

def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--staging-root',type=Path);a=p.parse_args();repo=a.repo.resolve();here=Path(__file__).resolve().parent;root=a.staging_root.resolve()if a.staging_root else here.parents[2];cap=here.parent/'hydrolysis-three-original-review-20261022';load=lambda p:json.loads(p.read_bytes());s=load(here/'selection.json')
 for folder in [here,cap]:
  for n,h in load(folder/'evidence-manifest.json')['files_sha256'].items():assert sha(folder/n)==h,n
 old=load(repo/s['previous_catalog']);new=load(root/s['new_catalog']);assert sha(repo/s['previous_catalog'])==s['previous_catalog_sha256']and sha(root/s['new_catalog'])==s['new_catalog_sha256']
 assert old['count']==len(old['experiments'])==339 and new['count']==len(new['experiments'])==342 and new['experiments'][:339]==old['experiments']
 assert len({x['id']for x in new['experiments']})==342 and new['counts_by_group']==dict(collections.Counter(x['group']for x in new['experiments']))
 originals_path=repo/s['original_corpus'];assert sha(originals_path)==s['original_corpus_sha256'];originals={x['id']:x for x in load(originals_path)['experiments']};design=load(cap/'design.public.json');assessment=load(cap/'assessment.json');ind=load(cap/'independent-qualification.json');adj=load(cap/'parent-adjudication.json');provenance=load(cap/'actual-provenance.json')
 assert sha(cap/'parent-adjudication.json')==s['parent_adjudication_sha256']and sha(cap/'independent-qualification.json')==s['independent_qualification_sha256']=='7758fa3177dfb090c0be8460d3e27a0aa276641ed627cd133c617de349b12322'
 assert ind['all_six_original_criteria_earned']is True and ind['independent_allphase_Decimal_atoms_charge_water_and_stoich_pass']is True and ind['no_hidden_clock_or_exports']is True
 assert adj['actual_all6criteria_and_owned_budgets_accepted']is True and adj['approved_fixed468_total']==338 and adj['engine']==design['accepted_engine']==ind['engine']==provenance['accepted_engine']
 ids=['new120:or-007','new120:or-008','new120:or-009'];assert adj['approved_originals']==[c['id']for c in design['cases']]==ids
 complete=load(cap/'completion.json');start=load(cap/'STARTED.json');assert start['design_sha256']==s['original_raw_bound_design_sha256']==provenance['design_sha256']==ind['design_sha256']and start['runner_sha256']==s['raw_runner_sha256']==provenance['runner_sha256']==ind['runner_sha256']
 assert start['accepted_engine']==design['accepted_engine']and complete['no_retries']is True
 for c,e,obs,proof in zip(design['cases'],new['experiments'][339:],assessment['results'],provenance['cases'],strict=True):
  oid=c['id'];orig=originals[oid];assert e['original_id']==obs['id']==proof['id']==oid
  assert c['script']==e['script']==orig['script']and c['script_sha256']==e['script_sha256']==hashlib.sha256(orig['script'].encode()).hexdigest()
  assert c['original_forecast']==e['expected_result']==orig['expected_result']and c['original_criteria']==orig['acceptance_checks']and c['original_limitations']==orig['limitations']
  assert e['limitations'][:len(orig['limitations'])]==orig['limitations']and e['validated_on']['source_head']==design['actual_source']
  for check,original in zip(e['acceptance_checks'],orig['acceptance_checks'],strict=True):assert{ k:v for k,v in check.items()if k!='result'}=={k:v for k,v in original.items()if k!='result'}and check['result']=='met_in_original_supported_scope'and check['id']in obs['criteria_passed']
  q=cap/'responses'/oid.replace(':','-');request=load(q/'request.json');body=load(q/'response.body');http=load(q/'http.json');issue=load(q/'issued.json')
  assert request=={'script':orig['script']}and body['engine']==design['accepted_engine']and body['script_sha256']==c['script_sha256']and body['exit_status']==0 and body['execution_status']=='completed'
  for name,key in [('request.json','request_sha256'),('response.body','response_sha256'),('http.json','http_sha256'),('issued.json','issue_sha256')]:assert sha(q/name)==proof[key]
  assert http['http_status']==200 and http['attempts']==issue['attempt']==1 and http in complete['cases']and issue['request_sha256']==sha(q/'request.json')and issue['script_sha256']==c['script_sha256']and issue['url']==design['origin']+'/api/run'
  assert e['evidence']['response_sha256']==obs['response_sha256']==sha(q/'response.body')
  assert e['observed_result']=={k:v for k,v in obs.items()if k not in ['id','thermal_diagnostics','source','thermal_limit','response_sha256','criteria_passed']}
  for k,v in design['accepted_engine'].items():assert e['validated_on'][k]==v
 for e in new['experiments']:assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
 target={x['id']for x in load(repo/'audits/target468-20261016/common-engine-all668.json')['cases']if not x['id'].startswith('curriculum200:')};assert len(target)==468
 allids=target|{x['id']for x in load(repo/'audits/target468-20261016/common-engine-all668.json')['cases']if x['id'].startswith('curriculum200:')};mapping=[];outside=[]
 for e in new['experiments']:
  oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
  if not match:outside.append(e['id']);continue
  assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
 assert len(mapping)==len({x['original_target_id']for x in mapping})==338 and len(outside)==4
 membership=load(here/'catalog-membership.json');assert membership['mapping']==mapping and membership['outside_target']==outside and membership['catalog_sha256']==sha(root/s['new_catalog'])and membership['selection_approved']is True
 prior=load(repo/'audits/target468-20261016/selection-TC008-20261021/remaining133.json');remaining=load(here/'remaining130.json');assert remaining['cases']==[x for x in prior['cases']if x['original_id']not in ids]and len(remaining['cases'])==remaining['remaining']==130 and remaining['additional_needed_to401']==63
 assert {x['original_id']for x in remaining['cases']}==target-{x['original_target_id']for x in mapping}
 oldtree=ast.parse((cap/'review_responses.predecessor.py').read_bytes());newtree=ast.parse((cap/'review_responses.py').read_bytes());asserts={ast.dump(n,include_attributes=False)for n in ast.walk(newtree)if isinstance(n,ast.Assert)};missing=[n for n in ast.walk(oldtree)if isinstance(n,ast.Assert)and ast.dump(n,include_attributes=False)not in asserts]
 assert len(missing)==1;oldsets=[x for x in ast.walk(missing[0])if isinstance(x,ast.Set)];assert len(oldsets)==1;oldset=ast.literal_eval(oldsets[0]);assert any(oldset<ast.literal_eval(x)for n in ast.walk(newtree)if isinstance(n,ast.Assert)for x in ast.walk(n)if isinstance(x,ast.Set))
 with tempfile.TemporaryDirectory(prefix='kero-hydrolysis-saved-')as tmp:
  t=Path(tmp);responses=t/'responses';responses.mkdir()
  for oid in ids:(responses/(oid.replace(':','-')+'.json')).write_bytes((cap/'responses'/oid.replace(':','-')/'response.body').read_bytes())
  env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1');argv=[sys.executable,'-B',str(cap/'review_responses.py'),'--repo',str(repo),'--design',str(cap/'design.public.json'),'--responses',str(responses),'--out',str(t/'assessment.json')]
  result=subprocess.run(argv,capture_output=True,env=env,timeout=120);assert result.returncode==0,result.stderr.decode()
  assert load(t/'assessment.json')==assessment,'Strict scientific saved-reader reproduction differs'
 print(json.dumps({'all339_predecessors_exact':True,'three_original_scripts_forecasts_criteria_limitations_preserved':True,'six_actual_criteria_and_everyphase_atom_charge_budgets_reproduced':True,'catalog_entries':342,'qualified_fixed468':338,'remaining':130,'additional_to401':63,'new_engine_API_browser_executions':0,'empirical_kinetic_full_heat_mechanism_claim':False}))
if __name__=='__main__':main()
