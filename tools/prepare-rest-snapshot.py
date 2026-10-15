"""Prepare a verified, secret-free native image for disposable Vercel microVMs.

Usage: python tools/prepare-rest-snapshot.py NATIVE_BUNDLE NEW_RECEIPT
Authenticate through project OIDC or standard Vercel SDK environment variables.
No API bearer keys, Redis credentials or client scripts enter the snapshot.
"""
import asyncio
import base64
import hashlib
import json
from pathlib import Path
import sys
from vercel import sandbox
from vercel.api import session

root = Path(__file__).resolve().parent.parent
artifacts = Path(sys.argv[1]).resolve(strict=True)
receipt = Path(sys.argv[2])
if receipt.exists():
    raise SystemExit("Refusing to overwrite a snapshot receipt")
manifest = json.loads((artifacts / "manifest.json").read_text())
if manifest.get("loader") != "ld-linux-x86-64.so.2":
    raise SystemExit("The reviewed x86_64 loader is required")
files = {"kero": (artifacts / "kero").read_bytes(), "manifest.json": (artifacts / "manifest.json").read_bytes(),
         "sandbox.runtime": (root / "services/rest_api/sandbox.py").read_bytes()}
if hashlib.sha256(files["kero"]).hexdigest() != manifest["binary_sha256"]:
    raise SystemExit("Engine identity mismatch")
for name, digest in manifest["libraries"].items():
    if Path(name).name != name:
        raise SystemExit("Invalid library name")
    content = (artifacts / "lib" / name).read_bytes()
    if hashlib.sha256(content).hexdigest() != digest:
        raise SystemExit("Runtime identity mismatch")
    files["lib/" + name] = content
job = (root / "services/rest_api/vm_job.py").read_bytes()

async def main():
    async with session():
        box = await sandbox.create_sandbox(resources=sandbox.SandboxResources(vcpus=1, memory=2048),
            execution_time_limit=120, network_policy=sandbox.NetworkPolicy.deny_all(),
            env={}, ports=[], persistent=False, region="iad1")
        try:
            await box.fs.mkdir("/vercel/sandbox/engine/lib")
            for name, content in files.items():
                await box.fs.write_bytes("/vercel/sandbox/engine/" + name, content,
                    mode=0o755 if name in ("kero", "lib/ld-linux-x86-64.so.2") else 0o644)
            await box.fs.write_bytes("/vercel/sandbox/engine-job.py", job, mode=0o644)
            await box.fs.write_text("/vercel/sandbox/engine/input.lab", "add v1 water 100mL @ 25C\ninspect v1", mode=0o644)
            result = await box.run_process("python3", ["-I", "/vercel/sandbox/engine-job.py"],
                sudo=True, capture_output=True, kill_after=15)
            if result.returncode:
                raise RuntimeError("Snapshot smoke failed: " + result.stderr[:1500])
            response = json.loads(result.stdout)
            if response["status"] != "completed" or response["code"] != 0:
                raise RuntimeError("Snapshot smoke failed: " + base64.b64decode(response["stderr"]).decode()[:1500])
            rows = [json.loads(line) for line in base64.b64decode(response["stdout"]).splitlines()]
            if len(rows) != 2 or response["binary_sha256"] != manifest["binary_sha256"]:
                raise RuntimeError("Invalid native smoke observations")
            await box.fs.remove("/vercel/sandbox/engine/input.lab")
            snapshot = await box.snapshot(expiration=30 * 86400)
            record = dict(snapshot_id=snapshot.id, engine=manifest,
                vm_job_sha256=hashlib.sha256(job).hexdigest(),
                sandbox_sha256=hashlib.sha256(files["sandbox.runtime"]).hexdigest(),
                network="deny-all", vcpus=1, memory_mib=2048,
                smoke=response, physical_verdict="not_assigned")
            receipt.write_text(json.dumps(record, indent=2) + "\n")
            print("Prepared snapshot", snapshot.id, "native observations", len(rows))
        finally:
            await box.destroy(delete_orphan_snapshots=False)

asyncio.run(main())
