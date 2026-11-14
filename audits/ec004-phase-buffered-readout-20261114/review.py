"""Independent original criteria and two-stage provenance reader; never executes product."""
import argparse,copy,hashlib,json,math,pathlib
SOURCE='4151791fafb05dbc88ce39baec83b33ca4dd7b05';TREE='42a1d68a2507062f81d201b3852b4f6b4e72d3ac';FIXTURE='2bdc16d1848513c8b7eb8d921e6f0b9e19aecfcbcd0bd796b2f3d4b2d46e8fff';PHYSICAL='4bd93add656c936b0724673c7ea8332cf21e906f'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_text())
def amount(v,s,phase):return sum(p['moles'] for p in v['contents'] if p['species']==s and p['phase']==phase)
def activity(v,s):
 solution=v['solution'];assert solution.get('scope','complete')=='complete'
 a=next(p['activity'] for p in solution['species'] if p['name']==s)
 assert math.isfinite(a) and a>0
 return a
def check(data):
 frames=data['frames'];fixture=data['fixture'];before=data['before'];after=data['after'];leads=data['leads'];neg=data['neg'];binding=data['binding'];binary=data['binary']
 assert binding['source']==SOURCE and binding['tree']==TREE and binding['fixture_sha256']==FIXTURE
 assert binding['physical_source']==PHYSICAL and binding['historical_native_binary_sha256'] is None
 assert fixture['physical_source']==PHYSICAL and fixture['historical_native_binary_sha256'] is None
 assert binding['cargo_lock_sha256']=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7'
 assert len(binary)==1 and binary[0]['source']==SOURCE and binary[0]['tree']==TREE and len(binary[0]['sha256'])==64
 assert binary[0]['cargo_lock_sha256']==binding['cargo_lock_sha256']
 assert fixture['before_chloride']==frames[7]['bench']['vessels'] and fixture['after_chloride']==frames[9]['bench']['vessels']
 compositions=data['compositions']['compositions'];budget={};residuals=[]
 def comp(s):
  row=compositions[s];return {**{e['element']:e['count']['value'] for e in row['elements']},'Q':row['net_charge']['value']}
 for f in frames:
  op=f['operator']
  if op['op']=='add':
   for e,n in comp(op['species']).items():budget[e]=budget.get(e,0)+n*op['moles']
  totals={}
  for v in f['bench']['vessels']:
   assert not v.get('unresolved_materials',[]) and v.get('pending_co2_transfer_mol',0)==0
   for p in v['contents']:
    assert math.isfinite(p['moles']) and p['moles']>=0
    for e,n in comp(p['species']).items():totals[e]=totals.get(e,0)+n*p['moles']
  assert not any(e['event']=='solver_failed' for e in f['events'])
  for e in set(budget)|set(totals):
   residual=totals.get(e,0)-budget.get(e,0);assert abs(residual)<=1e-10
   residuals.append(dict(step=f['step'],element=e,residual=residual))
 untreated,treated=fixture['after_chloride']
 assert amount(treated,'AgCl','solid')>1e-6 and amount(treated,'Ag','solid')==.001 and amount(untreated,'Ag','solid')==.001
 assert 0<activity(treated,'Ag+')<activity(untreated,'Ag+')
 for stage,key in [(before,'before_chloride'),(after,'after_chloride')]:
  assert stage['physical_source']==PHYSICAL and stage['historical_native_binary_sha256'] is None
  assert 'not rerun' in stage['stage']
  assert stage['before']==stage['after']==fixture[key]
 assert any(e['event']=='no_cell' and '0 V' in e['why'] for e in before['events'])
 event=next(e for e in after['events'] if e['event']=='cell_voltage')
 expected=8.31446261815324/96485.33212*(untreated['temperature']*math.log(activity(untreated,'Ag+'))-treated['temperature']*math.log(activity(treated,'Ag+')))
 assert expected>.1 and abs(event['volts']-expected)<1e-12
 assert event['anode']==1 and event['cathode']==0 and event['standard_volts']==0
 assert leads['before']==leads['after']==fixture['after_chloride']
 a,b=leads['readings'];assert a>0 and a==-b and abs(a-expected)<1e-12
 assert len(neg)==27 and all(r==dict(case=i,refused=True,unchanged=True) for i,r in enumerate(neg))
 assert any(e['event']=='no_cell' for e in frames[9]['events'])
 return dict(criteria={'EC-004-C1':True,'EC-004-C2':True},historical_criteria={'EC-004-C1':True,'EC-004-C2':False},volts=event['volts'],independent_expected_volts=expected,ledger_checks=len(residuals),max_abs_residual=max(abs(r['residual']) for r in residuals),ledger=residuals,negative_cases=27,physical_source=PHYSICAL,historical_native_binary_sha256=None,readout_source=SOURCE,readout_executable_sha256=binary[0]['sha256'],chemical_prefix_replayed=False)
