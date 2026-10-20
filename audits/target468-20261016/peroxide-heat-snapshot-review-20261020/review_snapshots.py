#!/usr/bin/env python3
"""Read-only paired CI snapshot review; no engine/build/network execution."""
import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import zipfile


def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=pathlib.Path,required=True);p.add_argument('--source',required=True);p.add_argument('--results-dir',type=pathlib.Path,required=True);p.add_argument('--actual-checkout');p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args()
 sha=lambda b:hashlib.sha256(b).hexdigest()
 get=lambda path,source=a.source:subprocess.check_output(['git','show',source+':'+path],cwd=a.repo)
 source=subprocess.check_output(['git','rev-parse',a.source],cwd=a.repo,text=True).strip()
 metadata=json.loads((a.results_dir/'artifacts.json').read_text());bindings={};actuals={}
 for label,artifact_id in [('linux',12276213904),('macos',12279436718)]:
  row=next(x for x in metadata['artifacts'] if x['id']==artifact_id);archive=(a.results_dir/(label+'.zip')).read_bytes();assert sha(archive)==row['digest'].split(':',1)[1];assert len(archive)==row['size_in_bytes'];assert row['workflow_run']['head_sha']==source
  with zipfile.ZipFile(a.results_dir/(label+'.zip')) as z:
   assert z.namelist()==['kerotakis-core/tests/golden/lessons.actual.json'];raw=z.read(z.namelist()[0])
  assert raw==(a.results_dir/(label+'-lessons.actual.json')).read_bytes();actuals[label]=raw
  bindings[label]={'artifact_id':artifact_id,'artifact_name':row['name'],'archive_sha256':sha(archive),'actual_snapshot_sha256':sha(raw),'workflow_run':row['workflow_run']['id'],'reported_head':row['workflow_run']['head_sha'],'checkout_sha_from_artifact_name':re.search(r'-([0-9a-f]{40})$',row['name']).group(1)}
 assert actuals['linux']==actuals['macos']
 baseline=get('crates/kerotakis-core/tests/golden/lessons.json');old=json.loads(baseline);new=json.loads(actuals['linux']);assert old.keys()==new.keys() and len(old)==113
 changed=[k for k in old if old[k]!=new[k]];assert changed==['elephant-toothpaste','elephant-toothpaste-catalyst-dose','longer-waits-advance-peroxide-kinetics','luminol-temperature']
 def spans(raw):
  text=raw.decode();out={};decoder=json.JSONDecoder()
  for match in re.finditer(r'^  ("[^"\\]*(?:\\.[^"\\]*)*"): ',text,re.M):
   key=json.loads(match.group(1));_,end=decoder.raw_decode(text,match.end());out[key]=text[match.end():end].encode()
  return out
 os,ns=spans(baseline),spans(actuals['linux']);unaffected=[k for k in old if k not in changed]
 assert len(os)==113 and all(os[k]==ns[k] for k in unaffected)
 differences=[]
 def diff(x,y,path):
  if isinstance(x,dict) and isinstance(y,dict):
   for k in sorted(x.keys()|y.keys()):diff(x.get(k),y.get(k),path+'/'+k)
  elif isinstance(x,list) and isinstance(y,list):
   if len(x)!=len(y):differences.append({'path':path+'/length','before':len(x),'after':len(y)})
   for i in range(max(len(x),len(y))):
    if i>=len(x) or i>=len(y):differences.append({'path':path+'/'+str(i),'before':x[i] if i<len(x) else None,'after':y[i] if i<len(y) else None})
    else:diff(x[i],y[i],path+'/'+str(i))
  elif x!=y:differences.append({'path':path,'before':x,'after':y})
 for key in changed:diff(old[key],new[key],'/'+key)
 def amount(lesson,vessel,species):return sum(x['moles'] for x in new[lesson]['vessels'][vessel]['contents'] if x['species']==species)
 heat_examples=[]
 for lesson,v,idx in [('elephant-toothpaste',0,12),('elephant-toothpaste-catalyst-dose',0,11),('elephant-toothpaste-catalyst-dose',1,16),('luminol-temperature',1,18)]:
  xi=amount(lesson,v,'O2');expected=196400*xi
  before=float(re.search(r'released ([\d.]+) J',old[lesson]['events_sample'][idx]).group(1));after=float(re.search(r'released ([\d.]+) J',new[lesson]['events_sample'][idx]).group(1))
  # Eight-place moles round by at most5e-9; event heat rounds by0.005J.
  assert abs(after-expected)<=.005+196400*5e-9 and abs(before-98200*xi)<=.005+98200*5e-9
  heat_examples.append({'lesson':lesson,'vessel_index':v,'snapshot_O2_mol':xi,'old_rendered_heat_J':before,'new_rendered_heat_J':after,'new_heat_from_consumed_2H2O2_J':expected,'rounding_tolerance_basis':'0.005J event formatting + coefficient*5e-9mol snapshot rounding'})
 for lesson in changed:
  for i,(x,y) in enumerate(zip(old[lesson]['vessels'],new[lesson]['vessels'])):
   for left,right in zip(x['contents'],y['contents']):
    if left!=right:assert lesson=='luminol-temperature' and i==1 and left['species']=='water' and right['species']=='water'
 water_delta=old['luminol-temperature']['vessels'][1]['contents'][2]['moles']-new['luminol-temperature']['vessels'][1]['contents'][2]['moles'];assert abs(water_delta-.00132034)<1e-12
 files=['crates/kerotakis-core/src/clock.rs','crates/kerotakis-core/src/solve.rs','crates/kerotakis-core/src/vessel.rs','crates/kerotakis-core/src/kinetics.rs','crates/kerotakis-core/tests/frozen_behavior.rs']+['lessons/'+x+'.lab' for x in changed]
 source_blobs={x:sha(get(x)) for x in files}
 checkout={'status':'PENDING_EXACT_CHECKOUT_BLOB_PROOF','explanation':'Artifact API reports PR head; artifact name identifies synthetic merge checkout. Do not conflate these identities.'}
 if a.actual_checkout:
  assert all(v['checkout_sha_from_artifact_name']==a.actual_checkout for v in bindings.values())
  for path in files+['crates/kerotakis-core/tests/golden/lessons.json']:assert get(path)==get(path,a.actual_checkout),path
  checkout={'status':'EXACT_RELEVANT_BLOBS_EQUAL','actual_checkout':a.actual_checkout,'candidate_head':source,'verified_paths':files+['crates/kerotakis-core/tests/golden/lessons.json']}
 result={'review_scope':'Independent saved snapshot assessment; no golden mutation, engine/build/network invocation','source':source,'artifacts':bindings,'paired_snapshot_bytes_equal':True,'baseline_snapshot_sha256':sha(baseline),'lesson_count':113,'changed_lessons':changed,'unchanged_lessons':unaffected,'unchanged_lesson_value_bytes_exact':True,'field_differences':differences,'heat_checks':heat_examples,'source_blobs_sha256':source_blobs,'actual_checkout_binding':checkout,'physical_interpretation':{'heat_basis':'Extent consumes2H2O2 and produces1O2;98.2kJ/mol consumed means196.4kJ/mol extent. Event heat doubles within original formatting precision.','inventory':'All four lessons retain identical rounded substrate/O2/catalyst inventories. Only luminol hot liquid water changes; the paired cold water inventory is unchanged. No new reaction extent or arbitrary chemistry acceptance is inferred.','temperatures':'Larger heat raises adiabatic temperature; ambient relaxation follows chemistry, so final rises need not double. Catalyst-dose extra20th event is existing ambient v2 cooling crossing unchanged0.01K narration floor.','kinetic_feedback':'Later waits start slightly warmer, giving a small Arrhenius-rate change; longer-waits rate sample4.057e-8→4.058e-8 while8-place inventories stay identical. This is model feedback, not calibrated kinetics.','luminol_hot_water':{'additional_absent_liquid_water_mol':water_delta,'latent_equivalent_J_at_40650J_per_mol':water_delta*40650,'final_temperature_K':373.15,'explanation':'Unchanged StateEquilibrator spends excess sensible enthalpy above boiling on finite water vaporization, exporting it from an open vessel. Higher reaction heat leaves less liquid; ambient exchange also changes, so extra latent energy is not asserted equal to extra reaction heat.','limit':'Only first20events are snapshotted: no complete raw export/thermal trace is available here. This review justifies a bounded regression expectation, not a new complete calorimetry or mass-transfer qualification.'}},'recommendation':'Refresh ONLY actual four-lesson snapshot fields after exact synthetic-checkout blob proof; preserve113lessons,109unchanged byte values, unchanged scripts/forecasts/test comparison and original failing artifacts. Do not bless broader changes.','new_engine_build_network_calls':0}
 a.out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'changed_lessons':changed,'field_differences':len(differences),'unaffected':len(unaffected),'checkout_binding':checkout['status'],'report':a.out.name}))
if __name__=='__main__':main()
