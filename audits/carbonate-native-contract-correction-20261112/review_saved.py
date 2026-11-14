#!/usr/bin/env python3
"""Independent saved-predicate material audit; runs no chemistry engine."""
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
# Explicit chemical formula counts; base_equivalents is the analytical OH-
# coordinate (one O, one H, charge -1), not measured free hydroxide.
FORMULAS = {
    'water': ({'H': 2, 'O': 1}, 0),
    'CH3COOH': ({'C': 2, 'H': 4, 'O': 2}, 0),
    'CH3COO-': ({'C': 2, 'H': 3, 'O': 2}, -1),
    'NaHCO3': ({'Na': 1, 'H': 1, 'C': 1, 'O': 3}, 0),
    'CaCO3': ({'Ca': 1, 'C': 1, 'O': 3}, 0),
    'CO2(aq)': ({'C': 1, 'O': 2}, 0),
    'CO2': ({'C': 1, 'O': 2}, 0),
    'HCO3-': ({'H': 1, 'C': 1, 'O': 3}, -1),
    'CO3-2': ({'C': 1, 'O': 3}, -2),
    'Ca+2': ({'Ca': 1}, 2),
    'Na+': ({'Na': 1}, 1),
    'H+': ({'H': 1}, 1),
    'base_equivalents': ({'O': 1, 'H': 1}, -1),
    'N2': ({'N': 2}, 0),
    'O2': ({'O': 2}, 0),
}

def add(total, species, moles):
    atoms, charge = FORMULAS[species]  # Unknown identities fail closed.
    assert math.isfinite(moles)
    for element, count in atoms.items():
        total[element] = total.get(element, 0.0) + count * moles
    total['charge'] = total.get('charge', 0.0) + charge * moles


def inventory(vessel):
    assert not vessel['surfaces']
    result = {}
    for portion in vessel['contents']:
        assert portion['moles'] >= 0
        add(result, portion['species'], portion['moles'])
    return result


def review(name, rows=None):
    if rows is None:
        rows = [json.loads(line) for line in (HERE / (name + '.jsonl')).read_text().splitlines()]
    previous = {}
    max_error = 0.0
    additions = 0
    warning_keys = set()
    gas_out = {}
    measurements = {}
    for row in rows:
        operator = row.get('operator', {})
        vessel_id = operator.get('vessel')
        current = {v['id']: v for v in row['bench']['vessels']}
        events = row.get('events', [])
        assert not any(e['event'] == 'solver_failed' for e in events)
        for event in events:
            if event['event'] == 'not_yet_modeled':
                warning_keys.add(event.get('reason', {}).get('key', 'untyped'))
            if event['event'] == 'measured':
                measurements[event['vessel']] = event['value']
        if operator.get('op') in ('add', 'add_material'):
            expected = inventory(previous[vessel_id]) if vessel_id in previous else {}
            if operator['op'] == 'add':
                add(expected, operator['species'], operator['moles'])
            else:
                material = [e for e in events if e['event'] == 'material_added']
                assert len(material) == 1 and material[0]['unresolved_amount'] == 0
                for component in material[0]['components']:
                    add(expected, component['species'], component['moles'])
            for event in events:
                if event['event'] in ('gas_evolved', 'gas_absorbed'):
                    assert event['vessel'] == vessel_id
                    sign = -1 if event['event'] == 'gas_evolved' else 1
                    add(expected, event['species'], sign * event['moles'])
                    gas_out[vessel_id] = gas_out.get(vessel_id, 0.0) - sign * event['moles']
            actual = inventory(current[vessel_id])
            errors = {element: actual.get(element, 0) - expected.get(element, 0)
                      for element in set(actual) | set(expected)}
            max_error = max(max_error, *(abs(x) for x in errors.values()))
            assert all(abs(x) <= 1e-10 for x in errors.values()), (name, errors)
            additions += 1
        previous = current
    final = current[1 if name == 'lesson' else 0]
    assert final['solution'].get('scope', 'complete') == 'complete'
    provenance = final['solution']['provenance']
    assert provenance['engine'] == 'PHREEQC (IPhreeqc, USGS)'
    routing_key = provenance['routing_phrase']['key']
    assert routing_key == 'routing.crystallisation-outside-activity-domain'
    assert 'not-modeled.carbonate-fast-endpoint' in warning_keys
    result = {'additions_audited': additions, 'max_element_or_charge_error_mol': max_error,
              'routing_hold': routing_key, 'model_limits': sorted(warning_keys),
              'scientific_prediction_status': 'HELD: concentrated activity model and uncalibrated endpoint',
              'gas_out_mol_by_vessel': gas_out}
    if name != 'lesson':
        retained = sum(p['moles'] for p in final['contents']
                       if p['species'] in ('CaCO3', 'NaHCO3', 'HCO3-', 'CO3-2', 'CO2(aq)'))
        result['retained_inorganic_carbon_mol'] = retained
        result['gas_plus_retained_carbon_mol'] = gas_out[0] + retained
        assert abs(gas_out[0] + retained - 0.05) <= 1e-10
    if name == 'lesson':
        # Registry molar-mass/CO2 is 44.009 g/mol, not a rounded gas amount.
        residual = measurements[1] + gas_out[1] * 44.009 - 55.3
        assert abs(residual) < 1e-4
        result.update({'open_balance_g': measurements[1], 'co2_molar_mass_g_mol': 44.009,
                       'input_minus_gas_balance_residual_g': residual,
                       'unchanged_mass_tolerance_g': 1e-4})
    if name == 'calcite':
        activities = {s['name']: s['activity'] for s in final['solution']['species']}
        # Exact source-bound MINTEQ Calcite definition: logK(298.15)=-8.48,
        # delta_h=-8 kJ/mol; van't Hoff correction. Printed activities have
        # four significant digits. This is a saved model consistency read,
        # not validation of the warned concentrated activity model.
        log_k = -8.48 + 8000 / (8.314462618 * math.log(10)) * (1 / final['temperature'] - 1 / 298.15)
        log_q = math.log10(activities['Ca+2'] * activities['CO3-2'])
        result.update({'calcite_solid_mol': sum(p['moles'] for p in final['contents'] if p['species'] == 'CaCO3'),
                       'calcite_logQ_from_printed_activities': log_q,
                       'calcite_logK_at_reported_temperature': log_k,
                       'calcite_printed_activity_log_residual': log_q - log_k})
    return result

if __name__ == '__main__':
    print(json.dumps({name: review(name) for name in ('soda', 'calcite', 'lesson')}, indent=2))
