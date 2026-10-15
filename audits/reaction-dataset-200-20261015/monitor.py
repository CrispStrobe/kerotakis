"""Collect this already-dispatched batch; never dispatch or execute chemistry.

Usage: monitor.py NEW_EVIDENCE_DIRECTORY
Polls up to 480 times at 30-second intervals, preserving failures and partial artifacts.
"""
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
import zipfile

RUN = 38763662200
HEAD = 'f4da8338fea4d00ac08c26117e783b699087cda4'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    output = Path(sys.argv[1]).resolve()
    output.mkdir()
    source = Path(__file__).resolve().parent
    frozen = output / 'ingestion-inputs'
    frozen.mkdir()
    for name in ['execution-binding.json', 'assess_execution.py', 'protocol-review.json']:
        shutil.copyfile(source / name, frozen / name)
    shutil.copytree(source / 'design', frozen / 'design')
    binding = json.loads((frozen / 'execution-binding.json').read_text())

    def state(status, **fields):
        result = {'status': status, 'run_id': RUN, 'audit_head': HEAD,
                  'updated_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), **fields}
        (output / 'state.json').write_text(json.dumps(result, indent=2) + '\n')

    def api(endpoint, destination):
        with destination.open('wb') as stream:
            subprocess.run(['gh', 'api', 'repos/CrispStrobe/kerotakis' + endpoint],
                           stdout=stream, check=True, timeout=120)

    try:
        expected = {}
        for key, path in [('producer_sha256', 'audits/reaction-dataset-200-20261015/hosted.py'),
                          ('caller_sha256', '.github/workflows/chemistry-audit.yml'),
                          ('callee_sha256', '.github/workflows/reaction-dataset-200.yml')]:
            data = subprocess.check_output(['git', 'show', HEAD + ':' + path])
            expected[key] = hashlib.sha256(data).hexdigest()
        (output / 'collector-binding.json').write_text(json.dumps({
            'monitor_sha256': sha(Path(__file__)),
            'reader_sha256': sha(frozen / 'assess_execution.py'),
            'execution_binding_sha256': sha(frozen / 'execution-binding.json'),
            'expected_routing_hashes': expected}, indent=2) + '\n')
        state('watching')
        for _ in range(480):
            api(f'/actions/runs/{RUN}', output / 'run.json')
            run = json.loads((output / 'run.json').read_text())
            assert run['head_sha'] == HEAD and run['event'] == 'workflow_dispatch'
            if run['status'] != 'completed':
                state('waiting_for_hosted_run', workflow_status=run['status'])
                time.sleep(30)
                continue
            api(f'/actions/runs/{RUN}/artifacts', output / 'artifacts.json')
            candidates = [artifact for artifact in json.loads((output / 'artifacts.json').read_text())['artifacts']
                          if artifact['name'] == f"reaction-dataset-200-{RUN}-{run['run_attempt']}"]
            if not candidates:
                state('terminal_run_without_artifact', conclusion=run['conclusion'])
                return
            assert len(candidates) == 1
            api(f"/actions/artifacts/{candidates[0]['id']}", output / 'artifact.json')
            artifact = json.loads((output / 'artifact.json').read_text())
            assert artifact['workflow_run']['id'] == RUN and not artifact['expired']
            assert artifact['size_in_bytes'] < 150_000_000
            archive = output / 'artifact.zip'
            api(f"/actions/artifacts/{artifact['id']}/zip", archive)
            assert 'sha256:' + sha(archive) == artifact['digest']
            with zipfile.ZipFile(archive) as zipped:
                members = zipped.infolist()
                assert len(members) < 2000 and len({member.filename for member in members}) == len(members)
                assert sum(member.file_size for member in members) < 500_000_000
                for member in members:
                    path = Path(member.filename)
                    assert not path.is_absolute() and '..' not in path.parts
                    assert member.external_attr >> 16 & 0o170000 != 0o120000
                zipped.extractall(output / 'verified-members')
            verified = output / 'verified-members'
            routing = json.loads((verified / 'routing.json').read_text())
            assert routing['audit_head'] == HEAD and str(routing['run_id']) == str(RUN)
            assert routing['binding_sha256'] == sha(frozen / 'execution-binding.json')
            for key, digest in expected.items():
                assert routing[key] == digest, key
            assert sha(verified / 'source-artifact.zip') == binding['artifact_sha256']
            assert sha(verified / 'source-artifact/kero') == binding['binary_sha256']
            assert sha(verified / 'source-artifact/Cargo.lock') == binding['cargo_lock_sha256']
            if not (verified / 'executions/completed.json').is_file():
                state('partial_execution_preserved', conclusion=run['conclusion'], artifact_id=artifact['id'])
                return
            with (output / 'assessment.log').open('wb') as log:
                subprocess.run([sys.executable, str(frozen / 'assess_execution.py'),
                                str(verified / 'executions'), str(output / 'report')],
                               stdout=log, stderr=subprocess.STDOUT, check=True, timeout=120)
            report = json.loads((output / 'report/execution-assessment.json').read_text())
            state('complete', executed_count=report['attempted_reaction_records'],
                  execution_counts=report['execution_counts'], artifact_id=artifact['id'],
                  artifact_sha256=sha(archive), physical_verdict='not_assigned')
            print((output / 'state.json').read_text(), flush=True)
            return
        state('monitor_timeout', note='Existing hosted run may still continue; do not duplicate it')
    except Exception as error:
        state('collector_failed', error_type=type(error).__name__, reason=str(error))
        raise


if __name__ == '__main__':
    main()
