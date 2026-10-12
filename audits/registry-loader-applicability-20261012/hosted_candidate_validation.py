#!/usr/bin/env python3
"""Collect source-bound paired applicability evidence for the fixed candidate."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import hosted_original_baseline as baseline

HEAD = '7cce5b8f861bbe3c5fabdffb5eb8c0ca4acae06f'
TREE = 'cbe987d1fba5eaeaecbd7a23232fb2105eee8684'
PREVIOUS = '62c70f634213bbad68a2ef3d99971625f9f0a2db'
BASE = baseline.SOURCE
require = baseline.require
digest = baseline.digest
write_json = baseline.json_file


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    require(__debug__, 'optimized Python is not supported')
    source = args.source.resolve(strict=True)
    out = args.out.resolve()
    require(not args.out.exists() and not args.out.is_symlink(), 'output already exists')
    require(not out.is_relative_to(source), 'evidence must be outside source')
    out.mkdir(parents=True, exist_ok=False)
    result = dict(source=HEAD, source_tree=TREE, base=BASE, collection_complete=False,
                  paired_contracts_passed=False, accepted_integration=False,
                  started_utc=datetime.now(timezone.utc).isoformat(),
                  hosted_environment={k: os.environ.get(k) for k in
                      ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_SHA', 'GITHUB_REPOSITORY']})
    try:
        inputs = baseline.frozen_inputs()
        audit_revision = baseline.git(baseline.AUDIT, 'rev-parse', 'HEAD').decode().strip()
        result['audit_revision'] = audit_revision
        if os.environ.get('GITHUB_SHA'):
            require(audit_revision == os.environ['GITHUB_SHA'], 'audit revision mismatch')
        for filename in [Path(__file__), Path(baseline.__file__)]:
            relative = str(filename.resolve().relative_to(baseline.AUDIT))
            require(baseline.git(baseline.AUDIT, 'show', audit_revision + ':' + relative)
                    == filename.read_bytes(), 'collector differs from audit revision')
        require(baseline.git(source, 'show', '-s', '--format=%P', 'HEAD').decode().strip() == PREVIOUS
                and baseline.git(source, 'show', '-s', '--format=%P', PREVIOUS).decode().strip() == BASE,
                'candidate must have exact original repair and baseline ancestry')
        changed = baseline.git(source, 'diff', '--name-only', BASE, HEAD).decode().splitlines()
        expected = {'crates/kerotakis-data/src/runtime_scalar_records.rs'} | {i['destination'] for i in inputs}
        require(set(changed) == expected, 'unexpected candidate changes')
        baseline.SOURCE, baseline.TREE, baseline.REGULAR_COUNT = HEAD, TREE, 2253
        inventory = baseline.source_inventory(source)
        baseline.SOURCE, baseline.TREE, baseline.REGULAR_COUNT = BASE, '22ab9d3ad14b824274c6c4d7c00e780808fc857f', 2251
        write_json(out / 'source-inventory.json', inventory)
        hashes = {row['path']: row['sha256'] for row in inventory['regular_files']}
        require(hashes['crates/kerotakis-data/src/runtime_scalar_records.rs'] ==
                'c1a6f6c67bfd2d469698f223eae71dddc07db3a2a3956dedd939752939c59cbb', 'wrong repair bytes')
        ports = []
        for item in inputs:
            original = item['fixture']
            if Path(item['destination']).stem == 'registry_loader_applicability_contracts':
                ported = original
                for expression in ['parse_document(doc)', 'parse_document(&unpacked)']:
                    old = (expression + '\n            .err()\n            .expect(').encode()
                    require(ported.count(old) == 1, 'ambiguous runtime assertion normalization')
                    ported = ported.replace(old, (expression + '\n            .expect_err(').encode())
            else:
                old = b'parse_document(doc)\n        .err()\n        .expect('
                require(original.count(old) == 1, 'ambiguous standalone assertion normalization')
                ported = original.replace(old, b'parse_document(doc)\n        .expect_err(')
            require(baseline.git(source, 'show', HEAD + ':' + item['destination']) == ported,
                    'production port differs beyond declared assertion spelling')
            require(hashes[item['destination']] == digest(ported), 'production port SHA differs')
            ports.append(dict(path=item['destination'], original_sha256=item['fixture_sha256'],
                              ported_sha256=digest(ported)))
            item['fixture_sha256'] = digest(ported)
            (out / item['freeze_name']).write_bytes((baseline.HERE / item['freeze_name']).read_bytes())
        result['assertion_spelling_ports'] = ports
        result['source_inventory_sha256'] = digest((out / 'source-inventory.json').read_bytes())
        require(not (source / 'Cargo.lock').exists(), 'pre-existing lock is not fresh hosted evidence')
        env = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_TERM_COLOR='never', RUST_BACKTRACE='0')
        for label, command in [('cargo-version', ['cargo', '--version', '--verbose']),
                               ('rustc-version', ['rustc', '--version', '--verbose']),
                               ('generate-lockfile', ['cargo', 'generate-lockfile'])]:
            require(baseline.run_command(out, label, command, source, env) == 0, label + ' failed')
        lock = (source / 'Cargo.lock').read_bytes()
        (out / 'Cargo.lock').write_bytes(lock)
        result['hosted_lock_sha256'] = digest(lock)
        require(baseline.run_command(out, 'fetch-locked', ['cargo', 'fetch', '--locked'], source, env) == 0,
                'locked dependency fetch failed')
        command = ['cargo', 'test', '--locked', '--no-fail-fast', '--color', 'never', '-p', 'kerotakis-core']
        for item in inputs:
            command += ['--test', Path(item['destination']).stem]
        code = baseline.run_command(out, 'candidate-controls', command, source, env)
        observations = baseline.test_observations((out / 'candidate-controls.log').read_text(errors='replace'), inputs)
        result['rust_targets'] = observations
        result['rust_test_exit'] = code
        require(code == 0 and all(t['failed'] == 0 for t in observations), 'candidate Rust contracts failed')
        # Invalidate only the build-script timestamp in this ephemeral checkout.
        # Bytes stay committed; the artifact report must show an actual rebuild.
        os.utime(source / 'crates/kerotakis-core/build.rs', None)
        require(baseline.run_command(out, 'generator-build',
            ['cargo', 'build', '--locked', '-p', 'kerotakis-core', '--message-format=json', '-vv'], source, env) == 0,
            'generator build failed')
        messages = []
        for line in (out / 'generator-build.log').read_text(errors='replace').splitlines():
            if line.startswith('{'):
                try:
                    value = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if value.get('reason') == 'compiler-artifact':
                    messages.append(value)
        scripts = [m for m in messages if m['target']['kind'] == ['custom-build']
                   and Path(m['target']['src_path']).resolve() == source / 'crates/kerotakis-core/build.rs']
        require(len(scripts) == 1 and scripts[0]['fresh'] is False, 'no uniquely observed new generator compilation')
        filenames = [Path(p) for p in scripts[0]['filenames'] if Path(p).name == 'build-script-build']
        if not filenames and scripts[0].get('executable'):
            filenames = [Path(scripts[0]['executable'])]
        require(len(filenames) == 1, 'ambiguous generator artifact')
        binary = filenames[0].resolve(strict=True)
        require(binary.is_relative_to(source / 'target'), 'unexpected generator artifact location')
        binary_sha = digest(binary.read_bytes())
        require(binary_sha != 'e9ae23f79b1b31231cba7e70c44681c1e302987acc5453d9c245ca030724a32c', 'old baseline artifact substituted')
        (out / 'generator-build-script').write_bytes(binary.read_bytes())
        dependencies = list(binary.parent.glob('*.d'))
        require(len(dependencies) == 1, 'ambiguous build-script dependency file')
        (out / 'generator-build-script.d').write_bytes(dependencies[0].read_bytes())
        result['generator_artifact'] = dict(binary_sha256=binary_sha,
            dependency_file_sha256=digest(dependencies[0].read_bytes()), compiler_artifact=scripts[0],
            scope='Actual new build-script compilation observed after timestamp-only invalidation; other cached dependency artifacts are not independently rebuilt.')
        generator = baseline.HERE / 'fixtures/generated_applicability_contracts.py'
        frozen = json.loads((baseline.HERE / 'generated-freeze.json').read_text())
        require(digest(generator.read_bytes()) == frozen['fixture_sha256'], 'generator fixture changed')
        command = [sys.executable, str(generator), '--binary', str(binary),
                   '--binary-sha256', binary_sha, '--registry', str(source / 'data/registry/registry-source-v1.json'),
                   '--registry-sha256', frozen['registry_sha256'], '--out', str(out / 'generated-candidate')]
        code = baseline.run_command(out, 'generator-controls', command, source, env)
        generated = json.loads((out / 'generated-candidate/results.json').read_text())
        result['generator_total'] = generated['total']
        result['generator_passed'] = generated['passed']
        result['generator_failed'] = generated['failed']
        require(code == 0 and generated['total'] == generated['passed'] == 24, 'candidate generator contracts failed')
        require((source / 'Cargo.lock').read_bytes() == lock, 'candidate lock changed')
        require(not baseline.git(source, 'status', '--porcelain=v1', '--untracked-files=all', '--ignore-submodules=none'),
                'source changed during candidate execution')
        result.update(collection_complete=True, paired_contracts_passed=True, actual_rust_count=26,
                      actual_generator_count=24, total_frozen_controls=50,
                      completed_utc=datetime.now(timezone.utc).isoformat())
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        result['collection_error'] = type(error).__name__ + ': ' + str(error)
    result['artifact_sha256'] = {p.name: digest(p.read_bytes()) for p in sorted(out.iterdir()) if p.is_file()}
    write_json(out / 'result.json', result)
    print(json.dumps({k: result.get(k) for k in ['collection_complete', 'paired_contracts_passed', 'collection_error']}))
    return 0 if result['collection_complete'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
