#!/usr/bin/env python3
"""Inspect the completed frozen review archive and close remaining aggregates.

Runs on a separately gated hosted worker. Never executes the app or a family
analyzer; source-owned aggregate scripts operate only on verified saved reports.
"""
import argparse
import collections
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import sys
import time
import zipfile

INPUT_RUN = 38450225092
INPUT_HEAD = 'c5c56f9cfa7381df63d7fa85a4ed64838d9eea79'
INPUT_JOB = 115478922130
INPUT_READER = '63481df0492e3dba7b4aa3c4f99ac8b929a0dfb3bf729c2f2f87d5f47f23653c'
OWN_RUN = 38387729183
HEAD = 'd06ffe3a1740728656878c4aea0e9507ca83c808'
TREE = '25d1dd490acc080a59f2a02e5b9399a22bd0f4bc'
TESTED = '4df5deab77506d372a8df45f5d517e2f070d7f5a'
BIN = 'b290b436f63dc026d3c9b1134cc6362eb501008a7a0c9ad46a4a0567378e9853'
LOCK = '5135e2cca3fa785a2d109409c0540eb1265cf7f6670637ab87e3d8af1b836a93'
ACID = '23eca87b0d200ecafd07294bc2ee5f72a79161dd9119044d633ff93a5f745e50'
RELEASE = '8ec124715750fbb0b316d7354f5b924ff9517ec6ff92ad7cdf59c879e0995c9c'
V2 = 'c1b1fbacd39fe73251d7982dea199810eea771077c80570378b17f0c36177883'
V2_AGGREGATE = 'e2c1110673b350dd81e1efec5c5d7eb4c542949df557264d4f835f20c04adaf1'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def reject(token):
    raise ValueError('nonfinite JSON token: ' + token)


def unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON key: ' + key)
        result[key] = value
    return result


def finite(value):
    if isinstance(value, float):
        assert math.isfinite(value), 'nonfinite numeric overflow'
    elif isinstance(value, dict):
        for item in value.values():
            finite(item)
    elif isinstance(value, list):
        for item in value:
            finite(item)


def parse(raw):
    value = json.loads(raw, parse_constant=reject, object_pairs_hook=unique)
    finite(value)
    return value


def load(path):
    return parse(path.read_bytes())


def write(path, value):
    with path.open('x') as stream:
        stream.write(json.dumps(value, indent=2, allow_nan=False) + '\n')


