"""Focused hosted native diagnostics; never an accepted CLI build receipt."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only; local work must use the resource gate')
    args.out.mkdir(parents=True, exist_ok=True)
    command = ['cargo', 'test', '-p', 'kerotakis-phreeqc', '--features', 'engine', '--lib',
               '--test', 'trace_inventory', '--test', 'order_invariance',
               '--test', 'engine_call_budget', '--test', 'settled_carbonate_transfer',
               '--no-fail-fast', '--', '--test-threads=1']
    env = dict(os.environ, CARGO_BUILD_JOBS='1', KERO_WORKERS='1', RUSTC_WRAPPER='', KERO_DUMP_INPUT='1')
    commands = [('native', command), ('hook', ['cargo', 'test', '-p', 'kerotakis-phreeqc', '--no-default-features', '--lib', '--no-fail-fast', '--', '--test-threads=1'])]
    stages = []
    for name, cmd in commands:
        log = args.out/(name+'.log')
        started = time.time()
        with log.open('w') as output:
            result = subprocess.run(cmd, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
        stages.append(dict(name=name, command=cmd, exit_code=result.returncode, seconds=time.time()-started, log=log.name, log_sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
    receipt = dict(diagnostic_only=True, commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(), stages=stages)
    (args.out/'diagnostic.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt,indent=2))
    raise SystemExit(0 if all(s['exit_code']==0 for s in stages) else 1)

if __name__ == '__main__':
    main()
