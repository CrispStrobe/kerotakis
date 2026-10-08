"""Synthetic checker controls; never launches the CLI or profiler."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

import review_profile

spec = importlib.util.spec_from_file_location('fifth_runner', Path(__file__).with_name('run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ProfileAcceptance(unittest.TestCase):
    def setUp(self):
        self.case = dict(id='F19', checks=[dict(
            lhs=dict(kind='temperature_k', variant='a', vessel=1), op='near', rhs=300., atol=.01, rtol=0.)])
        self.warm = dict(a=dict(timeout=False, final_inspection=True, json_parse_errors=[],
                               exit_code=0, notices=[], vessels=[dict(id=0, temperature_k=300.)]))

    def check_sample(self, **changes):
        sample = copy.deepcopy(self.warm['a'])
        sample.update(changes)
        return runner.validate_samples(self.case, self.warm, 'a', [sample])[0]['outcome']

    def test_clean_exit_with_wrong_result_is_rejected(self):
        self.assertEqual(self.check_sample(vessels=[dict(id=0, temperature_k=310.)]), 'unmet_expectation')

    def test_missing_inspection_is_rejected(self):
        self.assertEqual(self.check_sample(final_inspection=False), 'missing_final_inspection_protocol_failure')

    def test_timeout_is_rejected(self):
        self.assertEqual(self.check_sample(timeout=True), 'timeout')

    def test_model_notice_remains_qualified(self):
        self.assertEqual(self.check_sample(notices=[dict(event='not_yet_modeled')]), 'qualified_agreement_with_model_notice')

    def test_missing_observable_is_rejected(self):
        self.assertEqual(self.check_sample(vessels=[dict(id=0)]), 'unmet_expectation')

    def test_successful_control(self):
        self.assertEqual(self.check_sample(), 'passed')

    def test_one_success_cannot_hide_failed_samples_or_profiles(self):
        measured = dict(status='measured', callgrind_status='passed')
        self.assertFalse(runner.profile_failed([measured]))
        self.assertTrue(runner.profile_failed([measured, dict(status='failed_sample')]))
        self.assertTrue(runner.profile_failed([measured, dict(status='measured', callgrind_status='failed')]))
        self.assertTrue(runner.profile_failed([dict(status='excluded_after_observed_behavior_failure')]))


class ArchivedAcceptance(unittest.TestCase):
    def write_run(self, directory, temperature=300.):
        directory.mkdir(parents=True)
        (directory / 'input.lab').write_text('inspect\n')
        output = dict(operator=dict(op='inspect'), bench=dict(vessels=[dict(id=0, temperature_k=temperature)]), events=[])
        (directory / 'stdout.ndjson').write_text(json.dumps(output) + '\n')
        (directory / 'stderr.txt').write_text('')
        envelope = dict(exit_code=0, timeout=False, seconds=.1, child_user_cpu_seconds=.01, child_system_cpu_seconds=.01,
                        records=1, json_parse_errors=[], final_inspection=True,
                        input_sha256=runner.digest(directory / 'input.lab'),
                        stdout_sha256=runner.digest(directory / 'stdout.ndjson'), stderr_sha256=runner.digest(directory / 'stderr.txt'))
        runner.save(directory / 'execution.json', envelope)

    def test_archive_hash_and_input_bindings(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp) / 'run'
            self.write_run(directory)
            self.assertTrue(review_profile.read_archived_run(directory, 'inspect\n')['final_inspection'])
            with self.assertRaisesRegex(ValueError, 'frozen forecast'):
                review_profile.read_archived_run(directory, 'new\ninspect\n')
            (directory / 'stdout.ndjson').write_text('{}\n')
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                review_profile.read_archived_run(directory, 'inspect\n')

    def test_archived_success_cannot_hide_changed_sample_behavior(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            path = directory / 'F26' / 'a'
            self.write_run(path / 'warm')
            for i in range(7):
                self.write_run(path / f'native-{i}')
            runner.save(directory / 'performance.json', dict(
                source_commit='synthetic', forecast_sha256=runner.EXPECTED_FORECAST, binary_sha256='0' * 64,
                results=[dict(id='F26', variant='a', status='measured', samples_seconds=[.1] * 7)]))
            forecast = dict(source_commit='synthetic', cases=[dict(id='F26', variants=dict(a='inspect\n'), checks=[dict(
                lhs=dict(kind='temperature_k', variant='a', vessel=1), op='near', rhs=300., atol=.01, rtol=0.)])])
            self.assertFalse(review_profile.review(forecast, directory)['failed'])
            # Rewrite a sample and its envelope consistently: integrity alone is
            # insufficient; the original behavior contract must also pass.
            sample = path / 'native-3'
            (sample / 'stdout.ndjson').write_text(json.dumps(dict(operator=dict(op='inspect'),
                bench=dict(vessels=[dict(id=0, temperature_k=310.)]), events=[])) + '\n')
            envelope = json.loads((sample / 'execution.json').read_text())
            envelope['stdout_sha256'] = runner.digest(sample / 'stdout.ndjson')
            runner.save(sample / 'execution.json', envelope)
            report = review_profile.review(forecast, directory)
            self.assertTrue(report['failed'])
            self.assertIsNone(report['results'][0]['median_seconds'])
            self.assertEqual(report['results'][0]['validations'][3]['outcome'], 'unmet_expectation')


if __name__ == '__main__':
    unittest.main()
