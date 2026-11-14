"""Independent unchanged KI011 source/sink inspection; no product execution."""
import json,pathlib,hashlib,math,copy
R=pathlib.Path(__file__).resolve().parent
def load(n):return json.loads((R/n).read_text())
def sha(n):return hashlib.sha256((R/n).read_bytes()).hexdigest()
def amount(v,s):return sum(p['moles'] for p in v['contents'] if p['species']==s)
def check(response,compositions):
 assert response['execution_status']=='completed' and response['exit_status']==0
 fs=response['observations'];assert len(fs)==21
 budget={};ledgers=[];gross=[0.,0.];acid=[0.,0.];oxygen=[0.,0.];samples=[];initial_sink=[.0001,.0002];known={'H','O','S','K','I','Na','Q'}
 def formula(s):
  c=compositions[s];return {**{a['element']:a['count']['value'] for a in c['elements']},'Q':c['net_charge']['value']}
 for index,f in enumerate(fs):
  op=f['operator'];assert f['step']==index
  if op['op']=='add':
   for e,n in formula(op['species']).items():budget[e]=budget.get(e,0)+n*op['moles']
  total={}
  for v in f['bench']['vessels']:
   assert not v.get('unresolved_materials',[]) and not v.get('material_objects',[]) and v.get('pending_co2_transfer_mol',0)==0
   for p in v['contents']:
    assert math.isfinite(p['moles']) and p['moles']>=0
    for e,n in formula(p['species']).items():total[e]=total.get(e,0)+n*p['moles']
  for e in known:
   residual=total.get(e,0)-budget.get(e,0);assert abs(residual)<=1e-10,(index,e,residual);ledgers.append(dict(step=index,element=e,residual=residual))
  assert all(e['event'] not in ['solver_failed','gas_released','gas_escaped','swept','evaporated'] for e in f['events'])
  if op['op'] in ['measure','inspect']:assert f['bench']==fs[index-1]['bench']
  for e in f['events']:
   if e['event']=='reacted':
    vi=e['vessel'];n=e['moles'];assert math.isfinite(n) and n>=0
    if e['reaction']=='iodide-peroxide-clock':gross[vi]+=n
    elif e['reaction']=='thiosulfate-acid':acid[vi]+=n
    elif e['reaction']=='peroxide-decomposition':oxygen[vi]+=n
    else:raise AssertionError('unexpected reaction')
  if op['op']=='wait':
   pair=[]
   for v in f['bench']['vessels']:
    vi=v['id'];assert amount(v,'I2')==0 and amount(v,'I-')==.002
    assert abs(amount(v,'S4O6-2')-gross[vi])<1e-12
    assert abs(sum(p['moles'] for p in v['contents'] if p['species']=='S')-acid[vi])<1e-12
    assert abs(amount(v,'SO2')-acid[vi])<1e-12
    assert abs(amount(v,'S2O3-2')-(initial_sink[vi]-acid[vi]-2*gross[vi]))<1e-12
    assert amount(v,'S2O3-2')>1e-6
    assert abs(amount(v,'H2O2')-(.002-2*oxygen[vi]-gross[vi]))<1e-12
    assert abs(amount(v,'O2')-oxygen[vi])<1e-12
    pair.append(dict(vessel=vi,time_s=v['elapsed_seconds'],gross_iodine_mol=gross[vi],surviving_iodine_mol=0,remaining_sink_mol=amount(v,'S2O3-2'),sink_initial_mol=initial_sink[vi],acid_sink_loss_mol=acid[vi],iodine_sink_loss_mol=2*gross[vi],remaining_peroxide_mol=amount(v,'H2O2'),peroxide_to_oxygen_mol=2*oxygen[vi],peroxide_to_clock_mol=gross[vi]))
   assert pair[0]['time_s']==pair[1]['time_s'];samples.append(pair)
 assert [p[0]['time_s'] for p in samples]==[10,40,100]
 assert samples[-1][0]['gross_iodine_mol']>samples[-1][1]['gross_iodine_mol']>0
 assert samples[-1][1]['remaining_sink_mol']>samples[-1][0]['remaining_sink_mol']
 warnings=[e for f in fs for e in f['events'] if e['event']=='not_yet_modeled'];assert sum('absorbance is unavailable' in e.get('what','') for e in warnings)==4
 return dict(criteria={'KI-011-C1':True,'KI-011-C2':True},samples=samples,known_element_charge_checks=len(ledgers),max_abs_residual=max(abs(x['residual']) for x in ledgers),ledger=ledgers,induction_time_observed=False,sink_exhaustion_observed=False,original_delay_forecast_confirmed=False,unsupported_absorbance_observations=4,non_consuming_observation=True)
def main():
 response=load('response.json');receipt=load('request-receipt.json');original=load('original.json');source=load('source-compositions.json')
 assert sha('response.json')==receipt['case']['response_body_sha256'] and sha('original.lab')==receipt['case']['original_sha256']==original['script_sha256']==response['script_sha256']
 assert (R/'original.lab').read_text()==original['script'];assert response['engine']==receipt['deployed'] or response['engine']=={k:v for k,v in receipt['deployed'].items() if k!='revision'}
 assert response['engine']['source_head']==source['source']=='abf4e9a34eb16e51f605a2009e542b62002a30ea'
 result=check(response,source['compositions']);mutations=[]
 def last(d):return d['observations'][-1]['bench']['vessels'][0]
 def pool(d,s):return next(p for p in last(d)['contents'] if p['species']==s)
 edits=[('lost_iodine',lambda d:pool(d,'I-').__setitem__('moles',.001)),('lost_sink',lambda d:pool(d,'S2O3-2').__setitem__('moles',0)),('lost_tetrathionate',lambda d:pool(d,'S4O6-2').__setitem__('moles',0)),('lost_SO2',lambda d:pool(d,'SO2').__setitem__('moles',0)),('lost_water',lambda d:pool(d,'water').__setitem__('moles',1)),('invent_I2',lambda d:last(d)['contents'].append(dict(species='I2',phase='aqueous',moles=1e-5))),('wrong_gross_event',lambda d:next(e for f in d['observations'] for e in f['events'] if e.get('reaction')=='iodide-peroxide-clock').__setitem__('moles',1e-4)),('solver_failure',lambda d:d['observations'][-1]['events'].append(dict(event='solver_failed'))),('lost_peroxide',lambda d:pool(d,'H2O2').__setitem__('moles',0)),('invent_exhaustion',lambda d:next(p for p in d['observations'][11]['bench']['vessels'][0]['contents'] if p['species']=='S2O3-2').__setitem__('moles',0))]
 for name,edit in edits:
  bad=copy.deepcopy(response);edit(bad)
  try:check(bad,source['compositions'])
  except (AssertionError,KeyError,StopIteration):mutations.append(dict(case=name,rejected=True))
  else:raise AssertionError(name+' survived')
 result.update(engine=response['engine'],source_tree=source['tree'],response_sha256=sha('response.json'),reader_mutations=mutations,new_job=False,new_chemical_replay=False)
 with (R/'review-result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['ledger','reader_mutations']},indent=2))
if __name__=='__main__':main()
