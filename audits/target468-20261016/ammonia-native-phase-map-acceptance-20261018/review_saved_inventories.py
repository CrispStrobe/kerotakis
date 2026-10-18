"""Independent formula ledger over saved native inventories; no engine import/call."""
import pathlib,json,decimal,hashlib
B=pathlib.Path(__file__).resolve().parent;D=lambda x:decimal.Decimal(str(x));decimal.getcontext().prec=40
raw=(B/'native.stderr').read_bytes();arms=[json.loads(l.split('AMMONIA_ARMS ',1)[1]) for l in raw.decode().splitlines() if l.startswith('AMMONIA_ARMS ')]
assert len(arms)==12
composition={'water':{'H':2,'O':1},'N2':{'N':2},'O2':{'O':2},'NH3':{'N':1,'H':3},'NH4+':{'N':1,'H':4,'Q':1},'H+':{'H':1,'Q':1},'base_equivalents':{'O':1,'H':1,'Q':-1},'Cl-':{'Cl':1,'Q':-1},'CO2':{'C':1,'O':2},'CO2(aq)':{'C':1,'O':2},'HCO3-':{'H':1,'C':1,'O':3,'Q':-1},'CO3-2':{'C':1,'O':3,'Q':-2}}
def ledger(v):
 out={}
 for p in v['contents']:
  for k,n in composition[p['species']].items():out[k]=out.get(k,D(0))+D(p['moles'])*n
 return out
def gas(v):return sum(D(p['moles']) for p in v['contents'] if p['species']=='NH3' and p['phase']=='gas')
metrics=[]
for a in arms:
 neutral=ledger(a['neutral']);ng=gas(a['neutral']);assert ng>0
 for name in ['acid_first','acid_after']:
  acid=ledger(a[name]);res={k:acid.get(k,D(0))-neutral.get(k,D(0))-(2*D(a['dose']) if k in ['H','Cl'] else 0) for k in set(acid)|set(neutral)}
  assert all(abs(n)<D('1e-8') for n in res.values()),res
  ratio=gas(a[name])/ng;assert ratio<D('1e-4')
  metrics.append({'volume_L':a['volume'],'dose_mol':a['dose'],'regulated':a['regulated'],'arm':name,'gas_ratio':str(ratio),'atom_charge_residual':{k:str(v) for k,v in res.items()}})
print(json.dumps({'source':json.loads((B/'frozen-design.json').read_text())['source'],'raw_native_stderr_sha256':hashlib.sha256(raw).hexdigest(),'preauthored_groups':12,'acid_arms':len(metrics),'max_gas_ratio':str(max(D(x['gas_ratio']) for x in metrics)),'max_atom_charge_residual':str(max(abs(D(v)) for x in metrics for v in x['atom_charge_residual'].values())),'new_engine_calls':0},indent=2))
