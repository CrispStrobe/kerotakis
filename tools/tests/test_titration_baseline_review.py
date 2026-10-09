"""Frozen evidence must bind both package identity and inherited safety outcomes."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    'titration_baseline_review', Path(__file__).resolve().parents[1] / 'titration-baseline-review.py')
reviewer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reviewer)


class ProductionSafetyEvidence(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.artifact = Path(self.temp.name)
        self.evidence = self.artifact / 'titration-baseline'
        self.evidence.mkdir()
        self.harness = 'a' * 40
        self.source = 'b' * 40
        self.tree = 'c' * 40
        self.freeze_path = 'audits/safety/freeze.json'
        self.fixture_path = 'audits/safety/contracts.rs'
        self.target = 'crates/kerotakis-safety/tests/contracts.rs'
        self.library_path = 'crates/kerotakis-safety/src/lib.rs'
        self.fixture = b'#[test]\nfn introduced() {}\n#[test]\nfn unchanged() {}\n'
        self.library = b'#[cfg(test)]\nmod tests { #[test] fn inherited() {} }\n'
        self.freeze = {
            'source_observed': self.source, 'source_tree': self.tree,
            'test_package': 'kerotakis-safety', 'fixture_path': self.fixture_path,
            'fixture_sha256': reviewer.sha(self.fixture), 'intended_test_target': self.target,
            'expected_test_functions': 2, 'functions': ['introduced', 'unchanged'],
            'additional_libraries': [{
                'package': 'kerotakis-safety', 'source_path': self.library_path,
                'source_sha256': reviewer.sha(self.library), 'expected_test_functions': 1,
                'functions': ['tests::inherited'], 'evidence_prefix': 'safety-library',
            }],
        }
        self.blobs = {
            (self.harness, reviewer.WORKFLOW): b'synthetic workflow\n',
            (self.harness, self.fixture_path): self.fixture,
            (self.source, self.library_path): self.library,
        }
        self.bind_freeze()
        for name, value in {
            'fixture.rs': self.fixture, 'source-commit.txt': self.source + '\n',
            'source-tree.txt': self.tree + '\n', 'harness-commit.txt': self.harness + '\n',
            'status-before.txt': '', 'tracked-diff.txt': '',
            'status-with-fixture.txt': '?? ' + self.target + '\n',
            'submodules.txt': ' ' + 'd' * 40 + ' donor\n ' + 'e' * 40 + ' vendor\n',
            'rustc.txt': 'synthetic compiler\n', 'cargo.txt': 'synthetic cargo\n',
            'Cargo.lock': 'synthetic lock\n',
            'controls-exit.txt': '101\n', 'inherited-exit.txt': '0\n',
            'safety-library-exit.txt': '0\n',
        }.items():
            self.put(name, value)
        self.put('lock-sha256.txt', reviewer.sha((self.evidence / 'Cargo.lock').read_bytes())
                 + '  titration-baseline/Cargo.lock\n')
        self.put('controls.log', self.log(['introduced', 'unchanged'], {'introduced'}))
        self.put('inherited.log', self.log([f'core_{n}' for n in range(604)]))
        self.put('safety-library.log', self.log(['tests::inherited']))
        self.result(['unchanged'], ['introduced'])
        self.run = self.artifact / 'run.json'
        self.run_record = {
            'databaseId': 0, 'url': 'synthetic archive, no hosted execution',
            'headSha': self.harness, 'event': 'workflow_dispatch',
            'status': 'completed', 'conclusion': 'success',
            'jobs': [{'name': 'Safety evidence', 'conclusion': 'success'}],
        }
        self.run.write_text(json.dumps(self.run_record))
        mocked = patch.object(reviewer, 'git', side_effect=self.git)
        mocked.start()
        self.addCleanup(mocked.stop)

    def put(self, name, value):
        (self.evidence / name).write_bytes(value if isinstance(value, bytes) else value.encode())

    @staticmethod
    def log(names, failed=()):
        return ''.join(f'test {n} ... {"FAILED" if n in failed else "ok"}\n' for n in names) + (
            f'test result: {"FAILED" if failed else "ok"}. {len(names)-len(failed)} passed; '
            f'{len(failed)} failed; 0 ignored; 0 measured; 0 filtered out;\n')

    def bind_freeze(self):
        self.blobs[self.harness, self.freeze_path] = json.dumps(self.freeze).encode()
        for path in [self.freeze_path, self.fixture_path]:
            target = self.artifact / 'harness' / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(self.blobs[self.harness, path])
        self.put('harness-sha256.txt', ''.join(
            reviewer.sha(self.blobs[self.harness, path]) + '  harness/' + path + '\n'
            for path in [reviewer.WORKFLOW, self.freeze_path, self.fixture_path]))

    def git(self, *args):
        if args[0] == 'show':
            commit, path = args[1].split(':', 1)
            return self.blobs[commit, path]
        if args[0] == 'rev-parse':
            return self.tree.encode()
        if args[:2] == ('ls-tree', '-r'):
            return ('160000 commit ' + 'd' * 40 + '\tdonor\n160000 commit '
                    + 'e' * 40 + '\tvendor\n').encode()
        if args[0] == 'ls-tree':
            return b''
        raise AssertionError(args)

    def result(self, passed, failed):
        self.put('results.json', json.dumps({
            'source': self.source, 'fixture_sha256': reviewer.sha(self.fixture),
            'exit_code': 101, 'passed': passed, 'failed': failed,
            'controls_sha256': reviewer.sha((self.evidence / 'controls.log').read_bytes()),
            'scope': 'Baseline named functions only; failed parameter loops may stop early. '
                     'Not chemistry or complete integration acceptance.',
        }))

    def review(self, prerequisite=False):
        return reviewer.review(self.artifact, self.run, self.harness, self.freeze_path,
                               'Safety evidence', prerequisite)

    def test_complete_collection_keeps_model_failures(self):
        result = self.review()
        self.assertTrue(result['accepted_baseline_collection'])
        self.assertEqual(result['failed'], ['introduced'])
        self.assertEqual(result['additional_libraries'][0]['passed'], ['tests::inherited'])

    def test_safety_library_must_be_complete_unfiltered_and_successful(self):
        original = (self.evidence / 'safety-library.log').read_bytes()
        for text in [b'', original + original,
                     original.replace(b'0 filtered out', b'1 filtered out'),
                     self.log(['tests::other']).encode(),
                     self.log(['tests::inherited'], {'tests::inherited'}).encode()]:
            with self.subTest(log=text), self.assertRaises(AssertionError):
                self.put('safety-library.log', text)
                self.review()
        self.put('safety-library.log', original)
        self.put('safety-library-exit.txt', '101\n')
        with self.assertRaises(AssertionError):
            self.review()

    def test_source_library_hash_is_bound(self):
        self.blobs[self.source, self.library_path] += b'changed source\n'
        with self.assertRaises(AssertionError):
            self.review()

    def compiler_archive(self, package):
        self.put('controls.log', 'error[E0407]: unsupported callback\n'
                 f'error: could not compile `{package}` (test "contracts")\n')
        self.result([], [])
        self.run_record['conclusion'] = 'failure'
        self.run_record['jobs'][0]['conclusion'] = 'failure'
        self.run.write_text(json.dumps(self.run_record))

    def test_exact_compiler_package_has_zero_behavioral_outcomes(self):
        self.compiler_archive('kerotakis-safety')
        result = self.review(prerequisite=True)
        self.assertFalse(result['accepted_baseline_collection'])
        self.assertEqual(result['passed'], [])
        self.assertEqual(result['failed'], [])
        with self.assertRaises(AssertionError):
            self.review()

    def test_wrong_compiler_package_cannot_establish_prerequisite_failure(self):
        self.compiler_archive('kerotakis-core')
        with self.assertRaises(AssertionError):
            self.review(prerequisite=True)

    def test_runtime_failure_cannot_be_reclassified_as_compiler_failure(self):
        with self.assertRaises(AssertionError):
            self.review(prerequisite=True)

    def test_library_failure_blocks_compiler_archive_acceptance_too(self):
        self.compiler_archive('kerotakis-safety')
        self.put('safety-library.log', self.log(['tests::inherited'], {'tests::inherited'}))
        with self.assertRaises(AssertionError):
            self.review(prerequisite=True)


if __name__ == '__main__':
    unittest.main()
