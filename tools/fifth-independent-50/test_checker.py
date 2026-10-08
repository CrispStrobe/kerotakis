"""Synthetic checker controls; never launches the CLI or profiler."""
import copy
import importlib.util
from pathlib import Path
import unittest

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


if __name__ == '__main__':
    unittest.main()
