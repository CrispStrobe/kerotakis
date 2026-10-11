#!/usr/bin/env python3
"""Review all declared chemistry fleet jobs against an explicit exact source head."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = '.github/workflows/chemistry-audit.yml'
REPOSITORY = 'CrispStrobe/kerotakis'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def inventory(workflow):
    jobs = yaml.safe_load(workflow)['jobs']
    expected = {}
    for job in jobs.values():
        matrix = job.get('strategy', {}).get('matrix')
        if matrix is None:
            names = [job['name']]
        else:
            require(set(matrix) == {'family'}, 'unsupported matrix shape')
            require('${{ matrix.family }}' in job['name'], 'unexpanded matrix job name')
            require(isinstance(matrix['family'], list) and bool(matrix['family']), 'empty matrix')
            require(all(isinstance(value, str) for value in matrix['family']), 'invalid matrix value')
            names = [job['name'].replace('${{ matrix.family }}', value)
                     for value in matrix['family']]
        steps = [dict(number=i + 2, name=step.get('name'))
                 for i, step in enumerate(job['steps'])]
        named = [step['name'] for step in steps if step['name'] is not None]
        require(len(set(named)) == len(named), 'duplicate named workflow step')
        for name in names:
            require('${{' not in name and name not in expected, 'duplicate or unresolved job name')
            expected[name] = steps
    require(len(expected) == 33, 'changed 33-job fleet contract requires explicit reviewer update')
    return expected


def source_workflow(head):
    require(re.fullmatch(r'[0-9a-f]{40}', head) is not None, 'head must be a full lowercase commit SHA')
    kind = subprocess.check_output(['git', 'cat-file', '-t', head], cwd=ROOT).strip()
    require(kind == b'commit', 'head must identify a known commit')
    return subprocess.check_output(['git', 'show', head + ':' + WORKFLOW], cwd=ROOT)


def review(run, workflow, *, head, run_id, pr=None):
    require(re.fullmatch(r'[0-9a-f]{40}', head) is not None, 'head must be a full lowercase commit SHA')
    require(type(run_id) is int and run_id > 0, 'run must be a positive integer')
    require(pr is None or (type(pr) is int and pr > 0), 'PR must be a positive integer')
    require(run['databaseId'] == run_id and run['headSha'] == head, 'wrong run or head')
    require(run['event'] == 'pull_request', 'wrong workflow event')
    require(run['url'] == f'https://github.com/{REPOSITORY}/actions/runs/{run_id}', 'wrong run URL')
    pr_verified = False
    if 'pullRequests' in run:
        numbers = [item['number'] for item in run['pullRequests']]
        require(len(numbers) == 1 and type(numbers[0]) is int, 'ambiguous or absent PR identity')
        if pr is not None:
            require(numbers == [pr], 'wrong PR identity')
        else:
            pr = numbers[0]
        pr_verified = True
    expected = inventory(workflow)
    observed = {}
    for job in run['jobs']:
        require(job['name'] in expected and job['name'] not in observed, 'unknown or duplicate fleet job')
        observed[job['name']] = job
    missing = sorted(set(expected) - set(observed))
    successful = []
    for name, job in observed.items():
        if job['status'] != 'completed' or job['conclusion'] != 'success':
            continue
        for required in expected[name]:
            matching = [step for step in job['steps'] if step['number'] == required['number']]
            require(len(matching) == 1, 'missing/duplicate workflow step ordinal')
            if required['name'] is not None:
                require(matching[0]['name'] == required['name'], 'wrong named workflow step')
            require(matching[0]['status'] == 'completed' and matching[0]['conclusion'] == 'success',
                    'declared workflow step did not succeed')
        require(all(step['status'] == 'completed' and step['conclusion'] in {'success', 'skipped'}
                    for step in job['steps']), 'unsuccessful extra workflow step')
        successful.append(name)
    if run['status'] == 'completed':
        require(not missing, 'completed run lacks declared fleet jobs')
    accepted = (run['status'] == 'completed' and run['conclusion'] == 'success'
                and len(successful) == len(expected))
    return dict(run=run_id, run_url=run['url'], head=head, pr=pr,
                pr_identity_verified_from_run_metadata=pr_verified,
                workflow_path=WORKFLOW, workflow_sha256=hashlib.sha256(workflow).hexdigest(),
                expected_jobs=sorted(expected), successful_jobs=sorted(successful),
                not_yet_created_jobs=missing,
                remaining_jobs=sorted(set(expected) - set(successful)),
                observed_jobs=[{key: job[key] for key in ('databaseId', 'name', 'status', 'conclusion', 'url')}
                               for job in observed.values()],
                accepted_complete_fleet_gate=accepted, accepted_integration=False,
                scope='Exact-head hosted fleet job gates and every declared workflow step only. '
                      'Requires all 33 jobs, including three aggregates. '
                      'No independent raw-law recount or integration acceptance. '
                      'PR identity is verified only when present in run metadata.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--head', required=True)
    parser.add_argument('--run', type=int, required=True)
    parser.add_argument('--pr', type=int)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    require(not args.report.exists(), 'report already exists')
    workflow = source_workflow(args.head)
    raw = args.run_metadata.read_bytes()
    result = review(json.loads(raw), workflow, head=args.head, run_id=args.run, pr=args.pr)
    result['run_metadata_sha256'] = hashlib.sha256(raw).hexdigest()
    result['reviewer_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with args.report.open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')


if __name__ == '__main__':
    main()
