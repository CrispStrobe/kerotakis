"""Portable public saved-evidence review; no engine calls or golden edits."""
from pathlib import Path
import argparse,collections,hashlib,json,re,subprocess

def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True);x=p.parse_args();B=Path(__file__).resolve().parent;O=x.out;O.mkdir(exist_ok=True);source='047d04513d1e55e162ad4269cbd5192e432e83e1';sha=lambda b:hashlib.sha256(b).hexdigest()
 def git(path):return subprocess.check_output(['git','show',source+':'+path],cwd=x.repo,timeout=60)
 def save(n,v):(O/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
 actual_raw=(B/'evidence/scene.actual.extracted.json').read_text();actual=json.loads(actual_raw)
 golden=git('crates/kerotakis-core/tests/golden/scene-five.json');expected_raw=golden.decode();expected=json.loads(golden)
 bindings=json.loads((B/'records/SCENE-STRUCTURED-DIFF.json').read_bytes());assert sha(actual_raw.encode())==bindings['actual_sha256']and sha(expected_raw.rstrip().encode())==bindings['expected_sha256']
 modified=[];projected=json.loads(actual_raw)
 for lesson,frames in projected.items():
  for i,frame in enumerate(frames):
   for j,v in enumerate(frame['scene']['vessels']):
    if not (v.get('liquid')or{}).get('optical_ph_gaps'):continue
    old=expected[lesson][i]['scene']['vessels'][j];gaps=v['liquid']['optical_ph_gaps'];assert gaps==['anthocyanin'];notes=v['notes'];assert len(notes)==1 and notes[0]['key']=='look.indicator-optics-missing-ph'and notes[0]['en']=='Colour is incomplete: a finite aqueous pH is required for {species}.'and notes[0]['slots']==[['species',{'list':{'items':[{'term':{'en':'anthocyanin','section':'species'}}]}}]]
    suffix=' Colour is incomplete: a finite aqueous pH is required for anthocyanin.';assert v['words']==old['words']+suffix
    modified.append({'lesson':lesson,'frame_index':i,'command':frame['command'],'line':frame['line'],'vessel_index':j,'gap':gaps})
    del v['liquid']['optical_ph_gaps'];del v['notes'];v['words']=old['words']
 assert len(modified)==74 and projected==expected
 unchanged=[k for k in expected if actual[k]==expected[k]];assert unchanged==['boiling-curve','rusting','slime','water-filter']
 save('SCENE-ADJUDICATION.json',{'source':source,'expected_blob_sha256':sha(golden),'actual_scene_sha256':sha(actual_raw.encode()),'changed_vessel_snapshots':len(modified),'changes_per_snapshot':['liquid.optical_ph_gaps added [anthocyanin]','notes added exact typed missing-pH phrase','words old caption plus missing-pH sentence'],'all_other_fields_exact_equal':True,'physical_numeric_fields_and_commands_and_event_tags_unchanged':True,'unchanged_lessons':unchanged,'modified_snapshots':modified,'limits':'Numerical equality is at the six-decimal presentation precision frozen by scene_golden.rs; not an independent chemistry calibration or proof of missing-pH equilibrium.'})
 # The actual source compares complete combined113-lesson JSON, emits ALL differing lines (cap600).
 test=git('crates/kerotakis-core/tests/frozen_behavior.rs');assert b'let shown = differences.len().min(600);'in test and b'expected_lines.get(index) != actual_lines.get(index)'in test
 raw=git('crates/kerotakis-core/tests/golden/lessons.json');base=raw.decode().splitlines();new=base.copy();report=json.loads((B/'evidence/frozen-five-line-diff.json').read_bytes());assert report['reported_total_differing_lines']==5 and report['all_reported_lines_included']is True and len(report['changes'])==5;changes=[]
 for row in report['changes']:
  number=row['line'];old=row['expected'];actual_line=row['actual'];assert base[number-1]==old;new[number-1]=actual_line;changes.append({'line':number,'expected':old.strip(),'actual':actual_line.strip()})
 reconstructed='\n'.join(new)+('\n'if raw.endswith(b'\n')else'');assert reconstructed.encode()==(B/'evidence/lessons.reconstructed-from-exhaustive-diff.json').read_bytes();(O/'lessons.reconstructed-from-exhaustive-diff.json').write_text(reconstructed);a=json.loads(reconstructed);e=json.loads(raw);deltas=[]
 def walk(u,v,path):
  assert type(u)==type(v)
  if isinstance(u,dict):
   assert u.keys()==v.keys()
   for k in u:walk(u[k],v[k],path+[k])
  elif isinstance(u,list):
   assert len(u)==len(v)
   for i,(q,r)in enumerate(zip(u,v)):walk(q,r,path+[i])
  elif u!=v:deltas.append({'path':path,'expected':u,'actual':v})
 walk(e,a,[]);assert len(deltas)==5
 for d in deltas:assert isinstance(d['expected'],str)and d['actual'].startswith(d['expected']+' Colour is incomplete: a finite aqueous pH is required for ')
 save('FROZEN-STRUCTURED-ADJUDICATION.json',{'source':source,'expected_blob_sha256':sha(raw),'actual_file_retained':False,'reconstruction_authority':'Exact source compares whole combined corpus and emitted all5 differing lines; reconstruct only those lines, no engine execution. Reconstructed file is not claimed as raw captured artifact.','combined_lessons':len(e),'changed_lessons':sorted({d['path'][0]for d in deltas}),'all_other_fields_exact_equal_by_exhaustive_line_diff':True,'changes':deltas,'textual_diff':changes,'frozen_behavior_source_sha256':sha(test)})
 sources={p:sha(git(p))for p in ['crates/kerotakis-core/tests/scene_golden.rs','crates/kerotakis-core/tests/frozen_behavior.rs','crates/kerotakis-core/src/solution_optics.rs','crates/kerotakis-core/src/appearance.rs','crates/kerotakis-core/src/scene.rs','crates/kerotakis-core/src/phrase.rs']};save('SOURCE-AUTHORITIES.json',{'source':source,'blobs_sha256':sources})
 save('PUBLIC-REVIEW-SCOPE.json',{'source':source,'public_saved_inputs_only':True,'raw_private_logs_or_binaries_reverified':False,'actual_extracted_scene_reviewed':True,'derived_lessons_reconstruction_reviewed':True,'full_acceptance':False,'scientific_selections':0})
 print(json.dumps({'scene_changed_vessels':len(modified),'scene_physical_fields_equal':True,'frozen_deltas':len(deltas),'frozen_changed_lessons':sorted({d['path'][0]for d in deltas})}))
if __name__=='__main__':main()
