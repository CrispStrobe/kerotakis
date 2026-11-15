"""Independent RX012 original/negative reader. No product imports or native calls."""
import argparse,copy,hashlib,json,math,re,subprocess
from pathlib import Path
SOURCE='c637d2b2f1d3e431cfa41e742214256d19cbbbc4';TREE='27f4470de6b3b1ee1183df2fba3b2235cda5212a'
ORIGINAL='01a5e02d3313c023412b20b4ee248966fb00d364521d472b64484474abe4df8d'
SCRIPT='62ba0274f50d7a1e422a2fcb2bb8ac8687df1fddd607d2c8f25c8b8c821e6023'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def readrows(p):return [json.loads(l)for l in p.read_text().splitlines()if l.strip()]
def require(value,label):
 if not value:raise AssertionError(label)
def physical(positive,negative,registry):
 comp={r['species_id']:r for r in registry['compositions']}
 props={(r['species_id'],r['property']):r['quantity']['value']for r in registry['phase_thermodynamics']if isinstance(r['property'],str)and r['property']in ['molar_mass','mass_density']}
 def add(out,s,n):
  require(s in comp,'unknown inventory species '+s);require(math.isfinite(n),'nonfinite inventory')
  for el in comp[s]['elements']:out[el['element']]=out.get(el['element'],0)+n*el['count']['value']
  out['Q']=out.get('Q',0)+n*comp[s]['net_charge']['value']
 water=100*props['water','mass_density']/props['water','molar_mass'];zinc=1/props['Zn','molar_mass']
 expected_feeds=[(0,'water',water),(0,'ZnSO4',.0001),(0,'Zn',zinc),(1,'water',water),(1,'ZnSO4',.01),(1,'Zn',zinc)]
 residuals=[];warnings=[]
 def frames(rows,feed):
  require(bool(rows),'missing native frames');supplied={};seen=[]
  for i,row in enumerate(rows):
   op=row['operator'];events=row['events'];vessels=row['bench']['vessels']
   require(not any(e['event']=='solver_failed'for e in events),'native solver failure')
   if op['op']=='add':
    seen.append((op['vessel'],op['species'],op['moles']));add(supplied,op['species'],op['moles'])
    added=[e for e in events if e['event']=='added'];require(len(added)==1 and added[0]['species']==op['species']and added[0]['moles']==op['moles'],'feed event/command mismatch')
   for event in events:
    if event['event']in ['gas_evolved','gas_absorbed']:
     require(event['moles']>=0,'negative gas crossing');add(supplied,event['species'],event['moles']*(1 if event['event']=='gas_absorbed'else -1))
    if event['event']=='not_yet_modeled':
     require(event['cause']=='rate-not-modelled'and event['reason']['key']=='not-modeled.stays-as-the-metal','unexpected model boundary')
     warnings.append(dict(frame=i,vessel=event['vessel'],cause=event['cause'],key=event['reason']['key']))
   actual={}
   for v in vessels:
    for key in ['surfaces','exchanges','solid_solutions','electrodes','adsorbed','unresolved_materials','material_objects']:
     require(not v.get(key),'unreviewed inventory owner '+key)
    for portion in v['contents']:require(portion['moles']>=0,'negative retained stock');add(actual,portion['species'],portion['moles'])
   delta={el:actual.get(el,0)-supplied.get(el,0)for el in set(actual)|set(supplied)}
   require(all(abs(n)<=1e-10 for n in delta.values()),'stock element/charge ledger exceeds unchanged1e-10')
   residuals.append(delta)
   if op['op']=='cell':
    require(i>0 and rows[i-1]['bench']==row['bench'],'voltmeter changed whole bench')
  require(len(seen)==len(feed),'original feed count')
  for actual,wanted in zip(seen,feed):require(actual[:2]==wanted[:2]and abs(actual[2]-wanted[2])<1e-12,'original feed amount/units')
 frames(positive,expected_feeds);frames(negative,[f for f in expected_feeds if f[:2]!=(0,'Zn')])
 cells=[r for r in positive if r['operator']['op']=='cell'];require(len(cells)==2,'RX-012-C1 two actual readouts')
 values=[];nernst=[];states=[]
 for row,pair in zip(cells,[(0,1),(1,0)]):
  op=row['operator'];require((op['a'],op['b'])==pair,'original electrode request order')
  events=[e for e in row['events']if e['event']=='cell_voltage'];require(len(events)==1,'RX-012-C1 cell voltage authority')
  event=events[0];reading=event.get('terminals');require(isinstance(reading,dict),'RX-012-C1 missing ordered reading')
  require((reading['first'],reading['second'])==pair,'RX-012-C1 lead-order binding')
  v=row['bench']['vessels'];require(len(v)==2,'two original vessels')
  potentials=[];details=[]
  for index,vessel in enumerate(v):
   info=vessel.get('solution');require(info and info.get('scope','complete')=='complete','RX-012-C2 complete native activity authority')
   provenance=info.get('provenance');require(provenance and 'PHREEQC'in provenance['engine']and 'wateq4f'in provenance['dataset'],'RX-012-C2 native provenance')
   require('WATEQ'in provenance['model']and 0<info['ionic_strength']<1,'native activity model domain')
   activity=[s for s in info['species']if s['name']=='Zn+2'];require(len(activity)==1,'RX-012-C2 actual zinc activity')
   a=activity[0]['activity'];t=vessel['temperature'];require(math.isfinite(a)and a>0 and t>0 and vessel['pressure']>0,'invalid activity/T/P')
   # Same Zn(II)/Zn reference potential cancels. Use each actual T;
   # do not impose 25C or nominal ZnSO4 molarity in place of activity.
   potentials.append(8.31446261815324*t/(2*96485.33212)*math.log(a))
   details.append(dict(vessel=index,activity=a,molality=activity[0]['molality'],temperature_K=t,pressure_Pa=vessel['pressure'],ionic_strength=info['ionic_strength'],native_dataset=provenance['dataset']))
   metal=[p for p in vessel['contents']if p['species']=='Zn'and p['phase']=='solid'];require(len(metal)==1 and metal[0]['moles']==zinc,'retained1g zinc electrode')
  prediction=potentials[pair[1]]-potentials[pair[0]];volts=reading['volts'];require(math.isfinite(volts)and abs(volts-prediction)<=1e-12,'RX-012-C2 independent Nernst activity comparison')
  require(event['anode']==0 and event['cathode']==1 and event['standard_volts']==0 and event['volts']>0,'spontaneous roles/reference changed')
  require(event['volts']==abs(volts),'spontaneous magnitude differs from lead readout')
  values.append(volts);nernst.append(prediction);states=details
 require(values[0]>0 and values[1]<0 and values[0]==-values[1],'RX-012-C1 equal magnitudes/opposite signs')
 ncells=[r for r in negative if r['operator']['op']=='cell'];require(len(ncells)==1,'missing-metal control request')
 require(any(e['event']=='no_cell'for e in ncells[0]['events'])and not any(e['event']=='cell_voltage'for row in negative for e in row['events']),'missing-metal false voltage authority')
 require(not any(p['species']=='Zn'and p['phase']=='solid'for p in ncells[0]['bench']['vessels'][0]['contents']),'negative missing-metal fixture changed')
 require(len(warnings)==3,'slow-zinc rate boundaries not preserved')
 return dict(criteria={'RX-012-C1':'PASS','RX-012-C2':'PASS'},qualification=True,ordered_volts=values,independent_nernst_volts=nernst,native_states=states,positive_frames=len(positive),negative_frames=len(negative),whole_bench_read_only=True,solid_Zn_g_each=1,missing_metal_refused=True,ledger_max_abs=max(abs(x)for d in residuals for x in d.values()),ledger_residuals=residuals,preserved_model_warnings=warnings,scope='Instantaneous model open-circuit ordered readout, native activities at actual T/P, ideal salt bridge; no current/lifetime/slow-zinc kinetics or empirical calibration.')

