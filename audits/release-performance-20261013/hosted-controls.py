#!/usr/bin/env python3
"""Hosted correctness collection for the independently reviewed PR785 artifact."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import zipfile

HEAD = 'd06ffe3a1740728656878c4aea0e9507ca83c808'
TREE = '25d1dd490acc080a59f2a02e5b9399a22bd0f4bc'
ZIP_SHA = '0bfa80773f3a00114855a3167bd074dd4cfb614c39c3ba36c687f490df054125'
REVIEW_SHA = '8ec124715750fbb0b316d7354f5b924ff9517ec6ff92ad7cdf59c879e0995c9c'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    require(__debug__ and os.environ.get('GITHUB_ACTIONS') == 'true', 'hosted runner required')
    require(os.environ.get('GITHUB_REPOSITORY') == 'CrispStrobe/kerotakis', 'repository differs')
    workspace = Path(os.environ['GITHUB_WORKSPACE'])
    audit, source = workspace / 'audit', workspace / 'candidate'
    definitions = audit / 'audits/release-performance-20261013'
    out = workspace / 'hosted-controls-evidence'
    out.mkdir(exist_ok=False)
    def write(name, value):
        (out / name).write_text(json.dumps(value, indent=2) + '\n')
    try:
        actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=audit).decode().strip()
        require(actual == os.environ['GITHUB_SHA'], 'audit source differs')
        require(not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'],
                                           cwd=audit).strip(), 'audit source modified')
        write('identity.json', dict(audit_head=actual, run=os.environ['GITHUB_RUN_ID'],
              attempt=os.environ['GITHUB_RUN_ATTEMPT'], collector_sha256=sha(Path(__file__)),
              candidate_head=HEAD, candidate_tree=TREE, performance_measurements_started=False))
        # Import only the reviewed shared resource gate; invoke the real V2 entry point below.
        import importlib.util
        spec = importlib.util.spec_from_file_location('paired_resource_gate', definitions / 'paired.py')
        paired = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(paired)
        paired.gate(out, workspace)
        review_path = definitions / 'candidate-artifact-independent-review-v2.json'
        require(sha(review_path) == REVIEW_SHA, 'accepted artifact review bytes differ')
        review = json.loads(review_path.read_bytes())
        require(review['all_checks_pass'] is True and review['candidate']['head'] == HEAD
                and review['candidate']['tree'] == TREE, 'artifact review identity differs')
        help_text = subprocess.check_output(['gh', 'api', '--help']).decode()
        flags = ['--allow-escape-sequences'] if '--allow-escape-sequences' in help_text else []
        def api(name, endpoint, binary=False):
            paired.gate(out, workspace)
            argv = ['gh', 'api', *flags, 'repos/CrispStrobe/kerotakis/' + endpoint]
            with (out / name).open('wb') as stream:
                subprocess.run(argv, stdout=stream, check=True, timeout=180)
            return None if binary else json.loads((out / name).read_bytes())
        metadata = api('artifact-api.json', 'actions/artifacts/11811008023')
        job = api('artifact-job.json', 'actions/jobs/115275834062')
        require(metadata['workflow_run']['id'] == 38387729183
                and metadata['workflow_run']['head_sha'] == HEAD
                and metadata['name'] == 'chemistry-audit-cli-4df5deab77506d372a8df45f5d517e2f070d7f5a'
                and not metadata['expired'] and metadata['digest'] == 'sha256:' + ZIP_SHA,
                'own artifact association differs')
        require(job['run_id'] == 38387729183 and job['head_sha'] == HEAD
                and job['conclusion'] == 'success', 'own artifact build job differs')
        api('artifact.zip', 'actions/artifacts/11811008023/zip', True)
        require(sha(out / 'artifact.zip') == ZIP_SHA
                and (out / 'artifact.zip').stat().st_size == metadata['size_in_bytes'], 'ZIP differs')
        paired.gate(out, workspace)
        extracted = workspace / 'reviewed-cli-artifact'
        extracted.mkdir(exist_ok=False)
        expected = {item['path']: item for item in review['artifact_members']}
        with zipfile.ZipFile(out / 'artifact.zip') as archive:
            regular = [i for i in archive.infolist() if not i.is_dir()]
            require(len(regular) == 4 and {i.filename for i in regular} == set(expected), 'member inventory differs')
            for info in regular:
                name = PurePosixPath(info.filename)
                require(not name.is_absolute() and '..' not in name.parts
                        and not stat.S_ISLNK(info.external_attr >> 16), 'unsafe member')
                data = archive.read(info)
                require(len(data) == expected[info.filename]['bytes']
                        and hashlib.sha256(data).hexdigest() == expected[info.filename]['sha256'],
                        'actual artifact member differs')
                path = extracted / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
        binary, lock = extracted / 'target/release/kero', extracted / 'Cargo.lock'
        require(sha(binary) == review['candidate']['binary_sha256']
                and sha(lock) == review['candidate']['lock_sha256'], 'actual ELF or lock differs')
        binary.chmod(0o500)
        lock.chmod(0o400)
        require(os.stat(source).st_dev == os.stat(binary).st_dev == os.stat(workspace).st_dev,
                'source and binary must share the fast filesystem')
        write('bindings.json', dict(independent_artifact_review_passed=True,
              artifact_review_receipt_sha256=REVIEW_SHA, candidate=review['candidate']))
        argv = ['python3', str(definitions / 'protocol-v2/controls.py'),
                '--binary', str(binary), '--source', str(source), '--lock', str(lock),
                '--repository', str(source), '--fast-volume', str(workspace),
                '--bindings', str(out / 'bindings.json'), '--artifact-review', str(review_path),
                '--out', str(out / 'controls')]
        write('control-command.json', dict(argv=argv, cwd='audit', purpose='Seven frozen correctness controls only'))
        subprocess.run(argv, cwd=audit, check=True)
        result = json.loads((out / 'controls/controls.json').read_bytes())
        require(result['all_checks_pass'] is True and len(result['results']) == 7, 'controls incomplete')
        write('collection-summary.json', dict(controls_collected=7, all_checks_pass=True,
              controls_sha256=sha(out / 'controls/controls.json'), candidate=review['candidate'],
              independent_raw_control_review_complete=False, performance_measured=False))
        print('Seven frozen candidate controls passed; raw output review pending. No performance measurement.')
    except Exception as error:
        write('failure.json', dict(type=type(error).__name__, message=str(error), performance_measured=False))
        raise


if __name__ == '__main__':
    main()
