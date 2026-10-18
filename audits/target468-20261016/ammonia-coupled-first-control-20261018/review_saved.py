import pathlib,json,hashlib,math
B=pathlib.Path(__file__).resolve().parent
d=json.loads((B/'frozen-design-v2.json').read_text());e=json.loads((B/'execution.json').read_text())
for name,key in [('control.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
 assert hashlib.sha256((B/name).read_bytes()).hexdigest()==(d if key=='script_sha256' else e)[key]
assert e['exit_status']==0
rows=[json.loads(l) for l in (B/'stdout.jsonl').read_text().splitlines()]
assert not any(x.get('event')=='solver_failed' for row in rows for x in row.get('events',[]))
latest={v['id']:v for row in rows for v in row['bench']['vessels']};assert set(latest)=={0,1,2}
final=[latest[i] for i in [0,1,2]];arms=[]
for v in final:
 amount=lambda key,phase:sum(p['moles'] for p in v['contents'] if p['species']==key and p['phase']==phase)
 baseline=next(x for row in rows for x in row['bench']['vessels'] if x['id']==v['id'] and x['headspace']['boundary']=='sealed')
 nitrogen=lambda x:sum(p['moles']*(2 if p['species']=='N2' else 1) for p in x['contents'] if p['species'] in ['N2','NH3','NH4+'])
 residual=nitrogen(v)-nitrogen(baseline)-0.0008;assert abs(residual)<1e-8
 chloride=amount('Cl-','aqueous');assert abs(chloride-(0.0016 if v['id'] else 0))<1e-8
 # Core Henry coefficient already declared by source, not fitted here.
 water=amount('water','liquid');vl=water*18.015/1000/0.997
 h=57*math.exp(4200*(1/v['temperature']-1/298.15))
 gas=amount('NH3','gas');aq=amount('NH3','aqueous')
 implied_gas=aq/(h*vl)*v['headspace']['volume']/(0.082057366*v['temperature'])
 arms.append({'id':v['id'],'temperature_K':v['temperature'],'pH':v['solution']['ph'],'NH3_gas_mol':gas,'NH3_aqueous_mol':aq,'NH4_aqueous_mol':amount('NH4+','aqueous'),'chloride_mol':chloride,'nitrogen_residual_mol':residual,'approx_core_Henry_gas_from_final_free_NH3_mol':implied_gas,'approx_partition_residual_ratio':gas/implied_gas})
assert len(arms)==3
assert all(a['NH3_aqueous_mol']<arms[0]['NH3_aqueous_mol']*1e-5 and a['NH3_gas_mol']>arms[0]['NH3_gas_mol']*0.9 for a in arms[1:])
result={'source':d['source'],'accepted_equivalent_main':d['accepted_equivalent_main'],'whole_tree_equality_claimed':False,'verdict':'FAIL: uncoupled finite ammonia partition/protonation','arms':arms,'qualification':'Saved-output assessment only. Henry diagnostic uses solvent-only approximate volume to show an orders-of-magnitude mismatch, not a fitted acceptance tolerance. Original pre-issue qualitative consistency forecast fails independently: acid suppresses free aqueous NH3 while over 90% of neutral headspace NH3 persists. No catalog selection or empirical calibration.'}
(B/'assessment.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
