"""Run one frozen CLI control in a disposable, network-denied remote microVM.

Authentication uses the standard parent-side Vercel SDK environment. Never pass
credentials, account environment, or a bearer key to the isolated engine.
"""
import argparse
import asyncio
import base64
import datetime
import hashlib
import json
from pathlib import Path

from vercel import sandbox
from vercel.api import session


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def save(path, value):
    with path.open('x') as handle:
        json.dump(value, handle, indent=2)
        handle.write('\n')


async def run(args):
    frozen_raw = args.design.read_bytes()
    design = json.loads(frozen_raw)
    snapshot_raw = args.snapshot_receipt.read_bytes()
    assert sha(snapshot_raw) == design['snapshot_receipt_sha256']
    assert sha(Path(__file__).read_bytes()) == design['runner_sha256']
    snapshot = json.loads(snapshot_raw)
    binary = (args.artifact / 'target/release/kero').read_bytes()
    lock = (args.artifact / 'Cargo.lock').read_bytes()
    assert sha(binary) == design['engine']['binary_sha256']
    assert sha(lock) == design['engine']['cargo_lock_sha256']
    assert snapshot['vm_job_sha256'] == design['supervisor_sha256']
    assert snapshot['sandbox_sha256'] == design['sandbox_sha256']
    assert snapshot['engine']['libraries'] == design['runtime_libraries_sha256']
    assert sha(design['script'].encode()) == design['script_sha256']
    args.out.mkdir(mode=0o700, exist_ok=False)
    (args.out / 'frozen-design.json').write_bytes(frozen_raw)
    (args.out / 'Cargo.lock').write_bytes(lock)
    manifest = {**snapshot['engine'], **design['engine']}
    save(args.out / 'manifest.json', manifest)
    async with session():
        box = await sandbox.create_sandbox(
            source=sandbox.SnapshotSource(snapshot_id=snapshot['snapshot_id']),
            execution_time_limit=60,
            resources=sandbox.SandboxResources(vcpus=1, memory=2048),
            network_policy=sandbox.NetworkPolicy.deny_all(), env={}, ports=[],
            persistent=False, region='iad1')
        try:
            save(args.out / 'private-vm.json', {'id': box.id})
            for name, expected in [('engine-job.py', design['supervisor_sha256']),
                                   ('engine/sandbox.runtime', design['sandbox_sha256'])]:
                assert sha(await box.fs.read_bytes('/vercel/sandbox/' + name)) == expected
            await box.fs.write_bytes('/vercel/sandbox/engine/kero', binary, mode=0o755)
            await box.fs.write_bytes('/vercel/sandbox/engine/manifest.json',
                                    json.dumps(manifest).encode(), mode=0o644)
            await box.fs.write_bytes('/vercel/sandbox/engine/input.lab',
                                    design['script'].encode(), mode=0o644)
            save(args.out / 'issued.json', {
                'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                'design_sha256': sha(frozen_raw), 'script_sha256': design['script_sha256'],
                'attempt': 1})
            result = await box.run_process('python3', ['-I', '/vercel/sandbox/engine-job.py'],
                                          sudo=True, capture_output=True, kill_after=25)
            envelope = result.stdout.encode()
            (args.out / 'supervisor.stdout').write_bytes(envelope)
            (args.out / 'supervisor.stderr').write_text(result.stderr)
            save(args.out / 'supervisor-process.json', {'returncode': result.returncode})
            assert len(envelope) <= 2_000_000
            assert result.returncode == 0
            response = json.loads(envelope)
            assert response['binary_sha256'] == design['engine']['binary_sha256']
            stdout = base64.b64decode(response['stdout'], validate=True)
            stderr = base64.b64decode(response['stderr'], validate=True)
            (args.out / 'stdout.jsonl').write_bytes(stdout)
            (args.out / 'stderr.txt').write_bytes(stderr)
            rows = [json.loads(line) for line in stdout.splitlines()]
            save(args.out / 'execution.json', {
                'engine': design['engine'], 'status': response['status'],
                'exit_code': response['code'], 'script_sha256': design['script_sha256'],
                'stdout_sha256': sha(stdout), 'stderr_sha256': sha(stderr),
                'observations': len(rows), 'elapsed_seconds': response['elapsed'],
                'resource_usage': response['resource_usage'], 'attempts': 1,
                'physical_verdict': 'not_assigned'})
            print('CLI replay:', response['status'], response['code'], len(rows), 'observations')
        finally:
            await box.destroy(delete_orphan_snapshots=False)
            save(args.out / 'destroy-returned.json', {
                'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                'sdk_destroy_returned': True, 'independent_deletion_receipt': False,
                'base_snapshot_modified': False})


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('design', 'snapshot-receipt', 'artifact', 'out'):
        parser.add_argument('--' + name, type=Path, required=True)
    asyncio.run(run(parser.parse_args()))
