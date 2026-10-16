#!/usr/bin/env python3
"""Read saved identity refusals and exact source; no engine or network execution."""
import collections
import hashlib
import json
from pathlib import Path
import re
import subprocess
import zipfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
SOURCE='1d24a43a8636e4cd1cf857e2834d74425b945b33'
BASE=HERE.parent/'curriculum200-common-engine'
ARCHIVE_SHA='918fde8fa17b89b131a58712381d86ee8b336a0990d411789549b13c15d57f73'
def sha(b):return hashlib.sha256(b).hexdigest()
def source(path):return subprocess.check_output(['git','-C',str(ROOT),'show',SOURCE+':'+path])
def rel(path):return path.relative_to(ROOT).as_posix()
def normalize_material(value):return ' '.join(p.lower() for p in re.split(r'[\s_-]+',value) if p)
paths=['data/registry/registry-source-v1.json','crates/kerotakis-core/src/species.rs',
 'crates/kerotakis-core/src/script.rs','crates/kerotakis-core/src/material.rs',
 'crates/kerotakis-data/src/schema.rs','crates/kerotakis-core/src/enzyme.rs',
 'crates/kerotakis-core/src/curated.rs','crates/kerotakis-core/src/displacement.rs',
 'crates/kerotakis-core/src/bench.rs']
sources={p:source(p) for p in paths}
registry=json.loads(sources[paths[0]])
identities={r['id']:r for r in registry['identities']}
compositions={r['species_id']:r for r in registry['compositions']}
section=sources[paths[1]].decode().split('const INPUT_ALIASES:',1)[1].split('];',1)[0]
aliases=dict(re.findall(r'\("([^"\n]+)", "([^"\n]+)"\)',section))
material_index=collections.defaultdict(list)
for r in registry['material_recipes']:
 for name in [r['canonical_key'],r['name']]+[n for names in r['aliases'].values() for n in names]:
  material_index[normalize_material(name)].append(r['canonical_key'])
enzymes=set(re.findall(r'enzyme_species\(\s*"([^"]+)"',sources[paths[5]].decode()))
manifest_path=ROOT/'audits/reaction-dataset-200-20261015/design/manifest.json'
manifest=json.loads(manifest_path.read_text());mrecords={c['rank']:c for c in manifest['records']}
freeze_path=manifest_path.with_name('freeze.json');freeze=json.loads(freeze_path.read_text())
assert sha(manifest_path.read_bytes())==freeze['files']['manifest.json']
prior_path=ROOT/'audits/reaction-dataset-200-20261015/identity-first15-review-20261016.json'
prior=json.loads(prior_path.read_text());prior_records={c['token']:c for c in prior['records']}
progress_path=ROOT/'audits/reaction-dataset-200-20261015/repair-progress.json'
progress=json.loads(progress_path.read_text());progress_records={c['rank']:c for c in progress['records']}
pr807_path=ROOT/'audits/reaction-dataset-200-20261015/pr807-integration.json'
pr807=json.loads(pr807_path.read_text())
subprocess.run(['git','-C',str(ROOT),'merge-base','--is-ancestor',pr807['merge_commit'],SOURCE],check=True)
verified=json.loads((BASE/'results/verified-results.json').read_text());receipts={c['id']:c for c in verified['cases']}
meta=json.loads((BASE/'results/artifact-metadata.json').read_text())
assert sha((BASE/'results/raw-artifact.zip').read_bytes())==ARCHIVE_SHA==verified['raw_artifact_sha256']==meta['digest'].removeprefix('sha256:')
failures=collections.defaultdict(list)
with zipfile.ZipFile(BASE/'results/raw-artifact.zip') as z:
 assert len(z.namelist())==len(set(z.namelist()))
 assert json.loads(z.read('binding.json'))==verified['binding']
 design_raw=(BASE/'native-screen-design.json').read_bytes()
 assert sha(design_raw)==verified['binding']['design_sha256']
 archived_design_raw=z.read('design.json')
 assert json.loads(design_raw)==json.loads(archived_design_raw)
 for c in verified['cases']:
  prefix=c['id'].replace(':','_');err=z.read(prefix+'/stderr.txt')
  match=re.search(r"unknown species or material '([^']+)'",err.decode())
  if not match:continue
  token=match[1];rank=int(c['id'].split(':')[1].split('-')[0]);original=mrecords[rank]
  receipt_raw=z.read(prefix+'/execution.json');receipt=json.loads(receipt_raw)
  assert all(receipt[k]==c[k] for k in receipt)
  assert receipt['exit_code']!=0 and receipt['stdout_parse_error'] is None
  script=z.read(prefix+'/experiment.lab');raw=z.read(prefix+'/stdout.jsonl')
  for raw_bytes,key in [(err,'stderr_sha256'),(script,'script_sha256'),(raw,'stdout_sha256')]:assert sha(raw_bytes)==c[key]
  assert sha(script)==freeze['files'][original['script']]
  # Retain exact error message but do not repeat runner/host filesystem paths.
  line=re.search(r'\.lab:(\d+): (.*)',err.decode())
  assert line and token in line[2]
  definitions=[r for r in original['reactants'] if r['species_id']==token]
  assert definitions
  failures[token].append({'original_rank':rank,'original_id':original['id'],'execution_id':c['id'],
   'raw_member_prefix':prefix,'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(err),
   'execution_receipt_sha256':sha(receipt_raw),'failed_script_line':int(line[1]),'error_without_runner_path':line[2],
   'script':script.decode(),'source_reactants_for_token':definitions,'original_equation':original['equation'],
   'original_expected_observations':original['expected_observations_from_source'],
   'frozen_progress_status':progress_records[rank]['status']})
