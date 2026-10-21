"""Narrow offline actual-release receipt/archive binder. No CONFIG/approval/issue.

This checks consistency of externally authenticated receipts and rehashes the
actual app archive projection. It does not replace production/API/raw readers.
"""
import argparse,hashlib,importlib.util,json,os,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
CANDIDATE='62f647275fd800d4992f5516651dfc9e61f47a05'
def sha(path):
 with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def require(value,message):
 if not value:raise ValueError(message)
def loads(raw):
 def pairs(rows):
  value={}
  for key,item in rows:require(key not in value,'duplicate JSON key');value[key]=item
  return value
 return json.loads(raw,object_pairs_hook=pairs,parse_constant=lambda s:(_ for _ in ()).throw(ValueError('nonfinite JSON constant: '+s)))
def pinned(path,want,maximum=8*1024**2):
 require(type(want)is str and re.fullmatch('[a-f0-9]{64}',want),'malformed external digest')
 path=Path(path);require(path.is_file()and not path.is_symlink()and path.stat().st_size<=maximum,'missing/linked/oversize pinned small input')
 raw=path.read_bytes();require(hashlib.sha256(raw).hexdigest()==want,'external input digest differs');return raw

def imported(name,relative,closure):
 path=ROOT/'frozen-inputs'/relative;require(sha(path)==closure['files'][relative]['sha256'],'frozen helper differs')
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 for name in ['receipt-manifest','integration','artifact','template','original-protocols']:
  parser.add_argument('--'+name,type=Path,required=True);parser.add_argument('--'+name+'-sha256',required=True)
 parser.add_argument('--runtime-source',required=True);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
 require(not args.out.exists(),'fresh unissued output required')
 require(re.fullmatch('[a-f0-9]{40}',args.runtime_source),'actual runtime source must be explicit')
 available=int(next(row.split()[1]for row in Path('/proc/meminfo').read_text().splitlines()if row.startswith('MemAvailable:')))//1024
 require(available>=3200 and os.getloadavg()[0]<=3.5,'saved archive resource gate closed')
 closure=loads((ROOT/'FROZEN-INPUTS.json').read_bytes())
 for name,row in closure['files'].items():
  rel=Path(name);require(not rel.is_absolute()and'..'not in rel.parts,'unsafe static source alias')
  path=ROOT/'frozen-inputs'/rel;require(path.is_file()and not path.is_symlink()and path.stat().st_size==row['bytes']and sha(path)==row['sha256'],'static source differs: '+name)
 metadata=loads(pinned(args.receipt_manifest,args.receipt_manifest_sha256));old=imported('receipt_consistency_only','private64-saved-readers/release_binding.py',closure)
 require(set(metadata['files'])==set(old.RECEIPTS),'exact eight actual receipt authorities required')
 raw={}
 for name,row in metadata['files'].items():
  require(set(row)=={'path','sha256'}and Path(row['path']).is_absolute(),'explicit private receipt path/hash required')
  raw[name]=pinned(row['path'],row['sha256']);loads(raw[name])
 integration=loads(pinned(args.integration,args.integration_sha256));shape=imported('prospective854_shape','delivery_binding854.py',closure)
 actual=shape.verify_integration_shape(integration);require(actual==args.runtime_source and integration['head']==CANDIDATE,'actual normal854 merge identity differs')
 tree=integration['actual_tree']
 template=loads(pinned(args.template,args.template_sha256));original=pinned(args.original_protocols,args.original_protocols_sha256)
 binding,prior=old.checked_binding(template['binding'],raw,actual,tree,original)
 require(binding['origin']=='https://kerotakis-web-payload.vercel.app','unreviewed product origin')
 verified=loads(raw['verified-release.json']);require(sha(args.artifact)==args.artifact_sha256==verified['artifact_sha256'],'actual artifact external digest differs')
 projector=imported('exact_optics_archive_projector','optics18/project_app_assets.py',closure)
 projection=projector.project(args.artifact,args.artifact_sha256,Path(metadata['files']['verified-release.json']['path']),metadata['files']['verified-release.json']['sha256'],actual)
 require(projection['tree']==tree,'actual app projection tree differs')
 canonical=lambda value:json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()+b'\n'
 closure_raw=(ROOT/'frozen-inputs/optics18/program-closure.json').read_bytes();programs=loads(closure_raw)
 for name,want in programs['files_sha256'].items():
  require(re.fullmatch('[A-Za-z0-9_.-]+',name)is not None,'unsafe optics alias')
  require(sha(ROOT/'frozen-inputs/optics18'/name)==want,'frozen optics program changed')
 optics=dict(origin=binding['origin'],expected_engine_source=actual,native_binary_sha256=prior['native_binary_sha256'],actual_producer_lock_sha256=prior['native_actual_producer_lock_sha256'],artifact_sha256=args.artifact_sha256,verified_release_sha256=prior['verified_artifact_receipt_sha256'],app_assets={name:row['sha256']for name,row in projection['app_assets'].items()},frontend_assets=prior['frontend_assets'],client_sha256=prior['client_sha256'])
 projection_raw=canonical(projection);optics_raw=canonical(optics)
 expected=dict(source=actual,tree=tree,artifact_sha256=args.artifact_sha256,api11_receipt_sha256=prior['API_receipt_sha256'],optics_binding_sha256=hashlib.sha256(optics_raw).hexdigest(),app_projection_sha256=hashlib.sha256(projection_raw).hexdigest(),optics_program_closure_sha256=hashlib.sha256(closure_raw).hexdigest())
 authority={'state':'ACTUAL_INPUT_CONSISTENCY_BOUND_NOT_AUTHORIZED_OR_CONFIGURED','source':actual,'tree':tree,'reviewed_candidate_source':CANDIDATE,'external_integration_sha256':args.integration_sha256,'external_receipt_manifest_sha256':args.receipt_manifest_sha256,'actual_receipt_sha256':{name:row['sha256']for name,row in metadata['files'].items()},'template_sha256':args.template_sha256,'original_protocols_sha256':args.original_protocols_sha256,'artifact_sha256':args.artifact_sha256,'static_closure_sha256':sha(ROOT/'FROZEN-INPUTS.json'),'binder_sha256':sha(Path(__file__)),'configuration_generated':False,'parent_approval_or_issue_generated':False,'production_and_API_invocation_provenance_still_requires_parent_authentication':True}
 output={'release-binding.json':canonical(binding),'heat-release-binding.json':canonical(dict(binding,reviewed_candidate_source=CANDIDATE)),'optics-binding.json':optics_raw,'app-projection.json':projection_raw,'EXPECTED_RELEASE.json':canonical(expected),'prior-release.json':canonical(prior),'BOUND-INPUTS-NOT-CONFIGURED.json':canonical(authority)}
 # Validate everything before creating a fresh output; retain partials on real I/O failure.
 args.out.mkdir(mode=0o700)
 for name,data in output.items():
  with(args.out/name).open('xb')as stream:stream.write(data)
  (args.out/name).chmod(0o600)
 print(json.dumps({'state':authority['state'],'files_written':list(output),'no_calls_or_issue':True}))
if __name__=='__main__':main()
