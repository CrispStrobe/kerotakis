"""Assess unchanged AN006/MO011 from saved output, with independent atoms."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,hashlib
root=Path(sys.argv[1]);bound=D('1e-10')
forms={'water':{'H':2,'O':1},'CH3COOH':{'C':2,'H':4,'O':2},'CH3COO-':{'C':2,'H':3,'O':2,'charge':-1},'ethanol':{'C':2,'H':6,'O':1},'ethyl_acetate':{'C':4,'H':8,'O':2},'NaOH':{'Na':1,'H':1,'O':1},'Na+':{'Na':1,'charge':1},'base_equivalents':{'H':1,'O':1,'charge':-1},'OH-':{'H':1,'O':1,'charge':-1},'H+':{'H':1,'charge':1}}
keys=['C','H','O','Na','charge']
def amount(v,key):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==key),D(0))
reports=[]
for oid in (sys.argv[2:] or ['batch3-diverse:an-006','batch2:mo-011']):
 case=oid.replace(':','-');rows=[json.loads(x) for x in (root/(case+'.stdout')).read_text().splitlines()];all_events=[e for r in rows for e in r.get('events',[])];expected={};residuals=[];previous=None;chrom=[];inventory_ok=True
 for i,row in enumerate(rows):
  op=row['operator'];vid=op.get('vessel')
  if op['op']=='add':
   target=expected.setdefault(vid,{k:D(0) for k in keys})
   for k,c in forms[op['species']].items():target[k]+=D(str(op['moles']))*c
  if 'bench' not in row:continue
  for v in row['bench']['vessels']:
   target=expected.setdefault(v['id'],{k:D(0) for k in keys});actual={k:D(0) for k in keys}
   for portion in v['contents']:
    n=D(str(portion['moles']));assert n.is_finite() and n>=0
    for k,c in forms[portion['species']].items():actual[k]+=n*c
   delta={k:actual[k]-target[k] for k in keys};inventory_ok &= all(abs(x)<=bound for x in delta.values());residuals.append({'step':i,'vessel':v['id'],'residual':{k:str(x) for k,x in delta.items()}})
  for event in row['events']:
   if event['event']=='chromatographed':
    v=next(v for v in row['bench']['vessels'] if v['id']==event['vessel']);old=next(prior for prior in previous['bench']['vessels'] if prior['id']==v['id']);inventory_ok &= v['contents']==old['contents'];chrom.append((v,event))
  previous=row
 inventory_ok &= not any(e['event'] in ['solver_failed','gas_evolved','gas_absorbed'] for e in all_events)
 warnings=[e for e in all_events if e['event']=='not_yet_modeled' and e.get('cause')=='not-parameterised']
 assert inventory_ok,(oid,residuals)
 if oid.endswith('an-006'):
  assert len(chrom)==2
  (acid,a),(salt,b)=chrom
  totals=[amount(v,'CH3COOH')+amount(v,'CH3COO-') for v in [acid,salt]]
  question=all(abs(n-D('0.001'))<bound for n in totals) and amount(salt,'CH3COOH')<amount(acid,'CH3COOH')/100 and amount(salt,'CH3COO-')>amount(acid,'CH3COO-')
  control=acid['solution']['ph']<salt['solution']['ph'] and all(amount(v,'ethanol')==D('0.001') for v in [acid,salt])
  coverage=(not warnings and all('CH3COO-' in e['outside_method'] for _,e in chrom) and all(any(p['species']=='CH3COOH' for p in e['peaks']) for _,e in chrom))
  criteria={'question':bool(question),'inventory':bool(inventory_ok),'coverage':bool(coverage),'controls':bool(control)}
  observed={'acid_neutral_mol':str(amount(acid,'CH3COOH')),'treated_neutral_mol':str(amount(salt,'CH3COOH')),'acid_ionic_mol':str(amount(acid,'CH3COO-')),'treated_ionic_mol':str(amount(salt,'CH3COO-')),'ph':[acid['solution']['ph'],salt['solution']['ph']]}
 else:
  assert len(chrom)==3
  vs=[v for v,_ in chrom];ester=[amount(v,'ethyl_acetate') for v in vs];ethanol=[amount(v,'ethanol') for v in vs];acetate=[amount(v,'CH3COOH')+amount(v,'CH3COO-') for v in vs]
  identity=ester[0]==D('0.001') and abs(ester[1]-D('0.0005'))<bound and ester[2]<bound and ethanol[0]<bound and abs(ethanol[1]-D('0.0005'))<bound and abs(ethanol[2]-D('0.001'))<bound
  identity &= all(abs((D('0.001')-est)-ac)<bound for est,ac in zip(ester,acetate))
  identity &= not any(p['species']=='ethanol' for p in chrom[0][1]['peaks']) and all(any(p['species']=='ethanol' for p in e['peaks']) for _,e in chrom[1:]) and not any(p['species']=='ethyl_acetate' for p in chrom[2][1]['peaks'])
  boundary=all('CH3COO-' in e['outside_method'] for _,e in chrom[1:])
  criteria={'MO-011-C1':bool(identity and inventory_ok),'MO-011-C2':bool(boundary)}
  observed={'ethyl_acetate_mol':list(map(str,ester)),'ethanol_mol':list(map(str,ethanol)),'acetic_acid_family_mol':list(map(str,acetate)),'area_rule':'No area-to-moles inference: only independently read inventories set stoichiometry. Per-run normalised areas are not compared across runs.'}
 reports.append({'id':oid,'criteria':criteria,'met':sum(criteria.values()),'total':len(criteria),'inventory_all_checkpoints_closed':bool(inventory_ok),'element_charge_bound_mol_eq':str(bound),'residuals':residuals,'unparameterised_warnings':[e.get('what') for e in warnings],'observed':observed,'solver_failures':sum(e['event']=='solver_failed' for e in all_events)})
print(json.dumps(reports,indent=2))
