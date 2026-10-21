#!/usr/bin/env python3
"""Recompute original TC008 saved math/criteria; never execute the simulator.

--focus optionally authenticates actual private trace/producer-lock/executable
bytes too. Without it, these are explicitly published prior-authentication
bindings, not freshly downloaded/rehashed binary claims.
"""
import argparse,gzip,hashlib,json,struct,subprocess,sys,tempfile
from decimal import Decimal as D
from pathlib import Path
S='ceff11dc9d9f86efdbe33bb33af4fe45807ddb22';F='46e1e63afeea33cd6cf14189179308e92324e7be';M='3162ab3f5fed6336a93976ba23dc58f3e0da4675'
def sha(p):
 with p.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--focus',type=Path);p.add_argument('--out-dir',type=Path,required=True);a=p.parse_args();assert not a.out_dir.exists()
 r=a.repo.resolve();root=r/'audits/target468-20261016';here=root/'TC008-qualified-original-review-20261021';prior=root/'phase-resolution-30pass-full-issued-20261021';assessment=json.loads((here/'assessment.json').read_bytes());parent=json.loads((here/'parent-adjudication.json').read_bytes());assert sha(here/'assessment.json')==parent['assessment_sha256'];assert parent['parent_own_closed127member_eightbinary_authentication_exit_status']==0 and parent['parent_own_392c_ledger_byte_identical']is True and parent['parent_own_qualifier_adjudication_byte_identical']is True
 binds=json.loads((here/'raw-bindings.json').read_bytes())
 for name,want in binds['public_files_sha256'].items():assert sha(r/name)==want,name
 trace=prior/'tc008-trace/tc008-trace.jsonl';assert sha(trace)==assessment['bindings']['trace_sha256'];rows=[json.loads(x,parse_float=D)for x in trace.read_bytes().splitlines()]
 o=next(x for x in json.loads((r/'audits/new-experiments-201-20261016/intake.json').read_bytes())if x['original_id']=='batch2:tc-008');assert o['script_sha256']==assessment['script_sha256']==hashlib.sha256(rows[0]['script'].encode()).hexdigest();assert o['original_expected_result']==assessment['original_forecast']and o['original_acceptance_checks']==assessment['original_criteria']and o['original_limitations']==assessment['original_limits'];assert len(o['commands'])==9
 exits={x['command_index']:x for x in rows if x['kind']=='command_exit'};assert set(exits)==set(range(3,12))
 def v(i):return exits[i]['bench']['vessels'][0]
 def n(i,ph):return sum((x['moles']for x in v(i)['contents']if x['species']=='water'and x['phase']==ph),D(0))
 before,after=v(7),v(8);assert after['temperature']<before['temperature']and after['temperature']>D('252');assert n(8,'solid')/(n(8,'solid')+n(8,'liquid'))<n(7,'solid')/(n(7,'solid')+n(7,'liquid'));assert abs(n(7,'solid')-n(8,'solid')-(n(8,'liquid')-n(7,'liquid')))<=D('1e-10')
 for i in [9,10,11]:assert v(i)==after
 for i in [6,9]:
  ev=exits[i]['events'];assert len(ev)==1 and ev[0]['event']=='measured'and ev[0]['instrument']=='thermometer'and ev[0]['vessel']==0 and abs(ev[0]['value']+D('273.15')-v(i)['temperature'])<D('1e-10')
 salt=exits[8]['operator']['moles'];assert all(x['species']in ['water','Na+','Cl-']for x in after['contents'])
 for key in ['Na+','Cl-']:assert abs(sum((x['moles']for x in after['contents']if x['species']==key and x['phase']=='aqueous'),D(0))-salt)<=D('1e-10')
 reader=root/'brine-phase-baseline-and-TC008-ledger-20261020/review_trace.py';assert sha(reader)=='392c7895df7b0c257ef5a198245c7c470f180373c14d26464fb58f331426f81e'
 a.out_dir.mkdir();args=[sys.executable,str(reader),'--repo',str(r),'--trace',str(trace),'--source',S,'--out',str(a.out_dir/'actual-stage-ledger.json')]
 with(a.out_dir/'saved-math.stdout').open('xb')as out,(a.out_dir/'saved-math.stderr').open('xb')as err:run=subprocess.run(args,stdout=out,stderr=err,timeout=180)
 assert run.returncode==0;ledger=json.loads((a.out_dir/'actual-stage-ledger.json').read_bytes());assert (a.out_dir/'actual-stage-ledger.json').read_bytes()==(prior/'actual-stage-ledger.json').read_bytes();assert ledger['observed_energy_contract_holds']is True
 assert len(ledger['whole_commands'])==9
 for x in ledger['whole_commands']:
  assert abs(D(x['whole_command_residual_J']))<=D('.001')and all(abs(D(q))<=D('1e-10')for q in x['owned_atom_charge_residual'].values())
 for x in ledger['intervening_phase_remainders']:assert abs(D(x['closed_adiabatic_stage_residual_J']))<=D('.001')and all(abs(D(q))<=D('1e-10')for q in x['atom_charge_delta'].values())
 def git(*args):return subprocess.check_output(['git','--no-replace-objects',*args],cwd=r)
 diff=git('diff','--name-only',S,F).decode().splitlines();assert diff==assessment['actual_source_equivalence']['ceff_to_phase_candidate_paths'];aq='crates/kerotakis-phreeqc/src/aqueous.rs';old=git('show',S+':'+aq);new=git('show',F+':'+aq);assert old.count(b'1.2345678912345678')==1 and old.replace(b'1.2345678912345678',b'1.234_567_891_234_568')==new and struct.pack('>d',1.2345678912345678)==struct.pack('>d',1.234_567_891_234_568)==bytes.fromhex('3ff3c0ca42d8b4db')
 def code(raw):return b'\n'.join(l.split(b'//',1)[0].strip()for l in raw.splitlines()if l.split(b'//',1)[0].strip())
 path='crates/kerotakis-phreeqc/tests/colligative_numbers.rs';assert code(git('show',S+':'+path)).replace(b'(freezing_c + 0.3516).abs() < 0.004,',b'(freezing_c + 0.3468).abs() < 0.0004,')==code(git('show',F+':'+path));assert git('diff','--name-only',F,M)==b''and git('rev-parse',F+'^{tree}')==git('rev-parse',M+'^{tree}')
 prod=json.loads((root/'phase-production-accepted-20261021/PRODUCTION852-ACCEPTED.json').read_bytes());assert prod['source']==M and prod['actual11API_and64browser_controls_pass']is True and prod['engine']==assessment['delivered_native_distinct_from_trace']
 binary_rehashed=False
 if a.focus:
  e=a.focus/'collection-phase-resolution/output/phase-resolution-focus-evidence';assert sha(e/'tc008-trace/tc008-trace.jsonl')==sha(trace);assert sha(e/'actual-producer-Cargo.lock')==assessment['actual_focus_lock_sha256'];meta=json.loads((e/'tc008_energy_trace-binary.json').read_bytes());assert sha(e/'tc008_energy_trace.gz')==meta['gzip_sha256'];h=hashlib.sha256();size=0
  with gzip.open(e/'tc008_energy_trace.gz','rb')as f:
   while chunk:=f.read(262144):h.update(chunk);size+=len(chunk)
  assert h.hexdigest()==assessment['actual_trace_binary_sha256']==meta['sha256']and size==meta['bytes'];binary_rehashed=True
 result={'original_id':o['original_id'],'original_nine_commands_unchanged':True,'original_criteria':['TC-008-C1','TC-008-C2'],'original_forecast_and_limits_unchanged':True,'literal_criteria_supported_in_declared_model_domain':True,'all_nine_whole_and_exclusive_phase_ledgers_close':True,'saved_392c_ledger_byte_identical':True,'same_runtime_source_data_proved':True,'private_binary_bytes_rehashed_by_this_invocation':binary_rehashed,'production_original_TC008_replay_claim':False,'catalog_selection_already_parent_approved':335,'new_product_engine_API_build_provider_calls':0,'empirical_calibration_claim':False}
 (a.out_dir/'assessment.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
if __name__=='__main__':main()
