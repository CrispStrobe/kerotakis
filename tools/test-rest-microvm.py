"""Live security controls in a fresh copy of the prepared Vercel microVM.

Usage: python tools/test-rest-microvm.py SNAPSHOT_RECEIPT NEW_EVIDENCE
Requires standard SDK authentication. Uses one bounded Vercel VM; never changes
or saves the production snapshot. Compile the tiny fixture, not the engine.
"""
import asyncio, base64, hashlib, json, subprocess, sys, tempfile
from pathlib import Path
from vercel import sandbox
from vercel.api import session
root=Path(__file__).resolve().parent.parent
binding=json.loads(Path(sys.argv[1]).read_text()); output=Path(sys.argv[2])
if output.exists(): raise SystemExit("Refusing to replace previous evidence")
async def main(binary):
 async with session():
  async with sandbox.create_sandbox(source=sandbox.SnapshotSource(snapshot_id=binding['snapshot_id']),
      execution_time_limit=30,resources=sandbox.SandboxResources(vcpus=1,memory=2048),
      network_policy=sandbox.NetworkPolicy.deny_all(),env={},ports=[],persistent=False,region='iad1') as box:
   await box.fs.write_bytes('/vercel/sandbox/engine/kero',binary,mode=0o755)
   manifest=dict(binding['engine']);manifest['binary_sha256']=hashlib.sha256(binary).hexdigest()
   await box.fs.write_text('/vercel/sandbox/engine/manifest.json',json.dumps(manifest),mode=0o644)
   await box.fs.write_text('/vercel/sandbox/engine/input.lab','immutable sentinel',mode=0o666)
   process=await box.run_process('python3',['-I','/vercel/sandbox/engine-job.py'],sudo=True,capture_output=True,kill_after=15)
   result=json.loads(process.stdout)
   observation=json.loads(base64.b64decode(result['stdout']))
   output.write_text(json.dumps(dict(snapshot_id=binding['snapshot_id'],result=result,observation=observation,
      probe_sha256=hashlib.sha256((root/'services/rest_api/probe.c').read_bytes()).hexdigest()),indent=2)+'\n')
   assert process.returncode==0 and result['code']==0 and observation=={'isolation_controls_pass':True},result
   print('Actual Vercel microVM isolation controls passed')
with tempfile.TemporaryDirectory() as temp:
 binary=Path(temp)/'probe'
 subprocess.run(['cc','-O0',str(root/'services/rest_api/probe.c'),'-o',str(binary)],check=True)
 asyncio.run(main(binary.read_bytes()))
