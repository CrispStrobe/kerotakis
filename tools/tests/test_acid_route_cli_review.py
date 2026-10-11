"""Reject mismatched reuse lineage and archives; no Kero or hosted runs."""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import py_compile
import tempfile
import unittest
import zipfile

SPEC = importlib.util.spec_from_file_location('acid_review', Path(__file__).resolve().parents[1] / 'acid-route-cli-review.py')
review = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(review)


def run():
    return dict(headSha='head', event='workflow_dispatch', status='completed', conclusion='success',
                jobs=[dict(name='intended', status='completed', conclusion='success')])


class AcidReuseReviewTests(unittest.TestCase):
    def test_dispatch_and_required_collection_steps_cannot_be_substituted(self):
        prepared = dict(harness='head', parent_run=123, parent_harness='parent',
                        source='source', campaign='routes', expected_cases=6,
                        expected_processes=12, job_name='intended')
        dispatch = dict(prepared, dispatched_once=True, compiled_source='source',
                        run=456, run_url='https://example.test/runs/456')
        data = run()
        data.update(databaseId=456, url=dispatch['run_url'])
        data['jobs'][0]['steps'] = [dict(name=name, status='completed', conclusion='success')
                                  for name in ('Download exact completed source-bound replay archive',
                                               'Bind six original route scripts and collect twelve raw processes')]
        review.dispatched_job(data, prepared, dispatch)
        for mutation in ('run', 'url', 'source', 'parent', 'count', 'missing',
                         'duplicate', 'skipped', 'failed', 'unfinished', 'other_failure'):
            changed = copy.deepcopy(data); receipt = copy.deepcopy(dispatch)
            steps = changed['jobs'][0]['steps']
            if mutation == 'run': changed['databaseId'] = 999
            if mutation == 'url': changed['url'] = 'https://example.test/runs/999'
            if mutation == 'source': receipt['compiled_source'] = 'other'
            if mutation == 'parent': receipt['parent_run'] = 999
            if mutation == 'count': receipt['expected_processes'] = 10
            if mutation == 'missing': steps.pop()
            if mutation == 'duplicate': steps.append(copy.deepcopy(steps[0]))
            if mutation == 'skipped': steps[0]['conclusion'] = 'skipped'
            if mutation == 'failed': steps[0]['conclusion'] = 'failure'
            if mutation == 'unfinished': steps[0]['status'] = 'in_progress'
            if mutation == 'other_failure':
                steps.append(dict(name='upload', status='completed', conclusion='failure'))
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                review.dispatched_job(changed, prepared, receipt)

    def test_recheck_uses_verified_source_even_when_old_bytecode_header_matches(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            helper = root / 'payload_helper.py'
            helper.write_text("VALUE='unsafe'\n")
            original = helper.stat()
            py_compile.compile(str(helper), doraise=True)
            helper.write_text("VALUE='source'\n")
            os.utime(helper, (original.st_atime, original.st_mtime))
            script = root / 'script.py'
            script.write_text('import sys\nfrom pathlib import Path\n'
                              'sys.path.insert(0,str(Path(__file__).parent))\n'
                              'import payload_helper\n'
                              'Path(sys.argv[1]).write_text(payload_helper.VALUE)\n')
            output = root / 'result.txt'
            review.recheck(script, [str(output)])
            self.assertEqual(output.read_text(), 'source')

    def test_correct_wording_with_wrong_severity_or_dose_narration_is_rejected(self):
        case = {'role': 'positive', 'expected_warning': {
            'severity': 'caution', 'hazard': 'frozen hazard', 'real_world': 'frozen context'}}
        text = 'HAZARD (Caution): frozen hazard — frozen context\n' \
               'v1: titrated with 1 mol/L hydrochloric acid; 1 step(s), 10.0 mL total\n'
        self.assertTrue(all(review.narration(text, case).values()))
        for changed in (text.replace('Caution', 'Danger'), text.replace('10.0', '5.0'),
                        text.replace('hydrochloric acid', 'sodium hydroxide'), text + text):
            self.assertFalse(all(review.narration(changed, case).values()))
        case['role'] = 'benign'
        benign = text.splitlines()[1]
        self.assertTrue(all(review.narration(benign, case).values()))
        self.assertFalse(all(review.narration(text, case).values()))

    def test_wrong_unfinished_failed_duplicate_and_skipped_job_rejected(self):
        self.assertEqual(review.completed_job(run(), 'head', 'intended')['name'], 'intended')
        for mutation in ('head', 'event', 'run_status', 'job_status', 'job_failure', 'job_name', 'duplicate', 'skipped'):
            data = run()
            if mutation == 'head': data['headSha'] = 'other'
            if mutation == 'event': data['event'] = 'pull_request'
            if mutation == 'run_status': data['status'] = 'in_progress'
            if mutation == 'job_status': data['jobs'][0]['status'] = 'in_progress'
            if mutation == 'job_failure': data['jobs'][0]['conclusion'] = 'failure'
            if mutation == 'job_name': data['jobs'][0]['name'] = 'other'
            if mutation == 'duplicate': data['jobs'].append(copy.deepcopy(data['jobs'][0]))
            if mutation == 'skipped': data['jobs'][0]['conclusion'] = 'skipped'
            with self.assertRaises(AssertionError): review.completed_job(data, 'head', 'intended')

    def test_extra_missing_or_modified_inherited_file_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            build = Path(temp)
            binding = dict(compiled_source='source', source_tree='tree', binary_sha256='synthetic')
            (build / 'binding.json').write_text(json.dumps(binding))
            (build / 'Cargo.lock').write_text('frozen lock')
            parent = dict(source_binding=binding,
                          build_evidence_sha256={p.name: review.digest(p) for p in build.iterdir()})
            review.inherited_build(build, parent)
            (build / 'Cargo.lock').write_text('modified lock')
            with self.assertRaises(AssertionError): review.inherited_build(build, parent)
            (build / 'Cargo.lock').write_text('frozen lock')
            (build / 'extra.txt').write_text('unbound')
            with self.assertRaises(AssertionError): review.inherited_build(build, parent)
            (build / 'extra.txt').unlink(); (build / 'Cargo.lock').unlink()
            with self.assertRaises(AssertionError): review.inherited_build(build, parent)

    def test_archive_byte_mismatch_traversal_and_unbound_parent_file_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); parent = root / 'parent'; parent.mkdir()
            (parent / 'file.txt').write_text('frozen')
            archive = root / 'archive.zip'
            with zipfile.ZipFile(archive, 'w') as zipped: zipped.writestr('file.txt', 'frozen')
            review.archive_matches_parent(archive, parent)
            cached = parent / '__pycache__'; cached.mkdir()
            (cached / 'review.pyc').write_bytes(b'regenerated offline cache')
            review.archive_matches_parent(archive, parent)
            (parent / 'file.txt').write_text('altered')
            with self.assertRaises(AssertionError): review.archive_matches_parent(archive, parent)
            (parent / 'file.txt').write_text('frozen'); (parent / 'extra').write_text('unbound')
            with self.assertRaises(AssertionError): review.archive_matches_parent(archive, parent)
            for name in ('../file.txt', '/file.txt'):
                with zipfile.ZipFile(archive, 'w') as zipped: zipped.writestr(name, 'frozen')
                with self.assertRaises(AssertionError): review.archive_matches_parent(archive, parent)

    def test_parent_identity_expiry_and_digest_cannot_be_substituted(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive = root / 'archive.zip'; archive.write_bytes(b'synthetic archive bytes')
            prepared = dict(parent_run=123, parent_harness='harness', parent_artifact='intended')
            run_row = dict(id=123, head_sha='harness', event='workflow_dispatch',
                           status='completed', conclusion='success')
            jobs = dict(total_count=1, jobs=[dict(name=review.PARENT_JOB, status='completed', conclusion='success')])
            artifact = dict(id=456, name='intended', expired=False,
                            workflow_run=dict(id=123, head_sha='harness'), digest='sha256:'+review.digest(archive))
            originals = {'run.json': run_row, 'jobs.json': jobs, 'artifact.json': artifact,
                         'artifacts.json': dict(total_count=1, artifacts=[artifact])}
            (root / 'artifact-id.txt').write_text('456\n')
            for name, data in originals.items(): (root / name).write_text(json.dumps(data))
            review.parent_metadata(root, prepared)
            for mutation in ('run', 'head', 'expiry', 'digest', 'artifact_head', 'count'):
                data = copy.deepcopy(originals)
                if mutation == 'run': data['run.json']['id'] = 999
                if mutation == 'head': data['run.json']['head_sha'] = 'other'
                if mutation in ('expiry', 'digest', 'artifact_head'):
                    a = data['artifact.json']
                    if mutation == 'expiry': a['expired'] = True
                    if mutation == 'digest': a['digest'] = 'sha256:'+'0'*64
                    if mutation == 'artifact_head': a['workflow_run']['head_sha'] = 'other'
                    data['artifacts.json']['artifacts'] = [a]
                if mutation == 'count': data['jobs.json']['total_count'] = 2
                for name, value in data.items(): (root / name).write_text(json.dumps(value))
                with self.assertRaises(AssertionError): review.parent_metadata(root, prepared)


if __name__ == '__main__':
    unittest.main()