def main():
    if not __debug__:
        raise RuntimeError('optimized Python disables provenance assertions')
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    assert os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'audit', 'out']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.out.exists(), 'preserve earlier outputs; use a new directory'
    args.out.mkdir()
    out, source, audit = args.out.resolve(), args.source.resolve(), args.audit.resolve()

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

    def api(path, endpoint, raw=False):
        gate('API ' + path.name)
        with path.open('xb') as stdout, path.with_name(path.name + '.stderr').open('xb') as stderr:
            completed = subprocess.run(['gh', 'api', *api_flags, 'repos/CrispStrobe/kerotakis/' + endpoint],
                                       stdout=stdout, stderr=stderr, timeout=60)
        assert completed.returncode == 0, endpoint
        return None if raw else load(path)

    def extract(archive, destination, max_members, max_bytes):
        gate('extract ' + archive.name)
        destination.mkdir()
        with zipfile.ZipFile(archive) as bundle:
            infos = bundle.infolist()
            assert len(infos) <= max_members
            assert len(infos) == len({item.filename for item in infos})
            assert sum(item.file_size for item in infos) <= max_bytes
            assert bundle.testzip() is None
            for item in infos:
                path = PurePosixPath(item.filename)
                assert not path.is_absolute() and '..' not in path.parts
                assert '\\' not in item.filename
                assert not stat.S_ISLNK(item.external_attr >> 16)
                if item.is_dir():
                    continue
                assert stat.S_IFMT(item.external_attr >> 16) in [0, stat.S_IFREG]
                target = destination / path
                target.parent.mkdir(parents=True, exist_ok=True)
                with bundle.open(item) as src, target.open('xb') as dst:
                    shutil.copyfileobj(src, dst, 1048576)
                target.chmod(0o400)

    def pin(path, digest):
        file = source / path
        assert sha(file) == digest, path
        assert file.read_bytes() == subprocess.check_output(
            ['git', 'show', HEAD + ':' + path], cwd=source), path

    try:
        gate('initial source and frozen-method guards')
        api_help = subprocess.check_output(['gh', 'api', '--help']).decode()
        api_flags = ['--allow-escape-sequences'] if '--allow-escape-sequences' in api_help else []
        write(out / 'api-client.json', dict(flags=api_flags,
                                           scope='Preserve original log/archive bytes; output goes to evidence files.'))
        assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=audit).decode().strip() == os.environ['GITHUB_SHA']
        assert subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=audit) == b''
        frozen = load(audit / 'audits/release-performance-20261013/hosted-fleet-evidence-freeze.json')
        for path, digest in frozen['files_sha256'].items():
            assert sha(audit / path) == digest, path
        assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source).decode().strip() == HEAD
        assert subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=source).decode().strip() == TREE
        assert subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=source) == b''
        run = api(out / 'input-run.json', 'actions/runs/' + str(INPUT_RUN))
        assert run['id'] == INPUT_RUN and run['head_sha'] == INPUT_HEAD
        assert run['run_attempt'] == 1 and run['event'] == 'workflow_dispatch'
        assert run['status'] == 'completed' and run['conclusion'] == 'success'
        job = api(out / 'input-job.json', 'actions/jobs/' + str(INPUT_JOB))
        assert job['run_id'] == INPUT_RUN and job['head_sha'] == INPUT_HEAD
        assert job['run_attempt'] == 1 and job['status'] == 'completed' and job['conclusion'] == 'success'
        api(out / 'input-job.log', 'actions/jobs/' + str(INPUT_JOB) + '/logs', True)
        prefix = 'REMAINING_FLEET_REVIEW_JSON='
        markers = [line.split(prefix, 1)[1] for line in (out / 'input-job.log').read_text().splitlines()
                   if prefix in line and not line.startswith('##')]
        assert len(markers) == 1, 'exactly one actual review marker required'
        marker = parse(markers[0])
        assert marker['all_checks_pass'] is True and marker['reviewer_sha256'] == INPUT_READER
        artifact_list = api(out / 'input-artifacts.json', 'actions/runs/' + str(INPUT_RUN) + '/artifacts?per_page=100')
        assert artifact_list['total_count'] == len(artifact_list['artifacts']) == 1
        artifact = artifact_list['artifacts'][0]
        for key in ['id', 'name', 'digest', 'size_in_bytes']:
            assert artifact[key] == frozen['input_artifact'][key], key
        assert artifact['name'] == 'remaining-fleet-saved-review-' + str(INPUT_RUN) + '-1'
        assert artifact['workflow_run']['id'] == INPUT_RUN and artifact['workflow_run']['head_sha'] == INPUT_HEAD
        assert not artifact['expired'] and artifact['size_in_bytes'] <= 8388608
        api(out / 'input.zip', 'actions/artifacts/' + str(artifact['id']) + '/zip', True)
        assert sha(out / 'input.zip') == artifact['digest'].removeprefix('sha256:')
        assert (out / 'input.zip').stat().st_size == artifact['size_in_bytes']
        families = out / 'families'
        extract(out / 'input.zip', families, 4096, 134217728)
        receipt_path = families / 'review.json'
        assert not (families / 'first-failure.json').exists()
        receipt = load(receipt_path)
        assert all(receipt[key] == value for key, value in marker.items())
        assert receipt['all_checks_pass'] is True and receipt['run'] == OWN_RUN
        assert receipt['candidate'] == dict(head=HEAD, tree=TREE, tested_checkout=TESTED,
                                           binary_sha256=BIN, lock_sha256=LOCK)
        assert receipt['reviewer_sha256'] == INPUT_READER
        assert receipt['families_complete'] == 17 and len(receipt['families']) == 17
        assert receipt['total_cases'] == 414 and receipt['total_passed_law_checks'] == 623
        assert receipt['newly_reviewed_cases'] == 390 and receipt['newly_reviewed_law_checks'] == 591
        assert receipt['accepted_partial_receipt_sha256'] == ACID
        assert receipt['accepted_own_release_receipt_sha256'] == RELEASE
        assert receipt['cli_executed'] is False and receipt['scientific_calibration_accepted'] is False
        for path, digest in receipt['source_sha256'].items():
            pin(path, digest)
        root = audit / 'audits/release-performance-20261013'
        partial_path = root / 'candidate-fleet-v1-v3-partial-review.json'
        assert sha(partial_path) == ACID
        acid = load(partial_path)['accepted_families'][0]
        assert sha(root / 'candidate-artifact-independent-review-v2.json') == RELEASE
        saved = out / 'saved-jobs'
        saved.mkdir()
        own = api(saved / 'run.json', 'actions/runs/' + str(OWN_RUN))
        assert own['head_sha'] == HEAD and own['status'] == 'completed' and own['conclusion'] == 'success'
        own_artifact_list = api(saved / 'artifacts.json', 'actions/runs/' + str(OWN_RUN) + '/artifacts?per_page=100')
        assert own_artifact_list['total_count'] == len(own_artifact_list['artifacts'])
        own_artifacts = {item['id']: item for item in own_artifact_list['artifacts']}
        sys.path.insert(0, str(source / 'tools/chemistry-audit'))
        import sixth_batch
        frozen6 = {spec['id']: spec for spec in sixth_batch.recorder.CASES}
        import seventh_batch
        frozen7 = {spec['id']: spec for spec in seventh_batch.recorder.CASES}
        reviews = []
        seen = set()
        acid_report = None
        for row in receipt['families']:
            family, generation = row['family'], row['generation']
            assert family not in seen
            seen.add(family)
            gate('independent saved raw review ' + family)
            item = own_artifacts[row['artifact_id']]
            assert item['workflow_run']['id'] == OWN_RUN and item['workflow_run']['head_sha'] == HEAD
            assert item['name'] == row['artifact_name'] and item['digest'] == 'sha256:' + row['artifact_zip_sha256']
            assert row['independent_raw_provenance_contracts_pass'] is True and row['source_owned_laws_pass'] is True
            if family == 'acid-base-buffers':
                assert row == acid
                folder = out / 'accepted-acid'
                folder.mkdir()
                assert item['size_in_bytes'] < 200000 and not item['expired']
                api(folder / 'artifact.zip', 'actions/artifacts/' + str(item['id']) + '/zip', True)
                assert sha(folder / 'artifact.zip') == row['artifact_zip_sha256']
                assert (folder / 'artifact.zip').stat().st_size == item['size_in_bytes']
                extracted = folder / 'extracted'
                extract(folder / 'artifact.zip', extracted, 512, 33554432)
            else:
                folder = families / (generation + '-' + family)
                assert sha(folder / 'artifact.zip') == row['artifact_zip_sha256']
                assert (folder / 'artifact.zip').stat().st_size == item['size_in_bytes']
                assert sha(folder / 'artifact-api.json') == row['artifact_api_sha256']
                original_api = load(folder / 'artifact-api.json')
                for key in ['id', 'name', 'size_in_bytes', 'digest']:
                    assert original_api[key] == item[key]
                extracted = folder / 'extracted'
            files = {str(path.relative_to(extracted)): sha(path)
                     for path in extracted.rglob('*') if path.is_file()}
            assert files == row['member_sha256'] and len(files) == row['archive_members']
            summaries = list(extracted.rglob('summary.json'))
            assert len(summaries) == 1
            directory, summary = summaries[0].parent, load(summaries[0])
            assert summary['commit'] == TESTED and summary['binary_sha256'] == BIN
            if generation == 'frozen':
                specs = frozen6 if family == 'sixth' else frozen7
                assert summary['lock_sha256'] == LOCK
                assert (directory / 'source.patch').read_bytes() == b''
                assert load(directory / 'new-source-files.json') == {}
            else:
                manifest = source / ('tools/chemistry-audit/source-fleets' +
                                     ('-3' if generation == 'v3' else '') + '/' + family + '.json')
                specs = {spec['id']: spec for spec in load(manifest)['cases']}
            records = summary['cases']
            assert len(records) == len(specs) == row['cases']
            assert {record['id'] for record in records} == set(specs)
            assert len({record['id'] for record in records}) == len(records)
            for record in records:
                case = directory / record['id']
                for name, digest in row['raw_evidence_sha256'][record['id']].items():
                    assert sha(case / name) == digest
                script = specs[record['id']]['script']
                if generation != 'frozen':
                    script = 'register lv3\n' + script.strip() + '\ninspect\n'
                assert (case / 'experiment.lab').read_text() == '# ' + specs[record['id']]['question'] + '\n' + script
                assert record['question'] == specs[record['id']]['question']
                lines = [parse(line) for line in (case / 'stdout.ndjson').read_text().splitlines()]
                assert lines and all(isinstance(line, dict) for line in lines)
                events = [event for line in lines for event in line.get('events', [])]
                final = next(line['bench'] for line in reversed(lines) if 'bench' in line)
                assert record['exit'] == 0 and record['non_json_lines'] == []
                assert record['steps'] == len(lines) and record['final'] == final
                assert record['stderr'] == (case / 'stderr.txt').read_text()
                assert record['event_counts'] == dict(collections.Counter(event['event'] for event in events))
                assert record['diagnostics'] == [event for event in events if event['event'] in
                                                 ['not_yet_modeled', 'solver_failed', 'hazard_warning']]
                assert not any(event['event'] == 'solver_failed' for event in events)
            reports = list(directory.glob('*law-checks.json'))
            assert len(reports) == 1 and sha(reports[0]) == row['original_law_receipt_sha256']
            report = load(reports[0])
            assert report['unmet'] == 0 and report['passed'] == len(report['checks']) == row['passed_law_checks']
            assert all(check['passed'] is True for check in report['checks'])
            if family == 'acid-base-buffers':
                acid_report = reports[0]
            else:
                recomputed = folder / 'source-owned-recomputed-law-checks.json'
                assert sha(recomputed) == row['source_owned_recomputed_law_receipt_sha256']
                assert load(recomputed) == report
            reviews.append(dict(family=family, cases=len(records), checks=report['passed'],
                                raw_contracts_verified=True, original_report_sha256=sha(reports[0])))
        assert sum(item['cases'] for item in reviews) == 414
        assert sum(item['checks'] for item in reviews) == 623
        assert acid_report is not None
        write(out / 'raw-family-review.json', dict(all_checks_pass=True, families=reviews,
                                                  input_receipt_sha256=sha(receipt_path)))
        for job_id in [115346519078, 115351275849]:
            directory = saved / str(job_id)
            directory.mkdir()
            api(directory / 'job.json', 'actions/jobs/' + str(job_id))
            api(directory / 'job.log', 'actions/jobs/' + str(job_id) + '/logs', True)
        gate('source-owned saved aggregate verification')
        aggregate_out = out / 'aggregate-review'
        command = [sys.executable, '-B', str(root / 'review-remaining-aggregates.py'),
                   '--source', str(source), '--families', str(families),
                   '--acid-report', str(acid_report), '--saved-jobs', str(saved),
                   '--audit', str(audit), '--out', str(aggregate_out),
                   '--accepted-receipt-sha256', sha(receipt_path)]
        with (out / 'aggregate.stdout').open('xb') as stdout, (out / 'aggregate.stderr').open('xb') as stderr:
            completed = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=180,
                                       env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'))
        assert completed.returncode == 0 and (out / 'aggregate.stderr').stat().st_size == 0
        aggregates = load(aggregate_out / 'review.json')
        assert aggregates['all_checks_pass'] is True
        assert [item['job'] for item in aggregates['aggregates']] == [115346519078, 115351275849]
        assert sha(root / 'candidate-fleet-v2-saved-output-review.json') == V2
        assert sha(root / 'candidate-fleet-v2-aggregate-independent-review.json') == V2_AGGREGATE
        gate('final source and receipt guards')
        for path, digest in receipt['source_sha256'].items():
            pin(path, digest)
        assert subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=source) == b''
        result = dict(schema_version=1, all_checks_pass=True, own_fleet_run=OWN_RUN,
                      candidate=receipt['candidate'], input_review_run=INPUT_RUN,
                      input_review_head=INPUT_HEAD, input_reviewer_sha256=INPUT_READER,
                      input_artifact_id=artifact['id'], input_artifact_zip_sha256=sha(out / 'input.zip'),
                      input_receipt_sha256=sha(receipt_path), independently_checked_cases=414,
                      independently_checked_saved_law_checks=623,
                      prior_v2_cases=288, prior_v2_checks=368,
                      combined_cases=702, combined_checks=991,
                      accepted_v2_receipt_sha256=V2, accepted_v2_aggregate_sha256=V2_AGGREGATE,
                      remaining_aggregate_receipt_sha256=sha(aggregate_out / 'review.json'),
                      reviewer_sha256=sha(Path(__file__)), cli_executed=False,
                      gh_api_flags=api_flags,
                      family_analyzers_reexecuted=False, scientific_calibration_accepted=False,
                      limitations=['Raw evidence integrity, original forecasts and serialized report equality independently inspected on this worker; family analyzer algorithms not rerun.',
                                   'Source-owned laws and aggregate algorithms remain engineering contracts, not independent chemical calibration.',
                                   'Ignored root lock bytes are bound by the accepted own-release receipt, not by Git or frozen family archives.',
                                   'Accepted acid/base archive is fetched once on this worker for raw/report verification and aggregation, without app or analyzer execution.',
                                   'Previously accepted v2 raw/aggregate receipts are digest-pinned external prerequisites; their raw archives are not reread here.',
                                   'No integration approval, deployment or new timing claim.'])
        write(out / 'review.json', result)
        print('HOSTED_FLEET_CLOSURE_JSON=' + json.dumps(result, sort_keys=True), flush=True)
    except Exception as error:
        write(out / 'first-failure.json', dict(type=type(error).__name__, message=str(error),
                                             cli_executed=False))
        raise


if __name__ == '__main__':
    main()
