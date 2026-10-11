#!/usr/bin/env python3
"""Review the entire pinned PR777 fleet, including dependency-delayed aggregates."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import yaml

ROOT = Path(__file__).resolve().parents[1]
HEAD = 'dbe8652117ed870e9fc945c8f6f42bfa0137d49d'
RUN = 38145873752
WORKFLOW = '.github/workflows/chemistry-audit.yml'


def inventory(workflow):
    jobs = yaml.safe_load(workflow)['jobs']
    expected = {}
    for job in jobs.values():
        matrix = job.get('strategy', {}).get('matrix')
        if matrix is None:
            names = [job['name']]
        else:
            assert set(matrix) == {'family'}, 'unsupported matrix shape'
            assert '${{ matrix.family }}' in job['name']
            assert isinstance(matrix['family'], list) and matrix['family']
            assert all(isinstance(value, str) for value in matrix['family'])
            names = [job['name'].replace('${{ matrix.family }}', value)
                     for value in matrix['family']]
        steps = [dict(number=i + 2, name=step.get('name'))
                 for i, step in enumerate(job['steps'])]
        names_of_steps = [step['name'] for step in steps if step['name'] is not None]
        assert len(set(names_of_steps)) == len(names_of_steps)
        for name in names:
            assert '${{' not in name and name not in expected
            expected[name] = steps
    return expected


def review(run, workflow):
    assert run['databaseId'] == RUN and run['headSha'] == HEAD
    assert run['event'] == 'pull_request'
    assert run['url'] == f'https://github.com/CrispStrobe/kerotakis/actions/runs/{RUN}'
    expected = inventory(workflow)
    assert len(expected) == 33, 'changed pinned fleet contract'
    observed = {}
    for job in run['jobs']:
        assert job['name'] in expected and job['name'] not in observed
        observed[job['name']] = job
    missing = sorted(set(expected) - set(observed))
    successful = []
    for name, job in observed.items():
        if job['status'] != 'completed' or job['conclusion'] != 'success':
            continue
        for required in expected[name]:
            matching = [step for step in job['steps'] if step['number'] == required['number']]
            assert len(matching) == 1, 'missing/duplicate workflow step ordinal'
            if required['name'] is not None:
                assert matching[0]['name'] == required['name'], 'wrong named workflow step'
            assert matching[0]['status'] == 'completed' and matching[0]['conclusion'] == 'success'
        assert all(step['status'] == 'completed' and step['conclusion'] in {'success', 'skipped'}
                   for step in job['steps'])
        successful.append(name)
    if run['status'] == 'completed':
        assert not missing, 'completed run lacks declared fleet jobs'
    accepted = (run['status'] == 'completed' and run['conclusion'] == 'success'
                and len(successful) == len(expected))
    return dict(run=RUN, run_url=run['url'], head=HEAD,
                workflow_sha256=hashlib.sha256(workflow).hexdigest(),
                expected_jobs=sorted(expected), successful_jobs=sorted(successful),
                not_yet_created_jobs=missing,
                remaining_jobs=sorted(set(expected) - set(successful)),
                observed_jobs=[{key: job[key] for key in ('databaseId', 'name', 'status', 'conclusion', 'url')}
                               for job in observed.values()],
                accepted_complete_fleet_gate=accepted, accepted_integration=False,
                scope='Exact-head hosted fleet job gates and every declared workflow step only. '
                      'Requires all 33 jobs, including three aggregates. '
                      'No independent raw-law recount or integration acceptance.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    assert not args.report.exists()
    workflow = subprocess.check_output(['git', 'show', HEAD + ':' + WORKFLOW], cwd=ROOT)
    result = review(json.loads(args.run_metadata.read_text()), workflow)
    result['run_metadata_sha256'] = hashlib.sha256(args.run_metadata.read_bytes()).hexdigest()
    result['reviewer_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with args.report.open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')


if __name__ == '__main__':
    main()
