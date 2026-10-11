"""Supplemental contracts reject setup scope, raw binding and warning substitutions."""
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('metal_followup_review', ROOT / 'tools/metal-acid-followup-review.py')
review = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(review)
collector = review.load_source('metal_test_collector', ROOT / review.COLLECTOR)


def fixture(role='positive'):
    original = review.read(ROOT / review.FOLDER / 'freeze.json')
    case = deepcopy(next(c for c in original['cases'] if c['path'].endswith('/metal-' + role + '.lab')))
    case['allowed_setup_notices'] = deepcopy(review.NOTICES if role == 'positive' else [])
    case['added_species'] = 'Mg' if role == 'positive' else 'MgSO4'
    metadata = case['expected_warning']
    def vessel(phase, acidic):
        positive_setup = role == 'positive' and not acidic
        return dict(id=0, solute_charge=-0.00998 if acidic else 0.0,
                    contents=[dict(species='Mg' if positive_setup else 'Mg+2',
                                   phase='solid' if positive_setup else 'aqueous', moles=1e-5)],
                    solution=dict(ph=1.12 if acidic else 6.997,
                                  scope='solvent_only' if positive_setup else 'fully_characterised',
                                  provenance=dict(engine='Kerotakis analytic equilibrium evaluator' if positive_setup
                                                  else original['expected_backend_engine'],
                                                  routing_phrase=dict(key='routing.solvent-relation-only'))))
    rows = [dict(step=0, operator=dict(op='add', vessel=0, species='water', moles=5.534276991396059),
                 bench=dict(vessels=[vessel('liquid', False)]), events=[]),
            dict(step=1, operator=dict(op='add', vessel=0, species=case['added_species'], moles=1e-5),
                 bench=dict(vessels=[vessel('solid', False)]), events=[]),
            dict(step=2, operator=dict(op='titrate', vessel=0, titrant='HCl', concentration=1.0,
                                       step=0.01, target_ph=0.0, max_steps=1),
                 bench=dict(vessels=[vessel('aqueous', True)]), events=[dict(
                     event='titrated', vessel=0, titrant='HCl', concentration=1.0, steps=1,
                     total_volume=0.01, curve=[[0.0, 6.997], [10.0, 1.12]], final_ph=1.12)])]
    for allowed in case['allowed_setup_notices']:
        rows[1]['events'].append(dict(event=allowed['event'], cause=allowed['cause'], vessel=0,
                                      what=review.NOTICE_TEMPLATES[allowed['reason_key']].replace('{name}', 'magnesium'),
                                      reason=dict(key=allowed['reason_key'], en=review.NOTICE_TEMPLATES[allowed['reason_key']],
                                      slots=[['name', {'term': {'en': 'magnesium', 'section': 'species'}}]])))
    text = '\n'.join('  v1: not yet modelled — ' + e['what'] for e in rows[1]['events'])
    text += '\ntitrated with 1 mol/L hydrochloric acid; 1 step(s), 10.0 mL total'
    if role == 'positive':
        rows[2]['events'].append(dict(event='hazard_warning', **metadata))
        text += '\nHAZARD (Caution): ' + metadata['hazard'] + ' — ' + metadata['real_world']
    processes = {mode: dict(exit_code=0, timeout=False) for mode in ('json', 'text')}
    return rows, processes, text, case, original


