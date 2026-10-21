"""Independent saved diagnostic diff; never executes a model or updates goldens."""
from pathlib import Path
import argparse,collections,copy,hashlib,json,subprocess

def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True);x=p.parse_args();x.evidence=Path(__file__).resolve().parent/'evidence';O=x.out;O.mkdir(exist_ok=False);source='e648249119e1fe028735bf31723da2867d87db77';sha=lambda b:hashlib.sha256(b).hexdigest()
 def git(path):return subprocess.check_output(['git','show',source+':'+path],cwd=x.repo,timeout=60)
 def save(n,v):(O/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
 def diff(a,b,path,rows):
  if type(a)!=type(b):rows.append({'path':path,'kind':'type','expected':a,'actual':b});return
  if isinstance(a,dict):
   for k in sorted(a.keys()|b.keys()):
    if k not in a:rows.append({'path':path+[k],'kind':'added','actual':b[k]})
    elif k not in b:rows.append({'path':path+[k],'kind':'removed','expected':a[k]})
    else:diff(a[k],b[k],path+[k],rows)
  elif isinstance(a,list):
   if len(a)!=len(b):rows.append({'path':path,'kind':'length','expected':len(a),'actual':len(b)})
   for i,(u,v)in enumerate(zip(a,b)):diff(u,v,path+[i],rows)
  elif a!=b:rows.append({'path':path,'kind':'value','expected':a,'actual':b})
 identities=json.loads(git('data/registry/registry-source-v1.json'))['identities'];names={row['id']:row['name']for row in identities};assert names['anthocyanin']=='anthocyanin (red-cabbage pigment)'and names['bromothymol_blue']=='bromothymol blue'
 authority={}
 for path in ['crates/kerotakis-core/src/species.rs','crates/kerotakis-core/src/phrase.rs','crates/kerotakis-core/src/render.rs','crates/kerotakis-core/src/scene.rs','crates/kerotakis-core/src/appearance.rs','crates/kerotakis-core/tests/scene_golden.rs','data/registry/registry-source-v1.json']:
  raw=git(path);terms=['pub fn english_name','crate::species::english_name','species::english_name','optical_species_names','look.indicator-optics-missing-ph','1_000_000.0','anthocyanin (red-cabbage pigment)','"bromothymol blue"'];authority[path]={'sha256':sha(raw),'coordinates':[{'line':i+1,'text':line.strip()}for i,line in enumerate(raw.decode().splitlines())if any(t in line for t in terms)]}
 expected_scene_raw=git('crates/kerotakis-core/tests/golden/scene-five.json');actual_scene_raw=(x.evidence/'diagnostic-scene.actual.json').read_bytes();a=json.loads(actual_scene_raw);e=json.loads(expected_scene_raw);assert(x.evidence/'diagnostic-scene.expected.json').read_bytes()==expected_scene_raw.rstrip(b'\r\n');changes=[];diff(e,a,[],changes)
 assert len(changes)==296 and collections.Counter(str(v['path'][-1])for v in changes)=={'optical_ph_gaps':74,'optical_species_names':74,'notes':74,'words':74};assert {v['path'][0]for v in changes}=={'cabbage-rainbow'}
 restored=copy.deepcopy(a);count=0
 for lesson,frames in restored.items():
  for i,f in enumerate(frames):
   for j,v in enumerate(f['scene']['vessels']):
    if not(v.get('liquid')or{}).get('optical_ph_gaps'):continue
    old=e[lesson][i]['scene']['vessels'][j];assert v['liquid']['optical_ph_gaps']==['anthocyanin']and v['liquid']['optical_species_names']=={'anthocyanin':names['anthocyanin']}
    assert v['notes']==[{'en':'Colour is incomplete: a finite aqueous pH is required for {species}.','key':'look.indicator-optics-missing-ph','slots':[['species',{'list':{'items':[{'term':{'en':'anthocyanin','section':'species'}}]}}]]}]
    assert v['words']==old['words']+' Colour is incomplete: a finite aqueous pH is required for '+names['anthocyanin']+'.';del v['liquid']['optical_ph_gaps'];del v['liquid']['optical_species_names'];del v['notes'];v['words']=old['words'];count+=1
 assert count==74 and restored==e
 save('SCENE-ALL-296-PATH-DIFFS.json',{'source':source,'expected_sha256':sha(expected_scene_raw),'actual_captured_sha256':sha(actual_scene_raw),'changes':changes})
 actual_lessons_raw=(x.evidence/'diagnostic-lessons.actual.json').read_bytes();expected_lessons_raw=git('crates/kerotakis-core/tests/golden/lessons.json');a=json.loads(actual_lessons_raw);e=json.loads(expected_lessons_raw);changes=[];diff(e,a,[],changes);assert len(a)==len(e)==113 and len(changes)==5 and collections.Counter(v['path'][0]for v in changes)=={'cabbage-rainbow':3,'lemonade-indicator':2}
 restored=copy.deepcopy(a)
 for d in changes:
  lesson,field,i=d['path'];assert field=='events_sample'and d['kind']=='value';name=names['anthocyanin'if lesson=='cabbage-rainbow'else'bromothymol_blue'];assert d['actual']==d['expected']+' Colour is incomplete: a finite aqueous pH is required for '+name+'.';restored[lesson][field][i]=d['expected']
 assert restored==e
 save('LESSONS-ALL-FIVE-PATH-DIFFS.json',{'source':source,'actual_file_captured_in_complete_diagnostic':True,'actual_captured_sha256':sha(actual_lessons_raw),'expected_sha256':sha(expected_lessons_raw),'changes':changes})
 save('SOURCE-AUTHORITIES.json',{'source':source,'registry_display_names':{k:names[k]for k in ['anthocyanin','bromothymol_blue']},'blobs':authority})
 summary={'source':source,'scene_changed_paths':296,'changed_scene_vessel_snapshots':74,'scene_only_cabbage_changed':True,'all_other_scene_fields_including_numbers_material_commands_events_exact_equal':True,'numeric_precision_limit':'scene_golden independently rounds floats to6decimal places; this is presentation equality, not calibration','lessons_total':113,'actual_complete_lesson_diagnostic_captured':True,'changed_lesson_strings':5,'all_other_lesson_fields_exact_equal':True,'new_names_map_registry_authoritative':True,'canonical_species_ids_and_typed_phrase_slots_preserved':True,'unresolved_snapshot_tests':2,'all_gates_pass':False,'source_goldens_modified':False,'scientific_selections':0};save('ADJUDICATION.json',summary);print(json.dumps(summary))
if __name__=='__main__':main()
