#!/usr/bin/env python3
"""Verify TC008-only catalogue addition and recompute saved evidence (no engine)."""
import argparse,collections,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
def sha(p):
 with p.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--focus',type=Path);a=p.parse_args();r=a.repo.resolve();root=r/'audits/target468-20261016';here=root/'selection-TC008-20261021';cap=root/'TC008-qualified-original-review-20261021';load=lambda p:json.loads(p.read_bytes());s=load(here/'selection.json');old=load(r/s['previous_catalog']);new=load(r/s['new_catalog']);assessment=load(cap/'assessment.json');adj=load(cap/'parent-adjudication.json')
 assert sha(r/s['previous_catalog'])==s['previous_catalog_sha256']and sha(r/s['new_catalog'])==s['new_catalog_sha256'];assert old['count']==len(old['experiments'])==338 and new['count']==len(new['experiments'])==339;assert new['experiments'][:338]==old['experiments'];assert new['counts_by_group']==dict(collections.Counter(x['group']for x in new['experiments']));assert sha(cap/'parent-adjudication.json')==s['parent_adjudication_sha256']and adj['approved_fixed468_total']==335
 e=new['experiments'][-1];o=next(x for x in load(r/'audits/new-experiments-201-20261016/intake.json')if x['original_id']=='batch2:tc-008');assert e['original_id']==o['original_id']==s['new_selected_originals'][0];assert e['script']==(r/'audits/new-experiments-201-20261016'/o['original_script']).read_text(encoding='utf-8');assert e['expected_result']==o['original_expected_result'];assert e['script_sha256']==o['script_sha256'];assert len(o['commands'])==9
 for x,y in zip(e['acceptance_checks'],o['original_acceptance_checks'],strict=True):assert {k:v for k,v in x.items()if k!='result'}=={k:v for k,v in y.items()if k!='result'}and x['result']=='met_in_original_supported_scope'
 assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations']and e['observed_result']==assessment['metrics'];assert e['validated_on']['source_head']==assessment['actual_focus_source']and e['validated_on']['binary_sha256']==assessment['actual_trace_binary_sha256']and e['validated_on']['cargo_lock_sha256']==assessment['actual_focus_lock_sha256']
 for x in new['experiments']:assert hashlib.sha256(x['script'].encode()).hexdigest()==x['script_sha256']
 assert len({x['id']for x in new['experiments']})==339
 ids={x['id']for x in load(root/'common-engine-all668.json')['cases']};target={x for x in ids if not x.startswith('curriculum200:')};assert len(target)==468;mapping=[];outside=[]
 for x in new['experiments']:
  oid=x.get('original_id',x['id']);matches={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&ids
  if not matches:outside.append(x['id']);continue
  assert len(matches)==1 and matches<=target;mapping.append({'catalog_id':x['id'],'original_target_id':matches.pop()})
 assert len(mapping)==len({x['original_target_id']for x in mapping})==335 and len(outside)==4;assert load(here/'catalog-membership.json')=={'catalog_sha256':sha(r/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':335}
 rem=load(here/'remaining133.json');prior=load(root/'selection-AD006-20261021/remaining134.json');assert rem['cases']==[x for x in prior['cases']if x['original_id']!='batch2:tc-008'];assert rem['remaining']==len(rem['cases'])==133 and rem['additional_needed_to401']==66;assert {x['original_id']for x in rem['cases']}==target-{x['original_target_id']for x in mapping};score=load(root/'scoreboard-20261021-reviewed-TC008.json');assert score['catalog_sha256']==sha(r/s['new_catalog'])and score['current_working_catalog_entries']==339 and score['source_qualified_selected_in_target']==335
 for directory in [cap,here]:
  for name,want in load(directory/'evidence-manifest.json')['files_sha256'].items():assert sha(directory/name)==want,name
 with tempfile.TemporaryDirectory(prefix='kero-tc008-saved-')as td:
  args=[sys.executable,str(cap/'review_saved.py'),'--repo',str(r),'--out-dir',str(Path(td)/'review')]
  if a.focus:args+=['--focus',str(a.focus)]
  result=subprocess.run(args,capture_output=True,text=True,encoding='utf-8',timeout=240);assert result.returncode==0,result.stderr
 print(json.dumps({'all338_predecessors_exact':True,'original_two_criteria_and_nine_commands_preserved':True,'saved_math_and_source_proof_reproduced':True,'catalog_entries':339,'unique_fixed468':335,'remaining':133,'additional_to401':66,'new_product_engine_API_executions':0,'deployed_original_replay_claim':False,'empirical_calibration_claim':False}))
if __name__=='__main__':main()
