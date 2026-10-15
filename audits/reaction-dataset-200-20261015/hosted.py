"""Authenticate the preserved main CLI; run all 200 probes without a rebuild."""
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
        raise RuntimeError('Restricted to the hosted Linux lane')
    root = Path(__file__).resolve().parent
    out = Path('reaction-evidence').resolve()
    out.mkdir()
    binding = json.loads((root / 'execution-binding.json').read_text())
    repository = 'CrispStrobe/kerotakis'
    assert os.environ['GITHUB_REPOSITORY'] == repository
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    assert head == os.environ['GITHUB_SHA']
    witness = {'audit_head': head, 'run_id': os.environ['GITHUB_RUN_ID'],
               'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
               'binding_sha256': sha(root / 'execution-binding.json'),
               'producer_sha256': sha(Path(__file__)),
               'caller_sha256': sha(Path('.github/workflows/chemistry-audit.yml')),
               'callee_sha256': sha(Path('.github/workflows/reaction-dataset-200.yml'))}
    (out / 'routing.json').write_text(json.dumps(witness, indent=2) + '\n')

    def api(endpoint, filename):
        target = out / filename
        with target.open('wb') as stream:
            subprocess.run(['gh', 'api', 'repos/' + repository + endpoint],
                           stdout=stream, check=True, timeout=120)
        return target

    run = json.loads(api('/actions/runs/38739278904', 'source-run.json').read_text())
    assert run['head_sha'] == binding['source_run_audit_head'] and run['conclusion'] == 'success'
    metadata = json.loads(api('/actions/artifacts/11977961282', 'source-artifact.json').read_text())
    assert metadata['workflow_run']['id'] == 38739278904 and not metadata['expired']
    assert metadata['digest'] == 'sha256:' + binding['artifact_sha256']
    archive = api('/actions/artifacts/11977961282/zip', 'source-artifact.zip')
    assert sha(archive) == binding['artifact_sha256']
    with zipfile.ZipFile(archive) as zipped:
        members = zipped.infolist()
        assert len(members) < 200 and len({i.filename for i in members}) == len(members)
        assert sum(i.file_size for i in members) < 100_000_000
        for member in members:
            name = Path(member.filename)
            assert not name.is_absolute() and '..' not in name.parts
            assert member.external_attr >> 16 & 0o170000 != 0o120000
        zipped.extractall(out / 'source-artifact')
    saved = out / 'source-artifact'
    identity = json.loads((saved / 'identity.json').read_text())
    assert identity['source']['head'] == binding['source_head']
    assert identity['source']['tree'] == binding['source_tree']
    binary_binding = json.loads((saved / 'binary-binding.json').read_text())
    binary = saved / 'kero'
    assert sha(binary) == binary_binding['sha256'] == binding['binary_sha256']
    assert sha(saved / 'Cargo.lock') == binary_binding['lock_sha256'] == binding['cargo_lock_sha256']
    binary.chmod(0o700)
    subprocess.run([sys.executable, '-B', str(root / 'run_all.py'), str(binary),
                    str(root / 'execution-binding.json'), str(out / 'executions')],
                   timeout=6600, check=True)


if __name__ == '__main__':
    main()
