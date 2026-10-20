#!/usr/bin/env python3
"""Exercise real complete Git diffs, conservative boundaries and workflow wiring."""
import importlib.util
import json
import os
import re
from unittest import mock
import pathlib
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('ci_changes', pathlib.Path(__file__).with_name('ci_changes.py'))
CI = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CI)


class ChangesTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = pathlib.Path(self.temp.name)
        self.git('init', '-q')
        self.git('config', 'user.email', 'ci-test@example.invalid')
        self.git('config', 'user.name', 'CI range test')
        for name in CI.DOCS_ONLY:
            self.write(name, 'before\n')
        self.base = self.commit()

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.repo, stderr=subprocess.PIPE).decode().strip()

    def write(self, name, data):
        p = self.repo / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(data, encoding='utf-8')

    def commit(self):
        self.git('add', '--all')
        self.git('commit', '-qm', 'fixture')
        return self.git('rev-parse', 'HEAD')

    def result(self, head, name='pull_request', **extra):
        event = ({'pull_request': {'base': {'sha': self.base}, 'head': {'sha': head}}}
                 if name == 'pull_request' else {'before': self.base, 'after': head})
        event.update(extra)
        return CI.classify(self.repo, name, event)['mode']

    def test_all_eight_reviewed_summaries_pr_and_push(self):
        for name in CI.DOCS_ONLY:
            self.write(name, 'after\n')
        head = self.commit()
        self.assertEqual(self.result(head), 'docs')
        self.assertEqual(self.result(head, 'push'), 'docs')

    def test_mixed_change_and_complete_range_past_300_paths(self):
        for n in range(310):
            self.write(f'docs/extra-{n:03}.md', 'before\n')
        self.write('zz-runtime.rs', 'before\n')
        self.base=self.commit()
        self.write('docs/STATUS.md', 'after\n')
        for n in range(310):
            self.write(f'docs/extra-{n:03}.md', 'after\n')
        self.write('zz-runtime.rs', 'changed\n')
        head=self.commit()
        extra={f'docs/extra-{n:03}.md' for n in range(310)}
        with mock.patch.object(CI, 'DOCS_ONLY', CI.DOCS_ONLY | extra):
            result=CI.classify(self.repo, 'push', {'before': self.base, 'after': head})
            self.assertEqual(result['mode'], 'full')
            self.assertEqual(result['changed_paths'], 312)

    def test_every_unknown_boundary_is_full(self):
        for name in ['README.md', 'CONTRIBUTING.md', 'docs/runtime.md', 'docs/data.json',
                     '.github/workflows/ci.yml', 'tools/ci_changes.py', 'crates/core.rs',
                     'docs/Überblick.md', 'docs/a\nb.md']:
            with self.subTest(name=name):
                self.write(name, 'new\n')
                self.assertEqual(self.result(self.commit()), 'full')
                self.git('reset', '--hard', self.base)

    def test_delete_and_rename_are_full_even_between_allowed_paths(self):
        (self.repo/'docs/STATUS.md').unlink()
        self.assertEqual(self.result(self.commit()), 'full')
        self.git('reset', '--hard', self.base)
        self.git('mv', '-f', 'docs/STATUS.md', 'docs/next-agent-handoff.md')
        self.assertEqual(self.result(self.commit()), 'full')

    def test_mode_change_and_symlink_are_full(self):
        p = self.repo/'docs/STATUS.md'
        p.chmod(0o755)
        self.assertEqual(self.result(self.commit()), 'full')
        self.git('reset', '--hard', self.base)
        p.unlink();p.symlink_to('../README.md')
        self.assertEqual(self.result(self.commit()), 'full')

    def test_missing_zero_noncommit_empty_and_newbranch_fail_closed(self):
        for head in ['0'*40, 'f'*40, 'HEAD', '', self.base, self.git('rev-parse', 'HEAD^{tree}')]:
            self.assertEqual(self.result(head), 'full')
        self.write('docs/STATUS.md', 'after\n');head=self.commit()
        self.assertEqual(self.result(head, 'push', created=True), 'full')
        self.assertEqual(self.result(head, 'push', deleted=True), 'full')
        for event in [{}, None, [], {'pull_request': {'base': {}}}]:
            self.assertEqual(CI.classify(self.repo, 'pull_request', event)['mode'], 'full')
        self.assertEqual(CI.classify(self.repo, 'workflow_dispatch', {})['mode'], 'full')

    def test_no_pr_merge_base_substitution(self):
        self.write('runtime.rs', 'base-only change\n');base2=self.commit()
        self.git('checkout', '-qb', 'head-branch', self.base)
        self.write('docs/STATUS.md', 'after\n');head=self.commit()
        event={'pull_request': {'base': {'sha': base2}, 'head': {'sha': head}}}
        self.assertEqual(CI.classify(self.repo, 'pull_request', event)['mode'], 'full')

    def test_invalid_utf8_path_is_full(self):
        with open(os.fsencode(self.repo)+b'/bad-\xff.md', 'wb') as stream:
            stream.write(b'data')
        self.assertEqual(self.result(self.commit()), 'full')

    def test_actual_required_gate_rejects_classifier_and_build_failures(self):
        workflow=pathlib.Path(__file__).resolve().parents[1]/'.github/workflows/ci.yml'
        text=workflow.read_text()
        script=re.search(r'          test "\$CHANGES_RESULT" = success\n(?:          .*\n)+?          esac', text).group()
        for classification in ['success','failure','cancelled','skipped']:
            for mode in ['docs','full','', 'unknown']:
                for build in ['success','failure','cancelled','skipped']:
                    env=dict(os.environ,CHANGES_RESULT=classification,CHANGE_MODE=mode,BUILD_RESULT=build)
                    result=subprocess.run(['bash','-e','-c',script],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
                    expected=classification=='success' and (mode,build) in [('docs','skipped'),('full','success')]
                    self.assertEqual(result.returncode==0,expected,(classification,mode,build))

    def test_native_matrix_preserves_protected_names_and_guards_every_heavy_step(self):
        workflow=pathlib.Path(__file__).resolve().parents[1]/'.github/workflows/ci.yml'
        native=workflow.read_text().split('  test:\n',1)[1].split('  my-basic-preview:',1)[0]
        self.assertIn('    if: ${{ always() }}',native)
        self.assertIn('    name: Test (native)',native)
        self.assertIn('os: [ubuntu-latest, macos-latest]',native)
        steps=re.split(r'(?=^      - )',native,flags=re.M)[1:]
        self.assertGreater(len(steps),10)
        for step in steps:
            if 'Reviewed documentation needs no native build' in step:
                self.assertIn("needs.changes.result == 'success' && needs.changes.outputs.mode == 'docs'",step)
            else:
                self.assertIn("needs.changes.result != 'success' || needs.changes.outputs.mode != 'docs'",step)
        artifact=next(step for step in steps if 'actions/upload-artifact' in step)
        self.assertIn('always() &&',artifact)

    def test_cli_bad_event_json_outputs_full(self):
        event=self.repo/'event.json';event.write_text('{')
        output=self.repo/'output'
        result=subprocess.check_output(['python3',str(pathlib.Path(CI.__file__).resolve()),
            '--repo',str(self.repo),'--event-name','push','--event-path',str(event),'--output',str(output)])
        self.assertEqual(json.loads(result)['mode'],'full')
        self.assertEqual(output.read_text(),'mode=full\n')


if __name__ == '__main__':
    unittest.main()
