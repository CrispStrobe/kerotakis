#!/usr/bin/env python3
"""Extract saved native output for review; never execute Kero or issue requests."""
import argparse,collections,hashlib,json,math,re
from pathlib import Path
H=lambda b:hashlib.sha256(b).hexdigest()
def load(p):return json.loads(p.read_text(),parse_constant=lambda x:(_ for _ in ()).throw(ValueError(x)))
def finite(x):
 if type(x)in(float,int):assert math.isfinite(x)
 elif isinstance(x,list):
  for v in x:finite(v)
 elif isinstance(x,dict):
  for v in x.values():finite(v)
def save(p,x):
 with p.open('x')as f:json.dump(x,f,indent=2,ensure_ascii=False,allow_nan=False);f.write('\n')
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--artifact',type=Path,required=True);a.add_argument('--out',type=Path,required=True);a.add_argument('--frozen-design',type=Path,required=True);args=a.parse_args();raw=args.artifact;out=args.out;out.mkdir(exist_ok=True);(out/'cases').mkdir(exist_ok=True)
 design=load(raw/'design.json');execution=load(raw/'execution.json')['results'];binding=load(raw/'binding.json');assert H(args.frozen_design.read_bytes())==binding['design_sha256'];assert load(args.frozen_design)==design;designs={x['id']:x for x in design['cases']};assert len(designs)==len(execution)==201;total=collections.Counter();limits=collections.Counter();case_limits=collections.Counter();classes=collections.Counter();summary=[];solver=[]
 for run in execution:
  ident=run['id'];path=raw/ident.replace(':','_');d=designs[ident];stdout=(path/'stdout.jsonl').read_bytes();stderr=(path/'stderr.txt').read_bytes();script=(path/'experiment.lab').read_bytes();assert H(stdout)==run['stdout_sha256']and H(stderr)==run['stderr_sha256']and H(script)==run['script_sha256']==d['selected_script_sha256'];assert load(path/'execution.json')==run
  rows=[json.loads(l)for l in stdout.splitlines()if l.strip()];finite(rows);assert len(rows)==run['parsed_observations'];counts=collections.Counter();notes={};final={};measurements=[];operators=[];failures=[];noted=set()
  for i,row in enumerate(rows):
   operators.append(row.get('operator'));seq=row.get('output_sequence',i)
   for v in row.get('bench',{}).get('vessels',[]):final[v['id']]={'observation_index':i,'output_sequence':seq,'step':row.get('step'),'state':v}
   for e in row.get('events',[]):
    event=e['event'];counts[event]+=1
    if event=='not_yet_modeled':
     key=e.get('reason',{}).get('key',e.get('what','unkeyed'));limits[key]+=1;noted.add(key)
     if key not in notes:notes[key]={'first_observation_index':i,'first_operator':row.get('operator'),'event':e,'occurrences':0}
     notes[key]['occurrences']+=1
    if event=='solver_failed':
     f={'observation_index':i,'output_sequence':seq,'operator':row.get('operator'),'event':e};failures.append(f);solver.append({'id':ident,**f})
    if event in ('measured','observed','titrated','transition_point_read','cell_voltage','no_cell','chromatographed','flame_test','chemiluminescence_observed','uv_attenuated','centrifuged','transported','polymer_swelled','gel_formed','foam_changed','enzyme_hydrolysed','fermented','org_reacted'):
     measurements.append({'observation_index':i,'operator':row.get('operator'),'event':e})
  for key in noted:case_limits[key]+=1
  classification='process_admission_refusal'if run['exit_code']else'completed_with_solver_failure'if failures else'completed_with_named_model_limits'if notes else'completed_requires_physical_assessment'
  if ident in ('batch3-diverse:so-001','batch3-diverse:so-003'):forecast='known_strong_base_selectivity_forecast_not_met_MODEL006'
  elif run['exit_code']:forecast='not_established_intended_later_operation_not_reached'
  elif failures:forecast='not_established_solver_failure_requires_manual_review'
  else:forecast='not_automatically_assessed'
  diag=re.sub(r'(?<=kero: )[^:\n]+(?=:\d+:)', '<selected-script>',stderr.decode())
  report={'id':ident,'engine':binding['engine'],'selected_script_sha256':run['script_sha256'],'original_script_sha256':d['original_script_sha256'],'selection':d['selection'],'original_physical_forecast':d['physical_forecast'],'original_acceptance_checks':d['acceptance_checks'],'execution':run,'classification':classification,'physical_forecast_status':forecast,'physical_verdict':'not_assigned','stderr_display':diag,'stderr_preservation':'Original stderr bytes/hash remain in authenticated raw artifact; host-specific script locator redacted only in this display.','actual_operator_sequence':operators,'event_counts':dict(counts),'named_model_limits':notes,'solver_failures':failures,'last_available_snapshots_by_vessel':list(final.values()),'actual_measurement_and_process_observations':measurements,'ledger_scope':'Authoritative contents plus unresolved materials, surfaces, exported/imported gases and operation transfers must all be counted. Solution species are diagnostic and must not be double-counted; free-ion fields are not new matter. No generic full-batch atom/energy proof is assigned.','catalog_case_closed':False}
  filename=ident.replace(':','_')+'.json';save(out/'cases'/filename,report);summary.append({'id':ident,'assessment_path':'cases/'+filename,'exit_code':run['exit_code'],'classification':classification,'physical_forecast_status':forecast,'solver_failure_count':len(failures),'named_limit_kinds':len(notes),'event_counts':dict(counts),'last_snapshot_vessels':len(final),'measurement_and_process_observations':len(measurements)});total.update(counts);classes[classification]+=1
 report={'schema':'kerotakis.native201-saved-review.v1','kind':'Saved-output extraction and triage, not 201 physical verdicts.','engine':binding['engine'],'run_id':binding['run_id'],'workflow_commit':binding['workflow_commit'],'input_checkout_commit':binding['input_checkout_commit'],'design_sha256':binding['design_sha256'],'artifact_design_copy_sha256':H((raw/'design.json').read_bytes()),'design_copy_note':'Runner reserialized the artifact design; original frozen bytes are separately hash-verified and JSON values must match exactly.','reviewed_cases':201,'all_case_input_stdout_stderr_receipts_verified':True,'exit_counts':dict(collections.Counter(r['exit_code']for r in execution)),'classifications':dict(classes),'event_occurrences':dict(total),'named_limit_occurrences':dict(limits),'named_limit_case_counts':dict(case_limits),'solver_failure_witnesses':solver,'cases':summary,'physical_passes_assigned':0,'catalog_cases_closed':0,'retries':0,'saved_evidence_only':True,'lock_qualification':binding['lock_qualification']};save(out/'assessment.json',report);print(json.dumps({'cases':201,'classifications':dict(classes),'solver_failures':solver,'physical_passes_assigned':0},indent=2))
if __name__=='__main__':main()
