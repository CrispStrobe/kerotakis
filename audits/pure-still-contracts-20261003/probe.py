"""Hosted thermodynamic diagnostics; not an accepted CLI validation receipt."""
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
    parser.add_argument('--scope', choices=['thermo', 'transport'], default='thermo')
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only; local execution requires resource headroom')
    args.out.mkdir(parents=True, exist_ok=True)
    command = ['cargo', 'test', '-p', 'kerotakis-thermo', '--lib', '--tests', '--no-fail-fast',
               '-j1', '--', '--test-threads=1']
    if args.scope == 'transport':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'transport', '--test', 'transport_precision_contracts',
                   '--test', 'still_donor_precision', '--test', 'atomic_distil_refusal',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1']
    log = args.out/('thermo.log' if args.scope == 'thermo' else 'transport.log')
    started = time.monotonic()
    with log.open('w') as output:
        result = subprocess.run(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
    receipt = dict(diagnostic_only=True, commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                   command=command, exit_code=result.returncode, seconds=time.monotonic()-started,
                   log=log.name, log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
    (args.out/'diagnostic.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt,indent=2))
    raise SystemExit(result.returncode)

if __name__ == '__main__':
    main()
