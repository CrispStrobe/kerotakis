"""Adversarial saved-output checks, with no engine replay."""
from pathlib import Path
import json,copy,tempfile,subprocess,sys
root=Path(sys.argv[1]).resolve();rows=[json.loads(l) for l in (root/'new120-sp-004.stdout').read_text().splitlines()]
def assess(data,alter_template=False):
 with tempfile.TemporaryDirectory(prefix='sp004-reader-') as directory:
  p=Path(directory)
  for name in ['binding.json','source-properties.json','source-excerpts.json','original.lab']:(p/name).write_bytes((root/name).read_bytes())
  if alter_template:(p/'source-excerpts.json').write_text((p/'source-excerpts.json').read_text().replace('outside the vessel ledger','inside the vessel ledger'))
  (p/'new120-sp-004.stdout').write_text(''.join(json.dumps(r)+'\n' for r in data));return subprocess.run([sys.executable,str(root/'review_original.py'),str(p)],capture_output=True,timeout=30).returncode
assert assess(rows)==0
report=[]
for name in ['receiver_water_loss','source_ethanol_loss','duplicate_components','wrong_legacy_mirror','wrong_energy','invented_full_heat_model','receiver_at_boil_temperature','missing_state_scene','solver_failure','changed_energy_disclosure']:
 data=copy.deepcopy(rows);r=data[7];e=next(e for e in r['events'] if e['event']=='distilled')
 if name=='receiver_water_loss':next(p for p in r['bench']['vessels'][2]['contents'] if p['species']=='water')['moles']*=.5
 elif name=='source_ethanol_loss':next(p for p in r['bench']['vessels'][0]['contents'] if p['species']=='ethanol')['moles']*=.5
 elif name=='duplicate_components':e['components'].append(copy.deepcopy(e['components'][0]))
 elif name=='wrong_legacy_mirror':e['water']*=2
 elif name=='wrong_energy':e['energy_kj']=20
 elif name=='invented_full_heat_model':e['model']='All supplied apparatus heat is known and equals the latent budget'
 elif name=='receiver_at_boil_temperature':r['bench']['vessels'][2]['temperature']=e['at']
 elif name=='missing_state_scene':r.pop('scene')
 elif name=='solver_failure':r['events'].append({'event':'solver_failed','detail':'negative control'})
 code=assess(data,alter_template=name=='changed_energy_disclosure');assert code!=0,name;report.append({'mutation':name,'rejected':True,'exit':code})
print(json.dumps(report,indent=2))
