#!/usr/bin/env python3
"""Assess the separately frozen carbonate controls from authenticated saved bytes."""
import hashlib
import json
import math
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
ATOMS = {
    'water': {'H': 2, 'O': 1}, 'H+': {'H': 1},
    'base_equivalents': {'H': 1, 'O': 1}, 'Na+': {'Na': 1}, 'Cl-': {'Cl': 1},
    'CO2(aq)': {'C': 1, 'O': 2}, 'CO2': {'C': 1, 'O': 2},
    'HCO3-': {'H': 1, 'C': 1, 'O': 3}, 'CO3-2': {'C': 1, 'O': 3},
    'O2': {'O': 2}, 'N2': {'N': 2},
}
CHARGE = {'H+': 1, 'base_equivalents': -1, 'Na+': 1, 'Cl-': -1, 'HCO3-': -1, 'CO3-2': -2}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def ledger(vessel):
    result = {k: 0.0 for k in ['H', 'O', 'C', 'Na', 'Cl', 'N', 'charge']}
    assert not vessel.get('unresolved_materials') and not vessel['surfaces']
    for portion in vessel['contents']:
        species, n = portion['species'], portion['moles']
        assert math.isfinite(n) and n >= 0
        for element, count in ATOMS[species].items():
            result[element] += count * n
        result['charge'] += CHARGE.get(species, 0) * n
    return result


def main():
    archive = (HERE / 'raw-artifact.zip').read_bytes()
    metadata = json.loads((HERE / 'artifact-metadata.json').read_text())
    assert sha(archive) == metadata['digest'].removeprefix('sha256:')
    design_bytes = (HERE.parent / 'design.json').read_bytes()
    design = json.loads(design_bytes)
    with zipfile.ZipFile(HERE / 'raw-artifact.zip') as zipped:
        binding = json.loads(zipped.read('binding.json'))
        assert binding['design_sha256'] == sha(design_bytes)
        assert binding['engine'] == design['engine']
        assert binding['workflow_commit'] == 'e4cc782e08180286fe5bdb1c6aa6ec36f904cab9'
        assert binding['input_checkout_commit'] == 'd847a0b112ffd0e85dd655ce98bf5448c9538008'
        assert binding['runner_sha256'] == sha((HERE.parent / 'hosted_screen.py').read_bytes())
        execution = json.loads(zipped.read('execution.json'))
        assert len(execution['results']) == 2
        results = []
        for case, run in zip(design['cases'], execution['results']):
            assert run['id'] == case['id'] and run['exit_code'] == 0
            folder = case['id'].replace(':', '_')
            raw = zipped.read(folder + '/stdout.jsonl')
            assert sha(raw) == run['stdout_sha256']
            assert sha(zipped.read(folder + '/stderr.txt')) == run['stderr_sha256']
            assert sha(zipped.read(folder + '/experiment.lab')) == case['selected_script_sha256']
            rows = [json.loads(line) for line in raw.splitlines()]
            sealed_index = next(i for i, row in enumerate(rows) if any(e['event'] == 'vessel_sealed' for e in row['events']))
            initial = rows[sealed_index]['bench']['vessels'][0]
            final = rows[-1]['bench']['vessels'][0]
            events = [e for row in rows[sealed_index:] for e in row['events']]
            endpoints = [e for e in events if e['event'] == 'titrated']
            assert len(endpoints) == 2
            doses = [e['concentration'] * e['total_volume'] for e in endpoints]
            volume = sum(e['total_volume'] for e in endpoints)
            # Both original scripts add100mL water. Titration uses the same
            # canonical water moles_from_liters conversion (bench.rs).
            assert rows[0]['operator']['op'] == 'add' and rows[0]['operator']['species'] == 'water'
            carrier = rows[0]['operator']['moles'] / 0.1 * volume
            expected = ledger(initial)
            expected['H'] += sum(doses) + 2 * carrier
            expected['O'] += carrier
            expected['Cl'] += sum(doses)
            final_elements = ledger(final)
            residuals = {k: final_elements[k] - expected[k] for k in expected}
            excluded = [e['event'] for e in events if e['event'] in ['solver_failed', 'not_yet_modeled', 'gas_evolved', 'gas_absorbed', 'burst', 'container_broken']]
            gas_moles = sum(p['moles'] for p in final['contents'] if p['phase'] == 'gas')
            # Existing model's R in litre pascal per mole kelvin, vessel.rs.
            ideal_pressure = gas_moles * 8314.462618 * final['temperature'] / final['headspace']['volume']
            scene = next(row['scene'] for row in reversed(rows) if 'scene' in row)
            liquid_volume = next(v['liquid']['volume_l'] for v in scene['vessels'] if v['id'] == 0)
            checks = {
                'first_endpoint_forecast': abs(doses[0] - case['expected_first_endpoint_acid_mol']) <= case['endpoint_amount_tolerance_mol'],
                'cumulative_endpoint_forecast': abs(sum(doses) - case['expected_cumulative_second_endpoint_acid_mol']) <= case['endpoint_amount_tolerance_mol'],
                'both_endpoints_reached': all(e['endpoint_reached'] for e in endpoints),
                'no_solver_coverage_or_export_events_after_seal': not excluded,
                'retained_sealed_headspace': final['headspace'] == {'boundary': 'sealed', 'volume': 0.001},
                'material_charge_with_carrier_and_trapped_air': all(math.isfinite(r) and abs(r) <= case['major_element_tolerance_mol'] for r in residuals.values()),
            }
            results.append({'id': case['id'], 'script_sha256': case['selected_script_sha256'],
                'stdout_sha256': sha(raw), 'endpoint_acid_mol': doses,
                'cumulative_acid_mol': sum(doses), 'original_open_first_endpoint_acid_mol': case['original_open_control']['first_endpoint_acid_mol'],
                'titrant_carrier_water_mol': carrier, 'initial_sealed_carbon_mol_including_air': ledger(initial)['C'],
                'final_carbon_mol_including_gas': final_elements['C'], 'element_charge_residual_mol': residuals,
                'excluded_events_after_seal': excluded, 'headspace_l': final['headspace']['volume'],
                'reported_liquid_volume_l': liquid_volume, 'reported_pressure_pa': final['pressure'],
                'inventory_ideal_pressure_pa': ideal_pressure, 'pressure_difference_pa': final['pressure'] - ideal_pressure,
                'checks': checks, 'verdict': 'bounded_forecasts_met' if all(checks.values()) else 'bounded_forecast_or_domain_failure'})
    print(json.dumps({'run': 38875794689, 'engine': design['engine'], 'raw_artifact_sha256': sha(archive),
        'cases': results, 'new_review_executions': 0, 'catalog_additions': 0,
        'scope': 'Finite existing model boundary controls; no general open-system titration, empirical or thermal calibration claim.'}, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
