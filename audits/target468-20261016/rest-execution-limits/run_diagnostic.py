#!/usr/bin/env python3
"""One changed-observer diagnostic in a fresh isolated Vercel VM; no retries."""
import argparse
import asyncio
import hashlib
import json
from pathlib import Path
from vercel import sandbox
from vercel.api import session


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot-receipt', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    raw = (root / 'design.json').read_bytes()
    design = json.loads(raw)
    receipt = args.snapshot_receipt.read_bytes()
    assert sha(receipt) == design['snapshot_receipt_sha256']
    binding = json.loads(receipt)
    assert binding['engine'] == design['engine']
    assert binding['vm_job_sha256'] == design['original_vm_job_sha256']
    observer = (root / 'vm_job_observer.py').read_bytes()
    assert sha(observer) == design['observer_sha256']
    script = design['case']['script'].encode()
    assert sha(script) == design['case']['script_sha256']
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'binding.json').write_text(json.dumps({
        'design_sha256': sha(raw), 'runner_sha256': sha(Path(__file__).read_bytes()),
        'observer_sha256': sha(observer), 'original_vm_job_sha256': binding['vm_job_sha256'],
        'engine': design['engine'], 'script_sha256': sha(script),
        'credentials_in_vm': False, 'retries': 0, 'physical_verdict': 'not_assigned'}, indent=2) + '\n')

    async def run():
        async with session():
            async with sandbox.create_sandbox(source=sandbox.SnapshotSource(snapshot_id=binding['snapshot_id']),
                    resources=sandbox.SandboxResources(vcpus=1, memory=2048), execution_time_limit=30,
                    network_policy=sandbox.NetworkPolicy.deny_all(), env={}, ports=[], persistent=False,
                    region='iad1') as box:
                await box.fs.write_bytes('/vercel/sandbox/engine/input.lab', script, mode=0o644)
                await box.fs.write_bytes('/vercel/sandbox/engine-job.py', observer, mode=0o644)
                (args.out / 'issued.json').write_text(json.dumps({'script_sha256': sha(script),
                    'observer_sha256': sha(observer), 'engine': design['engine']}, indent=2) + '\n')
                completed = await box.run_process('python3', ['-I', '/vercel/sandbox/engine-job.py'],
                    sudo=True, capture_output=True, kill_after=15)
                assert len(completed.stdout.encode()) <= 2_100_000
                (args.out / 'stdout.json').write_text(completed.stdout)
                # SDK stderr may contain paths; this directory is private evidence.
                (args.out / 'stderr.txt').write_text(completed.stderr)
                result = json.loads(completed.stdout)
                assert completed.returncode == 0
                assert result['binary_sha256'] == design['engine']['binary_sha256']
                return result

    result = asyncio.run(run())
    print(json.dumps({k: result[k] for k in ('status', 'code', 'elapsed', 'resource_usage')}))


if __name__ == '__main__':
    main()
