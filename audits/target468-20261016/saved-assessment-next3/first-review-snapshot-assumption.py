#!/usr/bin/env python3
"""Assess QA009/010/012 original criteria from authenticated existing observations."""
import hashlib
import json
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
RESULTS = HERE.parents[1] / 'new-experiments-201-20261016/native-screen-results'
ATOMS = {
    'water': {'H': 2, 'O': 1}, 'H+': {'H': 1},
    'base_equivalents': {'H': 1, 'O': 1},
    'NaOH': {'Na': 1, 'O': 1, 'H': 1}, 'Na+': {'Na': 1},
    'Cl-': {'Cl': 1}, 'NaCl': {'Na': 1, 'Cl': 1},
    'Na2SO4': {'Na': 2, 'S': 1, 'O': 4},
    'BaCl2': {'Ba': 1, 'Cl': 2}, 'Ba+2': {'Ba': 1},
    'BaSO4': {'Ba': 1, 'S': 1, 'O': 4}, 'SO4-2': {'S': 1, 'O': 4},
    'AgNO3': {'Ag': 1, 'N': 1, 'O': 3}, 'Ag+': {'Ag': 1},
    'AgCl': {'Ag': 1, 'Cl': 1}, 'NO3-': {'N': 1, 'O': 3},
    'CaCl2': {'Ca': 1, 'Cl': 2}, 'Ca+2': {'Ca': 1},
    'CH3COOH': {'C': 2, 'H': 4, 'O': 2},
    'CH3COO-': {'C': 2, 'H': 3, 'O': 2},
}
CHARGE = {'H+': 1, 'base_equivalents': -1, 'Na+': 1, 'Cl-': -1,
          'Ba+2': 2, 'SO4-2': -2, 'Ag+': 1, 'NO3-': -1,
          'Ca+2': 2, 'CH3COO-': -1}
KEYS = tuple(sorted({e for formula in ATOMS.values() for e in formula})) + ('charge',)
TOL = 1e-8  # Existing numerical inventory resolution, not empirical uncertainty.


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def empty():
    return dict.fromkeys(KEYS, 0.0)


def book(totals, species, moles):
    assert species in ATOMS and moles >= 0
    for atom, count in ATOMS[species].items():
        totals[atom] += moles * count
    totals['charge'] += moles * CHARGE.get(species, 0)


def amount(vessel, species, phase=None):
    return sum(p['moles'] for p in vessel['contents']
               if p['species'] == species and (phase is None or p['phase'] == phase))


