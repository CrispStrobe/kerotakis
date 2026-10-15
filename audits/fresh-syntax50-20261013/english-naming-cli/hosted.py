"""Hosted-only artifact reuse; no compilation and no automatic verdicts."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import zipfile


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_OS') != 'Linux':
        raise RuntimeError('This producer is restricted to the hosted Linux lane')
    root = Path(__file__).resolve().parent
    out = Path('naming-evidence').resolve()
    out.mkdir()
    binding = json.loads((root / 'syntax-normalized-binding.json').read_text())
    repository = 'CrispStrobe/kerotakis'
    assert os.environ['GITHUB_REPOSITORY'] == repository
    audit_head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    assert audit_head == os.environ['GITHUB_SHA']
    witness = {'audit_head': audit_head, 'run_id': os.environ['GITHUB_RUN_ID'],
               'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
               'binding_sha256': sha(root / 'syntax-normalized-binding.json'),
               'producer_sha256': sha(Path(__file__)),
               'caller_sha256': sha(Path('.github/workflows/chemistry-audit.yml')),
               'callee_sha256': sha(Path('.github/workflows/english-naming-controls.yml'))}
    (out / 'routing.json').write_text(json.dumps(witness, indent=2) + '\n')

    def api(endpoint, filename):
        target = out / filename
        with target.open('wb') as stream:
            subprocess.run(['gh', 'api', 'repos/' + repository + endpoint],
                           stdout=stream, check=True, timeout=120)
        return target

    run = json.loads(api('/actions/runs/38721945844', 'source-run.json').read_text())
    assert run['head_sha'] == binding['candidate_head']
    assert run['event'] == 'pull_request'
    metadata = json.loads(api('/actions/artifacts/11967894898', 'source-artifact.json').read_text())
    assert metadata['workflow_run']['id'] == 38721945844 and not metadata['expired']
    assert metadata['digest'] == 'sha256:' + binding['artifact_sha256']
    archive = api('/actions/artifacts/11967894898/zip', 'source-artifact.zip')
    assert sha(archive) == binding['artifact_sha256']
    expected = {'target/release/kero', 'Cargo.lock', 'binary-sha256.txt', 'submodules.txt'}
    with zipfile.ZipFile(archive) as zipped:
        members = zipped.infolist()
        assert len(members) == len(expected) and {i.filename for i in members} == expected
        assert sum(i.file_size for i in members) < 100_000_000
        assert all(i.external_attr >> 16 & 0o170000 != 0o120000 for i in members)
        zipped.extractall(out / 'source-artifact')
    saved = out / 'source-artifact'
    binary = saved / 'target/release/kero'
    assert sha(binary) == binding['binary_sha256']
    assert sha(saved / 'Cargo.lock') == binding['cargo_lock_sha256']
    assert sha(saved / 'submodules.txt') == binding['submodules_sha256']
    assert (saved / 'binary-sha256.txt').read_text().split()[0] == binding['binary_sha256']
    binary.chmod(0o700)
    completed = subprocess.run([sys.executable, '-B', str(root / 'run.py'), str(binary),
                                str(root / 'syntax-normalized-binding.json'), str(out / 'executions')],
                               timeout=600, check=False)
    if completed.returncode != 0:
        raise RuntimeError('Preserved runner stopped; inspect first failure or resource refusal')
    observations = json.loads((out / 'executions/executions.json').read_text())
    assert len(observations) == 8
    (out / 'completed.json').write_text(json.dumps({'executed_count': 8,
        'binary_sha256': sha(binary), 'physical_verdict': 'not_assigned'}, indent=2) + '\n')


if __name__ == '__main__':
    main()
