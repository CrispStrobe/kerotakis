"""Saved-only original TC008 qualification; no simulator/binary/provider execution."""
import argparse, hashlib, json, math, struct, subprocess
from pathlib import Path
from decimal import Decimal as D, getcontext
getcontext().prec=45
S='ceff11dc9d9f86efdbe33bb33af4fe45807ddb22'; F='46e1e63afeea33cd6cf14189179308e92324e7be'; M='3162ab3f5fed6336a93976ba23dc58f3e0da4675'
def digest(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--focus',type=Path,required=True);p.add_argument('--reproduced',type=Path,required=True);p.add_argument('--production',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();assert not a.out.exists()
 def git(*args):return subprocess.check_output(['git','--no-replace-objects',*args],cwd=a.repo)
 def blob(s,p):return git('show',s+':'+p)
 def h(b):return hashlib.sha256(b).hexdigest()
 e=a.focus/'collection-phase-resolution/output/phase-resolution-focus-evidence';trace=e/'tc008-trace/tc008-trace.jsonl';rows=[json.loads(x,parse_float=D)for x in trace.read_bytes().splitlines()]
 intake=a.repo/'audits/new-experiments-201-20261016/intake.json';original=next(x for x in json.loads(intake.read_bytes())if x['original_id']=='batch2:tc-008');script=rows[0]['script'];assert h(script.encode())==original['script_sha256']=='e4b74c4bf0faf8dcbcf0adf6957df668ae62b3a87ad716293e92c0ac4347dfd5'
 assert [x.strip()for x in script.splitlines()if x.strip()and not x.startswith('#')]==[x['command']for x in original['commands']]
 assert len(original['commands'])==9
 receipt=json.loads((a.reproduced/'INDEPENDENT-FOCUSED-ACCEPTED.json').read_bytes());assert receipt['source']==S and receipt['actual_controls']==30 and receipt['actual_test_executables']==8 and receipt['saved_math_exit_status']==0
 ledger=json.loads((a.reproduced/'actual-stage-ledger.json').read_bytes());assert ledger['trace_sha256']==digest(trace)and ledger['source']==S and ledger['observed_energy_contract_holds']is True
 assert ledger['original_script_sha256']==original['script_sha256']
 assert [x['command_index']for x in ledger['whole_commands']]==list(range(3,12))
 for x in ledger['whole_commands']:
  assert abs(D(x['whole_command_residual_J']))<=D('.001')
  assert all(abs(D(v))<=D('1e-10')for v in x['owned_atom_charge_residual'].values())
 for x in ledger['intervening_phase_remainders']:
  assert abs(D(x['closed_adiabatic_stage_residual_J']))<=D('.001')
  assert all(abs(D(v))<=D('1e-10')for v in x['atom_charge_delta'].values())
 exits={x['command_index']:x for x in rows if x['kind']=='command_exit'};assert set(exits)==set(range(3,12))
 for row in rows:
  if 'bench'in row:
   b=row['bench'];assert not b['broken_vessels']and not b['spills']and len(b['vessels'])==1
   vs=b['vessels']
  elif 'state'in row:vs=[row['state']['vessel']]
  else:continue
  for v in vs:
   assert v['elapsed_seconds']==0 and v['thermal_mode']=='adiabatic'and v['headspace']=={'boundary':'open'}
   assert v['excess_enthalpy_j']==0 and not any(v['foam'].values())and v['pending_co2_transfer_mol']==0 and v['nuclides']=={'inventory':{}}
   assert not any(v.get(k)for k in ['surfaces','unresolved_materials','material_objects','electrodes','exchanges','adsorbed','solid_solutions','soap_scum','surface_particles','emulsion','charred_materials','surface_colours'])
   for q in v['contents']:assert q['species']in ['water','NaCl','Na+','Cl-']and q['phase']in ['liquid','solid','aqueous']and q['moles']>=0 and q['moles'].is_finite()
 def vessel(i):return exits[i]['bench']['vessels'][0]
 def pool(v,k,ph):return sum((q['moles']for q in v['contents']if q['species']==k and q['phase']==ph),D(0))
 before,after=vessel(7),vessel(8); bi=pool(before,'water','solid');bl=pool(before,'water','liquid');ai=pool(after,'water','solid');al=pool(after,'water','liquid')
 def measured(i,instrument):
  es=exits[i]['events'];assert len(es)==1;ev=es[0];assert ev['event']=='measured'and ev['instrument']==instrument and ev['vessel']==0;return ev['value']
 t0=measured(6,'thermometer');t1=measured(9,'thermometer');assert t0==0 and t1<0 and abs(t1+ D('273.15')-after['temperature'])<D('1e-10')
 assert after['temperature']<before['temperature']and ai/(ai+al)<bi/(bi+bl)and ai>0 and al>0
 assert abs((bi-ai)-(al-bl))<=D('1e-10')
 salt=exits[8]['operator']['moles'];assert pool(after,'NaCl','solid')==pool(after,'NaCl','aqueous')==0
 assert abs(pool(after,'Na+','aqueous')-salt)<D('1e-10')and abs(pool(after,'Cl-','aqueous')-salt)<D('1e-10')
 for i in [9,10,11]:assert vessel(i)==after,'A readout changed physical state'
 events=[ev for x in rows for ev in x.get('events',[])];assert not any(ev['event']in ['solver_failed','gas_evolved','gas_absorbed','evaporated','released','vented']for ev in events)
 assert after['temperature']>D('252')
 # Actual whole-tree identity is stronger than receipt booleans. Only one
 # test-only same-f64 literal and one test snapshot/comments differ from ceff.
 diff=git('diff','--name-only',S,F).decode().splitlines();special=['crates/kerotakis-phreeqc/src/aqueous.rs','crates/kerotakis-phreeqc/tests/colligative_numbers.rs'];assert set(diff)==set(special+['docs/'+x for x in ['STATUS.md','improvement-agenda-20261014.md','next-agent-handoff.md','peroxide-medium-and-i18n-20261019.md','post-hydrolysis-gap-lanes-20261019.md','scientific-coverage-expansion-20261020.md','unifac-input-admission-20261019.md','wrap-up-and-open-lanes-20261019.md']])
 aq0=blob(S,special[0]);aq1=blob(F,special[0]);old=b'1.2345678912345678';new=b'1.234_567_891_234_568';assert aq0.count(old)==1 and aq0.replace(old,new)==aq1;assert b'#[cfg(test)]'in aq0[:aq0.index(old)]
 assert struct.pack('>d',float(old))==struct.pack('>d',float(new))==bytes.fromhex('3ff3c0ca42d8b4db')
 # Test comments may differ; require the exact sole executable assertion change.
 def uncomment(raw):return b'\n'.join(line.split(b'//',1)[0].strip()for line in raw.splitlines()if line.split(b'//',1)[0].strip())
 c0=uncomment(blob(S,special[1]));c1=uncomment(blob(F,special[1]));assert c0.count(b'(freezing_c + 0.3516).abs() < 0.004,')==1;assert c0.replace(b'(freezing_c + 0.3516).abs() < 0.004,',b'(freezing_c + 0.3468).abs() < 0.0004,')==c1
 assert git('diff','--name-only',F,M)==b'' and git('rev-parse',F+'^{tree}')==git('rev-parse',M+'^{tree}')
 rawprod=(a.production/'PRODUCTION852-ACCEPTED.json').read_bytes();prod=json.loads(rawprod);assert prod['source']==prod['engine']['source_head']==M and prod['actual11API_and64browser_controls_pass']is True and prod['scientific_selections']==0
 assert prod['engine']['binary_sha256']=='fafa5569b7fe4ee4279cf6d6494e0cb38cc115aa1166a490c4d7ed58f7242e46'
 assert prod['engine']['cargo_lock_sha256']=='41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb'
 whole=next(x for x in ledger['whole_commands']if x['command_index']==8)
 metrics={'before_temperature_K':str(before['temperature']),'after_temperature_K':str(after['temperature']),'thermometer_before_C':str(t0),'thermometer_after_C':str(t1),'ice_before_mol':str(bi),'ice_after_mol':str(ai),'liquid_before_mol':str(bl),'liquid_after_mol':str(al),'ice_fraction_before':str(bi/(bi+bl)),'ice_fraction_after':str(ai/(ai+al)),'melted_mol':str(bi-ai),'dissolved_salt_mol':str(salt),'undissolved_salt_mol':'0','reference_endothermic_dissolution_J':str(salt*D('3880')),'incoming_salt_standard_reference_H_J':whole['incoming_declared_species_H_J'],'incoming_salt_sensible298_to273_J':str(D(whole['incoming_declared_species_H_J'])+salt*D('3880')),'whole_salt_command_residual_J':whole['whole_command_residual_J'],'max_exclusive_phase_residual_J':str(max(abs(D(x['closed_adiabatic_stage_residual_J']))for x in ledger['intervening_phase_remainders'])),'maximum_whole_command_atom_charge_residual_mol':str(max(abs(D(v))for x in ledger['whole_commands']for v in x['owned_atom_charge_residual'].values()))}
 result={'state':'EARNED_ORIGINAL_QUALIFICATION_PROPOSAL_NOT_CATALOG_EDIT','original_id':'batch2:tc-008','proposed_catalog_total_if_parent_selects':335,'original_forecast':original['original_expected_result'],'original_criteria':original['original_acceptance_checks'],'original_limits':original['original_limitations'],'script_sha256':original['script_sha256'],'commands_exactly_original':9,'criteria':{'TC-008-C1':{'supported':True,'coordinates':'trace command_exit7/8 phase inventory; measured command_exit6/9; final11 unchanged','reason':'Actual cooling and reduced owned ice fraction are both observed; no event label is substituted for material state.'},'TC-008-C2':{'supported':True,'coordinates':'trace command_enter/exit8; all native/displacement/phase adapters and every wholecommand; independent392c ledger','reason':'All salt is dissolved Na+/Cl−; incoming cold solid and standard dissolution enthalpy are explicitly priced under a single reference, phase Cp/fusion and all intermediate stages retained; no hidden sink/gas/lost material.'}},'metrics':metrics,'actual_focus_source':S,'actual_focus_tree':receipt['tree'],'actual_focus_lock_sha256':receipt['actual_lock_sha256'],'actual_trace_binary_sha256':next(x['actual_binary_sha256']for x in receipt['authenticated_targets']if x['target']=='tc008_energy_trace'),'actual_source_equivalence':{'ceff_to_phase_candidate_paths':diff,'aqueous_change_test_only_same_f64_bits':'3ff3c0ca42d8b4db','colligative_change_test_only_tenfold_tighter_snapshot':True,'phase_candidate_and_actual_merge_tree_identical':True,'actual_merge_source':M,'actual_merge_tree':git('rev-parse',M+'^{tree}').decode().strip()},'delivery_receipt_sha256':h(rawprod),'delivered_native_distinct_from_trace':prod['engine'],'bindings':{'trace_sha256':digest(trace),'actual_intake_sha256':digest(intake),'closed_manifest_sha256':digest(e/'manifest.json'),'saved_reader_sha256':digest(e/'external-saved-reader/review_trace.py'),'new_independent_ledger_sha256':digest(a.reproduced/'actual-stage-ledger.json'),'new_closed_focus_receipt_sha256':digest(a.reproduced/'INDEPENDENT-FOCUSED-ACCEPTED.json')},'limits':['This is supported native-model coupling and declared enthalpy bookkeeping, not measured/calibrated nonideal subzero brine calorimetry.','Actual260.795725K observation is above declared252K cap; eutectic/salt precipitation and cap/refusal energy remain separate uncovered lanes.','Actual original replay is ceff observer binary+668lock. Production3162/nativefafa+41bb passed11API+64controls; originalTC008 was NOT replayed on that deployed binary. Identical runtime/data source is proved, dependency/build identities remain distinct.','The old236.388609J residual and reference-corrected128.316712J failure are immutable separate evidence; reference correction alone was not a repair.','Conductivity was observed but no quantitative conductivity calibration is earned.','Originalforecast/criteria unchanged; no catalog selection assigned by this review.'],'product_engine_provider_build_calls':0,'catalog_edited':False}
 a.out.mkdir();(a.out/'adjudication.json').write_text(json.dumps(result,indent=2)+'\n');(a.out/'ceff-to-phase.diff').write_bytes(git('diff',S,F));print(json.dumps(result,indent=2))
if __name__=='__main__':main()