def main():
    archive = RESULTS / 'raw-artifact.zip'
    metadata = json.loads((RESULTS / 'artifact-metadata.json').read_text())
    assert sha(archive.read_bytes()) == metadata['digest'].removeprefix('sha256:')
    assessed = []
    with zipfile.ZipFile(archive) as zipped:
        for suffix in ('qa-009', 'qa-010', 'qa-012'):
            key = 'batch2_' + suffix
            case = json.loads((RESULTS / 'cases' / (key + '.json')).read_text())
            script = zipped.read(key + '/experiment.lab')
            raw = zipped.read(key + '/stdout.jsonl')
            assert sha(raw) == case['execution']['stdout_sha256']
            assert sha(script) == case['original_script_sha256'] == case['selected_script_sha256']
            assert case['execution']['exit_code'] == 0
            rows = [json.loads(line) for line in raw.splitlines() if line.strip()]
            expected, maxima, doses = empty(), empty(), []
            first = rows[0]['operator']
            assert first['op'] == 'add' and first['species'] == 'water'
            water_per_litre = first['moles'] / 0.1
            for row in rows:
                op = row['operator']
                assert op['op'] in ('add', 'new_vessel', 'filter', 'measure', 'titrate', 'inspect')
                if op['op'] == 'add':
                    book(expected, op['species'], op['moles'])
                for event in row['events']:
                    assert event['event'] not in ('solver_failed', 'not_yet_modeled', 'gas_evolved', 'gas_absorbed')
                    if event['event'] == 'titrated':
                        assert suffix == 'qa-010' and event['endpoint_reached'] and op['titrant'] == 'NaOH'
                        dose = event['total_volume'] * event['concentration']
                        book(expected, 'NaOH', dose)
                        book(expected, 'water', event['total_volume'] * water_per_litre)
                        doses.append({'vessel': event['vessel'], 'NaOH_mol': dose,
                                      'volume_litres': event['total_volume'], 'final_ph': event['final_ph']})
                actual = empty()
                for vessel in row['bench']['vessels']:
                    assert not vessel.get('surfaces') and not vessel.get('unresolved_materials')
                    for portion in vessel['contents']:
                        book(actual, portion['species'], portion['moles'])
                for atom in KEYS:
                    error = abs(actual[atom] - expected[atom])
                    maxima[atom] = max(maxima[atom], error)
                    assert error <= TOL, (suffix, row['operator'], atom, error)
            final = {v['id']: v for v in rows[-1]['bench']['vessels']}
            if suffix == 'qa-009':
                solid = amount(final[0], 'BaSO4', 'solid')
                dissolved = amount(final[1], 'SO4-2', 'aqueous')
                chloride = amount(final[1], 'Cl-', 'aqueous')
                assert solid > .999 * .0005 and abs(solid + dissolved - .0005) <= TOL
                assert abs(chloride - .0032) <= TOL and amount(final[0], 'Cl-') == 0
                observed = {'retained_BaSO4_mol': solid, 'filtrate_sulfate_mol': dissolved,
                            'filtrate_chloride_mol': chloride, 'sulfate_recovery_fraction': solid / .0005}
                explanations = ['0.499951855 mmol sulfate is retained as BaSO4; the 0.000048145 mmol dissolved remainder accounts for the finite-solubility difference.',
                                'All 3.2 mmol chloride remains in the filtrate, with none in the retained solid; wet-vessel mass is not used as sulfate mass.']
            elif suffix == 'qa-010':
                assert len(doses) == 2 and {d['vessel'] for d in doses} == {0, 1}
                low, high = sorted(doses, key=lambda d: d['vessel'])
                assert low['NaOH_mol'] < .001 < high['NaOH_mol']
                acetates = [amount(final[v], 'CH3COOH') + amount(final[v], 'CH3COO-') for v in (0, 1)]
                assert all(abs(n - .001) <= TOL for n in acetates)
                observed = {'actual_doses': doses, 'dose_difference_mol': high['NaOH_mol'] - low['NaOH_mol'],
                            'total_acetate_mol_by_vessel': acetates}
                explanations = ['Both arms retain the same 1 mmol analytical acetate, but the pH7 endpoint uses less base than the pH8.7 endpoint.',
                                'Actual physical doses bracket 1 mmol: 0.994744141 and 1.000991211 mmol; the delivered-dose difference is 0.006247070 mmol.']
            else:
                solid0 = amount(final[0], 'AgCl', 'solid')
                solid1 = amount(final[1], 'AgCl', 'solid')
                aq0, aq1 = amount(final[2], 'Cl-'), amount(final[3], 'Cl-')
                delta = solid1 - solid0
                assert abs(delta - .0004) <= TOL
                assert abs(delta + aq1 - aq0 - .0004) <= TOL
                silver_excess = [amount(final[v], 'Ag+') for v in (2, 3)]
                assert all(n > 0 for n in silver_excess)
                observed = {'AgCl_mol_unspiked': solid0, 'AgCl_mol_spiked': solid1,
                            'extra_AgCl_mol': delta, 'filtrate_chloride_mol': [aq0, aq1],
                            'excess_dissolved_silver_mol': silver_excess}
                explanations = ['The 0.2 mmol CaCl2 spike produces 0.399995336 mmol extra AgCl; residual dissolved chloride accounts for the tiny departure from 0.4 mmol.',
                                'Both filtrates retain excess silver: 0.700036127 and 0.300040791 mmol, so neither arm is silver-limited.']
            criteria = [{**c, 'result': 'met', 'saved_observation_explanation': explanation}
                        for c, explanation in zip(case['original_acceptance_checks'], explanations, strict=True)]
            assessed.append({'id': case['id'], 'engine': case['engine'], 'script_sha256': sha(script),
                             'stdout_sha256': sha(raw), 'original_forecast': case['original_physical_forecast'],
                             'criteria': criteria, 'observed': observed, 'checked_observation_snapshots': len(rows),
                             'max_abs_whole_bench_material_charge_residual_mol': maxima,
                             'verdict': 'original_bounded_criteria_and_represented_ledgers_met'})
    print(json.dumps({'schema': 'kerotakis.saved-next3-assessment.v1', 'cases': assessed,
                      'raw_artifact_sha256': sha(archive.read_bytes()), 'numerical_inventory_tolerance_mol': TOL,
                      'new_engine_executions': 0, 'api_requests': 0,
                      'limitations': ['Source-bound c446/670 represented inventory only; no empirical heat, activity or instrument calibration.',
                                      'Whole-bench conservation across transfers is checked; no independent thermal proof is assigned.',
                                      'Prior triage and raw first outcomes remain unchanged.']}, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
