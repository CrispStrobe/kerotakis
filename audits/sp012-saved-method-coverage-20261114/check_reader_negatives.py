"""Saved-output mutations; no engine execution."""
from pathlib import Path
import json,copy,tempfile,subprocess,sys
root=Path(sys.argv[1]).resolve();rows=[json.loads(l) for l in (root/'new120-sp-012.stdout').read_text().splitlines()]
def assess(data):
 with tempfile.TemporaryDirectory(prefix='sp012-reader-') as directory:
  d=Path(directory)
  for name in ['binding.json','source-properties.json','source-excerpts.json']:(d/name).write_bytes((root/name).read_bytes())
  (d/'new120-sp-012.stdout').write_text(''.join(json.dumps(r)+'\n' for r in data));return subprocess.run([sys.executable,str(root/'review_original.py'),str(d)],capture_output=True,timeout=30).returncode
assert assess(rows)==0
report=[]
for name in ['lost_ethanol','lost_glucose','hidden_gap','invented_glucose_peak','missing_supported_peak','measurement_changed_clock','mass_fraction_areas','wrong_retention','wrong_outside_method','solver_failure']:
 data=copy.deepcopy(rows);r=data[5];e=next(e for e in r['events'] if e['event']=='chromatographed');v=r['bench']['vessels'][0]
 if name in ['lost_ethanol','lost_glucose']:
  sp='ethanol' if name=='lost_ethanol' else 'glucose';next(p for p in v['contents'] if p['species']==sp)['moles']*=.5
 elif name=='hidden_gap':r['events']=[e for e in r['events'] if e['event']!='not_yet_modeled']
 elif name=='invented_glucose_peak':p=copy.deepcopy(e['peaks'][0]);p['species']='glucose';e['peaks'].append(p)
 elif name=='missing_supported_peak':e['peaks'].pop()
 elif name=='measurement_changed_clock':v['elapsed_seconds']+=1
 elif name=='mass_fraction_areas':
  for p in e['peaks']:p['relative_area']=.5
 elif name=='wrong_retention':e['peaks'][0]['retention_time_s']+=1
 elif name=='wrong_outside_method':e['outside_method']=['glucose']
 elif name=='solver_failure':r['events'].append({'event':'solver_failed','detail':'negative control'})
 code=assess(data);assert code!=0,name;report.append({'mutation':name,'rejected':True,'exit':code})
print(json.dumps(report,indent=2))
