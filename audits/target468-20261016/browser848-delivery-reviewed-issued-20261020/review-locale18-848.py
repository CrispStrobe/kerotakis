"""Saved-byte acceptance; no browser, API, deployment or dispatch."""
import argparse,hashlib,json,pathlib
from locale18_saved_predicates import validate_family_rows
N=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--evidence',type=pathlib.Path,required=True);p.add_argument('--design',type=pathlib.Path,required=True);p.add_argument('--prior44',type=pathlib.Path,required=True);p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args()
def digest(path):
 with path.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load(path):return json.loads(path.read_text())
e=a.evidence;design=load(a.design);binding=design['binding'];source=design['source'];prior=load(a.prior44)
assert digest(pathlib.Path(__file__))==design['saved_reader_sha256'] and digest(N/'locale18_saved_predicates.py')==design['saved_predicates_sha256']
prior_dir=a.design.parent/'prior-acceptance'
for name,want in design['prior_acceptance_files_sha256'].items():assert digest(prior_dir/name)==want
api=load(prior_dir/'API-ACCEPTED.json');base37=load(prior_dir/'BROWSER37-ACCEPTED.json');prior_saved=load(prior_dir/'PRIOR44-ACCEPTED.json');verified=load(prior_dir/'verified-release.json')
assert prior_saved==prior and api['source']==base37['source']==prior['source']==source and api['engine']==base37['engine']==prior['engine']==binding['engine']
assert api['eleven_original_protocols_pass'] and api['concurrency']==[200,429] and base37['unchanged_checks']==37
assert prior['unchanged_API_controls']==11 and prior['unchanged_browser_controls']==37 and prior['fresh_targeted_browser_controls']==7
assert digest(prior_dir/'BROWSER37-ACCEPTED.json')==prior['unchanged_browser_acceptance_sha256']
assert digest(prior_dir/'verified-release.json')==binding['binding']['producer_artifact_receipt_sha256']
assert all(verified['engine'][key]==value for key,value in binding['engine'].items())
assert {name:row['sha256']for name,row in verified['frontend_assets'].items()if name!='rest.html'}==binding['binding']['frontend_assets'] and verified['frontend_assets']['rest.html']['sha256']==binding['binding']['client_sha256']
assert digest(a.prior44)==design['prior44_receipt_sha256'] and prior['source']==source and prior['engine']==binding['engine']
for n,h in load(e/'SHA256.json').items():
 path=pathlib.PurePosixPath(n);assert not path.is_absolute()and'..'not in path.parts;assert digest(e/n)==h
assert load(e/'STARTED.json')['script_sha256']==design['worker_sha256'];done=load(e/'COMPLETED.json');assert done['browser_exit_status']==0 and done['runtime_source']==source and done['harness_head']==binding['harness_head'];assert load(e/'source-unchanged.json')['exit_status']==0
src=load(e/'source.json');assert all(src[k]==binding[k]for k in ['harness_head','harness_tree','binding_sha256','harness_script_sha256'])
assert digest(N/'locale-controls.mjs')==design['locale_helper_sha256'] and digest(N/'run-locale18.mjs')==design['wrapper_sha256']
r=load(e/'actual-browser/report.json');assert r['binding']==binding['binding'] and r['all_checks_passed'] and r['no_owner_credentials_supplied'] and len(r['checks'])==18 and all(c['passed']for c in r['checks'])
assert set(r['assets'])==set(binding['binding']['frontend_assets'])
for n,h in binding['binding']['frontend_assets'].items():assert r['assets'][n]['status']==200 and r['assets'][n]['sha256']==h
assert r['browser']['web_security_disabled'] is False
assert r['client']['status']==200 and r['client']['sha256']==binding['binding']['client_sha256']
expected=[]
for locale in ['de','fr']:
 expected += [locale+' ordinary live water uses its actual analytical characterization',locale+' restored uncharacterized water blank caveat renders at LV2 and LV3',locale+' restored uncharacterized glucose blank caveat renders at LV2 and LV3']
 expected +=[locale+' uncharacterized '+name+' state remains outside the blank domain'for name in ['warm','acid','dry']]
 expected +=[locale+' shipped Fischer '+name+' gate renders actual dynamic conditions'for name in ['cold15','cold25','warm65']]
