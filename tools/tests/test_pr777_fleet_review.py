"""The fleet gate cannot omit delayed aggregates or skipped collection steps."""
import copy
import importlib.util
from pathlib import Path
import subprocess
import unittest

SPEC = importlib.util.spec_from_file_location('fleet', Path(__file__).resolve().parents[1] / 'pr777-fleet-review.py')
fleet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fleet)
WORKFLOW = subprocess.check_output(['git', 'show', fleet.HEAD + ':' + fleet.WORKFLOW], cwd=fleet.ROOT)


def complete():
    jobs = [dict(databaseId=i, name=name, status='completed', conclusion='success', url='job',
                 steps=[dict(number=step['number'], name=step['name'] or 'Run command',
                             status='completed', conclusion='success') for step in steps])
            for i, (name, steps) in enumerate(fleet.inventory(WORKFLOW).items())]
    return dict(databaseId=fleet.RUN, headSha=fleet.HEAD, event='pull_request', status='completed',
                conclusion='success', url=f'https://github.com/CrispStrobe/kerotakis/actions/runs/{fleet.RUN}', jobs=jobs)


class FleetReviewTests(unittest.TestCase):
    def test_all_33_declared_jobs_required(self):
        run = complete()
        self.assertTrue(fleet.review(run, WORKFLOW)['accepted_complete_fleet_gate'])
        self.assertEqual(len(run['jobs']), 33)
        for aggregate in ('All 240 source-informed cases', 'All 288 source-informed v2 cases',
                          'All 120 source-informed v3 cases'):
            changed = copy.deepcopy(run)
            changed['jobs'] = [j for j in changed['jobs'] if j['name'] != aggregate]
            with self.assertRaises(AssertionError): fleet.review(changed, WORKFLOW)

    def test_partial_success_does_not_accept_fleet(self):
        run = complete(); run.update(status='in_progress', conclusion='')
        run['jobs'] = run['jobs'][:12]
        result = fleet.review(run, WORKFLOW)
        self.assertFalse(result['accepted_complete_fleet_gate'])
        self.assertEqual(len(result['remaining_jobs']), 21)

    def test_skipped_failed_or_unfinished_job_never_qualifies(self):
        for status, conclusion in (('completed', 'skipped'), ('completed', 'failure'), ('queued', '')):
            run = complete(); run['jobs'][0].update(status=status, conclusion=conclusion)
            self.assertFalse(fleet.review(run, WORKFLOW)['accepted_complete_fleet_gate'])

    def test_wrong_run_head_event_url_and_job_inventory_rejected(self):
        for field in ('databaseId', 'headSha', 'event', 'url'):
            run = complete(); run[field] = 'substituted'
            with self.assertRaises(AssertionError): fleet.review(run, WORKFLOW)
        for mutation in ('duplicate', 'unknown'):
            run = complete()
            if mutation == 'duplicate': run['jobs'].append(copy.deepcopy(run['jobs'][0]))
            else: run['jobs'][0]['name'] = 'unbound job'
            with self.assertRaises(AssertionError): fleet.review(run, WORKFLOW)

    def test_missing_duplicate_skipped_or_failed_named_step_rejected(self):
        for mutation in ('missing', 'duplicate', 'skipped', 'failed', 'unfinished'):
            run = complete(); job = next(j for j in run['jobs'] if j['steps']); steps = job['steps']
            if mutation == 'missing': steps.pop()
            if mutation == 'duplicate': steps.append(copy.deepcopy(steps[0]))
            if mutation == 'skipped': steps[0]['conclusion'] = 'skipped'
            if mutation == 'failed': steps[0]['conclusion'] = 'failure'
            if mutation == 'unfinished': steps[0]['status'] = 'in_progress'
            with self.assertRaises(AssertionError): fleet.review(run, WORKFLOW)

    def test_build_and_aggregate_commands_cannot_be_missing_or_skipped(self):
        for name in ('Build audited CLI once', 'All 240 source-informed cases',
                     'All 288 source-informed v2 cases', 'All 120 source-informed v3 cases'):
            for mutation in ('empty', 'missing', 'duplicate', 'skipped', 'failed', 'wrong_ordinal'):
                run = complete(); job = next(j for j in run['jobs'] if j['name'] == name)
                if mutation == 'empty': job['steps'] = []
                if mutation == 'missing': job['steps'].pop()
                if mutation == 'duplicate': job['steps'].append(copy.deepcopy(job['steps'][0]))
                if mutation == 'skipped': job['steps'][0]['conclusion'] = 'skipped'
                if mutation == 'failed': job['steps'][0]['conclusion'] = 'failure'
                if mutation == 'wrong_ordinal': job['steps'][0]['number'] = 999
                with self.subTest(name=name, mutation=mutation), self.assertRaises(AssertionError):
                    fleet.review(run, WORKFLOW)

    def test_unhandled_matrix_shape_rejected(self):
        with self.assertRaises(AssertionError):
            fleet.inventory(b'jobs:\n  job:\n    name: matrix\n    strategy:\n      matrix:\n        include: []\n    steps: []\n')


if __name__ == '__main__':
    unittest.main()
