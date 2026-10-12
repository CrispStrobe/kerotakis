#!/usr/bin/env python3
"""Capture unchanged-source contracts, including failures; capture is not acceptance."""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
AUDIT = HERE.parents[1]
HELPER = AUDIT / 'audits/registry-loader-applicability-20261012/hosted_original_baseline.py'
spec = importlib.util.spec_from_file_location('source_binding', HELPER)
b = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b)
b.SOURCE = 'a125c88d5e4e97cfc55eb2f51e0490e9cb0ed21b'
b.TREE = 'cbe987d1fba5eaeaecbd7a23232fb2105eee8684'
b.REGULAR_COUNT = 2253
FREEZE_SHA = '7618b5dcb4b5a1485ee67b3103408c6e9e0d5d77f3d0d44605a71602d74fbaee'
HELPER_SHA = '53b22c431429b091a2f2539039e970d822de7a965605e37677c7c5824bc557d4'


def observations(raw, expected_names=None, library_counts=None):
    log = b.ANSI.sub('', raw)
    headers = list(re.finditer(r'^\s*Running (?:tests/[^\n]+\.rs|unittests src/lib\.rs)\s+\([^\n]+\)', log, re.M))
    b.require(len(headers) == (1 if expected_names is not None else 2), 'wrong binary inventory')
    expected_binaries = ['yaml_mechanism_parser_contracts'] if expected_names is not None else ['kerotakis_core', 'kerotakis_safety']
    for header, binary in zip(headers, expected_binaries):
        b.require(re.search(r'/'+re.escape(binary)+r'-[0-9a-f]+\)', header.group()) is not None, 'wrong actual binary identity')
        if expected_names is not None:
            b.require('Running tests/yaml_mechanism_parser_contracts.rs' in header.group(), 'wrong contract header')
    running = list(re.finditer(r'^\s*running (\d+) tests\s*$', log, re.M))
    counts = [len(expected_names)] if expected_names is not None else library_counts
    b.require([int(marker[1]) for marker in running] == counts, 'wrong running count inventory')
    reports = []
    for index, marker in enumerate(running):
        # stderr may print the next binary header before buffered stdout results.
        # Actual running markers and complete summaries delimit stdout blocks.
        block = log[marker.end():running[index + 1].start() if index + 1 < len(running) else len(log)]
        names = re.findall(r'^test ([\w:]+) \.\.\. (ok|FAILED)$', block, re.M)
        b.require(len(names) == len({name for name, _ in names}), 'duplicate result identities')
        count = counts[index]
        b.require(len(names) == count, 'incomplete outcomes')
        if expected_names is not None:
            b.require({name for name, _ in names} == set(expected_names), 'wrong frozen test identities')
        passed = sum(outcome == 'ok' for _, outcome in names)
        failed = count - passed
        summaries = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', block)
        b.require(summaries == [('FAILED' if failed else 'ok', str(passed), str(failed), '0', '0', '0')], 'wrong unfiltered summary')
        reports.append(dict(binary=expected_binaries[index], tests=[dict(name=name, outcome=outcome) for name, outcome in names], passed=passed, failed=failed, ignored=0, filtered_out=0))
    b.require(len(re.findall(r'^test [\w:]+ \.\.\. (?:ok|FAILED)$', log, re.M)) == sum(len(r['tests']) for r in reports), 'extra outcomes')
    return reports


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    source = args.source.resolve(strict=True)
    out = args.out.resolve()
    b.require(__debug__ and not args.out.exists() and not args.out.is_symlink(), 'output must be new; optimized Python unsupported')
    b.require(not out.is_relative_to(source), 'output must be outside source')
    out.mkdir(parents=True, exist_ok=False)
    result = dict(source=b.SOURCE, source_tree=b.TREE, collection_complete=False, accepted_integration=False,
                  started_utc=datetime.now(timezone.utc).isoformat(), hosted_environment={key: os.environ.get(key) for key in ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_SHA', 'GITHUB_REPOSITORY']})
    try:
        revision = b.git(AUDIT, 'rev-parse', 'HEAD').decode().strip()
        b.require(revision == os.environ.get('GITHUB_SHA', revision), 'audit revision differs')
        result['audit_revision'] = revision
        for path in [Path(__file__).resolve(), HELPER]:
            relative = str(path.relative_to(AUDIT))
            b.require(b.git(AUDIT, 'show', revision + ':' + relative) == path.read_bytes(), 'collector differs from audit Git')
            (out / path.name).write_bytes(path.read_bytes())
        b.require(b.digest(HELPER.read_bytes()) == HELPER_SHA, 'source-binding helper changed')
        frozen = (HERE / 'freeze.json').read_bytes()
        freeze = json.loads(frozen)
        b.require(b.digest(frozen) == FREEZE_SHA, 'freeze changed')
        fixture = (AUDIT / freeze['fixture_path']).read_bytes()
        names = re.findall(rb'#\[test\]\s*fn (\w+)\(', fixture)
        b.require([name.decode() for name in names] == freeze['expected_tests'] and len(names) == len(set(names)) == fixture.count(b'#[test]') == 50, 'wrong frozen identities')
        b.require(b.digest(fixture) == freeze['fixture_sha256'], 'fixture changed')
        b.require(freeze['source'] == b.SOURCE and freeze['source_tree'] == b.TREE, 'wrong frozen source')
        destination = 'crates/kerotakis-core/tests/yaml_mechanism_parser_contracts.rs'
        b.require(freeze['injected_fixture_path'] == destination, 'wrong injection destination')
        (out / 'freeze.json').write_bytes(frozen)
        (out / 'yaml_mechanism_parser_contracts.rs').write_bytes(fixture)
        b.json_file(out / 'source-inventory.json', b.source_inventory(source))
        b.require(not (source / 'Cargo.lock').exists(), 'preexisting lock is not fresh hosted resolution')
        env = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_TERM_COLOR='never', RUST_BACKTRACE='0')
        for label, command in [('cargo-version', ['cargo', '--version', '--verbose']), ('rustc-version', ['rustc', '--version', '--verbose']), ('generate-lockfile', ['cargo', 'generate-lockfile'])]:
            b.require(b.run_command(out, label, command, source, env) == 0, label + ' failed')
        lock = (source / 'Cargo.lock').read_bytes()
        (out / 'Cargo.lock').write_bytes(lock)
        result['actual_lock_sha256'] = b.digest(lock)
        with (source / destination).open('xb') as stream:
            stream.write(fixture)
        commands = [('contracts', ['cargo', 'test', '--locked', '--no-fail-fast', '--color', 'never', '-p', 'kerotakis-core', '--test', 'yaml_mechanism_parser_contracts', '--', '--test-threads=1']), ('libraries', ['cargo', 'test', '--locked', '--no-fail-fast', '--color', 'never', '-p', 'kerotakis-core', '-p', 'kerotakis-safety', '--lib', '--', '--test-threads=1'])]
        for label, command in commands:
            code = b.run_command(out, label, command, source, env)
            parsed = observations((out / (label + '.log')).read_text(), freeze['expected_tests'] if label == 'contracts' else None, [604, 28] if label == 'libraries' else None)
            failed = sum(report['failed'] for report in parsed)
            b.require(code == (101 if failed else 0), 'exit disagrees with test outcomes')
            result[label] = parsed
            b.require((source / 'Cargo.lock').read_bytes() == lock, 'actual lock changed during execution')
        b.require((source / destination).read_bytes() == fixture, 'injected fixture changed')
        status = b.git(source, 'status', '--porcelain=v1', '--untracked-files=all', '--ignore-submodules=none').decode().splitlines()
        b.require(status == ['?? ' + destination], 'postexecution source changed beyond injection/lock')
        result['postexecution_source_status'] = status
        result['collection_complete'] = True
        result['contracts_passed'] = sum(r['failed'] for r in result['contracts']) == 0
        result['libraries_passed'] = sum(r['failed'] for r in result['libraries']) == 0
        result['scope'] = 'Successful collection preserves first failures; synthetic YAML admission contracts are not chemical accuracy, CLI, WASM or parser-migration acceptance. Cached dependencies and ignored submodule contents are qualified by source inventory.'
    except Exception as error:
        result['error'] = str(error)
        raise
    finally:
        result['finished_utc'] = datetime.now(timezone.utc).isoformat()
        b.json_file(out / 'result.json', result)
        b.json_file(out / 'artifact-manifest.json', {str(path.relative_to(out)): b.digest(path.read_bytes()) for path in sorted(out.rglob('*')) if path.is_file()})


if __name__ == '__main__':
    main()
