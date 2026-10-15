"""Small saved-evidence fixtures; these do not execute or score chemistry experiments."""
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('review_prepare', HERE / 'prepare.py')
reader = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(reader)


class ReviewEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.controls = root / 'controls'
        shutil.copytree(HERE.parent / 'normalized40', self.controls)
        self.evidence = root / 'evidence'
        self.evidence.mkdir()
        shutil.copyfile(self.controls / 'execution-binding.json', self.evidence / 'binding.json')
        self.binding = json.loads((self.controls / 'execution-binding.json').read_text())
        self.scripts = sorted(x for x in self.binding['design_files'] if x.endswith('.lab'))

    def record(self, ordinal=23, stdout=b'{"step":3,"output_sequence":0,"events":[]}\n', exit_code=0, timeout=False):
        stem = f'{ordinal:02d}'
        result = {'case': self.scripts[ordinal - 1], 'ordinal': ordinal, 'started_utc': 'fixture',
                  'elapsed_seconds': 0.01, 'exit_code': exit_code, 'timeout': timeout,
                  'script_sha256': self.binding['design_files'][self.scripts[ordinal - 1]],
                  'stdout_sha256': reader.digest(stdout), 'stderr_sha256': reader.digest(b''),
                  'stdout_bytes': len(stdout), 'stderr_bytes': 0}
        (self.evidence / (stem + '.stdout.jsonl')).write_bytes(stdout)
        (self.evidence / (stem + '.stderr.txt')).write_bytes(b'')
        (self.evidence / (stem + '.result.json')).write_text(json.dumps(result))
        return self.evidence / (stem + '.result.json')

    def review(self):
        return reader.prepare(self.controls / 'design', self.evidence)

    def test_partial_ordinal_is_not_original_experiment_id(self):
        self.record()
        report = self.review()
        self.assertEqual((report['recorded_cases'], report['missing_cases']), (1, 39))
        case = report['cases'][22]
        self.assertEqual((case['ordinal'], case['experiment_id'], case['script']), (23, '24', 'scripts/24.lab'))
        self.assertEqual(case['physical_verdict'], 'pending_review')
        self.assertTrue(case['contiguous_output_sequence'])
        self.assertEqual(case['stdout_rows'][0]['step'], 3)

    def test_completed_successful_empty_streams_are_not_physical_passes(self):
        for ordinal in range(1, 41):
            self.record(ordinal, stdout=b'')
        (self.evidence / 'completed.json').write_text(json.dumps({'cases': 40,
            'binding_sha256': reader.digest((self.evidence / 'binding.json').read_bytes()),
            'original_outcomes_preserved': True}))
        report = self.review()
        self.assertTrue(report['completion_marker_verified'])
        self.assertEqual(report['physical_verdicts_assigned'], 0)
        self.assertTrue(all(c['physical_verdict'] == 'pending_review' for c in report['cases']))

    def test_tampered_stream_refused(self):
        self.record()
        (self.evidence / '23.stdout.jsonl').write_bytes(b'changed')
        with self.assertRaises(reader.EvidenceError):
            self.review()

    def test_wrong_source_binding_refused(self):
        changed = dict(self.binding, binary_sha256='0' * 64)
        (self.evidence / 'binding.json').write_text(json.dumps(changed))
        with self.assertRaisesRegex(reader.EvidenceError, 'binding mismatch'):
            self.review()

    def test_false_completion_refused(self):
        self.record()
        (self.evidence / 'completed.json').write_text('{"cases":40}')
        with self.assertRaisesRegex(reader.EvidenceError, 'false completion'):
            self.review()

    def test_orphan_output_refused(self):
        (self.evidence / '01.stdout.jsonl').write_bytes(b'')
        with self.assertRaisesRegex(reader.EvidenceError, 'orphan'):
            self.review()

    def test_timeout_retained_without_verdict(self):
        self.record(exit_code=-9, timeout=True)
        case = self.review()['cases'][22]
        self.assertTrue(case['process']['timeout'])
        self.assertEqual(case['physical_verdict'], 'pending_review')

    def test_malformed_and_nonfinite_streams_retained_as_diagnostics(self):
        for raw in [b'{"x":1,"x":2}\n', b'{"x":1e999}\n', b'{"x":NaN}\n', b'not JSON\n', b'\xff']:
            with self.subTest(raw=raw):
                self.record(stdout=raw)
                case = self.review()['cases'][22]
                self.assertIsNotNone(case['stdout_parse_error'])
                self.assertEqual(case['physical_verdict'], 'pending_review')

    def test_changed_frozen_input_refused(self):
        with (self.controls / 'design/scripts/01.lab').open('a') as output:
            output.write('\n# changed\n')
        with self.assertRaisesRegex(reader.EvidenceError, 'frozen member changed'):
            self.review()

    def test_wrong_ordinal_refused(self):
        path = self.record()
        result = json.loads(path.read_text())
        result['ordinal'] = 24
        path.write_text(json.dumps(result))
        with self.assertRaisesRegex(reader.EvidenceError, 'ordinal mismatch'):
            self.review()

    def test_duplicate_binding_key_refused(self):
        path = self.evidence / 'binding.json'
        path.write_text('{"a":1,"a":2}')
        with self.assertRaisesRegex(reader.EvidenceError, 'duplicate JSON key'):
            self.review()

    def test_symlinked_stream_refused(self):
        self.record()
        path = self.evidence / '23.stdout.jsonl'
        elsewhere = Path(self.temp.name) / 'outside.jsonl'
        elsewhere.write_bytes(path.read_bytes())
        path.unlink()
        path.symlink_to(elsewhere)
        with self.assertRaises(reader.EvidenceError):
            self.review()

    def test_broken_symlink_is_not_silently_missing(self):
        (self.evidence / '01.stdout.jsonl').symlink_to(self.evidence / 'nonexistent')
        with self.assertRaises(reader.EvidenceError):
            self.review()

    def test_unexpected_case_refused(self):
        (self.evidence / '41.result.json').write_text('{}')
        with self.assertRaisesRegex(reader.EvidenceError, 'unexpected case'):
            self.review()

    def test_repeated_bench_steps_do_not_substitute_for_stream_sequence(self):
        self.record(stdout=b'{"step":3,"output_sequence":0}\n{"step":3,"output_sequence":0}\n')
        case = self.review()['cases'][22]
        self.assertFalse(case['contiguous_output_sequence'])
        self.assertEqual(case['physical_verdict'], 'pending_review')

    def test_completion_with_timeout_contradicts_frozen_runner(self):
        for ordinal in range(1, 41):
            self.record(ordinal, timeout=ordinal == 23, exit_code=-9 if ordinal == 23 else 0)
        (self.evidence / 'completed.json').write_text(json.dumps({'cases': 40,
            'binding_sha256': reader.digest((self.evidence / 'binding.json').read_bytes()),
            'original_outcomes_preserved': True}))
        with self.assertRaisesRegex(reader.EvidenceError, 'completion contradicts timeout'):
            self.review()

    def test_valid_rows_survive_malformed_middle_line(self):
        for middle in [b'not JSON', b'\xff', b'{"x":NaN}']:
            with self.subTest(middle=middle):
                self.record(stdout=b'{"output_sequence":0}\n' + middle + b'\n{"output_sequence":1}\n')
                case = self.review()['cases'][22]
                self.assertEqual([r['output_sequence'] for r in case['stdout_rows']], [0, 1])
                self.assertEqual(case['stdout_row_line_numbers'], [1, 3])
                self.assertEqual([e['line'] for e in case['stdout_parse_errors']], [2])
                self.assertFalse(case['contiguous_output_sequence'])
                self.assertEqual(case['physical_verdict'], 'pending_review')


if __name__ == '__main__':
    unittest.main()
