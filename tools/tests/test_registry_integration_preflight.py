"""Offline integration readiness rejects substituted evidence and partial gates."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('registry_preflight', Path(__file__).resolve().parents[1] / 'registry-integration-preflight.py')
preflight = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(preflight)
PLAN_PATH = preflight.ROOT / 'audits/registry-loader-properties-20261011/integration-preparation-f515.json'
PLAN = json.loads(PLAN_PATH.read_text())
DOCS = preflight.bound_inputs(PLAN)
WORKFLOW = preflight.fleet.source_workflow(PLAN['head'])


def complete_fleet():
    jobs = [dict(databaseId=100000 + i, name=name, status='completed', conclusion='success', url='job',
                 steps=[dict(number=step['number'], name=step['name'] or 'Run command',
                             status='completed', conclusion='success') for step in steps])
            for i, (name, steps) in enumerate(preflight.fleet.inventory(WORKFLOW).items())]
    return dict(databaseId=PLAN['fleet_run'], headSha=PLAN['head'], event='pull_request', status='completed',
                conclusion='success', url=f'https://github.com/{preflight.REPOSITORY}/actions/runs/{PLAN["fleet_run"]}', jobs=jobs)


def api_run(run):
    return dict(id=run['databaseId'], head_sha=run['headSha'], event=run['event'], status=run['status'],
                conclusion=run['conclusion'], html_url=run['url'], pull_requests=[dict(number=PLAN['pr'])])


def complete_evidence():
    bound_ci = DOCS[PLAN['full_ci_receipt_path']]
    ci = dict(databaseId=PLAN['ci_run'], headSha=PLAN['head'], event='pull_request', status='completed',
              conclusion='success', url=bound_ci['run_url'], jobs=copy.deepcopy(bound_ci['jobs']))
    fleet = complete_fleet()
    checks = [dict(name=job['name'], status='COMPLETED', conclusion=job['conclusion'].upper())
              for job in ci['jobs'] + fleet['jobs']]
    commits = {}
    for path in PLAN['native_receipt_paths']:
        receipt = DOCS[path]
        commits[receipt['tested_checkout']] = dict(sha=receipt['tested_checkout'], tree=dict(sha=PLAN['tree']),
                                                  parents=[dict(sha=PLAN['native_tested_base']), dict(sha=PLAN['head'])])
    checkout = next(iter(commits))
    return dict(fleet=fleet, fleet_api=api_run(fleet), ci=ci, ci_api=api_run(ci),
                pr=dict(number=PLAN['pr'], state='open', draft=False, mergeable=True, merge_commit_sha=checkout,
                        head=dict(sha=PLAN['head']), base=dict(ref='main', sha=PLAN['base'])),
                main=dict(object=dict(sha=PLAN['base'])), commits=commits, current_synthetic=copy.deepcopy(commits[checkout]),
                rollup=dict(statusCheckRollup=checks, headRefOid=PLAN['head'], baseRefOid=PLAN['base'], state='OPEN', isDraft=False))


def evaluate(evidence=None, docs=None, plan=None):
    return preflight.evaluate(plan or copy.deepcopy(PLAN), docs or copy.deepcopy(DOCS),
                              evidence or complete_evidence())


class RegistryIntegrationPreflightTests(unittest.TestCase):
    def test_complete_is_premerge_only_and_retains_old_native_parents(self):
        result = evaluate()
        self.assertTrue(result['ready_for_merge'])
        self.assertTrue(result['ready_except_draft'])
        self.assertFalse(result['accepted_integration'])
        self.assertEqual(len(result['native']), 2)
        for native in result['native']:
            self.assertFalse(native['new_base_context_test_execution'])
            self.assertEqual(native['tested_checkout_parents'], [PLAN['native_tested_base'], PLAN['head']])

    def test_draft_is_last_gate(self):
        evidence = complete_evidence(); evidence['pr']['draft'] = True; evidence['rollup']['isDraft'] = True
        result = evaluate(evidence)
        self.assertFalse(result['ready_for_merge'])
        self.assertTrue(result['ready_except_draft'])
        evidence['ci']['jobs'][0]['conclusion'] = 'failure'
        with self.assertRaises(ValueError): evaluate(evidence)

    def test_pending_fleet_short_circuits_before_draft_or_ci_assumptions(self):
        evidence = complete_evidence()
        evidence['fleet'].update(status='queued', conclusion='')
        evidence['fleet']['jobs'] = []
        evidence['fleet_api'] = api_run(evidence['fleet'])
        for key in ('ci', 'ci_api', 'pr', 'main', 'commits', 'rollup', 'current_synthetic'): evidence.pop(key)
        result = evaluate(evidence)
        self.assertFalse(result['ready_for_merge'])
        self.assertFalse(result['ready_except_draft'])
        self.assertEqual(len(result['fleet']['remaining_jobs']), 33)

    def test_wrong_run_api_head_event_url_or_pr_rejected_even_pending(self):
        for field, value in [('id', 1), ('head_sha', '0' * 40), ('event', 'push'),
                             ('html_url', 'wrong'), ('pull_requests', [{'number': 779}])]:
            for kind in ('ci_api', 'fleet_api'):
                evidence = complete_evidence(); evidence[kind][field] = value
                with self.subTest(field=field, kind=kind), self.assertRaises(ValueError): evaluate(evidence)

    def test_wrong_normalized_run_or_head_rejected(self):
        for kind in ('ci', 'fleet'):
            for field, value in [('databaseId', 1), ('headSha', '0' * 40), ('event', 'push'), ('url', 'wrong')]:
                evidence = complete_evidence(); evidence[kind][field] = value
                with self.subTest(kind=kind, field=field), self.assertRaises(ValueError): evaluate(evidence)

    def test_api_and_normalized_state_must_agree(self):
        evidence = complete_evidence(); evidence['fleet_api']['status'] = 'in_progress'
        with self.assertRaises(ValueError): evaluate(evidence)

    def test_fresh_pr_head_base_main_mergeability_and_state_rejected(self):
        mutations = [lambda e: e['pr']['head'].update(sha='0' * 40),
                     lambda e: e['pr']['base'].update(sha='0' * 40),
                     lambda e: e['pr']['base'].update(ref='other'),
                     lambda e: e['main']['object'].update(sha='0' * 40),
                     lambda e: e['pr'].update(state='closed'),
                     lambda e: e['pr'].update(mergeable=None),
                     lambda e: e['pr'].update(number=779)]
        for mutation in mutations:
            evidence = complete_evidence(); mutation(evidence)
            with self.assertRaises(ValueError): evaluate(evidence)

    def test_wrong_merge_tree_rejected(self):
        actual_git = preflight.git
        def altered(*args):
            return b'0' * 40 + b'\n' if args[0] == 'merge-tree' else actual_git(*args)
        with patch.object(preflight, 'git', side_effect=altered), self.assertRaises(ValueError): evaluate()
        plan = copy.deepcopy(PLAN); plan['tree'] = '0' * 40
        with self.assertRaises(ValueError): evaluate(plan=plan)

    def test_missing_duplicate_unknown_failed_skipped_ci_jobs_rejected(self):
        for mutation in ('missing', 'duplicate', 'unknown', 'failed', 'skipped'):
            evidence = complete_evidence(); jobs = evidence['ci']['jobs']
            if mutation == 'missing': jobs.pop()
            if mutation == 'duplicate': jobs.append(copy.deepcopy(jobs[0]))
            if mutation == 'unknown': jobs[0]['name'] = 'unknown'
            if mutation == 'failed': jobs[0]['conclusion'] = 'failure'
            if mutation == 'skipped': jobs[0]['conclusion'] = 'skipped'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): evaluate(evidence)

    def test_ci_step_missing_duplicate_skipped_failed_or_renamed_rejected(self):
        for mutation in ('missing', 'duplicate', 'skipped', 'failed', 'rename'):
            evidence = complete_evidence(); steps = evidence['ci']['jobs'][0]['steps']
            if mutation == 'missing': steps.pop()
            if mutation == 'duplicate': steps.append(copy.deepcopy(steps[0]))
            if mutation == 'skipped': steps[0]['conclusion'] = 'skipped'
            if mutation == 'failed': steps[0]['conclusion'] = 'failure'
            if mutation == 'rename': steps[0]['name'] = 'wrong'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): evaluate(evidence)

    def test_fleet_missing_duplicate_or_skipped_ordinal_rejected(self):
        for mutation in ('missing', 'duplicate', 'skipped'):
            evidence = complete_evidence(); steps = evidence['fleet']['jobs'][0]['steps']
            if mutation == 'missing': steps.pop()
            if mutation == 'duplicate': steps.append(copy.deepcopy(steps[0]))
            if mutation == 'skipped': steps[0]['conclusion'] = 'skipped'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): evaluate(evidence)

    def test_native_receipt_count_run_head_hash_platform_or_fixture_rejected(self):
        path = PLAN['native_receipt_paths'][0]
        mutations = [lambda r: r.update(run=1), lambda r: r.update(head='0' * 40),
                     lambda r: r.update(total_focused_passed=332), lambda r: r.update(plan_sha256='0' * 64),
                     lambda r: r['targets'].pop(), lambda r: r['targets'][0].update(fixture_sha256='0' * 64),
                     lambda r: r['targets'][0]['passed'].pop(), lambda r: r['targets'][0].update(failed=['x'])]
        for mutation in mutations:
            docs = copy.deepcopy(DOCS); mutation(docs[path])
            with self.assertRaises(ValueError): evaluate(docs=docs)
        docs = copy.deepcopy(DOCS); docs[path] = copy.deepcopy(docs[PLAN['native_receipt_paths'][1]])
        with self.assertRaises(ValueError): evaluate(docs=docs)

    def test_wrong_synthetic_tree_parents_or_stale_base_tree_rejected(self):
        for mutation in ('tree', 'head_parent', 'parents'):
            evidence = complete_evidence(); commit = next(iter(evidence['commits'].values()))
            if mutation == 'tree': commit['tree']['sha'] = '0' * 40
            if mutation == 'head_parent': commit['parents'][1]['sha'] = '0' * 40
            if mutation == 'parents': commit['parents'].pop()
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): evaluate(evidence)
        actual_git = preflight.git
        def changed_base(*args):
            if args == ('rev-parse', PLAN['native_tested_base'] + '^{tree}'):
                return b'0' * 40 + b'\n'
            return actual_git(*args)
        with patch.object(preflight, 'git', side_effect=changed_base), self.assertRaises(ValueError): evaluate()

    def test_rollup_missing_unfinished_failed_or_empty_rejected(self):
        for mutation in ('missing', 'unfinished', 'failed', 'empty'):
            evidence = complete_evidence(); checks = evidence['rollup']['statusCheckRollup']
            if mutation == 'missing': checks.pop()
            if mutation == 'unfinished': checks[0]['status'] = 'QUEUED'
            if mutation == 'failed': checks[0]['conclusion'] = 'FAILURE'
            if mutation == 'empty': checks.clear()
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): evaluate(evidence)

    def test_current_synthetic_sha_tree_parent_or_unpinned_stale_base_rejected(self):
        for mutation in ('sha', 'tree', 'head_parent', 'base_parent', 'parents'):
            evidence = complete_evidence(); commit = evidence['current_synthetic']
            if mutation == 'sha': commit['sha'] = '0' * 40
            if mutation == 'tree': commit['tree']['sha'] = '0' * 40
            if mutation == 'head_parent': commit['parents'][1]['sha'] = '0' * 40
            if mutation == 'base_parent': commit['parents'][0]['sha'] = '0' * 40
            if mutation == 'parents': commit['parents'].pop()
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): evaluate(evidence)
        plan = copy.deepcopy(PLAN); del plan['native_tested_base']
        with self.assertRaises(ValueError): evaluate(plan=plan)

    def test_current_synthetic_with_actual_new_base_parents_retains_old_native_execution(self):
        evidence = complete_evidence()
        evidence['current_synthetic']['parents'][0]['sha'] = PLAN['base']
        result = evaluate(evidence)
        self.assertTrue(result['ready_for_merge'])
        self.assertEqual(result['current_synthetic_parents'], [PLAN['base'], PLAN['head']])
        self.assertTrue(all(not entry['new_base_context_test_execution'] for entry in result['native']))

    def test_rollup_pr_envelope_must_match_actual_api(self):
        for field, value in [('headRefOid', '0' * 40), ('baseRefOid', '0' * 40),
                             ('state', 'CLOSED'), ('isDraft', True)]:
            evidence = complete_evidence(); evidence['rollup'][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError): evaluate(evidence)

    def test_native_fixture_count_and_base_pin_rejected(self):
        plan = copy.deepcopy(PLAN); plan['expected_native_fixture_files'] = 29
        with self.assertRaises(ValueError): evaluate(plan=plan)
        plan = copy.deepcopy(PLAN); plan['native_tested_base'] = PLAN['base']
        evidence = complete_evidence(); evidence['current_synthetic']['parents'][0]['sha'] = PLAN['base']
        with self.assertRaises(ValueError): evaluate(evidence, plan=plan)

    def test_bound_receipt_and_source_sha_rejected(self):
        plan = copy.deepcopy(PLAN); path = plan['native_plan_path']; plan['receipt_sha256'][path] = '0' * 64
        with self.assertRaises(ValueError): preflight.bound_inputs(plan)
        plan = copy.deepcopy(PLAN); path = next(iter(plan['changed_source_sha256'])); plan['changed_source_sha256'][path] = '0' * 64
        with self.assertRaises(ValueError): preflight.bound_inputs(plan)
        plan = copy.deepcopy(PLAN); del plan['receipt_sha256'][plan['native_plan_path']]
        with self.assertRaises(ValueError): preflight.bound_inputs(plan)

    def test_public_plan_paths_cannot_escape_repository(self):
        for value in ('/private/path', '../elsewhere', 'audits/../../elsewhere'):
            with self.assertRaises(ValueError): preflight.portable_path(value)

    def test_collector_only_issues_read_commands_and_exclusive_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            collector = preflight.Collector(Path(directory))
            with patch.object(preflight.subprocess, 'check_output', return_value=b'{}') as command:
                collector.api('pr', 'pulls/778')
                self.assertEqual(command.call_args.args[0], ['gh', 'api', 'repos/CrispStrobe/kerotakis/pulls/778'])
                collector.run('ci', PLAN['ci_run'])
                self.assertEqual(command.call_args.args[0][:3], ['gh', 'run', 'view'])
                with self.assertRaises(FileExistsError): collector.api('pr', 'pulls/778')


if __name__ == '__main__':
    unittest.main()
