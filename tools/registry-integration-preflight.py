#!/usr/bin/env python3
"""Read-only registry PR integration preflight; never merges or edits a PR."""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = 'CrispStrobe/kerotakis'
SPEC = importlib.util.spec_from_file_location('chemistry_fleet', ROOT / 'tools/chemistry-fleet-review.py')
fleet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fleet)
require = fleet.require
NATIVE_NAMES = {'Test (native) (ubuntu-latest)', 'Test (native) (macos-latest)'}
NATIVE_STEPS = {'Format', 'Clippy', 'Test', 'Codex — every claim must still compute',
                'Curiosity coverage — the recorded route is still the route'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def portable_path(value):
    path = PurePosixPath(value)
    require(bool(value) and not path.is_absolute() and '..' not in path.parts,
            'plan paths must be repository relative')
    return ROOT / path


def validate_plan(plan):
    for key in ('head', 'base', 'tree'):
        require(re.fullmatch('[0-9a-f]{40}', plan[key]) is not None, 'invalid pinned ' + key)
    for key in ('pr', 'ci_run', 'fleet_run'):
        require(type(plan[key]) is int and plan[key] > 0, 'invalid pinned ' + key)
    require(plan['expected_final_fleet_jobs'] == 33, 'wrong fleet contract count')
    require(type(plan['expected_native_fixture_files']) is int and plan['expected_native_fixture_files'] > 0,
            'invalid native fixture file count')
    if 'native_tested_base' in plan:
        require(re.fullmatch('[0-9a-f]{40}', plan['native_tested_base']) is not None, 'invalid native tested base')
    require(type(plan['expected_native_functions']) is int and plan['expected_native_functions'] > 0,
            'invalid native count')
    require(len(plan['required_successful_jobs']) == 5 and len(set(plan['required_successful_jobs'])) == 5,
            'wrong required CI gates')
    require(NATIVE_NAMES <= set(plan['required_successful_jobs']), 'missing required native gates')
    require(len(plan['native_receipt_paths']) == 2 and len(set(plan['native_receipt_paths'])) == 2,
            'both independent native receipts required')
    for field in ('receipt_sha256', 'changed_source_sha256'):
        require(bool(plan[field]), 'missing bound hashes')
        for path, digest in plan[field].items():
            portable_path(path)
            require(re.fullmatch('[0-9a-f]{64}', digest) is not None, 'invalid SHA256')
    required = {plan['native_plan_path'], plan['full_ci_receipt_path'], *plan['native_receipt_paths']}
    require(required <= set(plan['receipt_sha256']), 'unbound required receipt')


def bound_inputs(plan):
    validate_plan(plan)
    docs = {}
    for path, digest in plan['receipt_sha256'].items():
        raw = portable_path(path).read_bytes()
        require(sha(raw) == digest, 'changed bound receipt: ' + path)
        if path.endswith('.json'):
            docs[path] = json.loads(raw)
    require(git('rev-parse', plan['head'] + '^{tree}').decode().strip() == plan['tree'], 'wrong head tree')
    for path, digest in plan['changed_source_sha256'].items():
        require(sha(git('show', plan['head'] + ':' + path)) == digest, 'wrong exact-head source: ' + path)
    return docs


def run_identity(api, normalized, plan, run_id, *, completed):
    url = f'https://github.com/{REPOSITORY}/actions/runs/{run_id}'
    require(api['id'] == run_id and api['head_sha'] == plan['head'], 'wrong API run/head')
    require(api['event'] == 'pull_request' and api['html_url'] == url, 'wrong API event/URL')
    require([item['number'] for item in api['pull_requests']] == [plan['pr']], 'wrong API PR association')
    require(normalized['databaseId'] == run_id and normalized['headSha'] == plan['head'], 'wrong normalized run/head')
    require(normalized['event'] == 'pull_request' and normalized['url'] == url, 'wrong normalized event/URL')
    require(api['status'] == normalized['status'] and (api['conclusion'] or '') == (normalized['conclusion'] or ''),
            'API and normalized run states disagree')
    if completed:
        require(api['status'] == 'completed' and api['conclusion'] == 'success', 'run is not completed successfully')


def indexed(items, key, message):
    result = {}
    for item in items:
        require(item[key] not in result, message)
        result[item[key]] = item
    return result


def validate_ci(plan, bound, fresh):
    require(bound['pr'] == plan['pr'] and bound['head'] == plan['head'] and bound['run'] == plan['ci_run'],
            'wrong bound CI identity')
    require(bound['accepted_full_ci_gate'] is True, 'bound CI receipt not accepted')
    require(set(bound['required_successful_jobs']) == set(plan['required_successful_jobs']), 'changed required CI gate list')
    expected = indexed(bound['jobs'], 'name', 'duplicate bound CI job')
    observed = indexed(fresh['jobs'], 'name', 'duplicate fresh CI job')
    require(len(expected) == 26 and set(observed) == set(expected), 'wrong complete CI job inventory')
    successful = []
    for name, job in observed.items():
        original = expected[name]
        require(job['databaseId'] == original['databaseId'] and job['url'] == original['url'], 'substituted CI job')
        require(job['status'] == 'completed', 'unfinished CI job')
        if name == 'Publish the demo':
            require(job['conclusion'] == 'skipped', 'unexpected publication gate outcome')
            continue
        require(job['conclusion'] == 'success', 'unsuccessful CI job')
        require(original['status'] == 'completed' and original['conclusion'] == 'success', 'bound CI job not successful')
        steps = indexed(job['steps'], 'number', 'duplicate CI step ordinal')
        bound_steps = indexed(original['steps'], 'number', 'duplicate bound CI step ordinal')
        require(set(steps) == set(bound_steps), 'changed CI step ordinal inventory')
        for number, step in steps.items():
            prior = bound_steps[number]
            require(step['name'] == prior['name'] and step['status'] == 'completed'
                    and step['conclusion'] == prior['conclusion']
                    and step['conclusion'] in {'success', 'skipped'}, 'changed/failed/skipped CI step')
        successful.append(name)
    require(len(successful) == 25 and set(plan['required_successful_jobs']) <= set(successful), 'missing required successful CI gate')
    return observed


def validate_native(plan, docs, jobs, commits):
    native_plan = docs[plan['native_plan_path']]
    require(native_plan['pr'] == plan['pr'] and native_plan['head'] == plan['head']
            and native_plan['head_tree'] == plan['tree'], 'wrong native plan identity')
    targets = indexed(native_plan['targets'], 'path', 'duplicate planned native fixture')
    require(len(targets) == plan['expected_native_fixture_files'], 'wrong native fixture file count')
    count = 0
    for path, target in targets.items():
        raw = git('show', plan['head'] + ':' + path)
        require(sha(raw) == target['fixture_sha256'], 'wrong source fixture SHA')
        identities = re.findall(r'#\[test\]\s*fn\s+(\w+)\s*\(', raw.decode())
        require(bool(identities) and len(identities) == raw.count(b'#[test]') and len(identities) == len(set(identities)),
                'unsupported or repeated source test declaration')
        require(identities == target['test_identities'] and len(identities) == target['functions'], 'changed planned fixture identities/count')
        count += len(identities)
    require(count == native_plan['expected_functions'] == plan['expected_native_functions'], 'wrong native inventory count')
    seen_jobs = set()
    summaries = []
    for path in plan['native_receipt_paths']:
        receipt = docs[path]
        require(receipt['pr'] == plan['pr'] and receipt['head'] == plan['head']
                and receipt['run'] == plan['ci_run'], 'wrong native receipt identity')
        require(receipt['plan_sha256'] == plan['receipt_sha256'][plan['native_plan_path']], 'wrong native plan hash')
        require(receipt['tested_tree'] == plan['tree'] and receipt['head_tree_equals_tested_checkout'] is True
                and receipt['conclusion'] == 'success', 'wrong native tested tree/outcome')
        matches = [job for job in jobs.values() if job['databaseId'] == receipt['job']]
        require(len(matches) == 1 and matches[0]['name'] in NATIVE_NAMES, 'unknown native job')
        job = matches[0]
        require(job['name'] not in seen_jobs and receipt['job_url'] == job['url'], 'duplicate/substituted native platform')
        seen_jobs.add(job['name'])
        require(set(receipt['successful_steps']) == NATIVE_STEPS, 'missing native control step')
        steps = indexed(job['steps'], 'name', 'duplicate native step name')
        require(all(name in steps and steps[name]['status'] == 'completed' and steps[name]['conclusion'] == 'success'
                    for name in NATIVE_STEPS), 'failed/skipped native control step')
        actual = indexed(receipt['targets'], 'path', 'duplicate native receipt fixture')
        require(set(actual) == set(targets), 'wrong native receipt fixture inventory')
        for fixture, target in targets.items():
            outcome = actual[fixture]
            require(outcome['fixture_sha256'] == target['fixture_sha256'] and outcome['passed'] == target['test_identities']
                    and outcome['failed'] == [], 'wrong native fixture outcome')
        require(receipt['total_focused_passed'] == count, 'wrong native receipt total')
        commit = commits[receipt['tested_checkout']]
        require(commit['sha'] == receipt['tested_checkout'] and commit['tree']['sha'] == plan['tree'], 'wrong synthetic tested commit')
        parents = [item['sha'] for item in commit['parents']]
        require(len(parents) == 2 and parents[1] == plan['head'], 'wrong synthetic tested parents')
        if 'native_tested_base' in plan:
            require(parents[0] == plan['native_tested_base'], 'wrong pinned native tested base')
        compatible = parents[0] == plan['base']
        if not compatible:
            require(parents[0] == plan.get('native_tested_base'), 'unpinned stale native base')
            require(git('rev-parse', parents[0] + '^{tree}') == git('rev-parse', plan['base'] + '^{tree}'),
                    'stale native base has different tracked tree')
        summaries.append(dict(job=job['name'], tested_checkout=receipt['tested_checkout'], tested_checkout_parents=parents,
                              tested_tree=plan['tree'], current_base_tree_compatible=True,
                              new_base_context_test_execution=compatible))
    require(seen_jobs == NATIVE_NAMES, 'both native platforms not accepted')
    return summaries


def validate_rollup(rollup, expected_names):
    require(bool(rollup), 'empty rollup')
    names = set()
    for check in rollup:
        if check.get('__typename') == 'StatusContext':
            require(check['state'] == 'SUCCESS', 'unsuccessful legacy status')
            names.add(check['context'])
        else:
            require(check['status'] == 'COMPLETED' and check['conclusion'] in {'SUCCESS', 'SKIPPED'}, 'unfinished/failed rollup check')
            names.add(check['name'])
    require(expected_names <= names, 'rollup missing expected CI/fleet checks')


def evaluate(plan, docs, evidence):
    run_identity(evidence['fleet_api'], evidence['fleet'], plan, plan['fleet_run'], completed=False)
    reviewed = fleet.review(evidence['fleet'], fleet.source_workflow(plan['head']),
                           head=plan['head'], run_id=plan['fleet_run'], pr=plan['pr'])
    result = dict(pr=plan['pr'], head=plan['head'], base=plan['base'], tree=plan['tree'],
                  ready_for_merge=False, ready_except_draft=False, accepted_integration=False, fleet=reviewed)
    if not reviewed['accepted_complete_fleet_gate']:
        result['pending_reason'] = 'own complete chemistry fleet not accepted'
        return result
    run_identity(evidence['fleet_api'], evidence['fleet'], plan, plan['fleet_run'], completed=True)
    pr = evidence['pr']
    require(pr['number'] == plan['pr'] and pr['state'] == 'open' and pr['head']['sha'] == plan['head'], 'wrong/closed PR head')
    require(pr['base']['ref'] == 'main' and pr['base']['sha'] == plan['base'], 'wrong PR base')
    require(pr['mergeable'] is True, 'PR mergeability not confirmed')
    require(evidence['main']['object']['sha'] == plan['base'], 'main advanced beyond pinned base')
    require(git('rev-parse', plan['head'] + '^{tree}').decode().strip() == plan['tree'], 'head tree changed')
    merge_tree = git('merge-tree', '--write-tree', plan['base'], plan['head']).decode().splitlines()[0]
    require(merge_tree == plan['tree'], 'proposed current-base merge tree differs from tested head')
    current_commit = evidence['current_synthetic']
    require(current_commit['sha'] == pr['merge_commit_sha'] and current_commit['tree']['sha'] == plan['tree'],
            'wrong current PR synthetic merge identity/tree')
    current_parents = [item['sha'] for item in current_commit['parents']]
    require(len(current_parents) == 2 and current_parents[1] == plan['head'], 'wrong current synthetic merge parents')
    if current_parents[0] != plan['base']:
        require(current_parents[0] == plan.get('native_tested_base'), 'unpinned stale current synthetic base')
        require(git('rev-parse', current_parents[0] + '^{tree}') == git('rev-parse', plan['base'] + '^{tree}'),
                'stale current synthetic base has different tracked tree')
    run_identity(evidence['ci_api'], evidence['ci'], plan, plan['ci_run'], completed=True)
    jobs = validate_ci(plan, docs[plan['full_ci_receipt_path']], evidence['ci'])
    result['native'] = validate_native(plan, docs, jobs, evidence['commits'])
    envelope = evidence['rollup']
    require(envelope['headRefOid'] == pr['head']['sha'] and envelope['baseRefOid'] == pr['base']['sha']
            and envelope['state'] == 'OPEN' and envelope['isDraft'] == pr['draft'], 'rollup PR envelope disagrees with fresh PR API')
    validate_rollup(evidence['rollup']['statusCheckRollup'], set(jobs) | set(reviewed['expected_jobs']))
    require(type(pr['draft']) is bool, 'missing draft state')
    result.update(ready_except_draft=True, ready_for_merge=not pr['draft'], draft=pr['draft'],
                  current_main_equals_base=True, computed_current_merge_tree=merge_tree,
                  current_synthetic_merge=pr['merge_commit_sha'], current_synthetic_parents=current_parents,
                  scope='Read-only premerge gates only. No merge, PR edit, or postmerge integration acceptance. '
                        'Native checkout parents remain recorded; base-tree compatibility does not claim a rerun.')
    return result


class Collector:
    def __init__(self, directory):
        self.directory = directory
        self.hashes = {}

    def get(self, name, command):
        raw = subprocess.check_output(command, cwd=ROOT)
        with (self.directory / (name + '.json')).open('xb') as stream:
            stream.write(raw)
        self.hashes[name + '.json'] = sha(raw)
        return json.loads(raw)

    def api(self, name, endpoint):
        return self.get(name, ['gh', 'api', 'repos/' + REPOSITORY + '/' + endpoint])

    def run(self, name, run_id):
        return self.get(name, ['gh', 'run', 'view', str(run_id), '--repo', REPOSITORY, '--json',
                               'databaseId,headSha,event,status,conclusion,url,jobs'])


def collect(plan, docs, collector):
    evidence = dict(fleet_api=collector.api('fleet-api', f'actions/runs/{plan["fleet_run"]}'),
                    fleet=collector.run('fleet', plan['fleet_run']))
    run_identity(evidence['fleet_api'], evidence['fleet'], plan, plan['fleet_run'], completed=False)
    initial = fleet.review(evidence['fleet'], fleet.source_workflow(plan['head']), head=plan['head'], run_id=plan['fleet_run'], pr=plan['pr'])
    if not initial['accepted_complete_fleet_gate']:
        return evidence
    evidence.update(pr=collector.api('pr', f'pulls/{plan["pr"]}'),
                    main=collector.api('main', 'git/ref/heads/main'),
                    ci_api=collector.api('ci-api', f'actions/runs/{plan["ci_run"]}'),
                    ci=collector.run('ci', plan['ci_run']),
                    rollup=collector.get('rollup', ['gh', 'pr', 'view', str(plan['pr']), '--repo', REPOSITORY,
                                                  '--json', 'statusCheckRollup,headRefOid,baseRefOid,state,isDraft']))
    require(bool(evidence['pr']['merge_commit_sha']), 'current PR synthetic merge unavailable')
    evidence['current_synthetic'] = collector.api('current-synthetic-merge', 'git/commits/' + evidence['pr']['merge_commit_sha'])
    evidence['commits'] = {}
    for path in plan['native_receipt_paths']:
        checkout = docs[path]['tested_checkout']
        if checkout not in evidence['commits']:
            evidence['commits'][checkout] = collector.api('tested-commit-' + checkout, 'git/commits/' + checkout)
    return evidence


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--evidence-dir', type=Path, required=True)
    args = parser.parse_args()
    raw = args.plan.read_bytes()
    plan = json.loads(raw)
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    directory = args.evidence_dir / ('registry-preflight-' + stamp + '-' + uuid.uuid4().hex[:8])
    directory.mkdir(mode=0o700, parents=True, exist_ok=False)
    with (directory / 'plan.json').open('xb') as stream:
        stream.write(raw)
    collector = Collector(directory)
    try:
        docs = bound_inputs(plan)
        result = evaluate(plan, docs, collect(plan, docs, collector))
    except (KeyError, ValueError, OSError, subprocess.CalledProcessError) as error:
        result = dict(ready_for_merge=False, ready_except_draft=False, accepted_integration=False,
                      rejected_reason=str(error))
    result.update(plan_sha256=sha(raw), reviewer_sha256=sha(Path(__file__).read_bytes()),
                  fleet_reviewer_sha256=sha((ROOT / 'tools/chemistry-fleet-review.py').read_bytes()),
                  evidence_sha256=collector.hashes)
    with (directory / 'result.json').open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps(result, allow_nan=False))
    return 1 if 'rejected_reason' in result else 0


if __name__ == '__main__':
    raise SystemExit(main())
