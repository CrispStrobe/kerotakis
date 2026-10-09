#!/usr/bin/env python3
"""Review frozen titration admission baseline evidence without executing the model."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FREEZE = 'audits/titration-trial-admission-20261009/freeze.json'
WORKFLOW = '.github/workflows/chemistry-audit.yml'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def outcomes(text, expected):
    rows = re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$', text, re.M)
    names = [name for name, _ in rows]
    assert len(names) == len(set(names)) == len(expected), 'Incomplete or duplicate outcomes'
    assert set(names) == set(expected), 'Unexpected function set'
    passed = [name for name, result in rows if result == 'ok']
    failed = [name for name, result in rows if result == 'FAILED']
    summaries = re.findall(
        r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; '
        r'(\d+) ignored; (\d+) measured; (\d+) filtered out;', text, re.M)
    assert summaries == [('FAILED' if failed else 'ok', str(len(passed)),
                          str(len(failed)), '0', '0', '0')], 'Filtered or mismatched summary'
    return passed, failed


def review(artifact, run_path, harness, freeze_path=FREEZE,
           job_name='Frozen titration admission baseline without production edits',
           prerequisite_failure=False):
    relative = PurePosixPath(freeze_path)
    assert not relative.is_absolute() and '..' not in relative.parts
    evidence = artifact / 'titration-baseline'
    archived = artifact / 'harness'
    freeze_bytes = git('show', f'{harness}:{freeze_path}')
    assert (archived / freeze_path).read_bytes() == freeze_bytes
    freeze = json.loads(freeze_bytes)
    frozen_path = freeze.get('frozen_manifest_path')
    if frozen_path:
        frozen_relative = PurePosixPath(frozen_path)
        assert not frozen_relative.is_absolute() and '..' not in frozen_relative.parts
        original = git('show', f'{harness}:{frozen_path}')
        assert (archived / frozen_path).read_bytes() == original
        assert sha(original) == freeze['frozen_manifest_sha256']
        original_freeze = json.loads(original)
        for key in ['fixture_path', 'fixture_sha256', 'intended_test_target',
                    'expected_test_functions', 'functions']:
            assert freeze[key] == original_freeze[key], 'Changed frozen expectation'
    fixture_path = freeze['fixture_path']
    fixture = git('show', f'{harness}:{fixture_path}')
    assert sha(fixture) == freeze['fixture_sha256']
    assert (archived / fixture_path).read_bytes() == fixture
    assert (evidence / 'fixture.rs').read_bytes() == fixture
    source = freeze['source_observed']
    tree = git('rev-parse', source + '^{tree}').decode().strip()
    assert tree == freeze['source_tree']
    assert (evidence / 'source-commit.txt').read_text().strip() == source
    assert (evidence / 'source-tree.txt').read_text().strip() == tree
    assert (evidence / 'harness-commit.txt').read_text().strip() == harness
    assert (evidence / 'status-before.txt').read_bytes() == b''
    assert (evidence / 'tracked-diff.txt').read_bytes() == b''
    target = freeze['intended_test_target']
    assert (evidence / 'status-with-fixture.txt').read_text() == f'?? {target}\n'
    assert not git('ls-tree', source, '--', target).strip(), 'Target already tracked'
    hashes = {}
    expected_paths = {WORKFLOW, freeze_path, fixture_path}
    if frozen_path:
        expected_paths.add(frozen_path)
    for line in (evidence / 'harness-sha256.txt').read_text().splitlines():
        recorded, path = line.split(maxsplit=1)
        assert path.startswith('harness/')
        path = path[len('harness/'):]
        assert path in expected_paths and path not in hashes
        assert recorded == sha(git('show', f'{harness}:{path}'))
        hashes[path] = recorded
    assert set(hashes) == expected_paths
    expected_links = {}
    for line in git('ls-tree', '-r', source).decode().splitlines():
        mode, _, commit, path = line.split()
        if mode == '160000':
            expected_links[path] = commit
    links = {}
    for line in (evidence / 'submodules.txt').read_text().splitlines():
        assert line.startswith(' '), 'Uninitialized or modified submodule'
        commit, path, *_ = line.split()
        assert path not in links
        links[path] = commit
    assert links == expected_links and len(links) == 2
    for name in ['rustc.txt', 'cargo.txt']:
        assert (evidence / name).read_text().strip(), name
    lock = sha((evidence / 'Cargo.lock').read_bytes())
    assert (evidence / 'lock-sha256.txt').read_text() == (
        f'{lock}  titration-baseline/Cargo.lock\n')
    controls_bytes = (evidence / 'controls.log').read_bytes()
    controls_text = controls_bytes.decode()
    compiler_errors = []
    if prerequisite_failure:
        assert not re.search(r'^test (\S+) \.\.\. ', controls_text, re.M)
        assert not re.search(r'^test result:', controls_text, re.M)
        compiler_errors = re.findall(r'^error\[(E\d+)\]: (.+)$', controls_text, re.M)
        assert compiler_errors, 'No Rust compiler diagnostic'
        target_name = Path(target).stem
        assert f'could not compile `kerotakis-core` (test "{target_name}")' in controls_text
        passed, failed = [], []
    else:
        passed, failed = outcomes(controls_text, freeze['functions'])
    assert len(freeze['functions']) == freeze['expected_test_functions'] > 0
    exit_code = int((evidence / 'controls-exit.txt').read_text())
    assert exit_code == (101 if failed or prerequisite_failure else 0), 'Outcome/exit disagreement'
    inherited_bytes = (evidence / 'inherited.log').read_bytes()
    inherited_rows = re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',
                                inherited_bytes.decode(), re.M)
    inherited_names = [name for name, _ in inherited_rows]
    assert len(inherited_names) == len(set(inherited_names)) == 604
    assert all(result == 'ok' for _, result in inherited_rows)
    outcomes(inherited_bytes.decode(), inherited_names)
    assert int((evidence / 'inherited-exit.txt').read_text()) == 0
    result = {
        'source': source, 'fixture_sha256': sha(fixture), 'exit_code': exit_code,
        'passed': passed, 'failed': failed, 'controls_sha256': sha(controls_bytes),
        'scope': 'Baseline named functions only; failed parameter loops may stop early. '
                 'Not chemistry or complete integration acceptance.',
    }
    assert json.loads((evidence / 'results.json').read_text()) == result
    run = json.loads(run_path.read_text())
    assert run['headSha'] == harness and run['event'] == 'workflow_dispatch'
    conclusion = 'failure' if prerequisite_failure else 'success'
    assert run['status'] == 'completed' and run['conclusion'] == conclusion
    jobs = [job for job in run['jobs'] if job['conclusion'] != 'skipped']
    assert len(jobs) == 1 and jobs[0]['conclusion'] == conclusion
    assert jobs[0]['name'] == job_name
    return {
        'run': run['databaseId'], 'run_url': run['url'], 'harness': harness,
        'source': source, 'source_tree': tree, 'fixture_sha256': sha(fixture),
        'freeze_path': freeze_path,
        'freeze_sha256': sha(freeze_bytes), 'generated_lock_sha256': lock,
        'submodules': links, 'harness_sha256': hashes,
        'passed': passed, 'failed': failed, 'inherited_passed': 604,
        'evidence_sha256': {p.name: sha(p.read_bytes())
                            for p in sorted(evidence.iterdir()) if p.is_file()},
        'run_metadata_sha256': sha(run_path.read_bytes()),
        'accepted_baseline_collection': not prerequisite_failure,
        'classification': 'compiler prerequisite failure' if prerequisite_failure else 'complete baseline collection',
        'compiler_errors': [{'code': code, 'message': message} for code, message in compiler_errors],
        'scope': 'Frozen orchestration baseline on unchanged tracked model plus a separately '
                 'bound injected test. Collection acceptance permits failed model functions; '
                 'not chemistry, repaired-source or integration acceptance. Declared parameter '
                 'subcases may stop early on function failure. Compiler prerequisite failure '
                 'verifies archive bindings only: zero executed control outcomes, no collection acceptance.',
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--harness', required=True)
    parser.add_argument('--freeze', default=FREEZE,
                        help='Repository-relative frozen manifest at the dispatched harness')
    parser.add_argument('--job-name',
                        default='Frozen titration admission baseline without production edits')
    parser.add_argument('--prerequisite-failure', action='store_true',
                        help='Verify a failed compiler/prerequisite archive without accepting behavioral outcomes')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    result = review(args.artifact, args.run_metadata, args.harness,
                    args.freeze, args.job_name, args.prerequisite_failure)
    with args.report.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'collection': 'accepted' if result['accepted_baseline_collection'] else 'not_executed',
                      'classification': result['classification'], 'passed': len(result['passed']),
                      'failed': len(result['failed']), 'inherited_passed': 604}))


if __name__ == '__main__':
    main()
