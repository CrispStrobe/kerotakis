"""Frozen prospective unit checks for an already compiled registry generator."""
import argparse,copy,hashlib,json,os,pathlib,re,subprocess
CASES=[('canonical_registry_supported',None,None,1.0,False,False,False),('cp_alias_values_supported','molar_heat_capacity','J.mol-1.K-1',1.0,False,False,False),('density_alias_values_supported','mass_density','g/cm3',1.0,False,False,False),('scaled_mass_values_normalized','molar_mass','kg/mol',1000.0,False,False,False),('scaled_density_values_normalized','mass_density','kg/m3',.001,False,False,False),('scaled_cp_values_normalized','molar_heat_capacity','kJ/(mol.K)',1000.0,False,False,False),('wrong_mass_symbol_refuses','molar_mass','J/mol',1.0,False,False,True),('wrong_density_dimension_refuses','mass_density','g/mL',1.0,True,False,True),('unknown_cp_symbol_refuses','molar_heat_capacity','unreviewed-unit',1.0,False,False,True),('converted_mass_overflow_refuses','molar_mass','kg/mol',1.0,False,True,True)]
def values(text,key):
 match=re.search(r'key:\s*"'+re.escape(key)+r'",',text);assert match,key
 row=text[match.end():].split('    SpeciesData {',1)[0]
 return {field:float(re.search(r'^\s*'+field+r':\s*([^,]+),$',row,re.M)[1]) for field in ['molar_mass','density','heat_capacity']}
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--binary',type=pathlib.Path,required=True);parser.add_argument('--registry',type=pathlib.Path,required=True);parser.add_argument('--out',type=pathlib.Path,required=True);args=parser.parse_args();binary=args.binary.resolve();doc=json.loads(args.registry.read_text());results=[]
 source_values={r['property']:r['quantity']['value'] for r in doc['phase_thermodynamics'] if r['species_id']=='MgSO4'};expected={'molar_mass':source_values['molar_mass'],'density':source_values['mass_density'],'heat_capacity':source_values['molar_heat_capacity']};zero_keys=[r['species_id'] for r in doc['phase_thermodynamics'] if r['property']=='molar_heat_capacity' and r['quantity']['value']==0.0]
 for name,prop,symbol,factor,wrong,overflow,refusal in CASES:
  case=args.out/name;case.mkdir(parents=True,exist_ok=False);manifest=case/'crates'/'kerotakis-core';manifest.mkdir(parents=True);data=case/'data'/'registry';data.mkdir(parents=True);out=case/'generated';out.mkdir();probe=copy.deepcopy(doc)
  if prop:
   rows=[r for r in probe['phase_thermodynamics'] if r['species_id']=='MgSO4' and r['property']==prop];assert len(rows)==1
   q=rows[0]['quantity'];q['value']=float.fromhex('0x1.fffffffffffffp+1023') if overflow else q['value']/factor;q['unit']['symbol']=symbol
   if wrong:q['unit']['dimension']='molar_mass'
  payload=json.dumps(probe).encode();(data/'registry-source-v1.json').write_bytes(payload);proc=subprocess.run([str(binary)],cwd=manifest,env=dict(os.environ,CARGO_MANIFEST_DIR=str(manifest.resolve()),OUT_DIR=str(out.resolve())),capture_output=True,timeout=15);(case/'stdout.log').write_bytes(proc.stdout);(case/'stderr.log').write_bytes(proc.stderr)
  if refusal:
   assert proc.returncode!=0 and b'MgSO4:' in proc.stderr and b'unit' in proc.stderr and prop.encode() in proc.stderr,(name,proc.stderr);assert not (out/'species_generated.rs').exists();actual=None
  else:
   assert proc.returncode==0,proc.stderr;text=(out/'species_generated.rs').read_text();actual=values(text,'MgSO4')
   assert all(abs(actual[k]-v)<=max(abs(v),1.0)*1e-12 for k,v in expected.items()),(name,actual,expected)
   assert len(re.findall(r'^\s*key:',text,re.M))==len(doc['identities'])
   assert all(values(text,key)['heat_capacity']==0.0 for key in zero_keys)
   for file in ['solvent_generated.rs','materials_generated.rs']:assert (out/file).stat().st_size>0
  results.append({'case':name,'passed':True,'exit_code':proc.returncode,'runtime_values':actual,'input_sha256':hashlib.sha256(payload).hexdigest(),'stdout_sha256':hashlib.sha256(proc.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(proc.stderr).hexdigest()})
 report={'passed':len(results),'cases':results,'zero_cp_rows_preserved':len(zero_keys),'expected_canonical_values':expected,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'registry_sha256':hashlib.sha256(args.registry.read_bytes()).hexdigest()};(args.out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'passed':len(results)}))
if __name__=='__main__':main()
