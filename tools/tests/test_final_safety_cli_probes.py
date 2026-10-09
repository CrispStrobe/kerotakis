"""Synthetic protocol checks only; no CLI execution or chemistry claims."""
import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    'probes', Path(__file__).resolve().parents[1] / 'final-safety-cli-probes.py')
probes = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probes)


def rows(positive=True, repaired=True):
    def vessel(nh3):
        contents = [{'species': 'NH3', 'moles': nh3, 'phase': 'aqueous'}]
        if positive:
            contents.append({'species': 'ClO-', 'moles': 1e-10, 'phase': 'aqueous'})
        return {'id': 0, 'contents': contents, 'solution': {'provenance': {'engine': 'synthetic'}}}
    result = [{'step': i, 'operator': {'op': 'add'}, 'events': [],
               'bench': {'vessels': [vessel(1e-13)]}} for i in range(3)]
    events = [{'event': 'titrated', 'vessel': 0, 'titrant': 'NaOH',
               'concentration': 0.001, 'total_volume': 0.001, 'steps': 1, 'final_ph': 9}]
    if positive and repaired:
        events.append({'event': 'hazard_warning', 'severity': 'danger', 'rule': probes.RULE,
                       'hazard': probes.HAZARD, 'real_world': probes.REAL_WORLD})
    result.append({'step': 3, 'operator': {'op': 'titrate'}, 'events': events,
                   'bench': {'vessels': [vessel(2e-12)]}})
    return result


class ProbeTests(unittest.TestCase):
    def analyse(self, value, positive=True, repaired=True):
        return probes.analyse(value, {'exit_code': 0, 'timeout': False}, positive, repaired)

    def test_positive_and_negative_observations(self):
        for positive, repaired in ((True, True), (True, False), (False, True)):
            self.assertEqual(self.analyse(rows(positive, repaired), positive, repaired)['classification'],
                             'qualifying_inventory_and_warning_observation')

    def test_zero_based_steps_match_cli_contract_and_reject_one_based(self):
        value = rows()
        self.assertTrue(self.analyse(value)['checks']['four_frozen_operators'])
        for row in value:
            row['step'] += 1
        self.assertEqual(self.analyse(value)['classification'], 'not_exercising_intended_route')

    def test_missing_inventory_witness_is_never_warning_pass(self):
        value = rows()
        value[3]['bench']['vessels'][0]['contents'][0]['moles'] = 1e-12
        self.assertEqual(self.analyse(value)['classification'], 'not_exercising_intended_route')
        negative = rows(False)
        negative[3]['bench']['vessels'][0]['contents'][0]['moles'] = 0
        self.assertEqual(self.analyse(negative, False)['classification'], 'not_exercising_intended_route')

    def test_refusal_and_partial_or_refined_dose_are_not_exercising(self):
        for change in ('refusal', 'partial', 'refined', 'duplicate', 'missing_row'):
            value = rows()
            if change == 'refusal':
                value[3]['events'].append({'event': 'solver_failed'})
            elif change == 'partial':
                value[3]['events'][0]['total_volume'] = 0.0005
            elif change == 'refined':
                value[3]['events'][0]['steps'] = 2
            elif change == 'duplicate':
                value[3]['events'].append(copy.deepcopy(value[3]['events'][0]))
            else:
                value.pop(1)
            self.assertEqual(self.analyse(value)['classification'], 'not_exercising_intended_route')

    def test_warning_missing_duplicate_wrong_metadata_and_setup_leak(self):
        for change in ('missing', 'duplicate', 'metadata', 'setup'):
            value = rows()
            if change == 'missing':
                value[3]['events'].pop()
            elif change == 'duplicate':
                value[3]['events'].append(copy.deepcopy(value[3]['events'][-1]))
            elif change == 'metadata':
                value[3]['events'][-1]['real_world'] = 'invented'
            else:
                value[0]['events'].append(copy.deepcopy(value[3]['events'][-1]))
            self.assertEqual(self.analyse(value)['classification'], 'warning_contract_mismatch')

    def test_nonfinite_negative_bool_and_duplicate_vessel_rejected(self):
        for amount in (float('nan'), float('inf'), -1, True):
            value = rows()
            value[2]['bench']['vessels'][0]['contents'][0]['moles'] = amount
            with self.assertRaises(ValueError):
                self.analyse(value)
        value = rows()
        value[3]['bench']['vessels'].append(copy.deepcopy(value[3]['bench']['vessels'][0]))
        with self.assertRaises(ValueError):
            self.analyse(value)


if __name__ == '__main__':
    unittest.main()