assert sum(map(len,failures.values()))==108 and len(failures)==80

protocols={
 'C2O4--':{'priority':1,'case':118,'class':'same_elemental_and_formal_charge_feed_in_existing_curated_basis',
  'replacement_commands':['add v1 H+ 0.006mol','add v1 H2C2O4 0.005mol','add v1 MnO4- 0.002mol'],
  'reason':'Original5mmolC2O4(2-)+16mmolH+ is atom/formal-charge equivalent to5mmolH2C2O4+6mmolH+. H2C2O4 is registered and curated.rs325-395 explicitly implements this feed basis. Mn feed unchanged. Prefer acid first in a separately frozen protocol; do not replace only the anion while leaving16mmol acid.',
  'limitations':'This is not an oxalate-ion alias. The existing curated row explicitly does not gate acid or the practical approximately60C initiation, and supplies no real rate/heat calibration. Original order and inputs remain frozen; no physical pass from source.'},
 'CuCl2':{'priority':2,'case':84,'class':'explicit_aqueous_analytical_components_control',
  'replacement_commands':['add v1 Cu+2 0.001mol','add v1 Cl- 0.002mol'],
  'reason':'Original specifies CuCl2(aq), already in100mL water. Existing Cu+2 and Cl- represent the same Cu/Cl count and netzero charge; retain original0.1A10s finite electrolysis. displacement.rs defines aqueous chloride oxidation and copper plating, including finite water and current bookkeeping.',
  'limitations':'Separate prepared-ion control, not a CuCl2 solid/solution stock alias or a dissolution-enthalpy repair. Source coverage does not prove output yields, electrode kinetics or final gas speciation.'},
 'NaCl_aq':{'priority':3,'case':86,'class':'explicit_salt_dissolution_then_missing_operator_control',
  'replacement_commands':['add v1 NaCl 0.002mol'],
  'reason':'The frozen script already has100mL water and another2mmol water. NaCl is the exact neutral formula; a prepared solution can be made by dissolving that amount. Original chlor_alkali script has NO electrolyse operator, so spelling alone cannot drive the forecast. A separate explicitly chosen finite current/duration is needed before any electrolysis replay.',
  'limitations':'Never globally alias phase-tagged NaCl_aq/NaCl_melt to solid NaCl. A solution prepared from solid changes preparation/heat history. No current or duration invented here; freeze those separately.'},
 'HSO4-':{'priority':4,'case':63,'class':'explicit_analytical_proton_sulfate_control',
  'replacement_commands':['add v1 SO4-2 0.001mol','add v1 H+ 0.001mol'],
  'reason':'Those admitted aqueous components preserve H/S/O and net-1 charge of1mmol HSO4-. This prepares an analytical sulfate/proton equilibrium question without adding Na from NaHSO4.',
  'limitations':'Not the same admitted protonation microstate or its preparation enthalpy. Direct HSO4- admission still needs its identity/properties and native ownership contract. No new Ka or exact original phase identity claim.'},
 'Cuaqua':{'priority':5,'case':97,'class':'explicit_hydrated_ion_analytical_basis_control',
  'replacement_commands':['add v1 Cu+2 0.001mol','add v1 water 0.006mol'],
  'reason':'The original token explicitly means Cu(H2O)6(2+). Cu+2 plus6bulk waters preserves Cu/H/O and formal+2 charge; keep the original4mmol NH3. Native reference-temperature copper-ammine distribution exists.',
  'limitations':'This does not create an explicit hexaaqua microstate or resolve which hydration waters remain coordinated. No silent sulfate/counterion addition, ammine spectrum, colour, heat or free-ligand ownership proof. Treat as a different prepared analytical-basis observation.'},
 'H2CO3':{'priority':6,'case':55,'class':'separate_dissolved_CO2_acidification_control',
  'replacement_commands':['add v1 CO2(aq) 0.001mol','add v1 water 0.001mol'],
  'reason':'The admitted CO2(aq)+water basis has the same H/C/O count as H2CO3. It can ask a separately declared aqueous-carbon acidification question with original extra water retained.',
  'limitations':'H2CO3 and CO2(aq) are NOT identical molecules or protonation constants. This cannot repair a molecular carbonic-acid identity or earn the original species-specific first-protolysis claim; retain as a distinct model-basis control.'}}

