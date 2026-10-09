#!/usr/bin/env python3
"""Verify archived CLI executable, source and harness bindings without running it."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def review(artifact, run_path, harness, manifest_path, results_path):
    relative = PurePosixPath(manifest_path)
    assert not relative.is_absolute() and '..' not in relative.parts
    build = artifact / 'combined-build'
    archived_harness = artifact / 'harness'
    manifest_bytes = git('show', f'{harness}:{manifest_path}')
    assert (archived_harness / manifest_path).read_bytes() == manifest_bytes
    manifest = json.loads(manifest_bytes)
    source = manifest['compiled_source']
    tree = git('rev-parse', source + '^{tree}').decode().strip()
    assert tree == manifest['source_tree']
    assert (build / 'source-commit.txt').read_text().strip() == source
    assert (build / 'source-tree.txt').read_text().strip() == tree
    assert not (build / 'status-before.txt').read_text().strip()
    assert (build / 'build-exit-code.txt').read_text().strip() == '0'
    assert (build / 'harness-commit.txt').read_text().strip() == harness
    for name in ['rustc.txt', 'cargo.txt', 'build.log', 'build-time.txt']:
        assert (build / name).read_text().strip(), name
    binding = json.loads((build / 'binding.json').read_text())
    for key in ['compiled_source', 'source_tree', 'source_kind']:
        assert binding[key] == manifest[key], key
    assert digest(build / 'kero') == binding['binary_sha256']
    assert digest(build / 'Cargo.lock') == binding['generated_lock_sha256']
    expected_links = {}
    for line in git('ls-tree', '-r', source).decode().splitlines():
        parts = line.split()
        if parts[0] == '160000':
            expected_links[parts[3]] = parts[2]
    actual_links = {}
    for line in (build / 'submodules.txt').read_text().splitlines():
        assert line.startswith(' '), 'Uninitialized or modified submodule'
        sha, path, *_ = line.split()
        assert path not in actual_links
        actual_links[path] = sha
    assert actual_links == expected_links and actual_links
    expected_paths = {
        manifest_path, manifest['forecast_path'], 'tools/combined-cli-replay.py',
        'tools/fifth-independent-50/run.py',
        'tools/fifth-independent-50/review_profile.py',
        '.github/workflows/chemistry-audit.yml',
    }
    hashes = {}
    for line in (build / 'harness-sha256.txt').read_text().splitlines():
        recorded, path = line.split(maxsplit=1)
        prefix = '../harness/'
        assert path.startswith(prefix)
        path = path[len(prefix):]
        assert path in expected_paths and path not in hashes
        data = git('show', f'{harness}:{path}')
        assert hashlib.sha256(data).hexdigest() == recorded
        # The workflow itself is bound through Git; the other five inputs
        # must also match their uploaded copies.
        if path != '.github/workflows/chemistry-audit.yml':
            assert (archived_harness / path).read_bytes() == data
        hashes[path] = recorded
    assert set(hashes) == expected_paths
    assert hashes[manifest['forecast_path']] == manifest['forecast_sha256']
    run = json.loads(run_path.read_text())
    assert run['headSha'] == harness and run['event'] == 'workflow_dispatch'
    assert run['status'] == 'completed' and run['conclusion'] == 'success'
    jobs = [job for job in run['jobs'] if job['conclusion'] != 'skipped']
    assert len(jobs) == 1 and jobs[0]['conclusion'] == 'success'
    results = json.loads(results_path.read_text())
    assert results == json.loads((artifact / 'combined-cli-replay/results.json').read_text())
    for key in binding:
        assert results[key] == binding[key], key
    assert results['build_binding_sha256'] == digest(build / 'binding.json')
    assert results['forecast_sha256'] == manifest['forecast_sha256']
    assert results['processes'] == manifest['expected_processes']
    assert len(results['results']) == manifest['expected_cases']
    assert len(results['strict_stage_checks']) == 8
    assert all(value is True for value in results['strict_stage_checks'].values())
    accepted = {'passed', 'expected_refusal', 'qualified_agreement_with_model_notice'}
    assert all(row['outcome'] in accepted for row in results['results'])
    return {
        'run': run['databaseId'], 'run_url': run['url'], 'harness': harness,
        'source_binding': binding, 'submodules': actual_links,
        'harness_sha256': hashes,
        'build_evidence_sha256': {p.name: digest(p) for p in sorted(build.iterdir()) if p.is_file()},
        'run_metadata_sha256': digest(run_path),
        'results_review_sha256': digest(results_path),
        'processes': results['processes'], 'counts': results['counts'],
        'strict_stage_checks': results['strict_stage_checks'],
        'accepted_executable_bindings': True,
        'scope': 'Executable and archive binding review; requires a separately successful offline process re-evaluation. Not integration gates or historical full-audit acceptance.',
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--harness', required=True)
    parser.add_argument('--source-manifest', required=True)
    parser.add_argument('--results-review', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    result = review(args.artifact, args.run_metadata, args.harness,
                    args.source_manifest, args.results_review)
    with args.report.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'source': result['source_binding']['compiled_source'],
                      'processes': result['processes'], 'bindings': 'accepted'}))


if __name__ == '__main__':
    main()
