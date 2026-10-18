"""Reproduce actual named-reaction snapshots; no engine/API execution."""
import argparse,copy,hashlib,json,math,subprocess,zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83';sha=lambda b:hashlib.sha256(b).hexdigest();p=argparse.ArgumentParser();p.add_argument('--record',action='store_true');a=p.parse_args();projection=json.loads((HERE.parent/'saved64-first-screen/material-projection.json').read_text());screen={c['id']:c for c in json.loads((HERE.parent/'saved64-first-screen/first-screen.json').read_text())['cases']};source=subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-core/src/curated.rs'],cwd=ROOT);assert b'equilibrium_log_k: Some(0.6020599913279624)' in source;archive=ROOT/'audits/target468-20261016/common-engine-completion/results/raw-artifact.zip';assert sha(archive.read_bytes())==projection['raw_artifact_sha256'];formulas=projection['formulas']
def metric(v):
 n={};carbon=0.
 for q in v['contents']:
  assert q['species'] in formulas and math.isfinite(q['moles']) and q['moles']>=0;n[q['species']]=n.get(q['species'],0)+q['moles'];carbon+=q['moles']*formulas[q['species']].get('C',0)
 A,E,P,W=[n.get(k,0) for k in ['CH3COOH','ethanol','ethyl_acetate','water']];den=A*E;inorganic=sum(n.get(k,0) for k in ['CO2','CO2(aq)','HCO3-','CO3-2'])
 return {'vessel':v['id'],'elapsed_seconds':v['elapsed_seconds'],'temperature_K':v['temperature'],'owned_species_mol':n,'owned_all_carbon_mol':carbon,'owned_inorganic_carbon_mol':inorganic,'ideal_mole_quotient_ester_water_over_acid_alcohol':P*W/den if den>0 else None,'reported_solution':v.get('solution'),'headspace':v['headspace']}
records=[]
with zipfile.ZipFile(archive) as z:
 for n in range(1,13):
  cid=f'new120:or-{n:03}';c=screen[cid];prefix=cid.replace(':','_');assert sha(z.read(prefix+'/experiment.lab'))==c['original_script_sha256'];raw=z.read(prefix+'/stdout.jsonl');assert sha(raw)==c['saved_execution_receipt']['stdout_sha256'];rows=[json.loads(l) for l in raw.splitlines()];state={};snapshots=[];inspections=[]
  for i,row in enumerate(rows):
   op=row['operator'];before=copy.deepcopy(state.get(op.get('vessel'))) if op['op']=='react' else None;state.update({v['id']:v for v in row['bench']['vessels']})
   if before is not None:
    pre,post=metric(before),metric(state[op['vessel']]);b=pre['owned_species_mol'];after=post['owned_species_mol'];snapshots.append({'row':i,'operator':op,'before':pre,'after':post,'owned_species_delta_mol':{s:after.get(s,0)-b.get(s,0) for s in sorted(set(b)|set(after))},'owned_carbon_delta_mol':post['owned_all_carbon_mol']-pre['owned_all_carbon_mol'],'inorganic_carbon_delta_mol':post['owned_inorganic_carbon_mol']-pre['owned_inorganic_carbon_mol'],'oxygen_before_divided_by_six_mol':b.get('O2',0)/6 if op['reaction']=='respiration' else None})
   if op['op']=='inspect':inspections.append({'row':i,'vessels':[metric(v) for v in row['bench']['vessels']]})
  records.append({'id':cid,'script_sha256':c['original_script_sha256'],'stdout_sha256':sha(raw),'original_criteria':c['original_criteria'],'reactions':snapshots,'inspections':inspections,'explicit_limits':[{'row':i,**e} for i,r in enumerate(rows) for e in r['events'] if e['event'] in ['solver_failed','not_yet_modeled']],'formal_selection':'not_assigned'})
report={'source':SOURCE,'curated_source_sha256':sha(source),'archive_sha256':projection['raw_artifact_sha256'],'source_ideal_ester_K':10**0.6020599913279624,'cases':records,'new_engine_or_api_executions':0,'catalog_additions':0,'qualification':'Actual owned reactant/product and carbon accounting for original criteria. K is the explicit ideal teaching model, not measured calibration. Named oxidation/respiration is a learner request, not spontaneous chemistry; no rate, ATP, cell mechanism or curated reaction heat is inferred. Temperature can still respond to a separate aqueous/physical tail. Saponification prerequisite gaps remain held.'}
out=HERE/'metrics.json'
if a.record:
 assert not out.exists();out.write_text(json.dumps(report,indent=2,allow_nan=False)+'\n')
else:assert json.loads(out.read_text())==report,'Saved organic projection changed'
print(json.dumps({'organic_cases':len(records),'named_reaction_snapshots':sum(len(c['reactions']) for c in records),'new_executions':0,'new_selections':0}))
