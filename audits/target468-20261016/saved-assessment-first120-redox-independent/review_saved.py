#!/usr/bin/env python3
"""Independent saved redox audit. No engine, API, download or build invocation."""
import hashlib
import json
import math
from collections import defaultdict
from decimal import Decimal as D
from pathlib import Path
import subprocess
import zipfile

HERE = Path(__file__).resolve().parent
BASE = HERE.parent / 'common-engine-completion'
SIBLING = HERE.parent / 'saved-assessment-first120-redox'
ROOT = HERE.parents[2]
SOURCE = 'c44692053a94c59dcf62e7fb67549f84c7d18f83'
DATA_SHA = '8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd'
ZIP_SHA = '9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f'
PROPOSE = [1, 3, 8, 9, 10, 11, 13, 14]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def file_sha(path):
    with path.open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def blob(path):
    return subprocess.check_output(['git', 'show', SOURCE + ':' + path], cwd=ROOT)


def strict(pairs):
    result = {}
    for key, value in pairs:
        assert key not in result, key
        result[key] = value
    return result


def loads(raw):
    def bad(value):
        raise ValueError(value)
    return json.loads(raw, object_pairs_hook=strict, parse_constant=bad)


def main():
    dataset = (BASE / 'originals/new-experiments-20261016.json').read_bytes()
    assert sha(dataset) == DATA_SHA
    archive = BASE / 'results/raw-artifact.zip'
    assert file_sha(archive) == ZIP_SHA
    verified = loads((BASE / 'results/verified-results.json').read_bytes())
    metadata = loads((BASE / 'results/artifact-metadata.json').read_bytes())
    assert verified['raw_artifact_sha256'] == metadata['digest'].removeprefix('sha256:') == ZIP_SHA
    sibling_raw = (SIBLING / 'assessment.json').read_bytes()
    sibling = loads(sibling_raw)
    engine = verified['engine']
    assert engine == sibling['engine']
    assert engine['source_head'] == SOURCE
    assert engine['binary_sha256'] == '670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f'
    assert engine['cargo_lock_sha256'] == '82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229'
    assert subprocess.check_output(['git', 'rev-parse', SOURCE + '^{tree}'], cwd=ROOT, text=True).strip() == engine['source_tree']
    registry_raw = blob('data/registry/registry-source-v1.json')
    compositions = {c['species_id']: c for c in loads(registry_raw)['compositions']}
    species = blob('crates/kerotakis-core/src/species.rs')
    assert b'fn basis_species_data' in species and b'.get("OH-")' in species
    compositions['base_equivalents'] = compositions['OH-']
    source_files = {}
    for binding in sibling['source_files']:
        raw = blob(binding['repository_path'])
        assert sha(raw) == binding['sha256']
        source_files[binding['repository_path']] = sha(raw)
    bench_source = blob('crates/kerotakis-core/src/bench.rs')
    for declaration in [b'const AIR_N2: f64 = 0.7901;', b'const AIR_O2: f64 = 0.2095;', b'const AIR_CO2: f64 = 0.0004;']:
        assert declaration in bench_source
    displacement_source = blob('crates/kerotakis-core/src/displacement.rs')
    assert b'let anode_is_first = ea.volts <= eb.volts;' in displacement_source
    assert b'let volts = cathode.volts - anode.volts;' in displacement_source
    originals = {c['id']: c for c in loads(dataset)['experiments']}
    receipts = {c['id']: c for c in verified['cases']}
    peer = {c['id']: c for c in sibling['cases']}

    def book(totals, key, amount):
        n = D(str(amount))
        assert n.is_finite()
        composition = compositions[key]
        for element in composition['elements']:
            totals[element['element']] += n * D(str(element['count']['value']))
        totals['charge'] += n * D(str(composition['net_charge']['value']))

    def inventory(vessel):
        totals = defaultdict(D)
        assert not vessel['surfaces'] and not vessel.get('unresolved_materials')
        for portion in vessel['contents']:
            assert math.isfinite(portion['moles']) and portion['moles'] >= 0
            book(totals, portion['species'], portion['moles'])
        return totals

    def amount(vessel, key, phase=None):
        names = {key} if isinstance(key, str) else key
        return sum(p['moles'] for p in vessel['contents'] if p['species'] in names and (phase is None or p['phase'] == phase))

    cases = []
    with zipfile.ZipFile(archive) as z:
        assert len(z.namelist()) == len(set(z.namelist()))
        binding = loads(z.read('binding.json'))
        assert binding == verified['binding'] == sibling['binding'] and binding['engine'] == engine
        design_raw = (BASE / 'native-screen-design.json').read_bytes()
        archived_design = z.read('design.json')
        assert sha(design_raw) == binding['design_sha256'] and loads(design_raw) == loads(archived_design)
        for n in range(1, 15):
            cid = f'new120:rx-{n:03}'
            original, assessment, receipt = originals[cid], peer[cid], receipts[cid]
            assert assessment['original_expected_result'] == original['expected_result']
            assert assessment['original_expected_processes'] == original['expected_processes']
            assert [c['original'] for c in assessment['criteria']] == original['acceptance_checks']
            prefix = cid.replace(':', '_')
            execution_raw = z.read(prefix + '/execution.json')
            execution = loads(execution_raw)
            assert all(receipt[key] == value for key, value in execution.items())
            assert receipt['engine'] == engine and execution['exit_code'] == 0 and execution['stdout_parse_error'] is None
            hashes = {}
            for filename, key in [('experiment.lab', 'script_sha256'), ('stdout.jsonl', 'stdout_sha256'), ('stderr.txt', 'stderr_sha256')]:
                raw = z.read(prefix + '/' + filename)
                assert sha(raw) == execution[key] == receipt[key]
                hashes[filename] = sha(raw)
            assert z.read(prefix + '/experiment.lab') == original['script'].encode()
            assert hashes['experiment.lab'] == original['script_sha256'] == assessment['script_sha256']
            assert assessment['authentication']['member_sha256'] == hashes
            assert assessment['authentication']['execution_receipt_sha256'] == sha(execution_raw)
            rows = [loads(line) for line in z.read(prefix + '/stdout.jsonl').splitlines()]
            assert len(rows) == receipt['parsed_observations']
            commands = [s.strip() for s in original['script'].splitlines() if s.strip() and not s.lstrip().startswith('#') and not s.strip().startswith('register ')]
            assert len(commands) == len(rows)
            states, current = [], {}
            incoming, exported = defaultdict(D), defaultdict(D)
            owned = defaultdict(lambda: defaultdict(D))
            events = []
            for i, (command, row) in enumerate(zip(commands, rows)):
                published = {v['id']: v for v in row['bench']['vessels']}
                assert len(published) == len(row['bench']['vessels'])
                if current.keys() - published.keys():
                    words = command.split()
                    assert len(words) == 2 and words[0] == 'inspect'
                    target = int(words[1][1:]) - 1
                    assert set(published) == {target} and row['operator']['op'] == 'inspect' and not row['events'] and published[target] == current[target]
                op = row['operator']
                if op['op'] == 'add':
                    book(incoming, op['species'], op['moles'])
                    book(owned[op['vessel']], op['species'], op['moles'])
                if op['op'] == 'filter':
                    for portion in current[op['from']]['contents']:
                        if portion['phase'] in ['liquid', 'aqueous']:
                            book(owned[op['from']], portion['species'], -portion['moles'])
                            book(owned[op['to']], portion['species'], portion['moles'])
                for event in row['events']:
                    events.append({'row': i, **event})
                    if event['event'] == 'vessel_sealed':
                        for key, fraction in [('N2', '.7901'), ('O2', '.2095'), ('CO2', '.0004')]:
                            moles = D(str(event['trapped_air'])) * D(fraction)
                            book(incoming, key, moles)
                            book(owned[event['vessel']], key, moles)
                    if event['event'] == 'gas_evolved':
                        book(exported, event['species'], event['moles'])
                        book(owned[event['vessel']], event['species'], -event['moles'])
                    assert event['event'] not in ['gas_absorbed', 'gas_vented', 'diluted']
                current.update(published)
                states.append(current.copy())
            total = defaultdict(D)
            per_vessel = []
            for vid, vessel in current.items():
                actual = inventory(vessel)
                for key, value in actual.items():
                    total[key] += value
                delta = {k: float(actual[k] - owned[vid][k]) for k in sorted(set(actual) | set(owned[vid]))}
                per_vessel.append({'vessel': vid, 'residual_mol': delta})
            residual = {k: float(total[k] + exported[k] - incoming[k]) for k in sorted(set(total) | set(exported) | set(incoming))}
            maximum = max([abs(x) for x in residual.values()] + [abs(x) for v in per_vessel for x in v['residual_mol'].values()])
            assert maximum <= 1e-8
            final = states[-1]
            cells = [e for e in events if e['event'] == 'cell_voltage']
            observation = {}
            if n == 1:
                ag = sum(amount(v, 'Ag', 'solid') for v in final.values())
                zn = sum(amount(v, 'Zn+2') for v in final.values())
                assert ag == .002 and zn == .001 and ag == 2 * zn
                observation = {'Ag_solid_mol': ag, 'ZnII_mol': zn}
            elif n == 2:
                assert amount(final[0], 'Cu', 'solid') == .001 and amount(final[0], 'Cu+2') == amount(final[0], 'Fe+2') == 0
                observation = {'Cu_solid_mol': .001, 'CuII_mol': 0, 'FeII_mol': 0, 'hold_reason': 'No positive requested coupled oxidation/reduction; zero conversion below a ceiling is insufficient.'}
            elif n == 3:
                assert amount(final[0], 'H2') == .001 and amount(final[0], 'Mg+2') > 0
                assert amount(final[1], 'H2') == 0 and amount(final[1], 'Cu', 'solid') == .001
                observation = {'Mg_vessel_H2_mol': amount(final[0], 'H2'), 'Cu_control_H2_mol': 0}
            elif n == 4:
                assert amount(final[0], 'KI') == .0004 and amount(final[0], {'I2', 'I3-'}) == amount(final[0], 'Fe+2') == 0
                observation = {'retained_KI_mol': .0004, 'oxidized_iodine_mol': 0, 'FeII_mol': 0, 'hold_reason': 'Unmapped KI and absent positive coupled products.'}
            elif n == 5:
                assert amount(final[0], 'I2') == .0001 and amount(final[0], 'S2O3-2') == .0002
                assert amount(final[0], {'S4O6-2', 'Na2S4O6'}) == 0
                observation = {'remaining_I2_mol': .0001, 'remaining_thiosulfate_mol': .0002, 'tetrathionate_mol': 0, 'hold_reason': 'No requested iodine-thiosulfate consumption; complexation coverage remains unsupported.'}
            elif n in [6, 7]:
                product = amount(final[0], 'dehydroascorbic_acid')
                assert product == .0001 and amount(final[0], 'ascorbic_acid') == 0
                assert amount(final[0], 'HI') == .0002
                if n == 6:
                    assert amount(final[0], 'I2') == 0
                else:
                    assert amount(states[4][0], 'ascorbic_acid') == .00005 and amount(states[4][0], 'I2') == 0
                    assert amount(final[0], 'I2') == .00005
                observation = {'oxidized_vitaminC_mol': product, 'remaining_I2_mol': amount(final[0], 'I2'), 'hold_reason': 'Literal curated conversion does not close missing KI/iodine-complex mixture coverage; RX007 also has initial dissolution limitation.'}
            elif n == 8:
                co2 = sum(e['moles'] for e in events if e['event'] == 'gas_evolved' and e['species'] == 'CO2')
                assert co2 == .001 and amount(final[0], 'Mn+2') == .0002 and amount(final[0], 'H2C2O4') == 0
                assert amount(final[0], {'KMnO4', 'MnO4-'}) == 0
                observation = {'exported_CO2_mol': co2, 'MnII_mol': .0002, 'oxalic_to_permanganate_input_ratio': 2.5}
            elif n == 9:
                assert amount(final[0], 'MnO2', 'solid') > 0 and amount(final[0], {'KMnO4', 'MnO4-'}) == 0
                observation = {'MnO2_mol': amount(final[0], 'MnO2', 'solid'), 'acetic_family_mol': amount(final[0], {'CH3COOH', 'CH3COO-'})}
            elif n in [10, 11, 12]:
                assert all(states[e['row']] == states[e['row'] - 1] for e in cells)
                observation = {'cell_volts': [e['volts'] for e in cells], 'anode_ids': [e['anode'] for e in cells], 'cathode_ids': [e['cathode'] for e in cells], 'no_state_change_during_measurement': True}
                if n == 10:
                    assert cells[1]['volts'] < cells[0]['volts'] and cells[0]['anode'] == cells[1]['anode'] == 0
                elif n == 11:
                    assert cells[0]['volts'] > 0 and cells[0]['anode'] == 0 and cells[0]['cathode'] == 1
                else:
                    assert cells[0]['volts'] == cells[1]['volts'] > 0 and cells[0]['anode'] == cells[1]['anode']
            elif n == 13:
                before, after = states[3][0], states[5][0]
                dh = amount(after, 'H2') - amount(before, 'H2')
                do = amount(after, 'O2') - amount(before, 'O2')
                observation = {'H2_increment_mol': dh, 'O2_increment_mol': do, 'H2_to_O2': dh / do, 'H2_Q_over_2F_residual': dh - 10 / (2 * 96485.33212), 'O2_Q_over_4F_residual': do - 10 / (4 * 96485.33212)}
                assert abs(observation['H2_Q_over_2F_residual']) < 1e-15 and abs(observation['O2_Q_over_4F_residual']) < 1e-15
                assert amount(after, 'H2', 'aqueous') == amount(after, 'O2', 'aqueous') == 0
            elif n == 14:
                before, after = states[2][0], final[0]
                cu = amount(after, 'Cu', 'solid')
                ion_loss = amount(before, 'Cu+2') - amount(after, 'Cu+2')
                mineral_loss = 3 * (amount(before, 'antlerite', 'solid') - amount(after, 'antlerite', 'solid'))
                observation = {'Cu_solid_mol': cu, 'aqueous_CuII_decrease_mol': ion_loss, 'Cu_from_antlerite_loss_mol': mineral_loss, 'Cu_phase_budget_residual_mol': cu - ion_loss - mineral_loss, 'Q_over_2F_residual_mol': cu - 10 / (2 * 96485.33212), 'exported_O2_mol': sum(e['moles'] for e in events if e['event'] == 'gas_evolved' and e['species'] == 'O2')}
                assert abs(observation['Q_over_2F_residual_mol']) < 1e-15 and abs(observation['Cu_phase_budget_residual_mol']) < 1e-8
            if n in PROPOSE:
                assert all(c['status'] == 'met' for c in assessment['criteria'])
            cases.append({'id': cid, 'script_sha256': hashes['experiment.lab'], 'raw_member_sha256': hashes, 'execution_receipt_sha256': sha(execution_raw), 'independent_recommendation': 'accept_bounded_selection' if n in PROPOSE else 'retain_hold', 'positive_observations': observation, 'final_whole_script_residual_mol': residual, 'final_per_vessel_residuals': per_vessel, 'max_abs_residual_mol': maximum, 'named_limits_preserved': assessment['named_limits']})
    report = {'schema_version': 1, 'scope': 'Independent saved-only review of14 first120 redox cases, not new execution or empirical calibration. Selection recommendations only.', 'engine': engine, 'original_dataset_sha256': DATA_SHA, 'raw_archive_sha256': ZIP_SHA, 'design_authentication': {'original_bytes_sha256': sha(design_raw), 'archive_bytes_sha256': sha(archived_design), 'complete_parsed_value_equal': True}, 'reviewed_sibling_files_sha256': {n: file_sha(SIBLING / n) for n in ['review_saved.py', 'assessment.json', 'README.md']}, 'source_files_sha256': source_files, 'registry_sha256': sha(registry_raw), 'cases': cases, 'accepted_ids': [c['id'] for c in cases if c['independent_recommendation'] == 'accept_bounded_selection'], 'held_ids': [c['id'] for c in cases if c['independent_recommendation'] == 'retain_hold'], 'material_blockers': [], 'qualifications': ['Archive carries producer-reported binary/lock identity; raw executable/lock bytes absent and not independently rehashed in this audit.', 'Whole-script final whole-bench and per-vessel represented material/formal-charge accounting; no energy or per-step closure claim.', 'Exact-source registry compositions and OH-derived base_equivalents; no double-counting of solution distributions, measured H/OH fields, lots or solute_charge.', 'RX008 only the warm acidic curated endpoint; missing acid/temperature gates and MnII spectrum do not become calibrated kinetics/heat.', 'RX010/011 ideal open-circuit Nernst measurements, with no discharge, junction-potential or empirical electrode claim.', 'RX013/014 ideal imposed-current model; floating extraction residuals are arithmetic checks, not newly authored physical acceptance tolerances.', 'RX012 original opposite-sign forecast fails; source sorts physical anode/cathode and reports positive spontaneous magnitude. No expectation rewrite.', 'RX006/007 positive literal curated conversion does not close unsupported KI/iodine-mixture and dissolution coverage.'], 'catalog_additions': 0, 'new_executions': 0}
    print(json.dumps(report, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
