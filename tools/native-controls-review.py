#!/usr/bin/env python3
"""Verify native CI checkout identity and unfiltered fixture outcomes offline."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
REQUIRED_STEPS = (
    'Format', 'Clippy', 'Test', 'Codex — every claim must still compute',
    'Curiosity coverage — the recorded route is still the route',
)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def fixture_result(log, path, raw):
    names = re.findall(r'#\[test\]\s*fn\s+(\w+)\s*\(', raw.decode())
    assert names and len(names) == raw.count(b'#[test]'), 'Unsupported test declaration'
    assert len(names) == len(set(names)), 'Duplicate fixture function'
    headers = list(re.finditer(r'Running tests/' + re.escape(Path(path).stem) + r'\.rs\s+\(', log))
    assert len(headers) == 1, f'Expected exactly one execution: {path}'
    start = headers[0].end()
    ends = []
    for name in names:
        matches = list(re.finditer(r'\btest ' + re.escape(name) + r' \.\.\. (\w+)\b', log[start:]))
        assert len(matches) == 1 and matches[0].group(1) == 'ok', f'Missing, repeated or failed: {name}'
        ends.append(start + matches[0].end())
    # stderr may announce the next binary before this binary flushes its stdout.
    # Bind the summary after the last expected function, not the next Running header.
    summary = re.search(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', log[max(ends):])
    assert summary and summary.start() < 4096, f'Missing nearby result summary: {path}'
    assert summary.groups() == ('ok', str(len(names)), '0', '0', '0', '0'), f'Filtered or incomplete fixture: {path}'
    return {'path': path, 'fixture_sha256': hashlib.sha256(raw).hexdigest(), 'passed': names, 'failed': []}


def review(log_path, run_path, commit_path, head, job_id, fixtures, pr):
    run = json.loads(run_path.read_text())
    assert run['headSha'] == head and run['event'] == 'pull_request'
    jobs = [job for job in run['jobs'] if job['databaseId'] == job_id]
    assert len(jobs) == 1
    job = jobs[0]
    assert job['name'].startswith('Test (native) (')
    assert job['status'] == 'completed' and job['conclusion'] == 'success'
    steps = {step['name']: step for step in job['steps']}
    for name in REQUIRED_STEPS:
        assert steps[name]['status'] == 'completed' and steps[name]['conclusion'] == 'success', name
    log = ANSI.sub('', log_path.read_text())
    checkout = re.findall(r'git log -1 --format=%H\s*\n(?:\S+\s+)?([0-9a-f]{40})\s*\n', log)
    assert len(checkout) == 1, 'Ambiguous actual checkout identity'
    commit = json.loads(commit_path.read_text())
    assert commit['sha'] == checkout[0]
    tree = git('rev-parse', head + '^{tree}').decode().strip()
    assert commit['tree']['sha'] == tree, 'Tested checkout tree differs from head'
    assert len(fixtures) == len(set(fixtures)), 'Repeated fixture path'
    targets = [fixture_result(log, path, git('show', head + ':' + path)) for path in fixtures]
    return {
        'pr': pr, 'run': int(re.search(r'/runs/(\d+)/', job['url']).group(1)),
        'job': job_id, 'head': head, 'tested_checkout': checkout[0],
        'tested_tree': tree, 'head_tree_equals_tested_checkout': True,
        'job_url': job['url'], 'conclusion': job['conclusion'],
        'log_sha256': digest(log_path), 'targets': targets,
        'total_focused_passed': sum(len(target['passed']) for target in targets),
        'successful_steps': list(REQUIRED_STEPS),
        'scope': 'One native integration gate only. Other required gates and historical full-audit acceptance are separate.',
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--log', type=Path, required=True)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--checkout-metadata', type=Path, required=True)
    parser.add_argument('--head', required=True)
    parser.add_argument('--job', type=int, required=True)
    parser.add_argument('--pr', type=int, required=True)
    parser.add_argument('--fixture', action='append', default=[])
    parser.add_argument('--plan', type=Path)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    assert bool(args.fixture) != bool(args.plan), 'Select fixtures or a bound plan'
    fixtures = args.fixture
    if args.plan:
        plan = json.loads(args.plan.read_text())
        assert plan['head'] == args.head and plan['pr'] == args.pr
        fixtures = [target['path'] for target in plan['targets']]
        for target in plan['targets']:
            assert hashlib.sha256(git('show', args.head + ':' + target['path'])).hexdigest() == target['fixture_sha256']
    report = review(args.log, args.run_metadata, args.checkout_metadata, args.head, args.job, fixtures, args.pr)
    if args.plan:
        assert report['total_focused_passed'] == plan['expected_functions']
        assert [len(target['passed']) for target in report['targets']] == [target['functions'] for target in plan['targets']]
        report['plan_sha256'] = digest(args.plan)
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps({'head': report['head'], 'focused_passed': report['total_focused_passed']}))


if __name__ == '__main__':
    main()
