"""Adversarial saved-data checks, with no engine execution."""
from pathlib import Path
import copy,json,tempfile,subprocess,sys
root=Path(sys.argv[1]).resolve();rows=[json.loads(l) for l in (root/'batch2-ki-003.stdout').read_text().splitlines()]
def assess(data):
 with tempfile.TemporaryDirectory(prefix='ki003-reader-') as directory:
  dst=Path(directory)
  for name in ['binding.json','source-properties.json','source-excerpts.json']:(dst/name).write_bytes((root/name).read_bytes())
  (dst/'batch2-ki-003.stdout').write_text(''.join(json.dumps(r)+'\n' for r in data))
  result=subprocess.run([sys.executable,str(root/'review_original.py'),str(dst)],capture_output=True,text=True,timeout=30);return result.returncode
assert assess(rows)==0
results=[]
for name in ['water_loss','peroxide_loss','catalyst_loss','oxygen_loss','default_diameter','uncoupled_grind','missing_active_settling','wrong_clock','solver_failure','balanced_wrong_kinetics']:
 data=copy.deepcopy(rows);r=data[9];v=r['bench']['vessels'][0]
 if name in ['water_loss','peroxide_loss','catalyst_loss','oxygen_loss']:
  species={'water_loss':'water','peroxide_loss':'H2O2','catalyst_loss':'MnO2','oxygen_loss':'O2'}[name];next(p for p in v['contents'] if p['species']==species)['moles']*=.5
 elif name=='default_diameter':next(l for l in v['lots'] if l['species']=='MnO2')['particle_size_um']=100
 elif name=='uncoupled_grind':next(e for e in data[3]['events'] if e['event']=='ground')['rate_coupled']=False
 elif name=='missing_active_settling':r['events']=[e for e in r['events'] if not(e['event']=='gravity_settled' and e['vessel']==0)]
 elif name=='wrong_clock':v['elapsed_seconds']=20
 elif name=='solver_failure':r['events'].append({'event':'solver_failed','detail':'reader negative control'})
 elif name=='balanced_wrong_kinetics':
  for p in v['contents']:
   p['moles']+= {'H2O2':-5e-6,'water':5e-6,'O2':2.5e-6}.get(p['species'],0)
  for e in r['events']:
   if e['event'] in ['reacted','gas_produced'] and e['vessel']==0:e['moles']+=2.5e-6
 code=assess(data);assert code!=0,name;results.append({'mutation':name,'rejected':True,'exit':code})
print(json.dumps(results,indent=2))