assert [c['name']for c in r['checks']]==expected
raw=r['raw'];assert set(raw)=={l+'-'+s for l in ['de','fr']for s in ['live','restored-false','restored-true','negative-warm','negative-acid','negative-dry','family-cold15','family-cold25','family-warm65']}
def events(result):return [e for s in result['steps']for e in s['events']]
def text(result):return '\n'.join(line for step in result['steps']for line in step.get('rendered',[]))
def slot(x,k):return next((v for n,v in x.get('slots',[])if n==k),None)
def amount(v,k):return sum(p['moles']for p in v['contents']if p['species']==k)
def physical(state):return [{k:v for k,v in vessel.items()if k not in ['solution','resolved','free_proton','free_hydroxide']}for vessel in state['vessels']]
for key,item in raw.items():assert len(item['meta']['git_rev'])>=7 and source.startswith(item['meta']['git_rev'])
for locale,blank in [('de','Autoprotolyse'),('fr','autoprotolyse')]:
 live=raw[locale+'-live'];measured=[e for e in events(live['preparation'])if e['event']=='measured'and e['instrument']=='ph_meter'];assert len(measured)==1 and not measured[0].get('note_reason') and isinstance(live['preparation']['bench']['vessels'][0]['solution']['ph'],(int,float))
 for suffix in ['restored-false','restored-true','negative-warm','negative-acid','negative-dry']:
  item=raw[locale+'-'+suffix];assert physical(item['original'])==physical(item['invalidated'])==physical(item['restored']);assert item['callsBeforeRead']==item['callsAfterRead'] and item['after']['vessels']==item['restored']['vessels']
  for result in [item['lv2'],item['lv3']]:
   ee=events(result)
   if suffix.startswith('restored'):
    m=[e for e in ee if e['event']=='measured'and e['instrument']=='ph_meter'];assert len(m)==1 and m[0]['value']==7 and m[0]['note_reason']['key']=='measured.neutral-water-baseline' and blank in text(result)
   else:assert not any(e['event']=='measured'and e['instrument']=='ph_meter'for e in ee)and any(e['event']=='not_yet_modeled'and e.get('reason',{}).get('key')=='not-modeled.ph-meter-has-no-solution'for e in ee)and blank not in text(result)
 actuals=[]
 for name,gate in [('cold15','temperature_k'),('cold25','temperature_k'),('warm65','catalyst')]:
  item=raw[locale+'-family-'+name];result=item['preparation'];v=result['bench']['vessels'][0];ee=events(result)
  found=[e for e in ee if e['event']=='not_yet_modeled'and e.get('reason',{}).get('key')=='not-modeled.family-declined-at-gate'and slot(e['reason'],'family')=={'text':{'text':'fischer-esterification'}}];assert found
  assert slot(found[0]['reason'],'gate')['term']['en']==gate;assert abs(v['elapsed_seconds']-1)<1e-12 and abs(amount(v,'CH3COOH')-.01)<1e-12 and abs(amount(v,'ethanol')-.01)<1e-12 and amount(v,'ethyl_acetate')==0
  assert not any(e['event']=='org_reacted'and e.get('name')=='fischer-esterification'for e in ee)
  if name=='warm65':assert v['temperature']>=333.15 and v['free_proton']==0 and amount(v,'H2SO4')==0
  else:actuals.append(v['temperature'])
 assert actuals[0]!=actuals[1]
independent_family_checks=validate_family_rows(raw)
result={'source':source,'engine':binding['engine'],'API_controls':11,'unchanged_browser_controls':37,'prior_targeted_browser_controls':7,'supplemental_locale_browser_controls':18,'all62_browser_checks_accepted':True,'independently_rechecked_family_dynamic_locale_cases':independent_family_checks,'actual_assets':r['assets'],'locale_report_sha256':digest(e/'actual-browser/report.json'),'prior44_receipt_sha256':digest(a.prior44),'manifest_sha256':digest(e/'SHA256.json'),'catalog_additions':0,'all_i18n_complete_claim':False,'scientific_yield_claim':False}
with a.out.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
