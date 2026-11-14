"""Independent saved-output review of unchanged FE003. No engine runs."""
from decimal import Decimal as D
from pathlib import Path
import json,hashlib,sys
root=Path(sys.argv[1]); rows=[json.loads(s) for s in (root/'batch3-diverse-fe-003.stdout').read_text().splitlines()]
original=(root/'batch3-diverse-fe-003.lab').read_bytes()
assert hashlib.sha256(original).hexdigest()=='bdbc26af0b823b7fd5169787561758a6760f464a7f325d26ed6f45917513c7ee'
assert not [e for r in rows for e in r['events'] if e['event']=='solver_failed']
formulas={'water':{'H':2,'O':1},'sucrose':{'C':12,'H':22,'O':11},'ethanol':{'C':2,'H':6,'O':1},'CO2':{'C':1,'O':2},'CO2(aq)':{'C':1,'O':2},'HCO3-':{'H':1,'C':1,'O':3,'charge':-1},'CO3-2':{'C':1,'O':3,'charge':-2},'H+':{'H':1,'charge':1},'OH-':{'H':1,'O':1,'charge':-1},'base_equivalents':{'H':1,'O':1,'charge':-1}}
keys=['C','H','O','charge'];bound=D('1e-10')
def ledger(v):
 d={k:D(0) for k in keys}
 for p in v['contents']:
  n=D(str(p['moles']));assert n.is_finite() and n>=0
  if p['species']=='catalase':continue # independently require exact retained dose
  assert p['species'] in formulas,p['species']
  for k,c in formulas[p['species']].items():d[k]+=n*c
 return d
def amount(v,key):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==key),D(0))
waits=[]
for i,r in enumerate(rows):
 if r['operator']['op']!='wait':continue
 before={v['id']:v for v in rows[i-1]['bench']['vessels']};after={v['id']:v for v in r['bench']['vessels']}
 elapsed=r['operator']['seconds'];assert elapsed in [300.0,3300.0]
 for vid,v in after.items():
  b=before[vid];assert v['unresolved_materials']==b['unresolved_materials']
  assert amount(v,'catalase')==amount(b,'catalase')
  expected=ledger(b);actual=ledger(v)
  for e in r['events']:
   if e.get('vessel')!=vid or e['event'] not in ['gas_evolved','gas_absorbed']:continue
   n=D(str(e['moles']));assert n>=0 and n.is_finite() and e['species']=='CO2'
   # The first unclocked pose reports finite internally generated uptake.
   # The second certified canonical pose publishes only external crossings.
   if elapsed==300.0 and e['event']=='gas_absorbed':continue
   sign=1 if e['event']=='gas_absorbed' else -1
   expected['C']+=sign*n;expected['O']+=2*sign*n
  residual={k:actual[k]-expected[k] for k in keys}
  assert all(abs(x)<=bound for x in residual.values()),(elapsed,vid,residual)
  consumed=amount(b,'sucrose')-amount(v,'sucrose')
  ethanol=amount(v,'ethanol')-amount(b,'ethanol')
  assert consumed>0 and abs(ethanol-4*consumed)<=bound
  assert v['pending_co2_transfer_mol']==0.0
  assert v['solution'] and v['solution']['ph']>0 and v['solution'].get('scope','complete')=='complete'
  waits.append({'seconds':elapsed,'vessel':vid,'element_charge_residual':{k:str(x) for k,x in residual.items()},'sucrose_consumed':str(consumed),'ethanol_formed':str(ethanol),'ph':v['solution']['ph']})
final=rows[-1]['bench']['vessels'];eth=[amount(v,'ethanol') for v in final]
assert len(final)==3 and all(v['elapsed_seconds']==3600 for v in final)
assert abs(eth[0]/eth[1]-1)<D('0.05') and eth[2]<eth[0]*D('0.5')
# These comparison diagnostics do not add a new empirical rate calibration.
report={'unchanged_script_sha256':hashlib.sha256(original).hexdigest(),'solver_failures':0,'waits':waits,'final_ethanol_moles':list(map(str,eth)),'original_comparison_met':True,'tolerance_mol_eq':str(bound)}
print(json.dumps(report,indent=2))
