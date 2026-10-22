"""Prepared scientific saved-reader; external release/issue/raw authentication is prerequisite."""
import argparse,json,hashlib,subprocess
from pathlib import Path
from decimal import Decimal as D, getcontext
getcontext().prec=45
ap=argparse.ArgumentParser();ap.add_argument('--repo',required=True,type=Path);ap.add_argument('--design',required=True,type=Path);ap.add_argument('--responses',required=True,type=Path);ap.add_argument('--out',required=True,type=Path);args=ap.parse_args()
assert not args.out.exists();load=lambda p:json.loads(p.read_bytes(),parse_float=D);sha=lambda b:hashlib.sha256(b).hexdigest();design=load(args.design)
assert design['accepted_engine'] is not None and design['actual_source']==design['accepted_engine']['source_head']
# Source bindings are populated only by a separately parent-reviewed actual-release binder.
assert set(design['actual_source_files_sha256'])==set(design['source_hint_files_sha256'])
for p,h in design['actual_source_files_sha256'].items():assert sha(subprocess.check_output(['git','show',design['actual_source']+':'+p],cwd=args.repo))==h
originals={c['id']:c for c in load(args.repo/'audits/target468-20261016/common-engine-completion/originals/new-experiments-20261016.json')['experiments']}
registry=json.loads(subprocess.check_output(['git','show',design['actual_source']+':data/registry/registry-source-v1.json'],cwd=args.repo),parse_float=D);compositions={x['species_id']:x for x in registry['compositions']}
def formula(s):
 if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
 r=compositions[s];return {**{x['element']:x['count']['value']for x in r['elements']},'charge':r['net_charge']['value']}
def ledger(v):
 out={}
 for p in v['contents']:
  n=p['moles'];assert n.is_finite()and n>=0
  for e,k in formula(p['species']).items():out[e]=out.get(e,D(0))+n*k
 return out
def n(v,*species):return sum((p['moles']for p in v['contents']if p['species']in species),D(0))
tol=D('1e-10');results=[]
for c in design['cases']:
 cid=c['id'];orig=originals[cid];assert c['script']==orig['script'] and c['original_forecast']==orig['expected_result']and c['original_criteria']==orig['acceptance_checks'];assert sha(c['script'].encode())==c['script_sha256']
 raw=(args.responses/(cid.replace(':','-')+'.json')).read_bytes();resp=json.loads(raw,parse_float=D);assert resp['engine']==design['accepted_engine']and resp['script_sha256']==c['script_sha256']and resp['exit_status']==0
 previous={};expected={};maxres=D(0);frames=[];thermal=[]
 for i,row in enumerate(resp['observations']):
  op=row['operator'];assert len(row['bench']['vessels'])==1;v=row['bench']['vessels'][0];assert v['id']==0
  # These originals have no transfer/phase/export/reaction-time operation.
  assert not any(e['event'] in {'solver_failed','gas_evolved','gas_vented','gas_absorbed','evaporated','state_changed'}for e in row['events'])
  assert not v.get('unresolved_materials') and not v.get('surfaces')
  for k,value in v.items():
   if any(s in k for s in ['electrode','exchange','adsorb','solid_solution']):assert not value,(cid,k)
  assert v['elapsed_seconds']==0
  if op['op']=='add':
   for e,k in formula(op['species']).items():expected[e]=expected.get(e,D(0))+k*op['moles']
  else:assert op['op']in {'react','inspect','measure'}
  actual=ledger(v);res={e:actual.get(e,D(0))-expected.get(e,D(0))for e in set(actual)|set(expected)};mx=max(map(abs,res.values()),default=D(0));maxres=max(maxres,mx);assert mx<=tol,(cid,i,res)
  if op['op']=='react':assert previous;frames.append((op['reaction'],previous,v,row['events']))
  thermal.append({'row':i,'operator':op,'temperature_K':v['temperature'],'excess_energy_J':v.get('excess_energy'),'events':[e for e in row['events']if 'heat' in e['event']or 'energy' in e['event']or 'temperature' in e['event']]});previous=v
 sap=[f for f in frames if f[0]=='saponification'];assert len(sap)==1;_,before,after,events=sap[0]
 consumed=n(before,'ethyl_acetate')-n(after,'ethyl_acetate');ethanol=n(after,'ethanol')-n(before,'ethanol');acetate=n(after,'CH3COOH','CH3COO-')-n(before,'CH3COOH','CH3COO-')
 assert consumed>tol and abs(consumed-ethanol)<=tol and abs(consumed-acetate)<=tol and n(after,'ethyl_acetate')<=tol
 actual_events=[e for e in events if e['event']=='org_reacted'and e['name']=='saponification'];assert len(actual_events)==1 and abs(actual_events[0]['extent']-consumed)<=tol
 final=previous;reserve=n(after,'Na+')-n(after,'Cl-')-n(after,'CH3COOH','CH3COO-')
 if cid=='new120:or-007':assert abs(consumed-D('.005'))<=tol and abs(reserve-D('.005'))<=tol
 elif cid=='new120:or-008':
  assert abs(consumed-D('.001'))<=tol and abs(reserve)<=tol;assert n(final,'ethyl_acetate')<=tol and abs(n(final,'ethanol')-D('.001'))<=tol and abs(n(final,'CH3COOH','CH3COO-')-D('.001'))<=tol
  assert n(final,'CH3COOH')>tol and n(final,'Cl-')>tol
 elif cid=='new120:or-009':
  ester=[f for f in frames if f[0]=='esterification'];assert len(ester)==1;fischer=n(ester[0][2],'ethyl_acetate')-n(ester[0][1],'ethyl_acetate');assert fischer>tol and abs(consumed-fischer)<=tol
  assert abs(n(final,'ethanol')-D('.01'))<=tol and abs(n(final,'CH3COOH','CH3COO-')-D('.01'))<=tol and abs(reserve-D('.01'))<=tol
 else:raise AssertionError(cid)
 results.append({'id':cid,'response_sha256':sha(raw),'criteria_passed':[x['id']for x in c['original_criteria']],'ester_consumed_mol':consumed,'ethanol_increment_mol':ethanol,'acetate_family_increment_mol':acetate,'restricted_strong_base_reserve_after_reaction_mol':reserve,'maximum_every_step_atom_charge_residual':maxres,'thermal_diagnostics':thermal,'thermal_limit':'No curated named-reaction heat; these diagnostics do not establish complete physical energy closure or kinetics.','source':design['actual_source']})
report={'state':'PREPARED_ORIGINAL_CRITERIA_REVIEW_ONLY','results':results,'engine':design['accepted_engine'],'scientific_catalog_selection':False,'external_issue_release_receipt_authentication':'Not performed by this narrow scientific reader; must separately be authenticated before invocation.','new_model_executions':0};args.out.write_text(json.dumps(report,indent=2,default=str)+'\n')
