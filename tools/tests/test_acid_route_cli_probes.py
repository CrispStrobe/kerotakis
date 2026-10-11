"""Witness and rejection controls; synthetic streams are not chemistry results."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('acid_probes', TOOLS / 'acid-route-cli-probes.py')
probes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probes)
FREEZE = json.loads(probes.FREEZE.read_text())
PROCESS = {m: {'exit_code': 0, 'timeout': False} for m in ('json', 'text')}


def protocol(case, consumed=False):
    def state(final=False):
        contents = [{'species': 'water', 'moles': 5.5, 'phase': 'liquid'}]
        if case['role'] == 'positive' and not (final and consumed):
            contents += [{'species': case['partner_keys'][0], 'moles': case['added_moles'],
                          'phase': case['partner_phase'] or 'aqueous'}]
        return {'id': 0, 'contents': contents, 'solute_charge': -0.009 if final else 0,
                'solution': {'ph': 1.1 if final else 7.0,
                             'provenance': {'engine': FREEZE['expected_backend_engine']}}}
    operators = [{'op': 'add', 'vessel': 0, 'species': 'water', 'moles': 5.534276991396059},
                 {'op': 'add', 'vessel': 0, 'species': case['added_species'], 'moles': case['added_moles']},
                 {'op': 'titrate', 'vessel': 0, 'titrant': 'HCl', 'concentration': 1.0,
                  'step': 0.01, 'target_ph': 0.0, 'max_steps': 1}]
    rows = [{'step': i, 'operator': op, 'events': [], 'bench': {'vessels': [state(i == 2)]}}
            for i, op in enumerate(operators)]
    rows[2]['events'] = [{k: v for k, v in FREEZE['dose'].items() if k != 'moles'} |
                         {'event': 'titrated', 'final_ph': 1.1, 'curve': [[0.0, 7.0], [10.0, 1.1]]}]
    text = ''
    if case['role'] == 'positive':
        rows[2]['events'].append({'event': 'hazard_warning', **case['expected_warning']})
        text = case['expected_warning']['hazard'] + ' — ' + case['expected_warning']['real_world']
    return rows, text


class AcidRouteTests(unittest.TestCase):
    def evaluate(self, rows, text, case, processes=PROCESS):
        return probes.analyse(rows, processes, text, case, FREEZE)

    def test_projection_benign_and_consumed_are_distinct(self):
        for case in FREEZE['cases']:
            rows, text = protocol(case)
            expected = ('qualifying_named_to_dissolved_projection' if case['role'] == 'positive'
                        else 'qualifying_benign_counterpart')
            self.assertEqual(self.evaluate(rows, text, case)['classification'], expected)
            if case['role'] == 'positive':
                rows, text = protocol(case, consumed=True)
                r = self.evaluate(rows, text, case)
                self.assertEqual(r['classification'], 'qualifying_prospective_only_partner_consumed')
                self.assertFalse(r['final_surviving_exposure'])

    def test_absent_setup_partner_wrong_phase_or_backend_is_non_exercising(self):
        case = FREEZE['cases'][2]
        for mutation in ('absent', 'cutoff', 'phase', 'backend', 'ph', 'charge'):
            rows, text = protocol(case)
            before = rows[1]['bench']['vessels'][0]
            after = rows[2]['bench']['vessels'][0]
            if mutation == 'absent': before['contents'] = before['contents'][:1]
            if mutation == 'cutoff': before['contents'][1]['moles'] = 1e-12
            if mutation == 'phase': before['contents'][1]['phase'] = 'aqueous'
            if mutation == 'backend': after['solution']['provenance']['engine'] = 'fallback'
            if mutation == 'ph': after['solution']['ph'] = 2.0001
            if mutation == 'charge': after['solute_charge'] = -1e-9
            self.assertEqual(self.evaluate(rows, text, case)['classification'], 'not_exercising_declared_route')

    def test_partial_dose_refusal_and_boolean_numeric_fields_rejected(self):
        case = FREEZE['cases'][0]
        for mutation in ('partial', 'refusal', 'operator', 'dose', 'step', 'process'):
            rows, text = protocol(case); process = copy.deepcopy(PROCESS)
            if mutation == 'partial': rows[2]['events'][0]['total_volume'] = 0.005
            if mutation == 'refusal': rows[2]['events'].append({'event': 'solver_failed'})
            if mutation == 'operator': rows[2]['operator']['concentration'] = True
            if mutation == 'dose': rows[2]['events'][0]['steps'] = True
            if mutation == 'step': rows[0]['step'] = False
            if mutation == 'process': process['text']['exit_code'] = False
            self.assertEqual(self.evaluate(rows, text, case, process)['classification'], 'not_exercising_declared_route')

    def test_missing_duplicate_metadata_setup_and_text_warning_mismatch(self):
        case = FREEZE['cases'][0]
        for mutation in ('missing', 'duplicate', 'metadata', 'setup', 'text'):
            rows, text = protocol(case)
            if mutation == 'missing': rows[2]['events'].pop()
            if mutation == 'duplicate': rows[2]['events'].append(copy.deepcopy(rows[2]['events'][-1]))
            if mutation == 'metadata': rows[2]['events'][-1]['severity'] = 'caution'
            if mutation == 'setup': rows[1]['events'].append(copy.deepcopy(rows[2]['events'][-1]))
            if mutation == 'text': text += text
            self.assertEqual(self.evaluate(rows, text, case)['classification'], 'warning_contract_mismatch')

    def test_negative_nonfinite_and_ambiguous_inventory_rejected(self):
        case = FREEZE['cases'][0]
        for mutation in (True, -1, float('nan'), float('inf'), 'duplicate_vessel'):
            rows, text = protocol(case)
            if mutation == 'duplicate_vessel':
                rows[1]['bench']['vessels'].append(copy.deepcopy(rows[1]['bench']['vessels'][0]))
            else: rows[1]['bench']['vessels'][0]['contents'][1]['moles'] = mutation
            with self.assertRaises(AssertionError): self.evaluate(rows, text, case)

    def test_weak_acid_dominated_state_does_not_pass_strong_acid_witness(self):
        case = FREEZE['cases'][0]
        rows, text = protocol(case)
        rows[2]['bench']['vessels'][0]['contents'].append(
            {'species': 'CH3COOH', 'moles': 0.02, 'phase': 'aqueous'})
        result = self.evaluate(rows, text, case)
        self.assertFalse(result['checks']['final_strong_acidity'])
        self.assertEqual(result['classification'], 'not_exercising_declared_route')

    def test_source_and_tree_must_match_the_freeze_not_only_each_other(self):
        binding = {'compiled_source': FREEZE['source_observed'],
                   'source_tree': FREEZE['source_tree'], 'source_kind': 'test'}
        probes.source_binding(binding, dict(binding), FREEZE)
        for key in ('compiled_source', 'source_tree'):
            wrong = dict(binding); wrong[key] = '0' * 40
            with self.assertRaises(AssertionError):
                probes.source_binding(wrong, dict(wrong), FREEZE)

    def test_nonfinite_noninventory_state_is_rejected(self):
        case = FREEZE['cases'][0]
        rows, text = protocol(case)
        rows[2]['bench']['vessels'][0]['pressure'] = float('inf')
        with self.assertRaises(AssertionError): self.evaluate(rows, text, case)

    def test_split_trace_portions_do_not_create_a_screened_partner(self):
        case = FREEZE['cases'][0]
        rows, text = protocol(case)
        v = rows[1]['bench']['vessels'][0]
        v['contents'][1]['moles'] = 1e-12
        v['contents'].append(copy.deepcopy(v['contents'][1]))
        result = self.evaluate(rows, text, case)
        self.assertFalse(result['checks']['setup_partner'])

    def test_solid_weak_acid_does_not_supply_ledger_protons(self):
        case = FREEZE['cases'][0]
        rows, text = protocol(case)
        rows[2]['bench']['vessels'][0]['contents'].append(
            {'species': 'CH3COOH', 'moles': 0.02, 'phase': 'solid'})
        self.assertTrue(self.evaluate(rows, text, case)['checks']['final_strong_acidity'])

    def test_incompatible_endpoint_event_or_curve_is_non_exercising(self):
        case = FREEZE['cases'][0]
        for mutation in ('endpoint', 'ph', 'curve_volume', 'curve_ph', 'missing_curve'):
            rows, text = protocol(case)
            if mutation == 'endpoint': rows[2]['operator']['endpoint'] = 'pe'
            if mutation == 'ph': rows[2]['events'][0]['final_ph'] = 1.2
            if mutation == 'curve_volume': rows[2]['events'][0]['curve'][1][0] = 5.0
            if mutation == 'curve_ph': rows[2]['events'][0]['curve'][1][1] = 1.2
            if mutation == 'missing_curve': rows[2]['events'][0].pop('curve')
            self.assertEqual(self.evaluate(rows, text, case)['classification'], 'not_exercising_declared_route')

    def test_boolean_identity_and_undefined_phase_are_invalid(self):
        case = FREEZE['cases'][0]
        for mutation in ('identity', 'phase'):
            rows, text = protocol(case)
            if mutation == 'identity': rows[1]['bench']['vessels'][0]['id'] = False
            if mutation == 'phase': rows[1]['bench']['vessels'][0]['contents'][1]['phase'] = 'invented'
            with self.assertRaises(AssertionError): self.evaluate(rows, text, case)

    def test_unreviewed_partner_projection_is_not_called_consumption(self):
        case = FREEZE['cases'][4]
        rows, text = protocol(case, consumed=True)
        rows[2]['bench']['vessels'][0]['contents'].append(
            {'species': 'CO3-2', 'moles': 1e-5, 'phase': 'aqueous'})
        result = self.evaluate(rows, text, case)
        self.assertFalse(result['checks']['no_unreviewed_final_partner'])
        self.assertEqual(result['classification'], 'not_exercising_declared_route')

    def test_benign_counterpart_with_another_screened_partner_is_non_exercising(self):
        case = FREEZE['cases'][3]
        rows, text = protocol(case)
        rows[1]['bench']['vessels'][0]['contents'].append(
            {'species': 'Zn', 'moles': 1e-5, 'phase': 'solid'})
        result = self.evaluate(rows, text, case)
        self.assertFalse(result['checks']['no_other_screened_partner'])
        self.assertEqual(result['classification'], 'not_exercising_declared_route')

    def test_synthetic_collection_can_be_moved_and_independently_rechecked(self):
        # A fake executable tests process/archive portability only; no Kero runs.
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            payload = {Path(c['path']).stem: protocol(c) for c in FREEZE['cases']}
            binary = directory / 'fake-cli'
            binary.write_text('#!' + sys.executable + '\nimport json,sys,pathlib\n'
                              + 'payload=' + repr(payload) + '\n'
                              + 'rows,text=payload[pathlib.Path(sys.argv[2]).parent.name]\n'
                              + "print('\\n'.join(json.dumps(r) for r in rows) if '--json' in sys.argv else text)\n")
            binary.chmod(0o755)
            manifest = dict(compiled_source=FREEZE['source_observed'], source_tree=FREEZE['source_tree'],
                            source_kind='synthetic archive portability test')
            binding = dict(manifest, binary_sha256=probes.digest(binary))
            for name, data in [('manifest.json', manifest), ('binding.json', binding)]:
                (directory / name).write_text(json.dumps(data))
            base = [sys.executable, str(TOOLS / 'acid-route-cli-probes.py'),
                    '--build-binding', str(directory / 'binding.json'),
                    '--source-manifest', str(directory / 'manifest.json')]
            subprocess.run(base + ['--binary', str(binary), '--out', 'observations', '--report', 'first.json'],
                           cwd=directory, check=True, capture_output=True)
            archive = directory / 'archive'; archive.mkdir()
            shutil.move(str(directory / 'observations'), archive)
            subprocess.run(base + ['--review', '--out', str(archive / 'observations'), '--report', 'review.json'],
                           cwd=directory, check=True, capture_output=True)
            first = json.loads((directory / 'first.json').read_text())
            self.assertEqual(first, json.loads((directory / 'review.json').read_text()))
            self.assertFalse(first['accepted_repair_proof'])
            self.assertEqual(len(first['observations']), 6)


if __name__ == '__main__':
    unittest.main()
