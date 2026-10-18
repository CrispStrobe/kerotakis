"""Independent saved separation ownership and observable projection."""
import argparse,copy,hashlib,json,math,subprocess,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83';sha=lambda b:hashlib.sha256(b).hexdigest();p=argparse.ArgumentParser();p.add_argument('--record',action='store_true');a=p.parse_args();projection=json.loads((HERE.parent/'saved64-first-screen/material-projection.json').read_text());screen={c['id']:c for c in json.loads((HERE.parent/'saved64-first-screen/first-screen.json').read_text())['cases']};registry_raw=subprocess.check_output(['git','show',SOURCE+':data/registry/registry-source-v1.json'],cwd=ROOT);registry=json.loads(registry_raw);mm={r['species_id']:r['quantity']['value'] for r in registry['phase_thermodynamics'] if r['property']=='molar_mass'};formulas=projection['formulas'];charges=projection['charges'];archive=ROOT/'audits/target468-20261016/common-engine-completion/results/raw-artifact.zip';assert sha(archive.read_bytes())==projection['raw_artifact_sha256']
def metric(v):
 n={};phases={};atoms={};charge=0.;mass=0.;missing=[]
 for q in v['contents']:
  s=q['species'];x=q['moles'];assert s in formulas and math.isfinite(x) and x>=0;n[s]=n.get(s,0)+x;phases.setdefault(q['phase'],{});phases[q['phase']][s]=phases[q['phase']].get(s,0)+x
  for e,k in formulas[s].items():atoms[e]=atoms.get(e,0)+k*x
  charge+=charges.get(s,0)*x
  if s in mm:mass+=mm[s]*x
  elif x>0:missing.append(s)
 total=n.get('water',0)+n.get('ethanol',0)
 return {'vessel':v['id'],'temperature_K':v['temperature'],'owned_species_mol':n,'owned_phase_species_mol':phases,'represented_atoms_mol':atoms,'represented_formal_charge_eq':charge,'known_property_mass_g':mass,'species_missing_mass_properties':sorted(set(missing)),'ethanol_mole_fraction_of_water_and_ethanol':n.get('ethanol',0)/total if total else None,'pure_AgCl_mass_g':n.get('AgCl',0)*mm['AgCl'],'headspace':v['headspace']}
records=[]
with zipfile.ZipFile(archive) as z:
 for i in range(1,13):
  cid=f'new120:sp-{i:03}';c=screen[cid];prefix=cid.replace(':','_');assert sha(z.read(prefix+'/experiment.lab'))==c['original_script_sha256'];raw=z.read(prefix+'/stdout.jsonl');assert sha(raw)==c['saved_execution_receipt']['stdout_sha256'];rows=[json.loads(l) for l in raw.splitlines()];state={};timeline=[]
  for index,row in enumerate(rows):
   op=row['operator'];before=[metric(v) for v in state.values()];state.update({v['id']:v for v in row['bench']['vessels']})
   if op['op'] not in ['inspect','distil','decant','filter','magnet','drain','extract','evaporate','measure']:continue
   timeline.append({'row':index,'operator':op,'before':before,'after':[metric(v) for v in state.values()],'events':row['events']})
  records.append({'id':cid,'script_sha256':c['original_script_sha256'],'stdout_sha256':sha(raw),'original_criteria':c['original_criteria'],'timeline':timeline,'formal_selection':'not_assigned'})
report={'source':SOURCE,'registry_sha256':sha(registry_raw),'archive_sha256':projection['raw_artifact_sha256'],'cases':records,'new_engine_or_api_executions':0,'catalog_additions':0,'qualification':'Whole owned-vessel state retained across partial inspect rows; solution/lot diagnostics are not extra matter. Atom/formal-charge projection is separate from chromatographic peaks or conductivity. Mass marked known-property only where required values exist; unknown mass inputs are explicit. This does not independently validate distillation enthalpy or energy-budget semantics, partition coefficients or empirical retention times. Held cases are not accepted for merely conserved matter.'};out=HERE/'metrics.json'
if a.record:
 assert not out.exists();out.write_text(json.dumps(report,indent=2,allow_nan=False)+'\n')
else:assert json.loads(out.read_text())==report,'Separation projection changed'
print(json.dumps({'separation_cases':len(records),'observed_operations':sum(len(c['timeline']) for c in records),'new_executions':0,'new_selections':0}))
