#!/usr/bin/env python3
"""Collect or recheck frozen final-safety CLI probes; never certify a build."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FREEZE = ROOT / 'audits/production-final-safety-20261009/cli-warning-probes/freeze.json'
RULE = 'bleach-ammonia-chloramine'
HYPO = ('NaOCl', 'ClO-', 'HClO')
HAZARD = 'mixing bleach with ammonia makes chloramine, a toxic gas'
REAL_WORLD = 'People are hospitalised every year from mixing these two household cleaners.'


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')


def finite(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def inventory(row):
    vessels = row['bench']['vessels']
    matches = [v for v in vessels if v['id'] == 0]
    if len(matches) != 1:
        raise ValueError('expected exactly one v1 inventory')
    amounts = {}
    for portion in matches[0]['contents']:
        amount = portion['moles']
        if not finite(amount) or amount < 0:
            raise ValueError('invalid raw inventory amount')
        species = portion['species']
        if not isinstance(species, str):
            raise ValueError('invalid species identity')
        amounts[species] = amounts.get(species, 0) + amount
        if not finite(amounts[species]):
            raise ValueError('inventory sum overflow')
    return amounts, matches[0].get('solution')


def analyse(rows, process, positive, repaired):
    """Compute observations independently of archived runner verdicts."""
    checks = {'process_completed': process['exit_code'] == 0 and not process['timeout']}
    checks['four_frozen_operators'] = (
        len(rows) == 4 and [r.get('operator', {}).get('op') for r in rows] ==
        ['add', 'add', 'add', 'titrate'] and
        all(r.get('step') == i + 1 for i, r in enumerate(rows)))
    events = [e for row in rows for e in row.get('events', [])]
    checks['no_refusal_or_solver_failure'] = not any(
        e.get('event') in ('solver_failed', 'not_yet_modeled', 'safety_veto') for e in events)
    observation = {'checks': checks}
    if not all(checks.values()):
        observation['classification'] = 'not_exercising_intended_route'
        return observation
    before, setup_solution = inventory(rows[2])
    after, final_solution = inventory(rows[3])
    cutoff = 1e-12
    checks['setup_nh3_below_or_at_cutoff'] = before.get('NH3', 0) <= cutoff
    checks['final_nh3_above_cutoff'] = after.get('NH3', 0) > cutoff
    checks['hypochlorite_witness'] = (
        any(before.get(k, 0) > cutoff for k in HYPO) and
        any(after.get(k, 0) > cutoff for k in HYPO)) if positive else (
        all(before.get(k, 0) == 0 and after.get(k, 0) == 0 for k in HYPO))
    titrated = [e for e in rows[3]['events'] if e.get('event') == 'titrated']
    checks['one_accepted_full_dose'] = len(titrated) == 1 and all(
        titrated[0].get(k) == v for k, v in {
            'vessel': 0, 'titrant': 'NaOH', 'concentration': 0.001,
            'steps': 1, 'total_volume': 0.001}.items())
    if checks['one_accepted_full_dose']:
        checks['finite_endpoint'] = finite(titrated[0].get('final_ph'))
    setup_warnings = [e for row in rows[:3] for e in row['events']
                      if e.get('event') == 'hazard_warning' and e.get('rule') == RULE]
    final_warnings = [e for e in rows[3]['events']
                      if e.get('event') == 'hazard_warning' and e.get('rule') == RULE]
    observation.update(setup_inventory=before, final_inventory=after,
                       setup_solution=setup_solution, final_solution=final_solution,
                       titrated=titrated, setup_warnings=setup_warnings,
                       final_warnings=final_warnings)
    if not all(checks.values()):
        observation['classification'] = 'not_exercising_intended_route'
        return observation
    warning_checks = {'no_setup_chloramine_warning': not setup_warnings}
    if positive and repaired:
        warning_checks['exact_final_warning'] = len(final_warnings) == 1 and all(
            final_warnings[0].get(k) == v for k, v in {
                'severity': 'danger', 'rule': RULE, 'hazard': HAZARD,
                'real_world': REAL_WORLD}.items())
    else:
        warning_checks['no_final_chloramine_warning'] = not final_warnings
    observation['warning_checks'] = warning_checks
    observation['classification'] = ('qualifying_inventory_and_warning_observation'
                                     if all(warning_checks.values()) else 'warning_contract_mismatch')
    return observation


def collect(binary, script, out, json_mode):
    out.mkdir()
    command = [str(binary), 'run', str(script)] + (['--json'] if json_mode else [])
    timeout = False
    with (out / 'stdout.txt').open('xb') as stdout, (out / 'stderr.txt').open('xb') as stderr:
        try:
            result = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=120)
            code = result.returncode
        except subprocess.TimeoutExpired:
            timeout, code = True, None
    row = {'command': command, 'exit_code': code, 'timeout': timeout,
           'stdout_sha256': digest(out / 'stdout.txt'),
           'stderr_sha256': digest(out / 'stderr.txt')}
    save(out / 'process.json', row)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--build-binding', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--review', action='store_true')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    freeze = json.loads(FREEZE.read_text())
    binding = json.loads(args.build_binding.read_text())
    source = binding['compiled_source']
    assert source in (freeze['source_observed'], freeze['prerepair_source'])
    for name, sha in freeze['scripts'].items():
        assert digest(ROOT / name) == sha
    if not args.review:
        assert args.binary and digest(args.binary) == binding['binary_sha256']
        args.out.mkdir(parents=True, exist_ok=False)
        save(args.out / 'collection-binding.json', {
            'build_binding_sha256': digest(args.build_binding),
            'freeze_sha256': digest(FREEZE), 'scripts': freeze['scripts']})
        for name in freeze['scripts']:
            script = ROOT / name
            directory = args.out / script.stem
            directory.mkdir()
            (directory / 'input.lab').write_bytes(script.read_bytes())
            for mode in ('json', 'text'):
                collect(args.binary.resolve(), directory / 'input.lab', directory / mode, mode == 'json')
    else:
        assert json.loads((args.out / 'collection-binding.json').read_text()) == {
            'build_binding_sha256': digest(args.build_binding),
            'freeze_sha256': digest(FREEZE), 'scripts': freeze['scripts']}
    observations = {}
    for name, sha in freeze['scripts'].items():
        directory = args.out / Path(name).stem
        assert digest(directory / 'input.lab') == sha
        processes = {}
        for mode in ('json', 'text'):
            mode_dir = directory / mode
            process = json.loads((mode_dir / 'process.json').read_text())
            for stream in ('stdout', 'stderr'):
                assert digest(mode_dir / (stream + '.txt')) == process[stream + '_sha256']
            processes[mode] = process
        try:
            rows = [json.loads(line) for line in (directory / 'json/stdout.txt').read_text().splitlines()]
            result = analyse(rows, processes['json'], 'introduced' in name,
                             source == freeze['source_observed'])
        except (ValueError, KeyError, TypeError, OverflowError) as error:
            result = {'classification': 'invalid_or_incomplete_cli_evidence', 'error': str(error)}
        result['text_process'] = processes['text']
        result['text_narration_review'] = 'pending_independent_review_of_archived_stdout'
        observations[Path(name).stem] = result
    save(args.report, {'compiled_source': source, 'source_tree': binding['source_tree'],
                       'binary_sha256': binding['binary_sha256'],
                       'build_binding_sha256': digest(args.build_binding),
                       'freeze_sha256': digest(FREEZE), 'observations': observations,
                       'accepted_repair_proof': False,
                       'remaining_acceptance': [
                           'Independent actual-workflow/executable/lock/clean-source/toolchain/gitlink binding review.',
                           'Verify actual solver identity, capabilities and absence of silent fallback from raw provenance.',
                           'Review separate text narration and pair qualifying baseline with repaired source.',
                           'All frozen witnesses must hold; missing witnesses never establish repair correctness.']})
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
