#!/usr/bin/env python3
"""Assess three original quantitative checks using saved authenticated output only."""
import hashlib
import json
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
RESULTS = HERE.parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    archive = RESULTS / 'raw-artifact.zip'
    metadata = json.loads((RESULTS / 'artifact-metadata.json').read_text())
    assert sha(archive.read_bytes()) == metadata['digest'].removeprefix('sha256:')
    cases = []
    with zipfile.ZipFile(archive) as zipped:
        for case_id in ['qa-001', 'qa-002', 'qa-003']:
            key = 'batch2_' + case_id
            saved = json.loads((RESULTS / 'cases' / (key + '.json')).read_text())
            raw = zipped.read(key + '/stdout.jsonl')
            script = zipped.read(key + '/experiment.lab')
            assert sha(raw) == saved['execution']['stdout_sha256']
            assert sha(script) == saved['selected_script_sha256'] == saved['original_script_sha256']
            rows = [json.loads(line) for line in raw.splitlines()]
            events = [event for row in rows for event in row['events']]
            assert not any(e['event'] == 'solver_failed' for e in events)
            endpoints = [e for e in events if e['event'] == 'titrated']
            assert len(endpoints) == 2 and all(e['endpoint_reached'] for e in endpoints)
            assert [e['vessel'] for e in endpoints] == [0, 1]
            doses = [e['concentration'] * e['total_volume'] for e in endpoints]
            states = {v['id']: v for v in rows[-1]['bench']['vessels']}
            def amount(vessel, species):
                return sum(p['moles'] for p in states[vessel]['contents'] if p['species'] in species)
            tolerance = 1e-8  # Inventory audit resolution, not empirical calibration.
            measured = {'endpoint_base_mol': doses}
            if case_id == 'qa-001':
                # Original C1 explicitly permits two burette increments.
                dose_tolerance = 2 * 0.1 * 0.0002
                recovered = (doses[0] - doses[1]) / 2
                silicon = amount(1, {'SiO2'})
                introduced_silica = sum(row['operator']['moles'] for row in rows
                                        if row['operator'].get('op') == 'add' and
                                        row['operator'].get('species') == 'SiO2' and
                                        row['operator'].get('vessel') == 1)
                calcium = amount(1, {'Ca+2'})
                exported_c = sum(e['moles'] for e in events if e['event'] == 'gas_evolved' and e['species'] == 'CO2')
                measured.update(recovered_CaCO3_mol=recovered, retained_SiO2_mol=silicon,
                                retained_Ca_mol=calcium, exported_CO2_mol=exported_c,
                                introduced_SiO2_mol=introduced_silica,
                                dose_tolerance_mol=dose_tolerance)
                checks = [abs(doses[0] - doses[1] - 0.002) <= dose_tolerance,
                          abs(recovered - 0.001) <= dose_tolerance / 2 and
                          abs(silicon - introduced_silica) <= tolerance and
                          abs(calcium - 0.001) <= tolerance and abs(exported_c - 0.001) <= tolerance]
            elif case_id == 'qa-002':
                dose_tolerance = 2 * 0.1 * 0.0001
                acetate = [amount(v, {'CH3COOH', 'CH3COO-'}) for v in [0, 1]]
                measured.update(recovered_spike_mol=doses[1] - doses[0],
                                retained_acetate_mol=acetate, dose_tolerance_mol=dose_tolerance)
                checks = [abs(doses[1] - doses[0] - 0.0005) <= dose_tolerance,
                          all(abs(n - 0.001) <= tolerance for n in acetate)]
            else:
                # Do not invent a liquid-volume measurement from the gas meter.
                dose_tolerance = 2 * 0.1 * 0.0001
                ratio = doses[0] / doses[1]
                refusals = [e for e in events if e['event'] == 'not_yet_modeled']
                assert sum(e.get('reason', {}).get('key') == 'not-modeled.volume-meter-on-an-open-vessel' for e in refusals) == 2
                measured.update(endpoint_dose_ratio=ratio, liquid_volume_measurement_available=False,
                                gas_meter_refusals=2, dose_tolerance_mol=dose_tolerance)
                checks = [abs(doses[0] - 0.0015) <= dose_tolerance and
                          abs(doses[1] - 0.0005) <= dose_tolerance, None]
            criteria = []
            for original, result in zip(saved['original_acceptance_checks'], checks):
                criteria.append({'id': original['id'], 'criterion': original['criterion'],
                                 'result': 'met' if result is True else 'unmet' if result is False else 'unsupported_measurement'})
            cases.append({'id': saved['id'], 'source_engine': saved['engine'],
                          'script_sha256': sha(script), 'stdout_sha256': sha(raw),
                          'original_forecast': saved['original_physical_forecast'],
                          'criteria': criteria, 'observations': measured,
                          'verdict': 'original_bounded_checks_met' if all(c is True for c in checks) else 'partial_checks_with_coverage_limit',
                          'catalog_case_closed': False})
    result = {'raw_artifact_sha256': sha(archive.read_bytes()), 'new_executions': 0,
              'scope': 'Saved evidence assessment of original criteria; no new forecasts, empirical calibration or full atom/energy certification.',
              'cases': cases, 'catalog_additions': 0}
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
