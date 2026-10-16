#!/usr/bin/env python3
"""Assess saved production delivery responses; never sends requests."""
import argparse,hashlib,json,math
from pathlib import Path
HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text(),parse_constant=lambda s:(_ for _ in ()).throw(ValueError(s)))
def finite(x):
 if type(x)in(float,int):assert math.isfinite(x)
 elif isinstance(x,dict):
  for v in x.values():finite(v)
 elif isinstance(x,list):
  for v in x:finite(v)
def state(row):return row['bench']['vessels'][0]
def amount(v,key):return sum(p['moles']for p in v['contents']if p['species']==key)
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--report',type=Path,default=HERE/'assessment.json');args=ap.parse_args();d=read(HERE/'design.json');b=read(HERE/'binding.json');e=read(HERE/'execution.json');assert sha(HERE/'binding.json')==d['binding_sha256'];assert sha(HERE/'design.json')==e['design_sha256'];checks={}
 for r in e['requests']:
  for suf,key in [('body','body_sha256'),('headers','headers_sha256'),('client.stderr','stderr_sha256')]:assert sha(HERE/(r['id']+'.'+suf))==r[key]
  checks[r['id']+'_http_status']=r['client_exit']==0 and r['http_status']==r['expected_http_status']
 rows={}
 for c in d['cases']:
  raw=read(HERE/(c['id']+'.body'));finite(raw);assert raw['engine']==d['engine']and raw['script_sha256']==c['script_sha256'];assert sha(HERE/(c['id']+'.lab'))==c['script_sha256'];checks[c['id']+'_completed']=raw['execution_status']=='completed'and raw['exit_status']==0;rows[c['id']]=raw['observations']
 base=rows['water-baseline'];v=state(base[-1]);checks['water_finite_input_retained']=abs(amount(v,'water')-base[0]['operator']['moles'])<=1e-8 and len(v['contents'])==1;checks['water_temperature']=abs(v['temperature']-298.15)<=1e-6;checks['water_solvent_scope']=v['solution']['scope']=='solvent_only'
 obs=rows['complete-stock-001'];water=state(obs[0]);acid=state(obs[2]);neutral=state(obs[4]);final=state(obs[-1]);gain=neutral['temperature']-acid['temperature'];watergain=amount(final,'water')-amount(water,'water');checks.update(acid_ph=1.8<=acid['solution']['ph']<=2.3,neutral_ph=6.5<=neutral['solution']['ph']<=7.5,wait_ph_stable=abs(final['solution']['ph']-neutral['solution']['ph'])<=.01,positive_modeled_heat_response=.1<=gain<=.5,sodium_retained=abs(amount(final,'Na+')-.001)<=1e-8,chloride_retained=abs(amount(final,'Cl-')-.001)<=1e-8,water_stoichiometry=abs(watergain-.001)<=1e-8,final_canonical_inventory=set(p['species']for p in final['contents'])=={'water','Na+','Cl-'})
 expected={'H':2*amount(water,'water')+.002,'O':amount(water,'water')+.001,'Na':.001,'Cl':.001};actual={'H':2*amount(final,'water'),'O':amount(final,'water'),'Na':amount(final,'Na+'),'Cl':amount(final,'Cl-')};residual={k:actual[k]-expected[k]for k in expected};charge=amount(final,'Na+')-amount(final,'Cl-');checks['element_charge_closure']=all(abs(n)<=1e-8 for n in residual.values())and abs(charge)<=1e-8
 events=[event for row in obs for event in row['events']];checks['no_solver_failed_or_safety_veto']=not any(x['event']in('solver_failed','safety_veto')for x in events)
 report={'schema':'kerotakis.production-assessment.v1','design_sha256':sha(HERE/'design.json'),'binding_sha256':sha(HERE/'binding.json'),'execution_sha256':sha(HERE/'execution.json'),'engine':d['engine'],'all_frozen_delivery_and_control_checks_pass':all(checks.values()),'checks':checks,'observations':{'water_mol':amount(v,'water'),'water_temperature_k':v['temperature'],'water_ph':v['solution']['ph'],'acid_ph':acid['solution']['ph'],'neutral_ph':neutral['solution']['ph'],'after_wait_ph':final['solution']['ph'],'temperature_before_base_k':acid['temperature'],'temperature_after_base_k':neutral['temperature'],'base_step_temperature_gain_k':gain,'after_wait_temperature_k':final['temperature'],'water_increment_mol':watergain,'final_contents':final['contents'],'element_residual_mol':residual,'charge_residual_mol':charge},'limits':['Production c446 delivery acceptance of these two finite controls only; not the entire200 or new201.','The old source/binary responses are reference context, not a required byte-equality oracle.','Modeled heat response and conservation do not establish empirical thermal calibration.','Producer-reported Cargo.lock hash is bound; raw producer lock bytes were not independently rehashed.','Actual producer isolation and completed requests plus source context/lease cleanup do not independently prove VM deletion.','Pages remains separately reported404; served production rest.html bytes match web/rest.html without a new browser test.'],'requests_retried':0,'chemistry_requests':2,'catalog_cases_closed':0,'local_cli_or_build':False,'production_configuration_changed':False}
 with args.report.open('x')as f:json.dump(report,f,indent=2,allow_nan=False);f.write('\n')
 print(json.dumps({'all_pass':all(checks.values()),'checks':checks,'observations':report['observations']},indent=2))
if __name__=='__main__':main()