related={'Na':['Na+','NaCl','NaOH'],'ethanal':['ethanol','propanone'],'Br2':['Br-','HBr'],
 'propene':['ethene','isobutylene','propane'],'benzene':['naphthalene'],'NO2':['NO2-','NO3-'],'NO':['NO2-','NO3-'],
 'I-':['I2','KI','HI'],'I3-':['I2','KI','lugol_solution_1_percent'],'triolein':['vegetable_oil','coconut_fat'],
 'tristearin':['vegetable_oil','coconut_fat'],'glucose_open':['glucose','fructose'],'butene2':['isobutylene'],
 'Al3+':['Al'],'AlOH3':['Al'],'Al2O3':['Al'],'hemihydrate':['gypsum'],'indigo':['indigo_carmine'],
 'methyl_acetate':['ethyl_acetate'],'decane':['dodecane','hexadecane'],'O_atom':['O2'],'O3':['O2'],
 'acetone_enolate':['propanone'],'ethoxide':['ethanol'],'NaCl_melt':['NaCl'],'Coaqua':[],
 'C2O4--':['H2C2O4'],'CuCl2':['Cu+2','Cl-'],'Cuaqua':['Cu+2','water'],'HSO4-':['NaHSO4','SO4-2'],
 'H2CO3':['CO2(aq)','CO2','water']}
