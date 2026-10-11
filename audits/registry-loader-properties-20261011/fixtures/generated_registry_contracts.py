"""Prospective generated-registry checks using an already compiled build script."""
import argparse,copy,hashlib,json,os,pathlib,subprocess
CASES=[('existing_registry_and_zero_cp_supported',None,None),('negative_molar_mass_refuses','molar_mass',-1.0),('zero_molar_mass_refuses','molar_mass',0.0),('negative_density_refuses','mass_density',-1.0),('zero_density_refuses','mass_density',0.0),('negative_heat_capacity_refuses','molar_heat_capacity',-1.0)]
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--binary',type=pathlib.Path,required=True);parser.add_argument('--registry',type=pathlib.Path,required=True);parser.add_argument('--out',type=pathlib.Path,required=True);args=parser.parse_args()
 doc=json.loads(args.registry.read_text());results=[];binary=args.binary.resolve()
 for name,prop,value in CASES:
  case=args.out/name;case.mkdir(parents=True,exist_ok=False);manifest=case/'crates'/'kerotakis-core';manifest.mkdir(parents=True);data=case/'data'/'registry';data.mkdir(parents=True);out=case/'generated';out.mkdir();probe=copy.deepcopy(doc)
  if prop:
   rows=[r for r in probe['phase_thermodynamics'] if r['species_id']=='MgSO4' and r['property']==prop];assert rows,'Missing target record'
   for row in rows:row['quantity']['value']=value
  payload=json.dumps(probe).encode();(data/'registry-source-v1.json').write_bytes(payload)
  proc=subprocess.run([str(binary)],cwd=manifest,env=dict(os.environ,CARGO_MANIFEST_DIR=str(manifest.resolve()),OUT_DIR=str(out.resolve())),capture_output=True,timeout=15)
  (case/'stdout.log').write_bytes(proc.stdout);(case/'stderr.log').write_bytes(proc.stderr)
  diagnostic=None
  if prop:
   reason='must be finite and non-negative' if prop=='molar_heat_capacity' else 'must be finite and positive';diagnostic=f'MgSO4: {prop} {reason}'
   assert proc.returncode!=0 and diagnostic.encode() in proc.stderr,(name,proc.returncode,proc.stderr)
   assert not (out/'species_generated.rs').exists(),name
  else:
   assert proc.returncode==0,proc.stderr
   for file in ['species_generated.rs','solvent_generated.rs','materials_generated.rs']:assert (out/file).stat().st_size>0,file
  results.append({'case':name,'passed':True,'exit_code':proc.returncode,'expected_diagnostic':diagnostic,'input_sha256':hashlib.sha256(payload).hexdigest(),'stdout_sha256':hashlib.sha256(proc.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(proc.stderr).hexdigest()})
 report={'cases':results,'passed':len(results),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'registry_sha256':hashlib.sha256(args.registry.read_bytes()).hexdigest()};(args.out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'passed':len(results)}))
if __name__=='__main__':main()
