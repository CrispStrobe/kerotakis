"""Explicit source/run bindings and delayed fleet jobs fail closed."""
import copy
import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    'chemistry_fleet', Path(__file__).resolve().parents[1] / 'chemistry-fleet-review.py')
fleet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fleet)
HEAD = 'f515781d1bf183a5602571e034fbbf2b5e18b7ff'
RUN = 38158962488
WORKFLOW = fleet.source_workflow(HEAD)


def complete():
    jobs = [dict(databaseId=i, name=name, status='completed', conclusion='success', url='job',
                 steps=[dict(number=step['number'], name=step['name'] or 'Run command',
                             status='completed', conclusion='success') for step in steps])
            for i, (name, steps) in enumerate(fleet.inventory(WORKFLOW).items())]
    return dict(databaseId=RUN, headSha=HEAD, event='pull_request', status='completed',
                conclusion='success', url=f'https://github.com/{fleet.REPOSITORY}/actions/runs/{RUN}', jobs=jobs)


def review(run, **kwargs):
    return fleet.review(run, WORKFLOW, head=HEAD, run_id=RUN, **kwargs)


class ChemistryFleetReviewTests(unittest.TestCase):
    def test_complete_gate_is_not_integration(self):
        result = review(complete(), pr=778)
        self.assertTrue(result['accepted_complete_fleet_gate'])
        self.assertFalse(result['accepted_integration'])
        self.assertFalse(result['pr_identity_verified_from_run_metadata'])
        self.assertEqual(result['head'], HEAD)
        self.assertEqual(result['run'], RUN)
        self.assertEqual(len(result['expected_jobs']), 33)

    def test_pending_partial_inventory_stays_pending(self):
        run = complete(); run.update(status='queued', conclusion='')
        run['jobs'] = run['jobs'][:12]
        result = review(run)
        self.assertFalse(result['accepted_complete_fleet_gate'])
        self.assertEqual(len(result['remaining_jobs']), 21)
        self.assertEqual(len(result['not_yet_created_jobs']), 21)

    def test_completed_missing_job_rejected(self):
        for name in ('All 240 source-informed cases', 'All 288 source-informed v2 cases',
                     'All 120 source-informed v3 cases', 'Build audited CLI once'):
            run = complete(); run['jobs'] = [job for job in run['jobs'] if job['name'] != name]
            with self.subTest(name=name), self.assertRaises(ValueError):
                review(run)

    def test_wrong_metadata_bindings_rejected(self):
        for field in ('databaseId', 'headSha', 'event', 'url'):
            run = complete(); run[field] = 'substituted'
            with self.subTest(field=field), self.assertRaises(ValueError):
                review(run)

    def test_pr_binding_when_metadata_available(self):
        run = complete(); run['pullRequests'] = [{'number': 778}]
        self.assertTrue(review(run, pr=778)['pr_identity_verified_from_run_metadata'])
        self.assertEqual(review(run)['pr'], 778)
        with self.assertRaises(ValueError): review(run, pr=779)
        for requests in ([], [{'number': 778}, {'number': 779}], [{'number': '778'}]):
            run['pullRequests'] = requests
            with self.assertRaises(ValueError): review(run, pr=778)

    def test_duplicate_unknown_jobs_rejected(self):
        for mutation in ('duplicate', 'unknown'):
            run = complete()
            if mutation == 'duplicate': run['jobs'].append(copy.deepcopy(run['jobs'][0]))
            else: run['jobs'][0]['name'] = 'unbound job'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): review(run)

    def test_job_or_run_non_success_never_qualifies(self):
        for status, conclusion in (('completed', 'skipped'), ('completed', 'failure'), ('queued', '')):
            run = complete(); run['jobs'][0].update(status=status, conclusion=conclusion)
            self.assertFalse(review(run)['accepted_complete_fleet_gate'])
        run = complete(); run['conclusion'] = 'failure'
        self.assertFalse(review(run)['accepted_complete_fleet_gate'])

    def test_every_named_and_unnamed_ordinal_required(self):
        for named in (True, False):
            for mutation in ('missing', 'duplicate', 'skipped', 'failed', 'unfinished', 'wrong_ordinal'):
                run = complete()
                name, requirements = next((name, steps) for name, steps in fleet.inventory(WORKFLOW).items()
                                          if any((step['name'] is not None) == named for step in steps))
                ordinal = next(step['number'] for step in requirements if (step['name'] is not None) == named)
                job = next(job for job in run['jobs'] if job['name'] == name)
                target = next(step for step in job['steps'] if step['number'] == ordinal)
                if mutation == 'missing': job['steps'].remove(target)
                if mutation == 'duplicate': job['steps'].append(copy.deepcopy(target))
                if mutation == 'skipped': target['conclusion'] = 'skipped'
                if mutation == 'failed': target['conclusion'] = 'failure'
                if mutation == 'unfinished': target['status'] = 'in_progress'
                if mutation == 'wrong_ordinal': target['number'] = 999
                with self.subTest(named=named, mutation=mutation), self.assertRaises(ValueError): review(run)

    def test_wrong_named_step_rejected(self):
        run = complete()
        name, required = next((name, step) for name, steps in fleet.inventory(WORKFLOW).items()
                              for step in steps if step['name'] is not None)
        job = next(job for job in run['jobs'] if job['name'] == name)
        next(step for step in job['steps'] if step['number'] == required['number'])['name'] = 'wrong'
        with self.assertRaises(ValueError): review(run)

    def test_source_is_exact_git_head_and_unknown_head_rejected(self):
        for head in ('main', HEAD[:8], '-x', 'z' * 40):
            with self.assertRaises(ValueError): fleet.source_workflow(head)
        with self.assertRaises(subprocess.CalledProcessError): fleet.source_workflow('0' * 40)
        with patch.object(fleet.subprocess, 'check_output', return_value=b'blob\n'):
            with self.assertRaises(ValueError): fleet.source_workflow(HEAD)
        self.assertEqual(WORKFLOW, subprocess.check_output(
            ['git', 'show', HEAD + ':' + fleet.WORKFLOW], cwd=fleet.ROOT))

    def test_inventory_shape_and_size_changes_rejected(self):
        workflows = (b'jobs:\n  job:\n    name: matrix\n    strategy:\n      matrix:\n        include: []\n    steps: []\n',
                     b'jobs:\n  job:\n    name: insufficient\n    steps: []\n')
        for workflow in workflows:
            with self.assertRaises(ValueError): fleet.inventory(workflow)

    def test_invalid_explicit_run_rejected(self):
        for run_id in (0, -1, '38158962488', True):
            with self.assertRaises(ValueError):
                fleet.review(complete(), WORKFLOW, head=HEAD, run_id=run_id)


if __name__ == '__main__':
    unittest.main()
