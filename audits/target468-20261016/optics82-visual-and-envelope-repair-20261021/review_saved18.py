"""Prospective independent saved18 review. No browser/provider/runtime operations."""
import argparse,datetime,hashlib,json,math,re
from pathlib import Path
from urllib.parse import urlsplit
from project_app_assets import project,digest,require
from optics_saved_predicates_v4 import review_case
ROOT=Path(__file__).parent

def pinned(path,sha):
 require(re.fullmatch('[a-f0-9]{64}',sha)is not None,'malformed external pin');require(digest(path)==sha,'external input SHA mismatch');return json.loads(Path(path).read_bytes())
def review_visual(row,report_path,binding):
 # PNG/styles are corroboration; the unchanged typed-event/inventory/geometry
 # review below remains mandatory and cannot be replaced by these objects.
 require(row['gui']['corroboration_worker_traffic_after']==row['gui']['worker_traffic'],'actual worker changed during visual corroboration: HOLD')
 visual=row['gui']['visual_corroboration'];viewport=visual['viewport']
 require(set(viewport)=={'width','height','device_pixel_ratio'}and type(viewport['width'])is int and type(viewport['height'])is int and type(viewport['device_pixel_ratio'])in[int,float] and viewport=={'width':1280,'height':960,'device_pixel_ratio':1},'actual configured viewport missing/different')
 image=visual['screenshot'];scene=row['gui']['scene_pair']
 require(image['stage']=='after-final-measure-and-scene'and image['worker']==scene['worker']and image['after_response_sequence']==scene['response_sequence'],'actual capture stage/worker sequence differs')
 origin=urlsplit(image['origin_url']);expected_origin=urlsplit(binding['origin'])
 require((origin.scheme,origin.netloc)==(expected_origin.scheme,expected_origin.netloc)and origin.scheme=='https'and origin.path=='/app/'and not origin.query and not origin.fragment,'actual GUI capture origin invalid')
 for field in ['capture_started_utc','capture_finished_utc']:require(type(image[field])is str and re.fullmatch(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z',image[field])is not None,'actual capture timestamp absent/malformed')
 require(datetime.datetime.fromisoformat(image['capture_started_utc'].replace('Z','+00:00'))<=datetime.datetime.fromisoformat(image['capture_finished_utc'].replace('Z','+00:00')),'actual capture time reversed')
 name='screenshots/'+row['case_id']+'-'+row['locale']+'.png'
 require(image['path']==name and re.fullmatch(r'screenshots/[a-z0-9-]+-(en|de|fr)\.png',name)is not None and image['type']=='png'and image['full_page']is False and image['animations']=='allow'and image['caret']=='initial','wrong actual PNG capture path/mode')
 root=report_path.parent.resolve();path=root/name
 require(all(not p.is_symlink()for p in [path,*path.parents]if p==root or root in p.parents)and path.is_file(),'actual screenshot missing/linked')
 size=path.stat().st_size;require(type(image['bytes'])is int and 24<size<=16*1024*1024 and size==image['bytes']and digest(path)==image['sha256'],'actual PNG bytes/SHA differ')
 with path.open('rb')as stream:header=stream.read(24)
 require(header[:16]==b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR'and type(image['pixel_width'])is int and type(image['pixel_height'])is int and int.from_bytes(header[16:20],'big')==image['pixel_width']==viewport['width']and int.from_bytes(header[20:24],'big')==image['pixel_height']==viewport['height'],'actual PNG header/viewport dimensions differ')
 def style(value):
  require(set(value)=={'display','visibility','opacity','fill','stroke','overflow','bounds'}and all(type(value[k])is str for k in ['display','visibility','opacity','fill','stroke','overflow']),'actual computed style fields absent')
  bounds=value['bounds'];require(all(type(bounds[k])in[int,float]and math.isfinite(bounds[k])for k in ['x','y','width','height'])and bounds['width']>0 and bounds['height']>0,'actual corroborating element geometry invalid')
  try:opacity=float(value['opacity'])
  except ValueError:raise ValueError('actual computed opacity invalid')
  require(value['display']!='none'and value['visibility']=='visible'and 0<opacity<=1,'rendered corroborating element hidden')
 style(visual['vessel']);dom=row['gui']['dom']
 for group in ['layers','hatches','ph_readouts']:
  require(len(visual[group])==len(dom[group]),'actual computed style/DOM element counts differ')
  for item in visual[group]:style(item)
 for css,layer in zip(visual['layers'],dom['layers']):require(re.sub(r'\s+','',css['fill'])==re.sub(r'\s+','',layer['fill']),'actual computed liquid fill differs from DOM fill')

def main():
 p=argparse.ArgumentParser()
 for name in ['report','binding','projection','closure','artifact','verified']:
  p.add_argument('--'+name,type=Path,required=True);p.add_argument('--'+name+'-sha256',required=True)
 p.add_argument('--source',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 closure=pinned(a.closure,a.closure_sha256)
 for name,sha in closure['files_sha256'].items():require(re.fullmatch('[A-Za-z0-9_.-]+',name)is not None and digest(ROOT/name)==sha,'program/protocol closure differs')
 required={'run-optics18.mjs','optics-controls.mjs','optics_saved_predicates_v4.py','PROTOCOLS.json','EXPECTED-WORDS.json','OPTICAL-ORACLE.json','review_saved18.py','project_app_assets.py'};require(required<=set(closure['files_sha256']),'incomplete externally pinned code closure')
 binding=pinned(a.binding,a.binding_sha256);projection=pinned(a.projection,a.projection_sha256);report=pinned(a.report,a.report_sha256)
 # Recompute actual archive proof, do not trust a producer-supplied asset map.
 actual=project(a.artifact,a.artifact_sha256,a.verified,a.verified_sha256,a.source);require(actual==projection,'actual authenticated archive projection differs')
 require(binding['expected_engine_source']==projection['source']==a.source and binding['native_binary_sha256']==projection['native_binary_sha256']and binding['actual_producer_lock_sha256']==projection['actual_producer_lock_sha256'],'release identity differs')
 require(binding['artifact_sha256']==a.artifact_sha256 and binding['verified_release_sha256']==a.verified_sha256,'external verifier/artifact authority differs')
 app={k:v['sha256']for k,v in projection['app_assets'].items()};engine={k:v['sha256']for k,v in projection['engine_and_rest_assets'].items()if k!='rest.html'}
 require(binding['app_assets']==app and binding['frontend_assets']==engine and binding['client_sha256']==projection['engine_and_rest_assets']['rest.html']['sha256'],'release assets/rest differ')
 origin=urlsplit(binding['origin']);require(origin.scheme=='https'and origin.netloc and origin.path in ['', '/'],'invalid expected secure origin')
 require(report['binding']==binding and report['program_closure_sha256']==a.closure_sha256 and report['binding_sha256']==a.binding_sha256 and report['projection_sha256']==a.projection_sha256,'actual report binding differs')
 require(report['browser']['web_security_disabled']is False and report['browser']['no_owner_credentials_supplied']is True and type(report['browser']['version'])is str,'browser envelope missing; external exact command/flags and worker/issue authentication still required')
 require(not report.get('error')and not report.get('wrapper_error')and report['actual_observation_rows_complete']is True and report['product_acceptance_claimed']is False,'producer did not complete genuine observations')
 protocols=json.loads((ROOT/'PROTOCOLS.json').read_bytes());words=json.loads((ROOT/'EXPECTED-WORDS.json').read_bytes());oracle=json.loads((ROOT/'OPTICAL-ORACLE.json').read_bytes())
 expected={(c['id'],locale):c for c in protocols['cases']for locale in c['locales']};require(len(expected)==18 and len(report['rows'])==18,'exact frozen18 rows absent');seen=set();results=[]
 for row in report['rows']:
  key=(row['case_id'],row['locale']);require(key in expected and key not in seen,'unexpected/duplicate actual case');seen.add(key)
  gui=row['gui'];loaded=gui['app_responses'];require(loaded,'actual loaded app response bodies missing')
  for response in loaded:
   u=urlsplit(response['url']);require((u.scheme,u.netloc)==(origin.scheme,origin.netloc)and not u.query and not u.fragment,'actual loaded app response origin/query differs')
   archive_path='app/index.html'if u.path=='/app/'else u.path.lstrip('/')
   require(response['state']=='ACTUAL_LOADED_BYTES_MATCH'and 200<=response['status']<300 and response['archive_path']==archive_path and archive_path in app,'failed/unbound actual loaded response')
   require(response['sha256']==app[archive_path]and type(response['bytes'])is int and response['bytes']==projection['app_assets'][archive_path]['bytes'],'actual loaded response SHA/length differs from archive')
  require(any(urlsplit(r['url']).path=='/app/'for r in loaded),'actual navigation body unobserved')
  for event in gui['worker_traffic']:
   if event['direction']=='created':require(any(r['url']==event['url']for r in loaded),'actual dedicated-worker response body unobserved')
  # Source identity is actual metadata, not a producer boolean.
  for meta in [row['lab']['meta'],json.loads(gui['hello']['response']['result_json'])]:
   rev=meta['git_rev'];require(type(rev)is str and re.fullmatch('[a-f0-9]{7,40}',rev)is not None and a.source.startswith(rev),'actual shipped Lab/GUI source differs')
  require(gui['external_solver_calls']is None and gui['external_solver_calls_not_exposed_by_public_GUI_protocol']is True,'GUI solver calls must not be fabricated from separate Lab')
  require(row['lab']['snapshot_before']==row['lab']['snapshot_after'],'passive independent Lab changed full snapshot')
  review_visual(row,a.report,binding)
  results.append(review_case(row,expected[key],words[row['locale']],oracle))
 require(seen==set(expected),'missing actual frozen case')
 result={'state':'SAVED18_CONJUNCTIVE_REVIEW_ONLY_REQUIRES_PARENT_AND_FULL_EXTERNAL_ENVELOPE','source':a.source,'report_sha256':a.report_sha256,'binding_sha256':a.binding_sha256,'projection_sha256':a.projection_sha256,'closure_sha256':a.closure_sha256,'actual_saved_rows_reviewed':18,'results':results,'scientific_selections':0,'product_or_production_acceptance_claimed':False,'unresolved_external_acceptance':'Parent must authenticate actual issued worker/config/command security/native/API receipts/manifests, both independent review outputs and unchanged prior64 before final82 acceptance.'}
 with a.out.open('x',encoding='utf-8')as out:json.dump(result,out,indent=2);out.write('\n')
if __name__=='__main__':main()