def main():
 p=argparse.ArgumentParser();p.add_argument('outputs',type=pathlib.Path);p.add_argument('report',type=pathlib.Path);a=p.parse_args();o=a.outputs;r=pathlib.Path(__file__).parent
 for name,expected in load(o/'SHA256.json').items():assert sha(o/name)==expected,name
 assert sha(o/'native-pair-fixture.json')==FIXTURE
 assert sha(o/'ec004-historical.stdout')=='a10ec1893d55d1313326916875d82c17afb8a2692a863f7f6977c79195f33455'
 assert (o/'ec004-historical.stdout').read_bytes()==(r/'batch2-ec-004.stdout').read_bytes()
 assert sha(o/'readout-core-test')==load(o/'readout-executable.json')[0]['sha256']
 completed=load(o/'COMPLETED.json');assert completed['source']==SOURCE and completed['tree']==TREE and completed['all_commands_passed'] and not completed['chemical_prefix_replayed']
 stdout=(o/'readout-controls.stdout').read_text();assert stdout.count('test result: ok.')==3 and 'test result: FAILED' not in stdout
 assert '5 passed; 0 failed' in stdout and '0 passed; 0 failed' not in stdout
 data=dict(frames=[json.loads(l) for l in (o/'ec004-historical.stdout').read_text().splitlines() if l.startswith('{')],fixture=load(o/'native-pair-fixture.json'),before=load(o/'readouts/before_chloride.json'),after=load(o/'readouts/after_chloride.json'),leads=load(o/'readouts/signed-leads.json'),neg=load(o/'readouts/negative-cases.json'),binding=load(o/'source-and-lock.json'),binary=load(o/'readout-executable.json'),compositions=load(r/'source-compositions.json'))
 result=check(data);mutations=[]
 edits=[('lost_AgCl',lambda d:d['fixture']['after_chloride'][1]['contents'].__setitem__(5,dict(species='AgCl',phase='solid',moles=0))),('wrong_voltage',lambda d:next(e for e in d['after']['events'] if e['event']=='cell_voltage').__setitem__('volts',0)),('wrong_sign',lambda d:d['leads']['readings'].__setitem__(0,-.2)),('mutated_readout',lambda d:d['after']['after'][1].__setitem__('temperature',300)),('wrong_source',lambda d:d['binding'].__setitem__('source',PHYSICAL)),('wrong_fixture',lambda d:d['binding'].__setitem__('fixture_sha256','0'*64)),('historical_binary_substitution',lambda d:d['binding'].__setitem__('historical_native_binary_sha256',d['binary'][0]['sha256'])),('missing_negative',lambda d:d['neg'].pop()),('solver_failure',lambda d:d['frames'][8]['events'].append(dict(event='solver_failed'))),('missing_native_activity',lambda d:next(x for x in d['fixture']['after_chloride'][1]['solution']['species'] if x['name']=='Ag+').__setitem__('activity',0))]
 for name,edit in edits:
  bad=copy.deepcopy(data);edit(bad)
  try:check(bad)
  except (AssertionError,KeyError,StopIteration):mutations.append(dict(case=name,rejected=True))
  else:raise AssertionError('mutation survived '+name)
 result['reader_mutations']=mutations;result['matched_core_tests']=10
 with a.report.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ('ledger','reader_mutations')},indent=2))
if __name__=='__main__':main()