def review(e,repo):
 require((e/'COMPLETED.json').exists()and not(e/'FAILED.json').exists(),'producer incomplete')
 manifest=json.loads((e/'SHA256.json').read_text());verified=[]
 for name,h in manifest.items():
  p=Path(name);require(not p.is_absolute()and '..'not in p.parts,'manifest path')
  if name=='native-kero':continue
  require((e/p).exists()and sha(e/p)==h,'producer hash '+name);verified.append(name)
 completed=json.loads((e/'COMPLETED.json').read_text());start=json.loads((e/'STARTED.json').read_text());source=json.loads((e/'source-and-lock.json').read_text())
 require(completed['source']==start['source']==source['source']==SOURCE and completed['tree']==start['tree']==source['tree']==TREE,'source/tree binding')
 require(start['script_sha256']==SCRIPT and sha(e/'original.lab')==ORIGINAL,'accepted runner/original hash')
 require(completed['original_exit']==completed['negative_exit']==0 and completed['original_id']=='new120:rx-012','native command exit/scope')
 require(not completed['PO002_replayed']and not completed['GS007_replayed'],'forbidden original replay')
 require(sha(e/'actual-producer-Cargo.lock')==source['cargo_lock_sha256']=='c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7','actual lock')
 require(sha(e/'compiler.stdout')=='be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6','actual compiler')
 require(subprocess.check_output(['git','-C',str(repo),'rev-parse',SOURCE+'^{tree}'],text=True).strip()==TREE,'independent source tree')
 for name,h in json.loads((e/'composed-source-files.json').read_text()).items():require(hashlib.sha256(subprocess.check_output(['git','-C',str(repo),'show',SOURCE+':'+name])).hexdigest()==h,'source file '+name)
 require(hashlib.sha256(subprocess.check_output(['git','-C',str(repo),'show',SOURCE+':data/registry/registry-source-v1.json'])).hexdigest()==sha(e/'registry-source-v1.json'),'actual registry')
 build=json.loads((e/'build-exact-cli.json').read_text());require(build['argv']==['cargo','build','--locked','-p','kerotakis-cli','--bin','kero','--message-format=json']and build['exit']==0,'fresh native build argv')
 artifact=[]
 for line in(e/'build-exact-cli.stdout').read_text().splitlines():
  try:r=json.loads(line)
  except ValueError:continue
  if r.get('reason')=='compiler-artifact'and r.get('executable')and r['target']['name']=='kero':artifact.append(r)
 require(len(artifact)==1 and artifact[0]['fresh']is False and artifact[0]['profile']['test']is False,'fresh Cargo production binary identity')
 exe=artifact[0]['executable']
 for name,script in [('rx012-original','original.lab'),('rx012-missing-metal','missing-metal.lab')]:
  step=json.loads((e/(name+'.json')).read_text());args=step['argv'];require(step['exit']==0 and args[0]==exe and args[1]=='run'and Path(args[2]).name==script and args[3:]==['--json'],'actual fresh binary/argv '+name)
 for name in ['build-exact-cli','rx012-original','rx012-missing-metal']:
  s=json.loads((e/(name+'.json')).read_text());require(sha(e/(name+'.stdout'))==s['stdout_sha256']and sha(e/(name+'.stderr'))==s['stderr_sha256'],'actual output hash '+name)
  g=json.loads((e/(name+'-resource-admission.json')).read_text());require(g['MemAvailable_MiB']>=4608 and g['load1']<3.5 and g['free_bytes']>12*1024**3,'unchanged resource guard '+name)
 native=json.loads((e/'native-executable.json').read_text());stream=json.loads((e/'streamed-executable.json').read_text());require(native['source']==stream['source']==SOURCE and native['tree']==stream['tree']==TREE and native['sha256']==stream['actual_sha256']and stream['verified'],'actual binary stream binding')
 positive=readrows(e/'rx012-original.stdout');negative=readrows(e/'rx012-missing-metal.stdout');registry=json.loads((e/'registry-source-v1.json').read_text());result=physical(positive,negative,registry)
 mutations={}
 def reject(label,p,n):
  try:physical(p,n,registry)
  except (AssertionError,KeyError):mutations[label]='REJECTED'
  else:raise AssertionError('mutation accepted '+label)
 cells=[i for i,r in enumerate(positive)if r['operator']['op']=='cell'];a,b=cells
 p=copy.deepcopy(positive);p[b]['events'][0]['terminals']['volts']=abs(p[b]['events'][0]['terminals']['volts']);reject('same_positive_sign',p,negative)
 p=copy.deepcopy(positive);p[a]['events'][0].pop('terminals');reject('missing_lead_order',p,negative)
 p=copy.deepcopy(positive);p[a]['events'][0]['terminals']['first']=1;reject('false_lead_binding',p,negative)
 p=copy.deepcopy(positive);p[a]['bench']['vessels'][0]['contents'][0]['moles']+=.001;reject('changed_stock',p,negative)
 p=copy.deepcopy(positive)
 for r in p:
  for v in r['bench']['vessels']:
   if v['id']==1 and v.get('solution'):
    for d in v['solution']['species']:
     if d['name']=='Zn+2':d['activity']*=2
 reject('false_zinc_activity',p,negative)
 p=copy.deepcopy(positive)
 for r in p:
  for v in r['bench']['vessels']:
   if v.get('solution'):v['solution']['provenance']['engine']='placeholder'
 reject('false_native_provenance',p,negative)
 n=copy.deepcopy(negative)
 for r in n:
  if r['operator']['op']=='cell':r['events']=[]
 reject('missing_refusal',positive,n)
 p=copy.deepcopy(positive);p[b]['bench']['vessels'][0]['temperature']+=1;reject('readout_mutates_bench',p,negative)
 result.update(source=SOURCE,tree=TREE,verified_producer_files=len(verified),actual_binary_sha256=stream['actual_sha256'],fresh_Cargo_binary=True,actual_argv_verified=True,compiler_lock_verified=True,reader_mutations=mutations,coverage_before=431,coverage_after=432,other_originals_replayed=False)
 return result

def main():
 p=argparse.ArgumentParser();p.add_argument('--evidence',type=Path,required=True);p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();result=review(a.evidence,a.repo)
 with a.out.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result))
if __name__=='__main__':main()
