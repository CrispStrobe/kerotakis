#!/usr/bin/env python3
"""QA011 saved physical-dose assessment; no simulator execution."""
import hashlib
import json
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
RESULTS = HERE.parent
ATOMS = {'water': {'H': 2, 'O': 1}, 'H+': {'H': 1},
         'base_equivalents': {'H': 1, 'O': 1}, 'Na+': {'Na': 1}, 'Cl-': {'Cl': 1}}
CHARGE = {'H+': 1, 'base_equivalents': -1, 'Na+': 1, 'Cl-': -1}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def ledger(v):
    totals = dict.fromkeys(['H', 'O', 'Na', 'Cl', 'charge'], 0.0)
    assert not v.get('unresolved_materials') and not v['surfaces']
    for p in v['contents']:
        for atom, n in ATOMS[p['species']].items():
            totals[atom] += p['moles'] * n
        totals['charge'] += p['moles'] * CHARGE.get(p['species'], 0)
    return totals


def main():
    metadata = json.loads((RESULTS / 'artifact-metadata.json').read_text())
    archive = RESULTS / 'raw-artifact.zip'
    assert sha(archive.read_bytes()) == metadata['digest'].removeprefix('sha256:')
    original = json.loads((RESULTS / 'cases/batch2_qa-011.json').read_text())
    with zipfile.ZipFile(archive) as z:
        raw = z.read('batch2_qa-011/stdout.jsonl')
        script = z.read('batch2_qa-011/experiment.lab')
        assert sha(raw) == original['execution']['stdout_sha256']
        assert sha(script) == original['original_script_sha256'] == original['selected_script_sha256']
        rows = [json.loads(l) for l in raw.splitlines()]
        assert not original['named_model_limits'] and not original['solver_failures']
        assert not any(e['event'] in ['gas_evolved', 'gas_absorbed', 'solver_failed', 'not_yet_modeled'] for r in rows for e in r['events'])
        before = {v['id']: v for v in rows[4]['bench']['vessels']}
        final = {v['id']: v for v in rows[-1]['bench']['vessels']}
        assert rows[4]['operator'] == {'moles': 0.001035, 'op': 'add', 'species': 'HCl', 'vessel': 1}
        water_per_liter = rows[0]['operator']['moles'] / 0.1
        arms = []
        for row in rows:
            if row['operator'].get('op') != 'titrate':
                continue
            op = row['operator']
            endpoint, = [e for e in row['events'] if e['event'] == 'titrated']
            vessel = endpoint['vessel']
            assert endpoint['endpoint_reached']
            dose = endpoint['total_volume'] * endpoint['concentration']
            water = endpoint['total_volume'] * water_per_liter
            expected = ledger(before[vessel])
            expected['H'] += dose + 2 * water
            expected['O'] += dose + water
            expected['Na'] += dose
            actual = ledger(final[vessel])
            residual = {k: actual[k] - expected[k] for k in actual}
            assert all(abs(v) <= 1e-8 for v in residual.values())
            sodium = sum(p['moles'] for p in final[vessel]['contents'] if p['species'] == 'Na+')
            assert abs(sodium - dose) <= 1e-8
            assert abs(dose - 0.001035) <= 1e-8
            final_increment = endpoint['total_volume'] - (endpoint['steps'] - 1) * op['step']
            assert 0 < final_increment < op['step']
            arms.append({'vessel': vessel, 'requested_increment_l': op['step'],
                'actual_delivered_volume_l': endpoint['total_volume'], 'steps': endpoint['steps'],
                'final_partial_increment_l': final_increment, 'actual_delivered_NaOH_mol': dose,
                'retained_sodium_mol': sodium, 'carrier_water_mol': water,
                'ideal_reference_volume_l': 0.001035 / 0.1, 'final_ph': endpoint['final_ph'],
                'material_charge_residual_mol': residual})
        assert len(arms) == 2
    result = {'id': original['id'], 'engine': original['engine'],
        'script_sha256': sha(script), 'stdout_sha256': sha(raw), 'raw_artifact_sha256': sha(archive.read_bytes()),
        'original_physical_forecast': original['original_physical_forecast'],
        'original_checks': [{'id': c['id'], 'criterion': c['criterion'], 'result': 'met'} for c in original['original_acceptance_checks']],
        'arms': arms, 'verdict': 'original_bounded_expectation_met',
        'interpretation': 'The fractional final increment is actually dispensed: retained sodium and carrier/atom balances match the event dose. It is not only an interpolated display estimate.',
        'limitations': ['Ideal computed endpoint; real burette resolution, experimental uncertainty and empirical calibration are not modeled by this check.', 'Native CLI source-qualified; no API execution of this presentation-bearing script is claimed.'],
        'new_engine_executions': 0}
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