class SupplementalReviewTests(unittest.TestCase):
    def test_scoped_pair_and_unchanged_original_backend(self):
        for role in ('positive', 'benign'):
            rows, processes, text, case, freeze = fixture(role)
            before = deepcopy(rows)
            result = review.analyse(rows, processes, text, case, freeze, collector)
            self.assertEqual(rows, before)
            self.assertEqual(result['scoped_collector_checks']['backend'], role == 'benign')
            self.assertEqual(result['original_collector_checks']['no_refusal'], role == 'benign')
            self.assertEqual(result['classification'], 'qualifying_prospective_only_partner_consumed'
                             if role == 'positive' else 'qualifying_benign_counterpart')

    def test_setup_notices_exact_location_cause_slot_and_count(self):
        for mutation in ('missing', 'duplicate', 'target', 'cause', 'slot', 'vessel', 'benign'):
            rows, processes, text, case, freeze = fixture()
            if mutation == 'missing': rows[1]['events'].pop()
            elif mutation == 'duplicate': rows[1]['events'].append(deepcopy(rows[1]['events'][0]))
            elif mutation == 'target': rows[2]['events'].append(rows[1]['events'].pop())
            elif mutation == 'cause': rows[1]['events'][0]['cause'] = 'not-speciated'
            elif mutation == 'slot': rows[1]['events'][0]['reason']['slots'][0][1]['term']['en'] = 'iron'
            elif mutation == 'vessel': rows[1]['events'][0]['vessel'] = True
            else: case['allowed_setup_notices'] = []
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                review.analyse(rows, processes, text, case, freeze, collector)

    def test_wrong_backends_fail(self):
        for role, step in [('positive', 1), ('positive', 2), ('benign', 1), ('benign', 2)]:
            rows, processes, text, case, freeze = fixture(role)
            rows[step]['bench']['vessels'][0]['solution']['provenance']['engine'] = 'fallback'
            with self.subTest(role=role, step=step), self.assertRaises(AssertionError):
                review.analyse(rows, processes, text, case, freeze, collector)

    def test_warning_severity_duplicates_and_dose_narration_fail(self):
        for mutation in ('severity', 'duplicate', 'absent', 'text-severity', 'dose', 'text-dose'):
            rows, processes, text, case, freeze = fixture()
            if mutation == 'severity': rows[2]['events'][-1]['severity'] = 'danger'
            elif mutation == 'duplicate': rows[2]['events'].append(deepcopy(rows[2]['events'][-1]))
            elif mutation == 'absent': rows[2]['events'].pop()
            elif mutation == 'text-severity': text = text.replace('(Caution)', '(Danger)')
            elif mutation == 'dose': rows[2]['events'][0]['steps'] = True
            else: text = text.replace('10.0 mL', '1.0 mL')
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                review.analyse(rows, processes, text, case, freeze, collector)

    def test_operator_inventory_and_endpoint_fail(self):
        for mutation in ('operator', 'inventory', 'endpoint', 'timeout'):
            rows, processes, text, case, freeze = fixture()
            if mutation == 'operator': rows[2]['operator']['max_steps'] = 2
            elif mutation == 'inventory': rows[1]['bench']['vessels'][0]['contents'][0]['moles'] = -1
            elif mutation == 'endpoint': rows[2]['events'][0]['final_ph'] = 1.13
            else: processes['text']['timeout'] = True
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                review.analyse(rows, processes, text, case, freeze, collector)

    def test_json_duplicates_and_nonfinite_fail(self):
        for value in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":Infinity}'):
            with self.assertRaises((AssertionError, ValueError)):
                review.decode(value)

    def test_rendered_setup_notice_or_refusal_substitutions_fail(self):
        for role in ('positive', 'benign'):
            for appended in ('  v1: not yet modelled — acid step refused',
                             "  v1: solver 'PHREEQC' failed: no convergence", 'SAFETY VETO (L0): refused'):
                rows, processes, text, case, freeze = fixture(role)
                with self.subTest(role=role, appended=appended), self.assertRaises(AssertionError):
                    review.analyse(rows, processes, text + '\n' + appended, case, freeze, collector)
        rows, processes, text, case, freeze = fixture()
        text = '\n'.join(line for line in text.splitlines() if 'not yet modelled' not in line)
        with self.assertRaises(AssertionError):
            review.analyse(rows, processes, text, case, freeze, collector)

    def test_raw_hash_command_and_stderr_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifact = Path(temporary)
            case = dict(path='metal-positive.lab')
            directory = artifact / 'metal-positive'
            directory.mkdir()
            (directory / 'input.lab').write_text('frozen input')
            case['sha256'] = review.digest(directory / 'input.lab')
            for mode in ('json', 'text'):
                folder = directory / mode
                folder.mkdir()
                (folder / 'stdout.txt').write_text('{}\n' if mode == 'json' else 'text')
                (folder / 'stderr.txt').write_text('')
                process = dict(command=['/verified/kero', 'run', 'metal-positive/input.lab']
                               + (['--json'] if mode == 'json' else []), exit_code=0, timeout=False,
                               stdout_sha256=review.digest(folder / 'stdout.txt'),
                               stderr_sha256=review.digest(folder / 'stderr.txt'))
                (folder / 'process.json').write_text(json.dumps(process))
            review.raw_case(artifact, case)
            process_path = directory / 'json/process.json'
            original = review.read(process_path)
            for field, value in [('stdout_sha256', '0' * 64), ('exit_code', True),
                                 ('command', ['/verified/kero', 'run', 'different/input.lab', '--json'])]:
                changed = dict(original, **{field: value})
                process_path.write_text(json.dumps(changed))
                with self.subTest(field=field), self.assertRaises(AssertionError):
                    review.raw_case(artifact, case)
            process_path.write_text(json.dumps(original))
            (directory / 'text/stderr.txt').write_text('warning')
            with self.assertRaises(AssertionError):
                review.raw_case(artifact, case)


if __name__ == '__main__':
    unittest.main()
