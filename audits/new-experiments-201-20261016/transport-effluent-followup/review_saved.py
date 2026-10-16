#!/usr/bin/env python3
"""Assess the frozen two-case baseline from saved artifacts; never run an engine."""
import argparse
import hashlib
import json
import math
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text())


def ledger(vessel):
    atoms = {'water': (2, 1), 'H+': (1, 0), 'base_equivalents': (1, 1)}
    charge = {'water': 0, 'H+': 1, 'base_equivalents': -1}
    assert not vessel.get('unresolved_materials') and not vessel.get('surfaces')
    totals = dict(H=0.0, O=0.0, charge=0.0)
    for portion in vessel['contents']:
        species, moles = portion['species'], portion['moles']
        assert species in atoms and math.isfinite(moles) and moles >= 0
        totals['H'] += atoms[species][0] * moles
        totals['O'] += atoms[species][1] * moles
        totals['charge'] += charge[species] * moles
    return totals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact_directory', type=Path)
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    design, dispatch = load(here / 'design.json'), load(here / 'dispatch.json')
    root = args.artifact_directory
    binding, execution = load(root / 'binding.json'), load(root / 'execution.json')
    completion = load(root / 'completion.json')
    assert load(root / 'design.json') == design
    assert sha(here / 'design.json') == binding['design_sha256'] == dispatch['design_sha256']
    assert sha(here / 'hosted_screen.py') == binding['runner_sha256'] == dispatch['runner_sha256']
    assert binding['workflow_commit'] == dispatch['isolated_validation_head']
    assert binding['input_checkout_commit'] == dispatch['input_checkout_commit']
    assert binding['engine'] == execution['engine'] == design['engine']
    assert completion['attempts'] == len(execution['results']) == len(design['cases']) == 2
    expected = {case['id']: case for case in design['cases']}
    assert set(expected) == {row['id'] for row in execution['results']}
    results = []
    for result in execution['results']:
        case = expected[result['id']]
        saved = root / result['id'].replace(':', '_')
        assert sha(saved / 'experiment.lab') == result['script_sha256'] == case['selected_script_sha256']
        assert sha(saved / 'stdout.jsonl') == result['stdout_sha256']
        assert sha(saved / 'stderr.txt') == result['stderr_sha256']
        rows = [json.loads(line) for line in (saved / 'stdout.jsonl').read_text().splitlines() if line.strip()]
        transport = [i for i, row in enumerate(rows) if row['operator']['op'] == 'transport']
        record = {'id': result['id'], 'stdout_sha256': result['stdout_sha256']}
        if len(transport) != 1:
            record.update(verdict='inconclusive_no_single_transport_snapshot', execution=result)
            results.append(record)
            continue
        index, = transport
        assert index > 0
        op = rows[index]['operator']
        assert op == {'op': 'transport', 'chain': [0], 'inlet': 1, 'receiver': 2, 'steps': 1, 'courant': 1.0}
        before = {v['id']: v for v in rows[index - 1]['bench']['vessels']}
        after = {v['id']: v for v in rows[index]['bench']['vessels']}
        receiver_delta = after[2]['temperature'] - before[0]['temperature']
        cell_delta = after[0]['temperature'] - before[1]['temperature']
        assert all(math.isfinite(t) for t in (receiver_delta, cell_delta))
        old_cell, inlet = ledger(before[0]), ledger(before[1])
        old_receiver, new_receiver = ledger(before[2]), ledger(after[2])
        new_cell = ledger(after[0])
        residuals = {
            'receiver_minus_previous_cell': {k: new_receiver[k] - old_receiver[k] - old_cell[k] for k in old_cell},
            'cell_minus_injected_inlet': {k: new_cell[k] - inlet[k] for k in inlet},
            'representative_inlet_change': {k: ledger(after[1])[k] - inlet[k] for k in inlet},
        }
        gaps = [event for row in rows for event in row['events']
                if event['event'] in ('solver_failed', 'not_yet_modeled')]
        checks = {
            'execution_completed': result['status'] == 'zero_exit' and result['exit_code'] == 0 and result['stdout_parse_error'] is None,
            'receiver_temperature': abs(receiver_delta) <= case['temperature_tolerance_k'],
            'cell_temperature': abs(cell_delta) <= case['temperature_tolerance_k'],
            'material_and_charge': all(abs(v) <= case['material_tolerance_mol'] for group in residuals.values() for v in group.values()),
            'no_named_model_or_solver_gap': not gaps,
        }
        record.update(checks=checks, receiver_temperature_error_k=receiver_delta,
                      cell_temperature_error_k=cell_delta, ledger_residuals_mol=residuals,
                      model_or_solver_gaps=gaps,
                      verdict='bounded_forecasts_met' if all(checks.values()) else 'bounded_expectation_not_met')
        results.append(record)
    print(json.dumps({'run': dispatch['run'], 'engine': design['engine'], 'cases': results,
                      'new_engine_executions': 0, 'catalog_additions': 0,
                      'scope': 'Saved one-cell controls only; no general hydraulic or empirical heat calibration claim.'},
                     indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