notes={
 'Na':'Neutral Na(0) solid is absent. Ionic sodium and sodium salts change oxidation state or composition.',
 'ethanal':'No ethanal/acetaldehyde identity; ethanol is a different alcohol and ethylene oxide would be a structural isomer, not an alias.',
 'bromobenzene':'No bromobenzene identity or aryl-halide coupling family. All seven blocked coupling protocols need more than a spelling.',
 'propene':'No C3H6 propene entry; ethene, propane and branched C4 isobutylene are distinct.',
 'benzene':'No C6H6 benzene entry. Naphthalene, petrol or a generic aromatic material is not equivalent.',
 'NO2':'NO2 gas is not nitrite NO2- or nitrate; new nitrogen-oxide identity and gas/domain data needed.',
 'NO':'NO gas is not NO2- or NO3-; add a sourced neutral nitrogen-monoxide identity and target gas/redox domain.',
 'I-':'KI/HI admit other components and do not establish isolated iodide or its aqueous reactions. No silent K+/H+ substitution.',
 'Al3+':'Neither Al3+ nor Al+3 is a registered ion; changing charge spelling cannot expose one. Aluminium metal is distinct.',
 'AlOH3':'Al(OH)3 is also absent. Parenthesis insertion alone cannot add a missing solid.',
 'Cr2O7--':'No Cr2O7-2 or equivalent registered dichromate basis; notation normalization cannot repair it.',
 'NaCl_melt':'NaCl solid is registered but molten-state preparation and molten electrolysis are separate domains; existing solvent electrolyser requires finite liquid water.',
 'hemihydrate':'Gypsum is CaSO4·2H2O; hemihydrate is CaSO4·0.5H2O. Hydration state and water balance differ.',
 'glucose_open':'Existing glucose is a D-glucose registry identity, not a declared open-chain microstate or ring-chain equilibrium; fructose has another structure.',
 'e-':'An electrode electron feed requires a circuit/counterreaction/current contract, not a weighable reagent or a silently omitted term.',
 'H_equiv':'Formal [H] is not a unique molecule, charge or finite reducing reagent.',
 'EDTA':'Source specifies Y4- (C10H12N2O8,-4), not an unspecified acid/salt; no registered Y4-/complex domain.',
 'peptide':'No chain length, sequence or unique molecular formula; gelatin/albumin are materials, not molar peptide substitutes.',
 'aromatic_aminoacid':'Unspecified aromatic residues/protein, not a unique amino-acid identity.',
 'diisocyanate':'The source gives a generic diisocyanate, not a unique monomer; choose one before molecular dosing.',
 'cope_start':'The source gives an unspecified substituted1,5-diene; no unique molecular identity.',
 'ninhydrin':'Concrete reagent name but no original formula supplied and no registered identity. Needs sourced identity/data, not a generic amino-acid reagent alias.'}
records=[]
for token,cases in sorted(failures.items(),key=lambda item:(-len(item[1]),item[0])):
 direct=identities.get(token);alias=aliases.get(token);materials=sorted(set(material_index.get(normalize_material(token),[])))
 assert not direct and not materials and token not in enzymes
 definitions=[{'rank':c['rank'],'original_id':c['id'],'reactant':r} for c in manifest['records'] for r in c['reactants'] if r['species_id']==token]
 first=cases[0]['source_reactants_for_token'][0]
 if alias: classification='already_repaired_by_PR807_in_current_main';reason='Exact reviewed alias is already in INPUT_ALIASES; replay on its accepted source/binary remains separate from c446 first outcomes.'
 elif token in protocols:classification='new_prospective_prepared_basis_control_available_not_global_alias';reason=protocols[token]['reason']
 elif token in ['peptide','aromatic_aminoacid','diisocyanate','cope_start','H_equiv']:classification='underspecified_or_schematic_entity';reason=notes[token]
 elif token=='e-':classification='missing_circuit_half_reaction_protocol_not_reagent_alias';reason=notes[token]
 elif token in ['hemihydrate','NaCl_melt','AlOH3','Al2O3','PbO2','LiC6']:classification='missing_specific_phase_hydrate_or_electrode_domain';reason=notes.get(token,'Requires the named composition/oxidation-state solid and its phase/electrode domain, not an existing metal/salt surrogate.')
 elif token in ['glycine','glycine_cation','glycine_zwitterion','glucose_open']:classification='missing_structure_or_protonation_microstate_and_domain';reason=notes.get(token,'The specified amino-acid/protonation microstate is absent; charged and zwitterionic forms must retain their own proton/charge ownership.')
 elif first['charge'] not in [None,0]:classification='missing_charge_specific_identity_properties_and_domain';reason=notes.get(token,'No admitted identity with this specified formula/charge. Its related neutral molecule, salt or a PHREEQC display species is not an equivalent shelf identity.')
 elif token in ['tristearin','triolein']:classification='missing_specific_triglyceride_and_reaction_domain';reason='The named pure triglyceride is absent; unresolved oil/coconut-fat recipes do not supply the same molar molecule or reaction stoichiometry.'
 elif token in ['Br2','NO','NO2','O3','O_atom','SO3']:classification='missing_neutral_gas_or_molecular_oxidation_identity_and_domain';reason=notes.get(token,'Add the specified neutral molecule/atom with sourced phase/properties and target reaction; do not exchange oxidation state or atomicity.')
 elif token=='Na':classification='missing_neutral_element_and_water_or_halogen_reaction';reason=notes[token]
 else:classification='missing_specific_molecular_identity_properties_and_reaction_domain';reason=notes.get(token,'No matching admitted molecular structure/identity; source identity/properties, then separately scope the original transformation. A chemically different registry neighbour is not an alias.')
 records.append({'token':token,'first_failure_multiplicity':len(cases),'classification':classification,'reason':reason,
   'registry_exact_key':None,'current_input_alias_target':alias,'current_material_matches':materials,'additional_enzyme_match':False,
   'source_reactant_occurrences':definitions,'first_failure_cases':cases,
   'related_registered_entities_not_identity_aliases':related.get(token,[]),
   'prior_first15_review':{'classification':prior_records[token]['classification'],'source_head':prior['source_head']} if token in prior_records else None,
   'prospective_control':protocols.get(token),'new_execution_performed':False,'original_case_closed':False})
