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
    parser.add_argument('--scope', choices=['thermo', 'transport', 'receiver', 'fractional', 'safety', 'titration', 'accounting'], default='thermo')
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        parser.error('Hosted execution only; local execution requires resource headroom')
    args.out.mkdir(parents=True, exist_ok=True)
    command = ['cargo', 'test', '-p', 'kerotakis-thermo', '--lib', '--tests', '--no-fail-fast',
               '-j1', '--', '--test-threads=1']
    if args.scope == 'transport':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'transport', '--test', 'transport_precision_contracts',
                   '--test', 'transport_charge_precision',
                   '--test', 'transport_subnormal_fraction',
                   '--test', 'still_donor_precision', '--test', 'atomic_distil_refusal',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1']
    if args.scope == 'receiver':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'transfer_receiver_precision', '--test', 'transfer_receiver_controls', '--test', 'transfer_trace',
                   '--test', 'mix', '--test', 'layers', '--test', 'vegetable_oil_layers', '--test', 'i18n_coverage', '--test', 'refusal_locale',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1']
    if args.scope == 'fractional':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'fractional_transfer_precision', '--test', 'fractional_transfer_controls', '--test', 'transfer_trace',
                   '--test', 'mix', '--test', 'magic_milk', '--test', 'vegetable_oil_layers',
                   '--test', 'transfer_receiver_precision', '--test', 'transfer_receiver_controls',
                   '--test', 'i18n_coverage', '--test', 'refusal_locale',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1']
    if args.scope == 'safety':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'safety_veto_contracts', '--test', 'safety_extraction_controls', '--test', 'transfer_probe_contracts',
                   '--test', 'mix', '--test', 'magic_milk', '--test', 'liquid_extraction',
                   '--test', 'spill_breakage', '--test', 'spill_persistence',
                   '--test', 'transfer_trace', '--test', 'solver_transactions',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1',
                   '--skip', 'extraction_veto_retains_its_atomic_contract']
    if args.scope == 'titration':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'titration_safety_contracts', '--test', 'titration_safety_controls', '--test', 'redox_titrimetry',
                   '--test', 'solver_transactions',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1']
    if args.scope == 'accounting':
        command = ['cargo', 'test', '-p', 'kerotakis-core', '--lib',
                   '--test', 'titration_quantity_contracts', '--test', 'titration_stock_contracts',
                   '--test', 'stock_precision_contracts', '--test', 'stock',
                   '--test', 'titration_safety_contracts', '--test', 'titration_safety_controls',
                   '--test', 'redox_titrimetry', '--test', 'solver_transactions',
                   '--test', 'i18n_coverage', '--test', 'refusal_locale',
                   '--no-fail-fast', '-j1', '--', '--test-threads=1']
    log = args.out/(args.scope + '.log')
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
