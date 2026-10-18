"""Independent saved-only high-precision energy/atom check; no engine calls."""
import pathlib,json,decimal,hashlib
B=pathlib.Path(__file__).resolve().parent;D=lambda x:decimal.Decimal(str(x));decimal.getcontext().prec=45
props=json.loads((B/'properties.json').read_text());design=json.loads((B/'design.json').read_text());raw=(B/'responses/new120-th-016.json').read_bytes();r=json.loads(raw);assert r['engine']==design['accepted_engine'] and r['script_sha256']==design['cases'][0]['script_sha256']
assert hashlib.sha256(design['cases'][0]['script'].encode()).hexdigest()==r['script_sha256']
rows=r['observations'];add=next(x for x in rows if x['operator']['op']=='add');last=next(x for x in reversed(rows) if x['operator']['op']=='inspect')['bench']['vessels'][0]
coef=list(map(D,props['Zn_solid_cp']['intervals'][0]['coefficients']));a0,a1,a2,a3,a4,a5,a6=coef
F=lambda x:-a0/x+a1*x.ln()+a2*x+a3*x*x/2+a4*x**3/3+a5*x**4/4+a6*x**5/5
n=D(add['operator']['moles']);tm=D(props['Zn_melting_k']);t0=D(add['bench']['vessels'][0]['temperature']);sensible=n*D(props['gas_constant_j_mol_k'])*(F(tm)-F(t0))
amount=lambda phase:sum(D(p['moles']) for p in last['contents'] if p['species']=='Zn' and p['phase']==phase)
solid=amount('solid');liquid=amount('liquid');residual=sensible+liquid*D(props['Zn_fusion_j_mol'])-D(design['cases'][0]['changed_source_expectations']['input_energy_J'])
assert abs(residual)<D('1e-8') and abs(liquid+solid-n)<D('1e-12') and solid>0 and liquid>0 and D(last['temperature'])==tm
for row in rows:
 for v in row['bench']['vessels']:
  assert all(p['species']=='Zn' for p in v['contents'])
  assert abs(sum(D(p['moles']) for p in v['contents'])-n)<D('1e-12')
assert not any(e['event']=='solver_failed' for row in rows for e in row['events'])
print(json.dumps({'source':r['engine'],'precision':45,'response_sha256':hashlib.sha256(raw).hexdigest(),'sensible_J':str(sensible),'energy_residual_J':str(residual),'zinc_residual_mol':str(solid+liquid-n),'original_criteria_C1_C2':'PASS within the declared paid-phase domain','new_engine_calls':0},indent=2))
