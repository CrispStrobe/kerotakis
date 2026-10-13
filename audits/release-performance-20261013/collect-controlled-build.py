#!/usr/bin/env python3
"""Collect exact controlled build bytes under resource gates; never execute ELF."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import time
import zipfile

HEAD = 'd49d9659030a1311442f3aaddf49dd6158731c7f'
ZIP_SHA = 'e121b7648165c121f8e9826ffac50f497cff46e884d505aa0a3ef08efabcaa16'
ZIP_SIZE = 19776062
LOCK_SHA = '5135e2cca3fa785a2d109409c0540eb1265cf7f6670637ab87e3d8af1b836a93'
SOURCES = {'baseline': ('2bf9c771975f24b9e5b933728ef440cbca9a8ec6',
                         '8613e46420a8f8d76f39732c5a50d3dcd8417811'),
           'candidate': ('d06ffe3a1740728656878c4aea0e9507ca83c808',
                          '25d1dd490acc080a59f2a02e5b9399a22bd0f4bc')}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1048576), b''):
            h.update(chunk)
    return h.hexdigest()


def gate(out):
    memory = int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines()
                      if line.startswith('MemAvailable:'))) // 1024
    value = dict(unix_time=time.time(), load1=os.getloadavg()[0], memory_mib=memory,
                 evidence_free_mib=shutil.disk_usage(out).free // 1048576)
    with (out / 'gates.jsonl').open('a') as stream:
        stream.write(json.dumps(value) + '\n')
    require(value['load1'] <= 3.5 and memory >= 3200 and value['evidence_free_mib'] >= 8192,
            'NAS artifact collection resource gate closed')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out
    require(not out.exists(), 'exclusive destination required')
    out.mkdir(parents=True)
    records = {}
    try:
        gate(out)
        flags = ['--allow-escape-sequences'] if '--allow-escape-sequences' in subprocess.check_output(
            ['gh', 'api', '--help']).decode() else []
        def api(name, endpoint, binary=False):
            gate(out)
            command = ['gh', 'api', *flags, 'repos/CrispStrobe/kerotakis/' + endpoint]
            with (out / name).open('wb') as stream, (out / (name + '.stderr')).open('wb') as error:
                subprocess.run(command, stdout=stream, stderr=error, check=True, timeout=180)
            records[name] = dict(argv=command, sha256=sha(out / name), bytes=(out / name).stat().st_size)
            return None if binary else json.loads((out / name).read_bytes())
        run = api('run.json', 'actions/runs/38396270148')
        job = api('job.json', 'actions/jobs/115292178103')
        artifact = api('artifact-api.json', 'actions/artifacts/11814954199')
        require(run['head_sha'] == HEAD and run['run_attempt'] == 1
                and run['event'] == 'workflow_dispatch' and run['conclusion'] == 'success', 'run differs')
        require(job['run_id'] == run['id'] and job['head_sha'] == HEAD
                and job['conclusion'] == 'success'
                and all(s['conclusion'] == 'success' for s in job['steps']), 'job differs')
        require(artifact['workflow_run']['id'] == run['id']
                and artifact['workflow_run']['head_sha'] == HEAD and not artifact['expired']
                and artifact['name'] == 'controlled-release-builds-38396270148-1'
                and artifact['size_in_bytes'] == ZIP_SIZE
                and artifact['digest'] == 'sha256:' + ZIP_SHA, 'artifact association differs')
        api('artifact.zip', 'actions/artifacts/11814954199/zip', True)
        require((out / 'artifact.zip').stat().st_size == ZIP_SIZE and sha(out / 'artifact.zip') == ZIP_SHA,
                'actual ZIP bytes differ')
        gate(out)
        extracted = out / 'extracted'
        extracted.mkdir()
        members = {}
        with zipfile.ZipFile(out / 'artifact.zip') as archive:
            infos = archive.infolist()
            require(len(infos) <= 256 and sum(i.file_size for i in infos) <= 134217728,
                    'artifact size/member budget exceeded')
            require(len({i.filename for i in infos}) == len(infos), 'duplicate member')
            for info in infos:
                name = PurePosixPath(info.filename)
                require(not name.is_absolute() and '..' not in name.parts
                        and not stat.S_ISLNK(info.external_attr >> 16), 'unsafe ZIP member')
                if info.is_dir():
                    continue
                require(stat.S_IFMT(info.external_attr >> 16) in (0, stat.S_IFREG), 'nonregular member')
                path = extracted / name
                path.parent.mkdir(parents=True, exist_ok=True)
                # Reading each member to EOF verifies its CRC, without holding an ELF in memory.
                with archive.open(info) as src, path.open('wb') as dst:
                    shutil.copyfileobj(src, dst, length=1048576)
                path.chmod(0o400)
                members[info.filename] = dict(bytes=path.stat().st_size, sha256=sha(path))
        load = lambda name: json.loads((extracted / name).read_bytes())
        identity = load('identity.json')
        require(identity['audit_head'] == HEAD and str(identity['run']) == '38396270148'
                and str(identity['attempt']) == '1', 'saved identity differs')
        pair = load('build-pair.json')
        require(pair['builds_complete'] is True and pair['observed_environments_equal'] is True
                and pair['candidate_executed'] is False, 'build-pair incomplete')
        binaries = {}
        for variant, (head, tree) in SOURCES.items():
            gate(out)
            receipt = load(variant + '/receipt.json')
            require(receipt == pair['variants'][variant], 'per-variant receipt differs')
            require(receipt['source_before'] == receipt['source_after']
                    and receipt['source_before']['head'] == head
                    and receipt['source_before']['tree'] == tree
                    and receipt['source_before']['lock_sha256'] == LOCK_SHA, 'source/lock guards differ')
            require(sha(extracted / (variant + '/Cargo.lock')) == LOCK_SHA, 'actual lock differs')
            binary = extracted / variant / 'kero'
            with binary.open('rb') as stream:
                require(stream.read(4) == b'\x7fELF', 'not ELF')
            require(sha(binary) == receipt['binary_sha256'], 'binary differs')
            require(sha(extracted / (variant + '/build.log')) == receipt['build_log_sha256'], 'build log differs')
            binaries[variant] = dict(head=head, tree=tree, binary_sha256=sha(binary), lock_sha256=LOCK_SHA)
        require(pair['variants']['baseline']['environment'] == pair['variants']['candidate']['environment'],
                'recorded environments differ')
        result = dict(structural_collection_pass=True, archive_members=members, variants=binaries,
                      collector_sha256=sha(Path(__file__)), effective_compiler_review_accepted=False,
                      independent_artifact_review_complete=False, candidate_executed=False,
                      scope='ZIP/ELF/lock/recorded-guard collection only; native effective flags, '
                            'source/workflow/routing/tool identities require independent review.')
        (out / 'collection.json').write_text(json.dumps(result, indent=2) + '\n')
    except Exception as error:
        (out / 'failure.json').write_text(json.dumps(dict(type=type(error).__name__, message=str(error),
                                  artifact_or_build_environment_accepted=False), indent=2) + '\n')
        raise
    finally:
        (out / 'commands.json').write_text(json.dumps(records, indent=2) + '\n')


if __name__ == '__main__':
    main()
