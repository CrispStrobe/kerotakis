"""Unchanged original MO003: requested reaction timing and complete stock ledgers."""
from pathlib import Path
from decimal import Decimal as D
import json,sys
root=Path(sys.argv[1]);props=Path(sys.argv[2]);data=json.loads(props.read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};forms['base_equivalents']=forms['OH-'];keys=['C','H','O','Na','N','charge'];bound=D('1e-10');expected={k:D(0) for k in keys};checks=[];timing=[];hydro=None;oxid=None;stable=True;warnings=[];fails=[]
rows=[json.loads(l) for l in (root/'batch2-mo-003.stdout').read_text().splitlines()]
def amount(v,s):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==s),D(0))
def add(sp,n):
 for k,c in forms[sp].items():expected[k]+=n*c
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':add(op['species'],D(str(op['moles'])))
 for e in r.get('events',[]):
  if e['event']=='vessel_sealed':
   n=D(str(e['trapped_air']))
   for s,f in [('N2','.7901'),('O2','.2095'),('CO2','.0004')]:add(s,n*D(f))
  if e['event']=='not_yet_modeled':warnings.append(e)
  if e['event']=='solver_failed':fails.append(e)
 if 'bench' not in r:continue
 assert len(r['bench']['vessels'])==1;v=r['bench']['vessels'][0];assert not v.get('unresolved_materials');actual={k:D(0) for k in keys}
 for p in v['contents']:
  n=D(str(p['moles']));assert n.is_finite() and n>=0
  for k,c in forms[p['species']].items():actual[k]+=n*c
 res={k:actual[k]-expected[k] for k in keys};checks.append({'step':i,'residual_mol_eq':{k:str(n) for k,n in res.items()}})
 ethanol=amount(v,'ethanol');ester=amount(v,'ethyl_acetate');acetate=sum((amount(v,s) for s in ['CH3COO-','CH3COOH','NaOAc']),D(0));timing.append({'step':i,'operator':op,'ethanol_mol':str(ethanol),'ester_mol':str(ester),'acetate_mol':str(acetate),'total_owned_carbon_mol':str(actual['C'])})
 if op['op']=='react' and op['reaction']=='saponification':hydro=ethanol
 elif op['op']=='react' and op['reaction']=='alcohol-oxidation':oxid=ethanol
 elif hydro is not None and oxid is None:stable &= abs(ethanol-hydro)<bound
c1=hydro is not None and oxid is not None and hydro>D('1e-12') and hydro-oxid>D('1e-12') and stable and not fails
c2=all(abs(D(n))<bound for c in checks for n in c['residual_mol_eq'].values()) and not fails
result={'criteria':{'MO-003-C1':bool(c1),'MO-003-C2':bool(c2)},'met':int(bool(c1))+int(bool(c2)),'total':2,'solver_failures':len(fails),'known_atom_charge_checks':checks,'reaction_timing':timing,'warning_keys':[e.get('reason',{}).get('key') for e in warnings],'scope':'Only the unchanged requested net conversions and owned stock closure, counting contents once and excluding mirrored solution.species; no empirical reaction rate or heat calibration.'}
print(json.dumps(result,indent=2))
if '--require-pass' in sys.argv and result['met']!=2:raise SystemExit(1)
