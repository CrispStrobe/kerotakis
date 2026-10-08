"""Acceptance log parsing must fail closed on missing or filtered controls."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('native_controls_review', Path(__file__).resolve().parents[1] / 'native-controls-review.py')
reviewer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reviewer)

FIXTURE = b'#[test]\nfn supported() {}\n#[test]\nfn refused_with_rollback() {}\n'
LOG = '''Running tests/contracts.rs (target/debug/deps/contracts-abc)
running 2 tests
test supported ... ok
Running tests/next.rs (target/debug/deps/next-def)
test refused_with_rollback ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
'''


class NativeLogContracts(unittest.TestCase):
    def test_next_binary_stderr_can_precede_previous_stdout_summary(self):
        result = reviewer.fixture_result(LOG, 'crates/kerotakis-core/tests/contracts.rs', FIXTURE)
        self.assertEqual(result['passed'], ['supported', 'refused_with_rollback'])

    def test_missing_refusal_control_cannot_borrow_an_unrelated_summary(self):
        log = LOG.replace('test refused_with_rollback ... ok\n', '')
        with self.assertRaises(AssertionError):
            reviewer.fixture_result(log, 'contracts.rs', FIXTURE)

    def test_filtered_controls_are_not_complete_acceptance(self):
        log = LOG.replace('2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out', '2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out')
        with self.assertRaises(AssertionError):
            reviewer.fixture_result(log, 'contracts.rs', FIXTURE)

    def test_failed_and_ignored_functions_refuse(self):
        for status in ['FAILED', 'ignored']:
            with self.subTest(status=status), self.assertRaises(AssertionError):
                reviewer.fixture_result(LOG.replace('refused_with_rollback ... ok', 'refused_with_rollback ... ' + status), 'contracts.rs', FIXTURE)

    def test_repeated_execution_is_ambiguous(self):
        with self.assertRaises(AssertionError):
            reviewer.fixture_result(LOG + LOG, 'contracts.rs', FIXTURE)

    def test_unrelated_summary_before_target_functions_is_skipped(self):
        prefix = 'test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
        log = LOG.replace('running 2 tests\n', prefix + 'running 2 tests\n')
        self.assertEqual(len(reviewer.fixture_result(log, 'contracts.rs', FIXTURE)['passed']), 2)


if __name__ == '__main__':
    unittest.main()
