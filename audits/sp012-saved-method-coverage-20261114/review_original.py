"""Original supported-peak/explicit-gap comparison and non-consuming measurement."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,math,hashlib
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'new120-sp-012.stdout').read_text().splitlines()];props={p['species_id']:p for p in json.loads((root/'source-properties.json').read_text())};binding=json.loads((root/'binding.json').read_text());h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert h(root/'source-properties.json')==binding['source_properties_sha256'];assert h(root/'source-excerpts.json')==binding['source_excerpts_sha256'];source=json.loads((root/'source-excerpts.json').read_text());code={k:''.join(x['text'] for x in v['excerpts']) for k,v in source.items()};assert 'p.relative_area /= largest' in code['bench'] and 'relative_area: moles' in code['bench'] and 'unparameterised.insert' in code['bench'];assert 'phase_ratio: 0.5' in code['column'] and 'void_time_s: 60.0' in code['column'] and 'plates: 10_000' in code['column']
keys=['C','H','O','charge'];expected={k:D(0) for k in keys};checks=[];bound=D('1e-10')
def add(total,sp,n):
 for k,c in props[sp]['atoms'].items():total[k]+=n*D(c)
 total['charge']+=n*D(props[sp]['charge'])
for i,r in enumerate(rows):
 op=r['operator'];assert len(r['bench']['vessels'])==1;v=r['bench']['vessels'][0]
 if op['op']=='add':
  assert op['vessel']==0;add(expected,op['species'],D(str(op['moles'])))
  if op['species']!='water':assert op['moles']==.001
 assert not any(v.get(k) for k in ['unresolved_materials','material_objects','surfaces','adsorbed','exchanges','electrodes','solid_solutions']);assert v['pending_co2_transfer_mol']==0
 actual={k:D(0) for k in keys}
 for p in v['contents']:
  n=D(str(p['moles']));assert n.is_finite() and n>=0;add(actual,p['species'],n)
 res={k:actual[k]-expected[k] for k in keys};assert all(abs(n)<bound for n in res.values());checks.append({'step':i,'residual_mol_eq':{k:str(n) for k,n in res.items()}})
 for e in r.get('events',[]):assert e['event'] not in ['solver_failed','gas_evolved','gas_absorbed','gas_vented','reacted']
measure=[i for i,r in enumerate(rows) if r['operator']['op']=='measure'];assert len(measure)==1;i=measure[0];r=rows[i];assert r['operator']['instrument']=='chromatograph';assert rows[i-1]['operator']['op']==rows[i+1]['operator']['op']=='inspect';assert rows[i-1]['bench']==r['bench']==rows[i+1]['bench'];v=r['bench']['vessels'][0];stock={p['species']:p for p in v['contents']};assert set(stock)=={'water','ethanol','ethyl_acetate','glucose'};assert stock['glucose']['phase']=='aqueous' and stock['glucose']['moles']==.001
notices=[e for row in rows for e in row.get('events',[]) if e['event']=='not_yet_modeled'];assert len(notices)==1;notice=notices[0];assert notice['reason']['key']=='not-modeled.no-group-decomposition';names=notice['reason']['slots'][0][1]['list']['items'];assert names==[{'text':{'text':'glucose'}}]
events=[e for e in r['events'] if e['event']=='chromatographed'];assert len(events)==1;e=events[0];assert e['outside_method']==[] and e['void_time_s']==60 and e['plates']==10000;peaks=e['peaks'];assert [p['species'] for p in peaks]==['ethanol','ethyl_acetate'];assert peaks[0]['retention_time_s']<peaks[1]['retention_time_s'];numeric=[]
for p in peaks:
 assert p['partition_k']>0 and math.isfinite(p['partition_k']);tr=60*(1+.5*p['partition_k']);width=4*tr/math.sqrt(10000);rf=p['partition_k']/(1+p['partition_k']);assert abs(tr-p['retention_time_s'])<1e-10 and abs(width-p['width_s'])<1e-10 and abs(rf-p['rf'])<1e-10;assert p['relative_area']==stock[p['species']]['moles']/max(stock[q['species']]['moles'] for q in peaks)==1;numeric.append({'species':p['species'],'retention_residual_s':tr-p['retention_time_s'],'width_residual_s':width-p['width_s'],'rf_residual':rf-p['rf']})
masses={sp:D(str(stock[sp]['moles']))*D(str(props[sp]['molar_mass'])) for sp in ['ethanol','ethyl_acetate','glucose']};assert masses['ethanol']!=masses['ethyl_acetate'];assert sum(p['relative_area'] for p in peaks)==2
print(json.dumps({'met':2,'total':2,'criteria':{'SP-012-C1':True,'SP-012-C2':True},'solver_failures':0,'known_atom_charge_checks':checks,'whole_bench_before_measure_after_exact':True,'dissolved_glucose_mol':stock['glucose']['moles'],'explicit_model_gap':notice['reason']['key'],'unsupported_species':['glucose'],'outside_method':e['outside_method'],'supported_peaks':peaks,'column_algebra_checks':numeric,'relative_area_sum':2,'known_solute_masses_g':{k:str(n) for k,n in masses.items()},'scope':'Original supported-peak identification, explicit missing-parameter distinction and non-consuming measurement comparison. No glucose retention/peak, complete sample coverage, empirical retention/detector response or calibrated mass-fraction prediction is qualified. Areas are scaled to the largest supported molar amount, not to total sample mass. Contents are counted once; lots and mirrored solution species are not extra stock. Historical executable checksum remains absent.'},indent=2))
