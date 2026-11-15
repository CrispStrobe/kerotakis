"""Read exact source API constraints only; no chemistry or native execution."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
SOURCE='52404e01c7bed8498516d76829e6feb87911f544'
FILES=['crates/kerotakis-core/src/'+n+'.rs' for n in ['ops','vessel','bench','clock','script']]+['crates/kerotakis-phreeqc/src/'+n+'.rs' for n in ['aqueous','inventory']]
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 source={f:subprocess.check_output(['git','-C',str(a.repo),'show',SOURCE+':'+f]).decode()for f in FILES}
 ops,vessel,bench,clock,script,aq,inventory=[source[f]for f in FILES]
 checks={
 'operator_only_vessel_and_pressure':bool(re.search(r'Sweep\s*\{\s*vessel: VesselId,\s*pressure: Pascal\s*\}',ops)),
 'headspace_swept_only_pressure':bool(re.search(r'Swept\s*\{\s*pressure: Pascal,?\s*\}',vessel)),
 'sweep_has_no_owned_headspace_volume':'Headspace::Open | Headspace::Swept { .. } => None' in vessel,
 'canonical_sweep_syntax_has_no_flow_or_budget':'sweep <vessel> <pressure><Pa|kPa|bar|atm>' in script,
 'initial_sweep_vents_owned_headspace':bool(re.search(r'Operator::Sweep \{ vessel, pressure \} => \{.*?let gases = vent_headspace\(v\);.*?v.headspace = Headspace::Swept',bench,re.S)),
 'film_clock_explicitly_excludes_swept':'if !matches!(vessel.headspace, crate::vessel::Headspace::Open)'in clock,
 'sweep_selects_existing_reservoir_target':'Headspace::Swept { .. } => Some(SWEPT_LOG_PARTIAL_PRESSURE)'in aq,
 'target_log_atm_unchanged_minus12':bool(re.search(r'const SWEPT_LOG_PARTIAL_PRESSURE: f64 = -12\.0;',aq)),
 'external_kinds_are_reservoir_and_dose':bool(re.search(r'enum ExternalGasKind \{.*?Reservoir,.*?Dose,\s*\}',aq,re.S)),
 'reservoir_starts_with_zero_moles':bool(re.search(r'external_gases.push\(ExternalGas \{\s*phase: phase.to_string\(\),\s*species: species.to_string\(\),\s*initial_moles: 0.0,\s*kind: ExternalGasKind::Reservoir,',aq)),
 'basis_accounts_external_h_o':all(s in inventory for s in ['pub(crate) fn complete_basis(', 'let h = formula.counts.get("H")', 'let o = formula.counts.get("O")','gas_excess += moles * (h - 2.0 * o)']),
 'no_source_edit_for_this_review':not subprocess.check_output(['git','-C',str(a.repo),'status','--porcelain']),
 }
 assert all(checks.values()),checks
 root=Path(__file__).resolve().parents[2]
 original=json.loads((root/'owned-split-gs007-20261115/original.json').read_text())
 assert original['id']=='new120:gs-007'
 assert hashlib.sha256(original['script'].encode()).hexdigest()==original['script_sha256']
 assert 'seal v1 500mL\n' in original['script']and 'sweep v1 1bar\nwait 60s\n'in original['script']
 r=dict(source=SOURCE,tree=subprocess.check_output(['git','-C',str(a.repo),'rev-parse',SOURCE+'^{tree}'],text=True).strip(),source_file_sha256={f:hashlib.sha256(s.encode()).hexdigest()for f,s in source.items()},source_contract_checks=checks,original_id=original['id'],original_script_sha256=original['script_sha256'],original_criteria=original['acceptance_checks'],missing_original_inputs=['finite inlet molar inventory or inlet molar-flow history and composition','a declared contact/transfer model with its apparatus-specific parameters and domain'],parameters_inferred_or_added=False,native_execution=False,scientific_qualification=False)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(dict(source=SOURCE,source_contract_checks_passed=len(checks),native_execution=False,scientific_qualification=False)))
if __name__=='__main__':main()
