#!/usr/bin/env python3
"""Recheck archived paired Rust diagnostics without executing the application."""
import argparse
import hashlib
import json
import pathlib
import re
import subprocess


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=pathlib.Path, required=True)
    parser.add_argument('--lane', choices=['inventory', 'thermal', 'snapshot', 'stack', 'boundary', 'nuclide', 'electrode', 'ordinary', 'transactions'], required=True)
    parser.add_argument('--harness', required=True)
    parser.add_argument('--manifest', help='Separately frozen repository-relative paired manifest')
    parser.add_argument('--report', type=pathlib.Path, required=True)
    args = parser.parse_args()
    root = args.artifact / 'delta-evidence'
    directory, fixture = {
        'inventory': ('delta-inventory-contracts', 'delta_inventory_contracts'),
        'thermal': ('delta-thermal-contracts', 'delta_thermal_contracts'),
        'snapshot': ('solver-snapshot-contracts', 'solver_snapshot_contracts'),
        'stack': ('solver-stack-atomic-contracts', 'solver_stack_atomic_contracts'),
        'boundary': ('solver-boundary-contracts', 'solver_boundary_contracts'),
        'nuclide': ('nuclide-inventory-serde', 'nuclide_inventory_serde_contracts'),
        'electrode': ('solver-electrode-schema', 'solver_electrode_schema_contracts'),
        'ordinary': ('ordinary-delta-state-contracts', 'ordinary_delta_state_contracts'),
        'transactions': ('historical-transaction-amount-controls', 'solver_transactions'),
    }[args.lane]
    manifest_path = args.manifest or f'audits/{directory}-20261008/paired-run.json'
    assert not pathlib.PurePosixPath(manifest_path).is_absolute()
    assert '..' not in pathlib.PurePosixPath(manifest_path).parts
    fixture = f'crates/kerotakis-core/tests/{fixture}.rs'
    manifest_bytes = (args.artifact / manifest_path).read_bytes()
    assert manifest_bytes == git('show', f'{args.harness}:{manifest_path}')
    manifest = json.loads(manifest_bytes)
    assert (root / 'harness/commit.txt').read_text().strip() == args.harness
    for line in (root / 'harness/sha256.txt').read_text().splitlines():
        expected, path = line.split(maxsplit=1)
        assert digest(git('show', f'{args.harness}:{path}')) == expected, path
    original = json.loads((root / 'results.json').read_bytes())
    report = {'lane': args.lane, 'harness': args.harness,
              'manifest_sha256': digest(manifest_bytes), 'sources': []}
    locks = []
    for label in ['baseline', 'repair']:
        folder = root / label
        source = manifest[label]
        assert (folder / 'source-commit.txt').read_text().strip() == source
        assert (folder / 'source-tree.txt').read_text().strip() == git('rev-parse', source + '^{tree}').decode().strip()
        assert not (folder / 'status-before.txt').read_text().strip()
        assert digest(git('show', f'{source}:{fixture}')) == manifest['fixture_sha256']
        assert (folder / 'fixture-sha256.txt').read_text().split()[0] == manifest['fixture_sha256']
        lock = digest((folder / 'Cargo.lock').read_bytes())
        assert (folder / 'generated-lock-sha256.txt').read_text().split()[0] == lock
        locks.append(lock)
        log = (folder / 'controls.log').read_text()
        outcomes = re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$', log, re.M)
        assert len(outcomes) == manifest['expected_test_functions']
        assert len({name for name, _ in outcomes}) == len(outcomes)
        if args.lane == 'transactions':
            expected_names = re.findall(rb'#\[test\]\s*fn\s+(\w+)\s*\(', git('show', f'{source}:{fixture}'))
            assert {name for name, _ in outcomes} == {name.decode() for name in expected_names}
            assert re.search(r'test result: .* 0 ignored; 0 measured; 0 filtered out;', log)
        passed = sorted(name for name, result in outcomes if result == 'ok')
        failed = sorted(name for name, result in outcomes if result == 'FAILED')
        code = int((folder / 'controls-exit.txt').read_text())
        assert (code == 0 and not failed) if label == 'repair' else (code != 0 and failed)
        assert re.search(rf'test result: .* {len(passed)} passed; {len(failed)} failed;', log)
        inherited = (folder / 'inherited.log').read_text()
        assert int((folder / 'inherited-exit.txt').read_text()) == 0
        assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', inherited)
        if args.lane == 'transactions':
            count = manifest['expected_inherited_test_functions']
            assert re.search(rf'test result: ok\. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;', inherited)
        recorded = next(row for row in original if row['label'] == label)
        assert sorted(recorded['passed']) == passed and sorted(recorded['failed']) == failed
        assert recorded['source'] == source and recorded['exit_code'] == code
        assert recorded['controls_sha256'] == digest((folder / 'controls.log').read_bytes())
        report['sources'].append({'label': label, 'source': source,
                                  'passed': passed, 'failed': failed,
                                  'evidence_sha256': {p.name: digest(p.read_bytes()) for p in sorted(folder.iterdir()) if p.is_file()}})
    assert locks[0] == locks[1]
    report['generated_lock_sha256'] = locks[0]
    report['accepted_bounded_diagnostic'] = True
    with args.report.open('x') as output:
        json.dump(report, output, indent=2)
        output.write('\n')
    print(json.dumps({row['label']: {'passed': len(row['passed']), 'failed': len(row['failed'])} for row in report['sources']}))


if __name__ == '__main__':
    main()
