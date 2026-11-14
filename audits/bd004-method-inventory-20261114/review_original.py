"""Saved original method-scope and independent physical inventory review."""
from pathlib import Path
from decimal import Decimal as D
import json,hashlib,sys,math
out=Path(sys.argv[1]);repo=Path(sys.argv[2]);bound=D('1e-10')
assert hashlib.sha256((out/'original.lab').read_bytes()).hexdigest()=='3bf8c92ef01ca69a3e4a5e9c41202726d369e68d0324f5f316fcb374b346b173'
rows=[json.loads(x) for x in (out/'observations.jsonl').read_text().splitlines()]
events=[e for r in rows for e in r['events']]
assert not any(e['event']=='solver_failed' for e in events)
assert not any(e['event'] in ['gas_absorbed','gas_evolved','transferred'] for e in events)
forms={'water':{'H':2,'O':1},'ethanol':{'C':2,'H':6,'O':1},'NaCl':{'Na':1,'Cl':1},'Na+':{'Na':1,'charge':1},'Cl-':{'Cl':1,'charge':-1}}
keys=['C','H','O','Na','Cl','charge'];expected={k:D(0) for k in keys};checks=[];opaque=D(0)
def add(amount,species):
 for k,c in forms[species].items():expected[k]+=amount*c
for i,r in enumerate(rows):
 if 'bench' not in r:continue
 op=r['operator']
 if op['op']=='add':add(D(str(op['moles'])),op['species'])
 if op['op']=='add_material':
  e=next(e for e in r['events'] if e['event']=='material_added');assert e['recipe_id']=='food/albumin' and e['recipe_version']==1
  assert D(str(e['total_amount']))==D('0.2') and D(str(e['unresolved_amount']))==D('0.188')
  assert e['components'][0]['species']=='water' and D(str(e['components'][0]['basis_amount']))==D('0.012')
  assert abs(D(str(e['components'][0]['moles']))*D('18.015')-D('0.012'))<bound
  add(D(str(e['components'][0]['moles'])),'water');opaque+=D('0.188')
 v=r['bench']['vessels'][0];actual={k:D(0) for k in keys}
 for part in v['contents']:
  n=D(str(part['moles']));assert n.is_finite() and n>=0
  for k,c in forms[part['species']].items():actual[k]+=n*c
 residual={k:actual[k]-expected[k] for k in keys};assert all(abs(x)<bound for x in residual.values()),(i,residual)
 assert sum((D(str(s['amount'])) for s in v.get('unresolved_materials',[])),D(0))==opaque
 checks.append({'step':i,'residual_mol_eq':{k:str(x) for k,x in residual.items()}})
# The observation instruments must preserve every physical stock entry.
for r in rows[4:]:
 if 'bench' in r:
  assert r['bench']['vessels'][0]['contents']==rows[3]['bench']['vessels'][0]['contents']
  assert r['bench']['vessels'][0]['unresolved_materials']==rows[3]['bench']['vessels'][0]['unresolved_materials']
chrom=next(e for e in events if e['event']=='chromatographed')
assert len(chrom['peaks'])==1 and chrom['peaks'][0]['species']=='ethanol'
assert set(chrom['outside_method'])=={'Na+','Cl-'}
peak=chrom['peaks'][0];assert all(math.isfinite(peak[k]) and peak[k]>0 for k in ['partition_k','relative_area','retention_time_s','width_s'])
final=rows[-1]['bench']['vessels'][0]
amounts={p['species']:D(str(p['moles'])) for p in final['contents']}
assert amounts['Na+']==amounts['Cl-']==amounts['ethanol']==D('0.001')
registry=repo/'crates/kerotakis-core/tests/golden/registry.json';mass={r['key']:D(str(r['molar_mass'])) for r in json.loads(registry.read_text())}
retained=sum((n*mass[k] for k,n in amounts.items()),D(0))+opaque
balance=next(e for e in events if e['event']=='measured' and e['instrument']=='balance');assert balance['unit']=='g' and abs(D(str(balance['value']))-retained)<bound
conductivity=next(e for e in events if e['event']=='measured' and e['instrument']=='conductivity_meter');assert conductivity['value']>0
report={'original_criteria_passed':['question','inventory','coverage'],'solver_failures':0,'known_element_charge_checks':checks,'retained_opaque_albumin_g':str(opaque),'retained_ethanol_sodium_chloride_mol':{k:str(amounts[k]) for k in ['ethanol','Na+','Cl-']},'peak_species':[p['species'] for p in chrom['peaks']],'outside_method':chrom['outside_method'],'balance_g':balance['value'],'independent_retained_mass_g':str(retained),'conductivity_uS_cm':conductivity['value'],'registry_sha256':hashlib.sha256(registry.read_bytes()).hexdigest(),'scope':'Original instrument-completeness comparison only: the neutral peak does not identify salt or unresolved protein; no protein formula, complete analysis or empirical retention calibration is asserted.'}
print(json.dumps(report,indent=2))
