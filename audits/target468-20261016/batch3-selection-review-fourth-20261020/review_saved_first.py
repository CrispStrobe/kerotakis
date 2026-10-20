#!/usr/bin/env python3
"""Eight disjoint historical saved cases. No engine, network, catalog or source edits."""
import collections,hashlib,inspect,json,os,pathlib,runpy,zipfile
OUT=pathlib.Path(__file__).resolve().parent;ROOT=OUT.parents[2]
PRIOR=ROOT/'audits/target468-20261016/batch3-selection-review-third-20261020/review_saved.py'
a=runpy.run_path(str(PRIOR));sha=a['sha'];source=a['source'];REG=a['REG'];RECIPES=a['RECIPES'];MASSES=a['MASSES'];close=a['close'];dump=a['dump'];SOURCE=a['SOURCE'];BASE=a['BASE'];COMPOSITION=a['COMPOSITION'];CHARGE=a['CHARGE'];R=a['R'];vecadd=a['vecadd'];atoms=a['atoms'];events=a['events'];measures=a['measures']
SELECT={'en-006','en-008','po-007','co-004','fo-004','su-002','su-003','su-005'}
# Each of these events has an explicit ownership role reviewed below. No gas
# export/inlet event is silently admitted; only operator-owned adds/sealed air.
ALLOWED={'added','material_added','vessel_created','solution_routed','solution_characterized','observed','measured','neutralised','dissolved','temperature_changed','energy_transferred','enzyme_hydrolysed','stirred','material_layers_formed','vessel_sealed','precipitated','headspace_equilibrated','gas_contained','hazard_warning','not_yet_modeled','gravity_settled','reaction_occurred','inert','curdling_changed','corroded','reacted'}
exec(inspect.getsource(a['ledger']),globals())
def file_digest(path):
 h=hashlib.sha256()
 with path.open('rb') as stream:
  while chunk:=stream.read(262144):h.update(chunk)
 return h.hexdigest()
def main():
 memory=int(next(line.split()[1] for line in pathlib.Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemAvailable:')))//1024
 assert memory>=3200 and os.getloadavg()[0]<=3.5,'Saved archive review resource gate closed'
 archive=BASE/'native-screen-results/raw-artifact.zip';verification=json.loads((BASE/'native-screen-results/verification.json').read_text());assert file_digest(archive)==verification['artifact_sha256']
 binding=json.loads((BASE/'native-screen-results/binding.json').read_text());intake=json.loads((BASE/'intake.json').read_text());index=[]
 with zipfile.ZipFile(archive) as z:
  for k in sorted(SELECT):
   oid='batch3-diverse:'+k;original=next(x for x in intake if x['original_id']==oid);prefix=oid.replace(':','_');summary=json.loads((BASE/'native-screen-results/cases'/f'{prefix}.json').read_text());execution=json.loads(z.read(prefix+'/execution.json'));assert summary['engine']==binding['engine'];hashes={}
   for name,key in [('experiment.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
    raw=z.read(prefix+'/'+name);digest=sha(raw);assert digest==execution[key]==summary['execution'][key];hashes[name]=digest
   assert z.read(prefix+'/experiment.lab')==(BASE/original['original_script']).read_bytes();assert hashes['experiment.lab']==original['script_sha256'];assert execution['exit_code']==0 and not summary['solver_failures']
   raw_rows=[json.loads(line) for line in z.read(prefix+'/stdout.jsonl').splitlines()];rows=[dict(row,raw_jsonl_line=i+1) for i,row in enumerate(raw_rows) if 'bench'in row];scenes=[{'raw_jsonl_line':i+1,'scene':row['scene']} for i,row in enumerate(raw_rows) if 'scene'in row]
   inventory=ledger(k,rows)
   report={'original_id':oid,'title':original['title'],'original_forecast':original['original_expected_result'],'original_limitations':original['original_limitations'],'original_script_sha256':original['script_sha256'],'raw_member_prefix':prefix,'raw_sha256':hashes,'engine':binding['engine'],'lock_qualification':binding['lock_qualification'],'status':'saved_inventory_reviewed_scientific_criteria_pending','no_catalog_selection':True,'original_criteria':original['original_acceptance_checks'],'inventory':inventory,'actual_events':[{'raw_jsonl_line':row['raw_jsonl_line'],'operator':row['operator'],'events':row['events']} for row in rows],'final_scene':scenes[-1] if scenes else None}
   dump(OUT/(k+'.json'),report);index.append({'original_id':oid,'assessment':k+'.json'})
 dump(OUT/'candidate-index.json',index);print(json.dumps({'saved_inventory_reports':len(index),'no_engine_execution':True}))
if __name__=='__main__':main()
