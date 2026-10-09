"""Rejection controls for paired-observation review; no Kero or builds."""
import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    'pair_review', Path(__file__).resolve().parents[1] / 'final-safety-cli-pair-review.py')
pair = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pair)
FIXTURE = importlib.util.spec_from_file_location(
    'fixture', Path(__file__).with_name('test_final_safety_cli_probes.py'))
fixture = importlib.util.module_from_spec(FIXTURE)
FIXTURE.loader.exec_module(fixture)
PROCESS = {'exit_code': 0, 'timeout': False}


def protocol(positive=True, repaired=True):
    rows = fixture.rows(positive, repaired)
    for row, key, amount in zip(rows[:3], ['water', 'NaOCl' if positive else 'NaCl', 'NH4Cl'],
                                [5.5, 1e-10, 1e-10]):
        row['operator'] = {'op': 'add', 'vessel': 0, 'species': key, 'moles': amount}
    rows[3]['operator'] = {'op': 'titrate', 'vessel': 0, 'titrant': 'NaOH',
                           'concentration': 0.001, 'step': 0.001, 'target_ph': 12, 'max_steps': 1}
    for row in rows:
        row['bench']['vessels'][0]['solution']['provenance'].update(
            dataset='synthetic dataset', model='synthetic model', routing='synthetic route')
    return rows


def baseline(positive=True):
    result = pair.probes.analyse(protocol(positive, False), PROCESS, positive, False)
    result['setup_backend'] = result['setup_solution']['provenance']
    result['final_backend'] = result['final_solution']['provenance']
    return result


def narration(positive=True):
    text = 'titrated with 0.001 mol/L sodium hydroxide; 1 step(s), 1.0 mL total\n'
    if positive:
        text += f'HAZARD (Danger): {pair.probes.HAZARD} — {pair.probes.REAL_WORLD}\n'
    return text


class PairReviewTests(unittest.TestCase):
    def review(self, rows, positive=True, text=None, before=None):
        return pair.qualifying_observation(rows, PROCESS, narration(positive) if text is None else text,
                                          positive, baseline(positive) if before is None else before)

    def test_positive_and_negative_require_distinct_warning_behavior(self):
        for positive in (True, False):
            result = self.review(protocol(positive), positive)
            self.assertEqual(result['classification'], 'qualifying_inventory_and_warning_observation')

    def test_unrepaired_positive_cannot_pass_as_repaired(self):
        with self.assertRaises(AssertionError):
            self.review(protocol(True, False))

    def test_benign_warning_cannot_pass(self):
        rows = protocol(False)
        rows[3]['events'].append(copy.deepcopy(protocol(True)[3]['events'][-1]))
        with self.assertRaises(AssertionError):
            self.review(rows, False)

    def test_missing_inventory_witness_cannot_pass(self):
        rows = protocol(True)
        rows[3]['bench']['vessels'][0]['contents'][0]['moles'] = 1e-12
        with self.assertRaises(AssertionError):
            self.review(rows)

    def test_changed_backend_cannot_borrow_baseline_provenance(self):
        for key in ('engine', 'dataset', 'model', 'routing'):
            rows = protocol(True)
            rows[3]['bench']['vessels'][0]['solution']['provenance'][key] = 'different'
            with self.assertRaises(AssertionError):
                self.review(rows)

    def test_nonqualifying_baseline_cannot_supply_contrast(self):
        before = baseline()
        before['checks']['final_nh3_above_cutoff'] = False
        with self.assertRaises(AssertionError):
            self.review(protocol(), before=before)

    def test_text_warning_missing_duplicate_wrong_metadata_or_severity_rejected(self):
        good = narration()
        for text in (narration(False), good + good, good.replace(pair.probes.REAL_WORLD, 'invented'),
                     good.replace('Danger', 'Caution')):
            with self.assertRaises(AssertionError):
                self.review(protocol(), text=text)
        with self.assertRaises(AssertionError):
            self.review(protocol(False), False, text=good)

    def test_wrong_parsed_input_rejected_even_with_matching_titrated_event(self):
        for change in ('reagent', 'amount', 'concentration', 'step', 'target', 'budget', 'vessel_type'):
            rows = protocol()
            if change == 'reagent':
                rows[1]['operator']['species'] = 'CaO'
            elif change == 'amount':
                rows[2]['operator']['moles'] = 1e-8
            else:
                key, value = {'concentration': ('concentration', 1), 'step': ('step', 1),
                              'target': ('target_ph', 7), 'budget': ('max_steps', 2),
                              'vessel_type': ('vessel', False)}[change]
                rows[3]['operator'][key] = value
            with self.assertRaises(AssertionError):
                self.review(rows)


if __name__ == '__main__':
    unittest.main()
