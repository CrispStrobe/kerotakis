#!/usr/bin/env python3
"""Supplement QA001/002 original-criterion review with saved per-step ledgers."""
import hashlib
import json
import math
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
RESULTS = HERE.parent
ATOMS = {
    'water': {'H': 2, 'O': 1}, 'H+': {'H': 1},
    'base_equivalents': {'H': 1, 'O': 1}, 'HCl': {'H': 1, 'Cl': 1},
    'NaOH': {'Na': 1, 'O': 1, 'H': 1}, 'Na+': {'Na': 1}, 'Cl-': {'Cl': 1},
    'CaCO3': {'Ca': 1, 'C': 1, 'O': 3}, 'Ca+2': {'Ca': 1},
    'SiO2': {'Si': 1, 'O': 2}, 'CO2': {'C': 1, 'O': 2},
    'CO2(aq)': {'C': 1, 'O': 2}, 'CO3-2': {'C': 1, 'O': 3},
    'HCO3-': {'H': 1, 'C': 1, 'O': 3},
    'CH3COOH': {'C': 2, 'H': 4, 'O': 2}, 'CH3COO-': {'C': 2, 'H': 3, 'O': 2},
}
CHARGE = {'H+': 1, 'base_equivalents': -1, 'Na+': 1, 'Cl-': -1,
          'Ca+2': 2, 'CO3-2': -2, 'HCO3-': -1, 'CH3COO-': -1}
KEYS = ('H', 'O', 'C', 'Na', 'Cl', 'Ca', 'Si', 'charge')
TOLERANCE = 1e-8  # Same inventory audit resolution; not empirical uncertainty.


def sha(data):
    return hashlib.sha256(data).hexdigest()


def empty():
    return dict.fromkeys(KEYS, 0.0)


def book(totals, species, moles):
    assert species in ATOMS and math.isfinite(moles)
    for element, count in ATOMS[species].items():
        totals[element] += count * moles
    totals['charge'] += CHARGE.get(species, 0) * moles


def main():
    metadata = json.loads((RESULTS / 'artifact-metadata.json').read_text())
    archive = (RESULTS / 'raw-artifact.zip').read_bytes()
    assert sha(archive) == metadata['digest'].removeprefix('sha256:')
    original_review = json.loads((HERE / 'assessment.json').read_text())
    cases = []
    with zipfile.ZipFile(RESULTS / 'raw-artifact.zip') as zipped:
        for case_id in ('qa-001', 'qa-002'):
            key = 'batch2_' + case_id
            intake = json.loads((RESULTS / 'cases' / (key + '.json')).read_text())
            raw = zipped.read(key + '/stdout.jsonl')
            script = zipped.read(key + '/experiment.lab')
            assert sha(raw) == intake['execution']['stdout_sha256']
            assert sha(script) == intake['original_script_sha256'] == intake['selected_script_sha256']
            assert intake['execution']['status'] == 'zero_exit' and intake['execution']['exit_code'] == 0
            rows = [json.loads(line) for line in raw.splitlines() if line.strip()]
            # Frozen protocol is all at the standard input-water temperature.
            first = rows[0]['operator']
            assert first == {'op': 'add', 'species': 'water', 'moles': first['moles'], 'vessel': 0}
            water_per_litre = first['moles'] / 0.1
            expected, maxima, exports, titrants = {}, empty(), [], []
            for row in rows:
                op = row['operator']
                assert op['op'] in ('add', 'new_vessel', 'sweep', 'wait', 'titrate', 'inspect')
                if op['op'] == 'add':
                    book(expected.setdefault(op['vessel'], empty()), op['species'], op['moles'])
                for event in row['events']:
                    assert event['event'] not in ('solver_failed', 'not_yet_modeled')
                    if event['event'] == 'titrated':
                        assert op['op'] == 'titrate' and op['titrant'] == 'NaOH' and event['endpoint_reached']
                        assert event['vessel'] == op['vessel']
                        target = expected.setdefault(event['vessel'], empty())
                        dose = event['total_volume'] * event['concentration']
                        carrier = event['total_volume'] * water_per_litre
                        book(target, 'NaOH', dose)
                        book(target, 'water', carrier)
                        titrants.append({'vessel': event['vessel'], 'NaOH_mol': dose, 'carrier_water_mol': carrier})
                    if event['event'] in ('gas_evolved', 'gas_absorbed'):
                        assert case_id == 'qa-001' and event['event'] == 'gas_evolved' and event['species'] == 'CO2'
                        book(expected[event['vessel']], event['species'], -event['moles'])
                        exports.append(event)
                for vessel in row['bench']['vessels']:
                    assert not vessel.get('surfaces') and not vessel.get('unresolved_materials')
                    actual = empty()
                    for portion in vessel['contents']:
                        assert portion['moles'] >= 0
                        book(actual, portion['species'], portion['moles'])
                    introduced = expected.setdefault(vessel['id'], empty())
                    for element in KEYS:
                        residual = actual[element] - introduced[element]
                        maxima[element] = max(maxima[element], abs(residual))
                        assert abs(residual) <= TOLERANCE, (case_id, row['operator'], element, residual)
            criterion_review, = [c for c in original_review['cases'] if c['id'] == intake['id']]
            assert criterion_review['verdict'] == 'original_bounded_checks_met'
            assert all(c['result'] == 'met' for c in criterion_review['criteria'])
            cases.append({'id': intake['id'], 'engine': intake['engine'], 'script_sha256': sha(script),
                          'stdout_sha256': sha(raw), 'checked_snapshots': len(rows),
                          'max_abs_per_step_material_charge_residual_mol': maxima,
                          'titrant_material': titrants, 'gas_exports': exports,
                          'original_criteria': criterion_review['criteria'],
                          'verdict': 'bounded_criteria_and_represented_per_step_ledgers_met'})
    print(json.dumps({'cases': cases, 'raw_artifact_sha256': sha(archive),
                      'material_tolerance_mol': TOLERANCE, 'new_engine_executions': 0,
                      'limitations': ['Represented inventory and original dose/retention criteria only.',
                                      'No empirical endpoint, activity, heat, rate or instrument calibration claim.',
                                      'Native c446/670 execution only; no API execution claim.']},
                     indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
