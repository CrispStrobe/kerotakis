#!/usr/bin/env python3
"""Bind the two remaining aggregate logs to separately accepted saved reports.

Requires a reviewed, digest-pinned completed remaining-family receipt. Does not
download archives, execute the CLI, or reexecute any family analyzer.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

HEAD = 'd06ffe3a1740728656878c4aea0e9507ca83c808'
TREE = '25d1dd490acc080a59f2a02e5b9399a22bd0f4bc'
TESTED = '4df5deab77506d372a8df45f5d517e2f070d7f5a'
BIN = 'b290b436f63dc026d3c9b1134cc6362eb501008a7a0c9ad46a4a0567378e9853'
LOCK = '5135e2cca3fa785a2d109409c0540eb1265cf7f6670637ab87e3d8af1b836a93'
RUN = 38387729183
ACID = '23eca87b0d200ecafd07294bc2ee5f72a79161dd9119044d633ff93a5f745e50'
RELEASE = '8ec124715750fbb0b316d7354f5b924ff9517ec6ff92ad7cdf59c879e0995c9c'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def reject(token):
    raise ValueError('nonfinite JSON token: ' + token)


def finite(value):
    if isinstance(value, float):
        assert math.isfinite(value), 'nonfinite numeric overflow'
    elif isinstance(value, dict):
        for item in value.values():
            finite(item)
    elif isinstance(value, list):
        for item in value:
            finite(item)


def load(path):
    value = json.loads(path.read_bytes(), parse_constant=reject)
    finite(value)
    return value


def write(path, value):
    with path.open('x') as stream:
        stream.write(json.dumps(value, indent=2, allow_nan=False) + '\n')


def main():
    if not __debug__:
        raise RuntimeError('optimized Python disables provenance assertions')
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'families', 'acid-report', 'saved-jobs', 'audit', 'out']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--accepted-receipt-sha256', required=True,
                        help='digest frozen AFTER separate family-receipt acceptance')
    args = parser.parse_args()
    assert re.fullmatch('[a-f0-9]{64}', args.accepted_receipt_sha256)
    assert not args.out.exists(), 'preserve first evidence; choose a new directory'
    args.out.mkdir()
    source, out = args.source.resolve(), args.out.resolve()
    source_pins = {}

    def gate(stage):
        memory = next(line.split()[1] for line in
                      Path('/proc/meminfo').read_text().splitlines()
                      if line.startswith('MemAvailable:'))
        values = dict(stage=stage, unix_time=time.time(), load1=os.getloadavg()[0],
                      available_memory_mib=int(memory) // 1024,
                      evidence_disk_free_mib=shutil.disk_usage(out).free // 1048576)
        with (out / 'resource-gates.jsonl').open('a') as stream:
            stream.write(json.dumps(values) + '\n')
        assert (values['load1'] <= 3.5 and values['available_memory_mib'] >= 3200
                and values['evidence_disk_free_mib'] >= 8192), values

    def git(*command):
        return subprocess.check_output(['git', *command], cwd=source)

    def pin(path):
        file = source / path
        assert file.read_bytes() == git('show', HEAD + ':' + path), path
        source_pins[path] = sha(file)
        return file

    try:
        gate('initial preparation')
        assert git('rev-parse', 'HEAD').decode().strip() == HEAD
        assert git('rev-parse', 'HEAD^{tree}').decode().strip() == TREE
        assert git('status', '--porcelain', '--untracked-files=all') == b''
        # Root Cargo.lock is ignored and absent from the pinned Git tree.
        # Its digest is bound by the separately accepted own-release receipt.
        receipt_path = args.families / 'review.json'
        assert sha(receipt_path) == args.accepted_receipt_sha256
        receipt = load(receipt_path)
        assert receipt['all_checks_pass'] is True and receipt['run'] == RUN
        assert receipt['candidate'] == dict(head=HEAD, tree=TREE,
                                           tested_checkout=TESTED,
                                           binary_sha256=BIN, lock_sha256=LOCK)
        assert receipt['families_complete'] == 17
        assert receipt['total_cases'] == 414 and receipt['total_passed_law_checks'] == 623
        assert receipt['newly_reviewed_cases'] == 390
        assert receipt['newly_reviewed_law_checks'] == 591
        assert receipt['accepted_partial_receipt_sha256'] == ACID
        assert receipt['accepted_own_release_receipt_sha256'] == RELEASE
        assert receipt['cli_executed'] is False
        partial = args.audit / 'audits/release-performance-20261013/candidate-fleet-v1-v3-partial-review.json'
        assert sha(partial) == ACID
        acid = load(partial)['accepted_families'][0]
        assert acid['family'] == 'acid-base-buffers'
        release = args.audit / 'audits/release-performance-20261013/candidate-artifact-independent-review-v2.json'
        assert sha(release) == RELEASE and load(release)['all_checks_pass'] is True
        for path, digest in receipt['source_sha256'].items():
            assert sha(pin(path)) == digest
        saved = args.saved_jobs
        run, artifacts = load(saved / 'run.json'), load(saved / 'artifacts.json')['artifacts']
        assert run['id'] == RUN and run['head_sha'] == HEAD
        assert run['head_commit']['tree_id'] == TREE
        assert run['run_attempt'] == 1 and run['conclusion'] == 'success'
        assert run['status'] == 'completed'
        rows = receipt['families']
        assert len({row['family'] for row in rows}) == 17
        assert sum(row['cases'] for row in rows) == 414
        assert sum(row['passed_law_checks'] for row in rows) == 623
        results = []
        for kind, job_id, count, downloads in [('v1', 115346519078, 10, 13),
                                               ('v3', 115351275849, 5, 5)]:
            gate('prepare ' + kind + ' accepted reports')
            report_dir = out / kind
            report_dir.mkdir()
            selected = [row for row in rows if row['generation'] == kind]
            assert len(selected) == count
            report_pins = {}
            for row in selected:
                family = row['family']
                if family == 'acid-base-buffers':
                    assert row == acid
                    report = args.acid_report
                else:
                    paths = list((args.families / (kind + '-' + family) / 'extracted')
                                 .rglob(family + '-law-checks.json'))
                    assert len(paths) == 1
                    report = paths[0]
                    recomputed = args.families / (kind + '-' + family) / 'source-owned-recomputed-law-checks.json'
                    assert sha(recomputed) == row['source_owned_recomputed_law_receipt_sha256']
                    assert load(recomputed) == load(report)
                assert sha(report) == row['original_law_receipt_sha256']
                data = load(report)
                assert data['cases'] == 24 and data['unmet'] == 0
                assert data['passed'] == len(data['checks']) == row['passed_law_checks']
                assert all(check['passed'] is True for check in data['checks'])
                if kind == 'v3':
                    assert data['commit'] == TESTED and data['binary_sha256'] == BIN
                target = report_dir / (family + '-law-checks.json')
                with target.open('xb') as stream:
                    stream.write(report.read_bytes())
                report_pins[family] = sha(report)
                manifest = ('tools/chemistry-audit/source-fleets' +
                            ('-3' if kind == 'v3' else '') + '/' + family + '.json')
                assert sha(pin(manifest)) == receipt['source_sha256'][manifest]
            helper = 'tools/chemistry-audit/aggregate_source_fleets' + ('_3' if kind == 'v3' else '') + '.py'
            runner = pin(helper)
            pin('tools/chemistry-audit/source_fleets' + ('_3' if kind == 'v3' else '') + '.py')
            gate('source-owned aggregation ' + kind)
            with (out / (kind + '.stdout')).open('xb') as stdout, (out / (kind + '.stderr')).open('xb') as stderr:
                completed = subprocess.run([sys.executable, '-B', str(runner), str(report_dir)],
                                           stdout=stdout, stderr=stderr, timeout=60,
                                           env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'))
            assert completed.returncode == 0
            assert (out / (kind + '.stderr')).stat().st_size == 0
            offline = load(out / (kind + '.stdout'))
            gate('actual aggregate log ' + kind)
            directory = saved / str(job_id)
            job = load(directory / 'job.json')
            assert job['id'] == job_id and job['run_id'] == RUN and job['head_sha'] == HEAD
            assert job['run_attempt'] == 1 and job['conclusion'] == 'success' and job['status'] == 'completed'
            assert all(step['conclusion'] == 'success' for step in job['steps'])
            text = re.sub(r'\x1b\[[0-9;]*m', '', (directory / 'job.log').read_text())
            text = '\n'.join(re.sub(r'^\ufeff?\d{4}-\S+Z ', '', line) for line in text.splitlines())
            assert re.search(r'git log -1 --format=%H\n' + TESTED + r'\n', text)
            actual_downloads = re.findall(r'^- (\S+) \(ID: (\d+), Size: (\d+), Expected Digest: sha256:([a-f0-9]{64})\)$', text, re.M)
            assert len(actual_downloads) == downloads
            assert len({int(row[1]) for row in actual_downloads}) == downloads
            artifact_names = set()
            for name, artifact_id, size, digest in actual_downloads:
                artifact = next(item for item in artifacts if item['id'] == int(artifact_id))
                assert artifact['name'] == name and artifact['size_in_bytes'] == int(size)
                assert artifact['digest'] == 'sha256:' + digest
                assert artifact['workflow_run']['id'] == RUN and artifact['workflow_run']['head_sha'] == HEAD
                assert 'SHA256 digest of downloaded artifact is ' + digest in text
                artifact_names.add(name)
                if name == 'chemistry-audit-cli-' + TESTED:
                    assert artifact['id'] == 11811008023
                    assert digest == '0bfa80773f3a00114855a3167bd074dd4cfb614c39c3ba36c687f490df054125'
                else:
                    accepted = next(row for row in rows if row['artifact_name'] == name)
                    assert accepted['artifact_id'] == int(artifact_id)
                    assert accepted['artifact_zip_sha256'] == digest
            expected_names = {row['artifact_name'] for row in selected}
            if kind == 'v1':
                expected_names |= {row['artifact_name'] for row in rows if row['generation'] == 'frozen'}
                expected_names.add('chemistry-audit-cli-' + TESTED)
            assert artifact_names == expected_names
            assert f'Total of {downloads} artifact(s) downloaded' in text
            assert 'python3 ' + helper + ' downloaded-evidence' + ('-v3' if kind == 'v3' else '') in text
            match = re.search(r'\{\n  "(?:families|schema)":.*?\n\}', text, re.S)
            assert match
            actual = json.loads(match.group(), parse_constant=reject)
            finite(actual)
            assert offline == actual
            expected = dict(families=count, cases=count * 24,
                            checks=338 if kind == 'v1' else 150, unmet_families=[])
            if kind == 'v3':
                expected.update(schema='kerotakis-source-aggregate-v3', relations=30,
                                commit=TESTED, binary_sha256=BIN)
            assert actual == expected
            results.append(dict(generation=kind, job=job_id, aggregate=actual,
                                downloaded_artifacts=downloads, accepted_report_sha256=report_pins,
                                job_api_sha256=sha(directory / 'job.json'),
                                job_log_sha256=sha(directory / 'job.log'),
                                aggregate_stdout_sha256=sha(out / (kind + '.stdout'))))
        gate('final source guard')
        for path, digest in source_pins.items():
            assert sha(source / path) == digest
        assert git('status', '--porcelain', '--untracked-files=all') == b''
        result = dict(schema_version=1, all_checks_pass=True, run=RUN, candidate=receipt['candidate'],
                      accepted_family_receipt_sha256=args.accepted_receipt_sha256,
                      accepted_acid_receipt_sha256=ACID, source_sha256=source_pins,
                      saved_run_api_sha256=sha(saved / 'run.json'),
                      saved_artifact_api_sha256=sha(saved / 'artifacts.json'),
                      aggregates=results, reviewer_sha256=sha(Path(__file__)),
                      cli_executed=False, family_analyzers_reexecuted=False,
                      limitations=['Requires separate acceptance of the digest-pinned family receipt; its existence alone does not certify its raw provenance.',
                                   'Exact source-owned aggregate algorithms reused: engineering consistency, not independent scientific calibration.',
                                   'V1 aggregate embeds no commit or binary; own checkout, accepted reports and all13 actual downloads provide the binding.',
                                   'No integration or performance approval; existing v2 aggregate acceptance is separate.'])
        write(out / 'review.json', result)
        print(json.dumps(dict(all_checks_pass=True, aggregate_jobs=[row['job'] for row in results],
                              receipt_sha256=sha(out / 'review.json'))), flush=True)
    except Exception as error:
        write(out / 'first-failure.json', dict(type=type(error).__name__, message=str(error),
                                             cli_executed=False))
        raise


if __name__ == '__main__':
    main()
