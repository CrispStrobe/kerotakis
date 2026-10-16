#!/usr/bin/env python3
"""Read saved responses only; no requests, simulator invocation or source mutation."""
import argparse,hashlib,json,math
from pathlib import Path
HERE=Path(__file__).resolve().parent
F={'water':({'H':2,'O':1},0),'H+':({'H':1},1),'Na+':({'Na':1},1),'Br-':({'Br':1},-1),'base_equivalents':({'H':1,'O':1},-1),'CO2':({'C':1,'O':2},0),'CO2(aq)':({'C':1,'O':2},0),'HCO3-':({'H':1,'C':1,'O':3},-1),'CO3-2':({'C':1,'O':3},-2),'O2':({'O':2},0),'N2':({'N':2},0),'bromoethane':({'C':2,'H':5,'Br':1},0),'tert_butyl_bromide':({'C':4,'H':9,'Br':1},0),'tert_butanol':({'C':4,'H':10,'O':1},0),'isobutylene':({'C':4,'H':8},0),'ethanol':({'C':2,'H':6,'O':1},0),'ethene':({'C':2,'H':4},0)}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text(),parse_constant=lambda s:(_ for _ in ()).throw(ValueError(s)))
def finite(x):
 if type(x)in(float,int):assert math.isfinite(x)
 elif isinstance(x,dict):
  for v in x.values():finite(v)
 elif isinstance(x,list):
  for v in x:finite(v)
def amount(v,k):return sum(x['moles'] for x in v['contents']if x['species']==k)
def ledger(v):
 el={};q=0
 for p in v['contents']:
  assert p['moles']>=0
  fs,z=F[p['species']];q+=z*p['moles']
  for k,n in fs.items():el[k]=el.get(k,0)+n*p['moles']
 return el,q
def balance(a,b,events):
 first,q1=ledger(a);last,q2=ledger(b);transfers=[]
 for e in events:
  if e['event'] not in ('gas_evolved','gas_absorbed'):continue
  sign=1 if e['event']=='gas_evolved'else-1;fs,z=F[e['species']];q2+=sign*z*e['moles'];transfers.append(e)
  for k,n in fs.items():last[k]=last.get(k,0)+sign*n*e['moles']
 residual={k:last.get(k,0)-first.get(k,0)for k in sorted(first.keys()|last.keys())}
 return {'elements_mol':residual,'charge_mol':q2-q1,'external_transfers':transfers,'accepted':all(abs(n)<=1e-8 for n in residual.values())and abs(q2-q1)<=1e-8,'basis_policy':'base_equivalents carries OH stoichiometric coordinate, not measured hydroxide; no addition of free_proton/free_hydroxide or solution species to authoritative contents.'}
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--report',type=Path,default=HERE/'assessment.json');args=ap.parse_args();d=read(HERE/'design.json');execution=read(HERE/'execution.json');assert d['engine']==execution['engine'];rows=[]
 for c in d['cases']:
  p=HERE/(c['id']+'.response.json');r=read(p);finite(r);receipt=next(x for x in execution['results']if x['id']==c['id']);assert sha(p)==receipt['response_sha256'];assert r['engine']==d['engine']and r['script_sha256']==c['script_sha256'];assert sha(HERE/(c['id']+'.lab'))==c['script_sha256'];reactions=[]
  for target in c['desired_per_vessel']:
   v=target['vessel'];i=next(i for i,o in enumerate(r['observations'])if o['operator']['op']=='react'and o['operator']['vessel']==v);o=r['observations'][i];before=next(x for x in reversed(r['observations'][:i])if'bench'in x)['bench']['vessels'][v];after=o['bench']['vessels'][v];events=[e for e in o['events']if e.get('vessel')==v];org=[e for e in events if e['event']=='org_reacted'];closure=balance(before,after,events);debit=amount(before,target['substrate'])-amount(after,target['substrate']);gain=amount(after,target['principal_product'])-amount(before,target['principal_product']);brgain=amount(after,'Br-')-amount(before,'Br-');tol=1e-9
   checks={'one_expected_mechanism':len(org)==1 and org[0]['name']==target['mechanism'],'expected_extent':len(org)==1 and abs(org[0]['extent']-.0002)<=tol,'substrate_debit':abs(debit-.0002)<=tol,'principal_product_gain':abs(gain-.0002)<=tol,'bromide_equivalent_gain':abs(brgain-.0002)<=tol,'material_charge_closure':closure['accepted'],'no_solver_failure':not any(e['event']=='solver_failed'for e in events)}
   reactions.append({'vessel':v,'desired':target,'checks':checks,'all_principal_expectations_met':all(checks.values()),'actual_temperature_before_k':before['temperature'],'actual_temperature_after_k':after['temperature'],'actual_mechanisms':[e['name']for e in org],'substrate_debit_mol':debit,'principal_product_gain_mol':gain,'bromide_gain_mol':brgain,'water_inventory_change_mol':amount(after,'water')-amount(before,'water'),'preexisting_base_equivalents_mol':amount(before,'base_equivalents'),'literal_NaOH_before_mol':amount(before,'NaOH'),'closure':closure,'before':before,'after':after,'events':events})
  passed=all(x['all_principal_expectations_met']for x in reactions);rows.append({'id':c['id'],'original_id':c['original_id'],'response_sha256':sha(p),'http_status':receipt['http_status'],'exit_status':r['exit_status'],'execution_status':r['execution_status'],'original_physical_forecast':c['original_physical_forecast'],'principal_forecast_met':passed,'classification':'bounded_explicit_selectivity_table_demonstrated'if passed else 'strong_base_ownership_selectivity_gap','reactions':reactions,'empirical_rates_heat_yields_validated':False,'catalog_case_closed':False})
 report={'schema':'kerotakis.haloalkane-api-assessment.v2','design_sha256':sha(HERE/'design.json'),'execution_sha256':sha(HERE/'execution.json'),'engine':d['engine'],'cases':rows,'requests':4,'retries':0,'original_refusal_preserved':'../D3-SO-001.response.json','principal_forecasts_met':sum(r['principal_forecast_met']for r in rows),'physical_domain_scope':'Explicit completion-on-command table only; no measured rates, reaction heat, quantitative selectivity or general yield validation.','model_issue':'MODEL-006','catalog_cases_closed':0,'production_changed':False}
 with args.report.open('x')as f:json.dump(report,f,ensure_ascii=False,indent=2,allow_nan=False);f.write('\n')
 print([(r['id'],r['principal_forecast_met'],[(x['vessel'],x['actual_mechanisms'],x['closure']['accepted'])for x in r['reactions']])for r in rows])
if __name__=='__main__':main()
