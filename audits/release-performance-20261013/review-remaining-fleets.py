from pathlib import Path,PurePosixPath
import os,sys,shutil,json,hashlib,subprocess,zipfile,stat,math,collections,datetime,importlib.util,time
import argparse
parser=argparse.ArgumentParser()
for key in ['source','audit','out']:parser.add_argument('--'+key,type=Path,required=True)
args=parser.parse_args()
if not __debug__:raise RuntimeError('optimized Python disables provenance assertions')
assert os.environ.get('GITHUB_ACTIONS')=='true' and os.environ.get('GITHUB_EVENT_NAME')=='workflow_dispatch','real hosted dispatch required'
assert not args.out.exists(),'exclusive output directory required'
args.out.mkdir()
OUT=args.out.resolve();SOURCE=args.source.resolve();REPO=SOURCE;AUDIT=args.audit.resolve()
HEAD='d06ffe3a1740728656878c4aea0e9507ca83c808';TREE='25d1dd490acc080a59f2a02e5b9399a22bd0f4bc';TESTED='4df5deab77506d372a8df45f5d517e2f070d7f5a';RUN=38387729183
BIN='b290b436f63dc026d3c9b1134cc6362eb501008a7a0c9ad46a4a0567378e9853';LOCK='5135e2cca3fa785a2d109409c0540eb1265cf7f6670637ab87e3d8af1b836a93'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def write(path,value):path.write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')
def gate(stage):
 d=dict(stage=stage,unix_time=time.time(),load1=os.getloadavg()[0],available_memory_mib=int(next(x.split()[1] for x in open('/proc/meminfo') if x.startswith('MemAvailable:')))//1024,nas_free_mib=shutil.disk_usage(OUT).free//1048576)
 with (OUT/'gates.jsonl').open('a') as f:f.write(json.dumps(d)+'\n')
 if not (d['load1']<=3.5 and d['available_memory_mib']>=3200 and d['nas_free_mib']>=8192):raise RuntimeError('resource gate closed: '+stage)
 return d
def reject(x):raise ValueError('nonfinite token '+x)
def finite(x):
 if isinstance(x,float):assert math.isfinite(x),'nonfinite overflow'
 elif isinstance(x,list):
  for z in x:finite(z)
 elif isinstance(x,dict):
  for z in x.values():finite(z)
def load(p):
 d=json.loads(p.read_bytes(),parse_constant=reject);finite(d);return d
def report_json_transport(value):
 # Match the source analyzer CLI's JSON writer, including integer evidence keys.
 # Refuse key collisions instead of silently discarding evidence on roundtrip.
 def unique(pairs):
  result={}
  for key,item in pairs:
   if key in result:raise ValueError('duplicate report key after JSON transport: '+key)
   result[key]=item
  return result
 return json.loads(json.dumps(value,allow_nan=False),parse_constant=reject,object_pairs_hook=unique)
def git(path):return subprocess.check_output(['git','show',HEAD+':'+path],cwd=REPO)
def verify_source(path):
 b=(SOURCE/path).read_bytes();assert b==git(path),path;return hashlib.sha256(b).hexdigest()
RECEIPT={'schema_version':1,'review_kind':'independent_saved_provenance_and_source_owned_fleet_law_recalculation','all_checks_pass':False,'run':RUN,'candidate':{'head':HEAD,'tree':TREE,'tested_checkout':TESTED,'binary_sha256':BIN,'lock_sha256':LOCK},'families':[],'source_sha256':{},'cli_executed':False,'scientific_calibration_accepted':False}
def checkpoint():write(OUT/'review-checkpoint.json',RECEIPT)
def api(folder,name,endpoint,raw=False):
 gate('API '+name)
 with (folder/name).open('wb') as f,(folder/(name+'.stderr')).open('wb') as err:subprocess.run(['gh','api',*FLAGS,'repos/CrispStrobe/kerotakis/'+endpoint],stdout=f,stderr=err,check=True,timeout=60)
 return None if raw else load(folder/name)
try:
 gate('initial analysis preparation')
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=AUDIT).decode().strip()==os.environ['GITHUB_SHA'],'audit source head differs'
 assert subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=AUDIT)==b'','audit source changed'
 frozen=load(AUDIT/'audits/release-performance-20261013/hosted-remaining-fleet-review-freeze.json')
 for path,digest in frozen['files_sha256'].items():assert sha(AUDIT/path)==digest,'frozen hosted definition differs: '+path
 paths=['tools/chemistry-audit/run.py','tools/chemistry-audit/source_fleets.py','tools/chemistry-audit/source_fleets_2.py','tools/chemistry-audit/source_fleets_3.py','tools/chemistry-audit/analyse_source_fleets.py','tools/chemistry-audit/analyse_source_fleets_3.py','tools/chemistry-audit/sixth_batch.py','tools/chemistry-audit/seventh_batch.py','tools/chemistry-audit/analyse_sixth.py','tools/chemistry-audit/analyse_seventh.py','data/registry/registry-source-v1.json','.github/workflows/chemistry-audit.yml']
 for path in paths:RECEIPT['source_sha256'][path]=verify_source(path)
 assert subprocess.check_output(['git','rev-parse',HEAD+'^{tree}'],cwd=REPO).decode().strip()==TREE
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=SOURCE).decode().strip()==HEAD
 assert subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=SOURCE)==b''
 sys.path.insert(0,str(SOURCE/'tools/chemistry-audit'))
 import analyse_source_fleets as a1,analyse_source_fleets_3 as a3,analyse_sixth as a6,analyse_seventh as a7,sixth_batch
 frozen6=list(sixth_batch.recorder.CASES)
 import seventh_batch
 frozen7=list(seventh_batch.recorder.CASES)
 helptext=subprocess.check_output(['gh','api','--help']).decode();FLAGS=['--allow-escape-sequences'] if '--allow-escape-sequences' in helptext else []
 run=api(OUT,'run.json','actions/runs/'+str(RUN));assert run['head_sha']==HEAD and run['run_attempt']==1 and run['status']=='completed' and run['conclusion']=='success' and run['event']=='pull_request'
 assert len(run['pull_requests'])==1 and run['pull_requests'][0]['number']==785 and run['pull_requests'][0]['base']['sha']=='7337bb58ce48285c54be5885dbe07afca1adbb7e'
 assert sha(AUDIT/'audits/release-performance-20261013/candidate-artifact-independent-review-v2.json')=='8ec124715750fbb0b316d7354f5b924ff9517ec6ff92ad7cdf59c879e0995c9c'
 prior=load((AUDIT/'audits/release-performance-20261013/candidate-artifact-independent-review-v2.json'));assert prior['all_checks_pass'] is True and prior['candidate']=={k:RECEIPT['candidate'][k] for k in ['head','tree','binary_sha256','lock_sha256']} and prior['tested_checkout']==TESTED
 RECEIPT['accepted_own_release_receipt_sha256']=sha((AUDIT/'audits/release-performance-20261013/candidate-artifact-independent-review-v2.json'))
 api(OUT,'artifacts.json','actions/runs/'+str(RUN)+'/artifacts?per_page=100');snapshot=OUT/'artifacts.json';artifact_list=load(snapshot);assert artifact_list['total_count']==len(artifact_list['artifacts']);arts=artifact_list['artifacts'];selected=[x for x in arts if (x['name'].startswith('chemistry-audit-') and not x['name'].startswith('chemistry-audit-cli-')) or x['name'].startswith('chemistry-source-v3-')];assert len(selected)==17
 RECEIPT['artifact_snapshot_sha256']=sha(snapshot);RECEIPT['current_run_api_sha256']=sha(OUT/'run.json');checkpoint()
 previous_path=AUDIT/'audits/release-performance-20261013/candidate-fleet-v1-v3-partial-review.json'
 assert sha(previous_path)=='23eca87b0d200ecafd07294bc2ee5f72a79161dd9119044d633ff93a5f745e50','accepted partial receipt bytes differ'
 previous=load(previous_path);assert previous['run']==RUN and previous['candidate']==RECEIPT['candidate']
 assert previous['accepted_own_release_receipt_sha256']==RECEIPT['accepted_own_release_receipt_sha256']
 assert len(previous['accepted_families'])==1 and previous['accepted_families'][0]['family']=='acid-base-buffers'
 for path,digest in previous['source_sha256'].items():assert verify_source(path)==digest
 RECEIPT['accepted_partial_receipt_sha256']=sha(previous_path)
 RECEIPT['source_sha256'].update(previous['source_sha256'])
 for artifact in sorted(selected,key=lambda x:x['name']):
  name=artifact['name'];v3=name.startswith('chemistry-source-v3-');family=name.removeprefix('chemistry-source-v3-' if v3 else 'chemistry-audit-').removesuffix('-'+TESTED);kind='v3' if v3 else ('frozen' if family in ['sixth','seventh'] else 'v1')
  if family=='acid-base-buffers':
   gate('bind externally accepted acid/base family')
   row=previous['accepted_families'][0]
   assert row['artifact_id']==artifact['id'] and row['artifact_name']==name
   assert artifact['digest']=='sha256:'+row['artifact_zip_sha256'] and row['cases']==24 and row['passed_law_checks']==32
   assert artifact['workflow_run']['id']==RUN and artifact['workflow_run']['head_sha']==HEAD
   assert row['independent_raw_provenance_contracts_pass'] and row['source_owned_laws_pass']
   RECEIPT['families'].append(row);RECEIPT['externally_accepted_family_not_downloaded_or_reanalyzed']='acid-base-buffers';checkpoint();continue
  folder=OUT/(kind+'-'+family);folder.mkdir(exist_ok=False)
  metadata=api(folder,'artifact-api.json','actions/artifacts/'+str(artifact['id']))
  for key in ['id','name','digest','size_in_bytes']:assert metadata[key]==artifact[key],key
  assert metadata['workflow_run']['id']==RUN and metadata['workflow_run']['head_sha']==HEAD and not metadata['expired'] and metadata['size_in_bytes']<200000
  api(folder,'artifact.zip','actions/artifacts/'+str(artifact['id'])+'/zip',True)
  assert sha(folder/'artifact.zip')==metadata['digest'].removeprefix('sha256:') and (folder/'artifact.zip').stat().st_size==metadata['size_in_bytes']
  gate('extract '+name);extracted=folder/'extracted';extracted.mkdir();members={}
  with zipfile.ZipFile(folder/'artifact.zip') as z:
   infos=z.infolist();assert len(infos)<=512 and len(infos)==len({x.filename for x in infos}) and sum(x.file_size for x in infos)<=33554432 and z.testzip() is None
   for info in infos:
    path=PurePosixPath(info.filename);assert not path.is_absolute() and '..' not in path.parts and not stat.S_ISLNK(info.external_attr>>16)
    if info.is_dir():continue
    assert stat.S_IFMT(info.external_attr>>16) in [0,stat.S_IFREG]
    target=extracted/path;target.parent.mkdir(parents=True,exist_ok=True)
    with z.open(info) as src,target.open('wb') as dst:shutil.copyfileobj(src,dst,1048576)
    target.chmod(0o400);members[info.filename]=sha(target)
  summaries=list(extracted.rglob('summary.json'));assert len(summaries)==1;directory=summaries[0].parent;summary=load(summaries[0]);assert summary['commit']==TESTED and summary['binary_sha256']==BIN
  manifest=None
  if kind=='frozen':
   assert summary['lock_sha256']==LOCK and summary['complete_source_patch_sha256']==sha(directory/'source.patch') and (directory/'source.patch').read_bytes()==b''
   assert summary['source_diff_sha256']==hashlib.sha256(b'').hexdigest() and summary['untracked_source_sha256']=={}
   newfiles=load(directory/'new-source-files.json');assert newfiles=={},'clean tracked checkout must have no untracked source snapshots'
   # Root Cargo.lock is ignored: summary lock digest binds the accepted own release, not a Git blob.
   specs={x['id']:x for x in (frozen6 if family=='sixth' else frozen7)};expected_count=24 if family=='sixth' else 30
  else:
   manifestpath='tools/chemistry-audit/'+('source-fleets-3' if v3 else 'source-fleets')+'/'+family+'.json';RECEIPT['source_sha256'][manifestpath]=verify_source(manifestpath);manifest=load(SOURCE/manifestpath);specs={x['id']:x for x in manifest['cases']};expected_count=24
   assert summary['manifest_sha256']=={family+'.json':sha(SOURCE/manifestpath)}
  records=summary['cases'];assert len(records)==len(specs)==expected_count and {x['id'] for x in records}==set(specs) and len({x['id'] for x in records})==expected_count
  gate('raw independent provenance '+name);raw={};counts=collections.Counter()
  for record in records:
   spec=specs[record['id']];d=directory/record['id'];stdout=(d/'stdout.ndjson').read_text();stderr=(d/'stderr.txt').read_text();rows=[json.loads(line,parse_constant=reject) for line in stdout.splitlines()];finite(rows);assert rows and all(isinstance(x,dict) for x in rows)
   events=[event for row in rows for event in row.get('events',[])];final=next(row['bench'] for row in reversed(rows) if 'bench' in row)
   assert record['exit']==0 and record['non_json_lines']==[] and record['steps']==len(rows) and record['stderr']==stderr and record['final']==final and record['event_counts']==dict(collections.Counter(x.get('event','<missing>') for x in events))
   assert record['diagnostics']==[x for x in events if x.get('event') in ['not_yet_modeled','solver_failed','hazard_warning']] and not any(x.get('event')=='solver_failed' for x in events)
   assert record['question']==spec['question']
   script='# '+spec['question']+'\n'+(spec['script'] if kind=='frozen' else 'register lv3\n'+spec['script'].strip()+'\ninspect\n');assert (d/'experiment.lab').read_text()==script
   if kind!='frozen':assert record['family']==family
   raw[record['id']]={n:sha(d/n) for n in ['experiment.lab','stdout.ndjson','stderr.txt']};counts.update(x.get('event','<missing>') for x in events)
  if (directory/'preflight-sha256.txt').exists():
   for line in (directory/'preflight-sha256.txt').read_text().splitlines():
    digest,path=line.split(None,1);path=path.strip();assert verify_source(path)==digest,path;RECEIPT['source_sha256'][path]=digest
  gate('source-owned reevaluation '+name)
  result=a3.analyse(directory,SOURCE/'tools/chemistry-audit/source-fleets-3',family) if v3 else (a1.analyse(directory,SOURCE/'tools/chemistry-audit/source-fleets',family) if kind=='v1' else (a6.analyse(directory) if family=='sixth' else a7.analyse(directory)))
  result=report_json_transport(result)
  write(folder/'source-owned-recomputed-law-checks.json',result)
  lawfiles=[x for x in directory.glob('*law-checks.json')];assert len(lawfiles)==1;reported=load(lawfiles[0]);assert result==reported,'source-owned result differs'
  assert result['unmet']==0 and result['passed']==len(result['checks']) and all(x['passed'] is True for x in result['checks'])
  row=dict(family=family,generation=kind,artifact_id=metadata['id'],artifact_name=name,artifact_zip_sha256=sha(folder/'artifact.zip'),artifact_api_sha256=sha(folder/'artifact-api.json'),archive_members=len(members),member_sha256=members,cases=expected_count,passed_law_checks=result['passed'],law_kinds=dict(collections.Counter(x['kind'] for x in result['checks'])),raw_evidence_sha256=raw,event_counts=dict(counts),original_law_receipt_sha256=sha(lawfiles[0]),source_owned_recomputed_law_receipt_sha256=sha(folder/'source-owned-recomputed-law-checks.json'),independent_raw_provenance_contracts_pass=True,source_owned_laws_pass=True)
  RECEIPT['families'].append(row);checkpoint();print(json.dumps({'accepted':family,'generation':kind,'cases':expected_count,'laws':result['passed']}),flush=True)
 gate('final source identity guards')
 for path,digest in RECEIPT['source_sha256'].items():assert verify_source(path)==digest
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=SOURCE).decode().strip()==HEAD
 assert subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=SOURCE)==b''
 RECEIPT.update(all_checks_pass=True,total_cases=sum(x['cases'] for x in RECEIPT['families']),total_passed_law_checks=sum(x['passed_law_checks'] for x in RECEIPT['families']),families_complete=len(RECEIPT['families']),reviewer_sha256=sha(Path(__file__)),resources_after=gate('final checkpoint'),limitations=['Accepted acid/base receipt bytes and identity trusted externally; its archive and laws were not re-read on this worker. Host execution fetches each remaining artifact once, including its own distillation copy; local cached bytes remain protected.','Exact source-owned analyzer algorithms rerun on saved evidence; report comparison uses the original analyzer JSON transport (integer evidence keys become strings), rejects key collisions, and preserves all values. Independent raw integrity/forecast/provenance checks are separate, not an independent scientific oracle.','Recorded commit and binary/lock envelopes tied to own accepted release artifact; sourcev1/v3 family summary does not itself capture actual lock or executable bytes.','Own complete source/tree/artifact API/checkouts proven by accepted release receipt; no whole family runner/native source reread or CLI rerun.','Frozen summaries capture the ignored root lock digest, not its bytes; accepted own release supplies the independent lock-byte binding. Git does not track root Cargo.lock.','Cases are frozen engineering contracts/source-informed laws, not new blind experiments or empirical chemical calibration.','This receipt covers only10v1+5v3+frozen sixth/seventh; v2 and actual aggregate job logs require separate closure.'])
 RECEIPT['newly_reviewed_cases']=RECEIPT['total_cases']-24;RECEIPT['newly_reviewed_law_checks']=RECEIPT['total_passed_law_checks']-32
 write(OUT/'review.json',RECEIPT);print('REMAINING_FLEET_REVIEW_JSON='+json.dumps({k:RECEIPT[k] for k in ['all_checks_pass','run','candidate','families_complete','total_cases','total_passed_law_checks','newly_reviewed_cases','newly_reviewed_law_checks','accepted_partial_receipt_sha256','accepted_own_release_receipt_sha256','reviewer_sha256','cli_executed','scientific_calibration_accepted','limitations']},sort_keys=True),flush=True);print(json.dumps({'all_checks_pass':True,'families':17,'cases':RECEIPT['total_cases'],'laws':RECEIPT['total_passed_law_checks'],'receipt_sha256':sha(OUT/'review.json')}),flush=True)
except Exception as error:
 write(OUT/'first-failure.json',{'type':type(error).__name__,'message':str(error),'accepted_families':[x['family'] for x in RECEIPT['families']],'cli_executed':False});checkpoint();raise
