"""Independently reproduce the saved64 material projection. No simulations."""
from pathlib import Path
import ast,collections,hashlib,json,math,zipfile
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
BASE=ROOT/'audits/target468-20261016'
def sha(b): return hashlib.sha256(b).hexdigest()
def load(b):
 def pairs(items):
  d={}
  for k,v in items:
   assert k not in d;k0=k;d[k0]=v
  return d
 return json.loads(b,object_pairs_hook=pairs,parse_constant=lambda s:(_ for _ in ()).throw(ValueError(s)))
formulas={};charges={}
for path in ['saved-assessment-first120-ab/review_saved.py','saved-assessment-first120-redox/review_saved.py']:
 for n in ast.parse((BASE/path).read_text()).body:
  if isinstance(n,ast.Assign):
   for target in n.targets:
    if isinstance(target,ast.Name) and target.id in ['FORMULAS','CHARGE','CHARGES']:
     d=ast.literal_eval(n.value); dest=formulas if target.id=='FORMULAS' else charges
     for k,v in d.items(): assert k not in dest or dest[k]==v
     dest.update(d)
formulas.update({'H2O2':{'H':2,'O':2},'catalase':{'catalase_tracer':1},'ethyl_acetate':{'C':4,'H':8,'O':2},'glucose':{'C':6,'H':12,'O':6},'sucrose':{'C':12,'H':22,'O':11},'hexane':{'C':6,'H':14},'naphthalene':{'C':10,'H':8},'S':{'S':1},'SO2':{'S':1,'O':2},'CaO':{'Ca':1,'O':1},'chalcanthite':{'Cu':1,'S':1,'O':9,'H':10},'AgCl':{'Ag':1,'Cl':1},'CuO':{'Cu':1,'O':1},'SiO2':{'Si':1,'O':2},'KCl':{'K':1,'Cl':1},'KNO3':{'K':1,'N':1,'O':3},'Fe':{'Fe':1}})
keys=sorted({e for f in formulas.values() for e in f}|{'charge'})
def zero():return dict.fromkeys(keys,0.)
def book(out,s,n):
 assert s in formulas, s
 assert math.isfinite(n)
 for e,v in formulas[s].items():out[e]+=v*n
 out['charge']+=charges.get(s,0)*n
screen=load((HERE/'first-screen.json').read_bytes())
manifest={'cases':[{'id':c['id']} for c in screen['cases']], 'engine':screen['engine'], 'raw_artifact_sha256':screen['raw_artifact_sha256']}
originals=load((BASE/'common-engine-completion/originals/new-experiments-20261016.json').read_bytes())
assert sha((BASE/'common-engine-completion/originals/new-experiments-20261016.json').read_bytes())==screen['originals_sha256']
cases={c['id']:c for c in originals['experiments']}
verified=load((BASE/'common-engine-completion/results/verified-results.json').read_bytes()); assert verified['engine']==manifest['engine']
receipts={c['id']:c for c in verified['cases']}
archive=BASE/'common-engine-completion/results/raw-artifact.zip'
assert sha(archive.read_bytes())==manifest['raw_artifact_sha256']
records=[]
with zipfile.ZipFile(archive) as z:
 for item in manifest['cases']:
  c={**cases[item['id']], 'receipt':receipts[item['id']]}
  assert sha(c['script'].encode())==c['script_sha256']==c['receipt']['script_sha256']
  prefix=c['id'].replace(':','_');stream=z.read(prefix+'/stdout.jsonl');assert sha(stream)==c['receipt']['stdout_sha256']
  rows=[load(b) for b in stream.splitlines()];incoming=zero();outgoing=zero();state={};steps=[];boundaries=[]
  for i,r in enumerate(rows):
   op=r['operator'];kind=op['op']
   if kind=='add':book(incoming,op['species'],op['moles'])
   if kind=='extract':book(incoming,op['solvent'],op['total_solvent'])
   for e in r['events']:
    typ=e['event']
    if typ in ['vessel_sealed','vessel_pressure_controlled']:
     n=e['trapped_air'] if typ=='vessel_sealed' else e['trapped_gas']
     for s,x in [('N2',.7901),('O2',.2095),('CO2',.0004)]:book(incoming,s,n*x)
     boundaries.append({'row':i,**e})
    if typ=='gas_absorbed':book(incoming,e['species'],e['moles']);boundaries.append({'row':i,**e})
    if typ=='gas_evolved':book(outgoing,e['species'],e['moles']);boundaries.append({'row':i,**e})
    if typ=='evaporated':book(outgoing,'water',e['moles']);boundaries.append({'row':i,**e})
    if typ=='diluted':book(incoming,'water',e['moles']);boundaries.append({'row':i,**e})
    assert typ not in ['gas_vented'], 'Unimplemented boundary'
   pub={v['id']:v for v in r['bench']['vessels']};assert len(pub)==len(r['bench']['vessels'])
   if state.keys()-pub.keys():assert kind=='inspect' and not r['events'] and all(v==state[k] for k,v in pub.items())
   state.update(pub);retained=zero()
   for v in state.values():
    assert not v['surfaces'] and not v.get('unresolved_materials')
    for p in v['contents']:
     assert p['moles']>=0;book(retained,p['species'],p['moles'])
   residual={k:retained[k]+outgoing[k]-incoming[k] for k in keys}
   steps.append({'row':i,'operator':op,'residual':{k:v for k,v in residual.items() if v!=0},'maximum_absolute_residual':max(map(abs,residual.values()))})
  records.append({'id':c['id'],'script_sha256':c['script_sha256'],'stdout_sha256':sha(stream),'retained':retained,'incoming':incoming,'exported':outgoing,'final_residual':residual,'final_maximum_absolute_residual':steps[-1]['maximum_absolute_residual'],'max_per_step_residual':max(s['maximum_absolute_residual'] for s in steps),'steps':steps,'boundary_events':boundaries,'physical_verdict':'not_assigned'})
saved=load((HERE/'material-projection.json').read_bytes())
assert saved['formulas']==formulas and saved['charges']==charges
assert {c['id']:c for c in saved['records']}=={c['id']:c for c in records}
print(json.dumps({'records_verified':len(records), 'material_records_exactly_reproduced':True, 'new_engine_or_api_executions':0, 'physical_verdicts_assigned':0}))