result={'scope':'All108 authenticated first unknown-token refusals,80 distinct tokens, compared with exact current source. No engine/API/build, alias additions, changed originals or chemical pass.',
 'review_source_head':SOURCE,'source_files':{p:sha(b) for p,b in sources.items()},
 'source_registry_identity_count':len(identities),'current_input_aliases':aliases,'additional_enzymes':sorted(enzymes),
 'saved_engine':verified['engine'],'saved_archive_sha256':ARCHIVE_SHA,'saved_run':verified['run_id'],
 'binding_qualification':'Raw archive/source/stream bindings checked. Producer lock/binary hash is attested; no executable/lock reconstructed. Current-source admission is not runtime replay.',
 'design_original_sha256':sha(design_raw),'design_archive_sha256':sha(archived_design_raw),'design_full_parsed_value_equality':True,
 'prior_files':{rel(p):sha(p.read_bytes()) for p in [prior_path,progress_path,manifest_path,freeze_path,pr807_path]},
 'PR807':{'merge_commit':pr807['merge_commit'],'head':pr807['head'],'ancestor_of_review_source':True,'already_handled_tokens':['CuSO4_anhydrous','acetate'],'remaining_c446_first_refusals_using_those_tokens':2},
 'summary':{'first_refusals':108,'distinct_tokens':80,'already_current_source_alias_tokens':2,'new_proven_alias_candidates':0,
  'separate_source_supportable_prepared_control_tokens':6,'other_tokens_requiring_new_identity_domain_or_procedure_design':72,'original_unknown_tokens_not_repaired_by_current_aliases':78,
  'highest_multiplicity_five_tokens':['bromobenzene','ethanal','propene','benzene','NO2'],'first_refusals_for_those_five':25},
 'prospective_control_queue':sorted(protocols.items(),key=lambda x:x[1]['priority']),
 'requirements':['No nearest-name substitutions, charge/oxidation changes, isomer swaps or unresolved-material-for-molecule swaps.',
 'Never alter old scripts/forecasts/first outputs. Freeze every new prepared-basis protocol, species/phase/dose/order and forecast separately before execution.',
 'Current aliases prove input spelling only, not the original hydration, reaction, heat, rate, spectrum or physical result.',
 'First refusal counts do not include later blocked reactants. Original source occurrences are included to expose potential later blockers.',
 'Global aliases for phase-tagged NaCl_aq/NaCl_melt or hydration/microstate entities remain unearned.'],
 'records':records,'new_engine_executions':0,'catalog_additions':0}
print(json.dumps(result,indent=2,allow_nan=False,ensure_ascii=False))
