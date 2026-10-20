"""Synthetic offline Git fixtures only; no actual release/provider execution."""
import argparse
import os
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import bind_stage as b


class LocalGit(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        b.git(self.repo, 'init', '-q')
        b.git(self.repo, 'config', 'user.name', 'Synthetic Fixture')
        b.git(self.repo, 'config', 'user.email', 'fixture@example.invalid')
        b.git(self.repo, 'config', 'commit.gpgsign', 'false')
        (self.repo / 'web').mkdir()
        (self.repo / 'web/rest.html').write_text('synthetic REST', encoding='utf-8')
        (self.repo / 'large-not-needed').write_bytes(b'not checked out')
        b.git(self.repo, 'add', '.')
        b.git(self.repo, 'commit', '-q', '-m', 'synthetic base')
        self.base = b.git(self.repo, 'rev-parse', 'HEAD').decode().strip()
        self.remote = self.root / 'remote.git'
        subprocess.run(['git', 'init', '--bare', '-q', str(self.remote)], check=True)
        b.git(self.repo, 'remote', 'add', 'origin', str(self.remote))
        self.wt = self.root / 'sparse'

    def tearDown(self):
        self.temp.cleanup()

    def create(self, branch='audit/synthetic'):
        b.absent_branch(self.repo, branch)
        b.sparse_create(self.repo, self.wt, branch, self.base)

    def public_commit(self, extra=False):
        expected = {p: ('synthetic ' + p).encode() for p in b.PUBLIC}
        for name, raw in expected.items():
            path = self.wt / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(raw)
        b.git(self.wt, 'add', '--', *b.PUBLIC)
        if extra:
            p = self.wt / 'extra'
            p.write_bytes(b'forbidden')
            b.git(self.wt, 'add', '--sparse', '--', 'extra')
        b.git(self.wt, '-c', 'commit.gpgsign=false', 'commit', '-q', '-s', '-m', 'synthetic evidence')
        return expected

    def test_sparse_checkout_omits_unrelated_source(self):
        self.create()
        self.assertFalse((self.wt / 'large-not-needed').exists())
        self.assertFalse((self.wt / 'web/rest.html').exists())
        self.assertEqual(b.git(self.wt, 'rev-parse', 'HEAD').decode().strip(), self.base)

    def test_exact_two_added_paths_actual_parent_and_signoff(self):
        self.create()
        expected = self.public_commit()
        head = b.audit_commit(self.wt, self.base, expected)
        self.assertIn(b'Signed-off-by: Synthetic Fixture', b.git(self.wt, 'show', '-s', '--format=%B', head))

    def test_extra_file_rejected(self):
        self.create()
        expected = self.public_commit(extra=True)
        with self.assertRaises(Exception):
            b.audit_commit(self.wt, self.base, expected)

    def test_wrong_parent_rejected(self):
        self.create()
        expected = self.public_commit()
        with self.assertRaises(Exception):
            b.audit_commit(self.wt, '0' * 40, expected)

    def test_local_used_branch_rejected(self):
        self.create()
        with self.assertRaises(Exception):
            b.absent_branch(self.repo, 'audit/synthetic')

    def test_remote_used_branch_rejected(self):
        b.git(self.repo, 'push', 'origin', 'HEAD:refs/heads/audit/remote-used')
        with self.assertRaises(Exception):
            b.absent_branch(self.repo, 'audit/remote-used')

    def test_non_audit_branch_rejected(self):
        with self.assertRaises(Exception):
            b.absent_branch(self.repo, 'main')


class OfflineGuards(unittest.TestCase):
    def test_missing_production_receipt_creates_nothing(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            template = root / 'template.json'
            template.write_bytes(b'{"binding":{}}')
            closure = root / 'closure.json'
            closure.write_bytes(b'{"files":{}}')
            review = root / 'review.json'
            review.write_text(json.dumps({'inputs': {}, 'worker_template_sha256': b.WORKER,
                                         'admission_programs_sha256': b.PINS,
                                         'template_sha256': b.digest(template.read_bytes())}), encoding='utf-8')
            args = argparse.Namespace(review=review, review_sha256=b.digest(review.read_bytes()),
                closure=closure, closure_sha256=b.digest(closure.read_bytes()), template=template,
                template_sha256=b.digest(template.read_bytes()),
                worker=Path(os.environ['KERO_GROUPED_WORKER_TEMPLATE']),
                production=root / 'absent-production', out=root / 'out', worktree=root / 'wt')
            with self.assertRaisesRegex(ValueError, "Missing or symlink input:.*API-ACCEPTED"):
                b.preflight(args)
            self.assertFalse(args.out.exists())
            self.assertFalse(args.worktree.exists())

    def test_bound_assignment_must_equal_admission(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            template = root / 'template.py'
            bound = root / 'bound.py'
            template.write_text('CONFIG=None\nCONFIG_SHA256=None\nassert False\n', encoding='utf-8')
            bound.write_text('CONFIG={"changed":True}\nCONFIG_SHA256="abc"\nassert False\n', encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'Bound assignments differ'):
                b.validate_bound(template, bound, {}, 'abc')

    def test_ast_change_outside_assignments_rejected_before_import(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            template = root / 'template.py'
            bound = root / 'bound.py'
            template.write_text('CONFIG=None\nCONFIG_SHA256=None\nassert False\n', encoding='utf-8')
            bound.write_text('CONFIG={}\nCONFIG_SHA256="abc"\nassert True\n', encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'Bound worker differs beyond configuration'):
                b.validate_bound(template, bound, {}, 'abc')

    def test_actual_worker_AST_boundary_with_explicit_schema_only_assignment(self):
        template = Path(os.environ['KERO_GROUPED_WORKER_TEMPLATE']).read_bytes()
        self.assertEqual(b.digest(template), b.WORKER)
        # This proves only that the ACTUAL full worker body is preserved. It
        # deliberately does not import/run or claim to validate this placeholder.
        config = {'SYNTHETIC_AST_ASSIGNMENT_ONLY_NOT_VALID_ADMISSION': True}
        body = template.decode().replace('\nCONFIG=None\n', '\nCONFIG=' + repr(config) + '\n')
        body = body.replace('\nCONFIG_SHA256=None\n', '\nCONFIG_SHA256="schema-only"\n')
        b.check_bound_source(template, body.encode(), config, 'schema-only')

    def test_matching_unknown_template_is_never_imported(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name);template = root/'template.py';bound = root/'bound.py'
            template.write_text('CONFIG=None\nCONFIG_SHA256=None\nraise AssertionError("must never import")\n')
            bound.write_text('CONFIG={}\nCONFIG_SHA256="abc"\nraise AssertionError("must never import")\n')
            with self.assertRaisesRegex(ValueError, 'Only the reviewed worker may be imported'):
                b.validate_bound(template,bound,{},'abc')


if __name__ == '__main__':
    unittest.main()
