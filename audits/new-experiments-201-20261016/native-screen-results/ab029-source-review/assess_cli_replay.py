"""Assess saved unchanged AB029 CLI output; no engine or network calls."""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import subprocess


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def assess(design_path, execution_dir):
    design_raw = design_path.read_bytes()
    design = json.loads(design_raw)
    receipt = json.loads((execution_dir / 'execution.json').read_text())
    stdout = (execution_dir / 'stdout.jsonl').read_bytes()
    stderr = (execution_dir / 'stderr.txt').read_bytes()
    assert receipt['engine'] == design['engine']
    assert receipt['stdout_sha256'] == sha(stdout) and receipt['stderr_sha256'] == sha(stderr)
    assert receipt['script_sha256'] == design['script_sha256'] == sha(design['script'].encode())
    assert receipt['status'] == 'completed' and receipt['exit_code'] == 0 and not stderr
    assert sha((execution_dir / 'Cargo.lock').read_bytes()) == design['engine']['cargo_lock_sha256']
    rows = [json.loads(line) for line in stdout.splitlines()]
    assert len(rows) == receipt['observations'] == 14
    source = design['engine']['source_head']
    registry_raw = subprocess.check_output(['git', 'show', source + ':data/registry/registry-source-v1.json'])
    registry = json.loads(registry_raw)
    compositions = {c['species_id']: c for c in registry['compositions']}
    # The exact source derives this analytical coordinate from OH-; it is not
    # another free-ion measurement or a stock on the registry shelf.
    species_raw = subprocess.check_output(['git', 'show', source + ':crates/kerotakis-core/src/species.rs'])
    assert b'pub const BASE_EQUIVALENTS: &str = "base_equivalents";' in species_raw
    compositions['base_equivalents'] = compositions['OH-']

    def add(totals, species, moles):
        composition = compositions[species]
        for element in composition['elements']:
            totals[element['element']] += moles * element['count']['value']
        totals['formal_charge'] += moles * composition['net_charge']['value']

    def vessel(row, identifier):
        return next(v for v in row['bench']['vessels'] if v['id'] == identifier)

    def amount(v, species):
        return sum(p['moles'] for p in v['contents'] if p['species'] == species)

    checks = []
    def check(quantity, actual, met):
        checks.append({'quantity': quantity, 'actual': actual, 'met': bool(met)})

    filtering = next(i for i, row in enumerate(rows) if row['operator']['op'] == 'filter')
    before = vessel(rows[filtering - 1], 1)
    filtrate = vessel(rows[filtering], 2)
    donor = vessel(rows[filtering], 1)
    limits = design['additional_preexisting_native_regression_checks']
    solution = filtrate['solution']
    fresh_complete = solution is not None and solution.get('scope', 'complete') == 'complete'
    check('fresh complete filtrate solution', fresh_complete, fresh_complete)
    delta_t = abs(filtrate['temperature'] - before['temperature'])
    check('same-state filter temperature difference K', delta_t,
          delta_t < limits['same_state_filter_temperature_max_difference_K'])
    delta_ph = abs(solution['ph'] - before['solution']['ph']) if solution else None
    check('same-state filter pH difference', delta_ph,
          delta_ph is not None and delta_ph < limits['same_state_filter_ph_max_difference'])
    delta_water = abs(amount(filtrate, 'water') - amount(before, 'water'))
    check('filtrate water difference mol', delta_water,
          delta_water < limits['filtrate_retained_water_difference_max_mol'])
    check('filtrate Mg(OH)2 mol', amount(filtrate, 'Mg(OH)2'),
          amount(filtrate, 'Mg(OH)2') < limits['filtrate_MgOH2_max_mol'])
    check('donor solid reserve mol', amount(donor, 'Mg(OH)2'),
          amount(donor, 'Mg(OH)2') > limits['donor_residual_MgOH2_min_mol'])
    events = [event for row in rows for event in row['events']]
    failures = [e for e in events if e['event'] == 'solver_failed']
    boundaries = [e for e in events if e['event'] == 'not_yet_modeled']
    check('solver_failed events', len(failures), not failures)
    measurements = [e for e in events if e['event'] == 'measured' and e['instrument'] == 'ph_meter']
    check('all four requested pH readings', len(measurements),
          len(measurements) == 4 and [e['vessel'] for e in measurements] == [0, 2, 0, 2])
    final_suspension = vessel(rows[-1], 0)
    final_filtrate = vessel(rows[-1], 2)
    ph_s, ph_f = final_suspension['solution']['ph'], final_filtrate['solution']['ph']
    check('final suspension pH', ph_s, ph_s > limits['final_suspension_ph_min'])
    check('final filtrate pH', ph_f, ph_f < limits['final_filtrate_ph_max'])
    check('final pH contrast', ph_s - ph_f, ph_s - ph_f > limits['final_ph_difference_min'])
    loss = amount(vessel(rows[1], 0), 'Mg(OH)2') - amount(final_suspension, 'Mg(OH)2')
    check('acid consumes suspension solid reserve mol', loss, loss > 0)

    incoming, exported = defaultdict(float), defaultdict(float)
    ledgers = []
    for index, row in enumerate(rows):
        op = row['operator']
        if op['op'] == 'add':
            add(incoming, op['species'], op['moles'])
        for event in row['events']:
            if event['event'] == 'gas_evolved':
                add(exported, event['species'], event['moles'])
            assert event['event'] != 'gas_absorbed', 'Unexpected external feed needs explicit accounting'
        retained = defaultdict(float)
        for v in row['bench']['vessels']:
            assert not v['surfaces'] and not v.get('unresolved_materials')
            for portion in v['contents']:
                add(retained, portion['species'], portion['moles'])
        residuals = {key: retained.get(key, 0) + exported.get(key, 0) - incoming.get(key, 0)
                     for key in sorted(set(retained) | set(incoming) | set(exported))}
        ledgers.append({'observation': index, 'residuals_mol': residuals,
                        'within_existing_1e-8_inventory_audit_resolution':
                            all(abs(value) < 1e-8 for value in residuals.values())})
    assert all(c['within_existing_1e-8_inventory_audit_resolution'] for c in ledgers)
    assert all(c['met'] for c in checks)
    return {'design_sha256': sha(design_raw), 'engine': design['engine'],
            'execution_receipt': receipt, 'checks': checks, 'all_frozen_checks_met': True,
            'original_acceptance_checks': design['original_acceptance_checks'],
            'initial_ph_measurements': measurements[:2], 'final_ph_measurements': measurements[2:],
            'model_boundaries': boundaries, 'per_observation_whole_bench_ledgers': ledgers,
            'registry_sha256': sha(registry_raw), 'basis_species_source_sha256': sha(species_raw),
            'qualification': 'Changed-runtime CLI acceptance on recorded tested merge; no empirical thermal calibration, REST production acceptance or original failure replacement.',
            'catalog_selection': 'pending_separate_review'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('design', 'execution', 'out'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    report = assess(args.design, args.execution)
    with args.out.open('x') as handle:
        json.dump(report, handle, indent=2)
        handle.write('\n')
