#!/usr/bin/env python3
"""Collect frozen acid-route streams; observational review never certifies a build."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FREEZE = ROOT / 'audits/production-acid-projection-20261010/cli-acid-routes/freeze.json'
PARTNER_FAMILIES = {
    'bleach-acid-chlorine': ('NaOCl', 'ClO-', 'HClO'),
    'acid-metal-hydrogen': ('Al', 'Mg', 'Zn', 'Fe', 'Pb', 'uranium'),
    'acid-carbonate-co2': ('CO3-2', 'CaCO3', 'Na2CO3', 'NaHCO3'),
}


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')


def finite(value):
    return type(value) in (int, float) and math.isfinite(value)


def vessel(row):
    assert all(type(v['id']) is int for v in row['bench']['vessels']), 'invalid vessel identity'
    found = [v for v in row['bench']['vessels'] if type(v['id']) is int and v['id'] == 0]
    assert len(found) == 1, 'ambiguous vessel'
    v = found[0]
    def finite_state(value):
        if type(value) in (int, float):
            assert finite(value), 'nonfinite vessel state'
        elif isinstance(value, dict):
            for nested in value.values():
                finite_state(nested)
        elif isinstance(value, list):
            for nested in value:
                finite_state(nested)
    finite_state(v)
    assert finite(v['solute_charge']), 'invalid charge ledger'
    for p in v['contents']:
        assert isinstance(p['species'], str) and p['phase'] in (
            'solid', 'liquid', 'gas', 'aqueous')
        assert finite(p['moles']) and p['moles'] >= 0, 'invalid inventory'
    return v


def partner(v, case, cutoff=0.0):
    amount = sum(p['moles'] for p in v['contents']
                 if p['species'] in case['partner_keys']
                 and p['moles'] > cutoff
                 and (case['partner_phase'] is None or p['phase'] == case['partner_phase']))
    assert finite(amount), 'inventory sum overflow'
    return amount


def strong_acidity(v, freeze):
    solution = v.get('solution')
    dissociated = max(-v['solute_charge'], 0.0)
    # Source dbe86521 displacement::LEDGER_ACIDS contains only CH3COOH.
    # Preserve its bound-proton contribution rather than equating charge
    # deficit plus low pH with the source's full strong-acidity predicate.
    titratable = dissociated + sum(p['moles'] for p in v['contents']
                                  if p['species'] == 'CH3COOH'
                                  and p['phase'] in ('aqueous', 'liquid'))
    return (solution is not None and finite(solution.get('ph'))
            and solution['ph'] <= freeze['strong_acid_ph_max']
            and finite(titratable) and titratable > freeze['acidity_gate_mol']
            and dissociated > 0.5 * titratable)


def source_binding(binding, manifest, freeze):
    for key in ('compiled_source', 'source_tree', 'source_kind'):
        assert binding[key] == manifest[key]
    assert binding['compiled_source'] == freeze['source_observed']
    assert binding['source_tree'] == freeze['source_tree']


def analyse(rows, processes, text, case, freeze):
    """Separate absent witnesses from failed warning contracts."""
    checks = {'processes_completed': all(type(p['exit_code']) is int and p['exit_code'] == 0
                                         and p['timeout'] is False
                                         for p in processes.values())}
    expected = [dict(op='add', vessel=0, species='water'),
                dict(op='add', vessel=0, species=case['added_species'], moles=case['added_moles']),
                dict(op='titrate', vessel=0, titrant='HCl', concentration=1.0,
                     step=0.01, target_ph=0.0, max_steps=1)]
    checks['parsed_inputs'] = len(rows) == 3 and all(
        type(row.get('step')) is int and row['step'] == i
        and all(row.get('operator', {}).get(k) == value
                and (finite(row['operator'].get(k)) if type(value) in (int, float) else True)
                                    for k, value in op.items())
        and type(row.get('operator', {}).get('vessel')) is int
        for i, (row, op) in enumerate(zip(rows, expected)))
    if checks['parsed_inputs']:
        water = rows[0]['operator'].get('moles')
        checks['water_amount'] = finite(water) and math.isclose(water, 5.534276991396059,
                                                               rel_tol=1e-12, abs_tol=1e-12)
        checks['integer_step_bound'] = type(rows[2]['operator'].get('max_steps')) is int
        checks['ph_endpoint'] = rows[2]['operator'].get('endpoint', 'ph') == 'ph'
    events = [e for row in rows for e in row.get('events', [])]
    checks['no_refusal'] = not any(e.get('event') in ('solver_failed', 'not_yet_modeled',
                                                    'safety_veto') for e in events)
    result = {'checks': checks, 'classification': 'not_exercising_declared_route'}
    if not all(checks.values()):
        return result
    before, after = vessel(rows[1]), vessel(rows[2])
    setup_partner, final_partner = partner(before, case, freeze['cutoff_mol']), \
        partner(after, case, freeze['cutoff_mol'])
    checks['setup_partner'] = (setup_partner > freeze['cutoff_mol']
                               if case['role'] == 'positive' else setup_partner == 0)
    checks['setup_not_strong_acid'] = not strong_acidity(before, freeze) and not any(
        p['species'] in ('HCl', 'H2SO4') and p['moles'] > freeze['cutoff_mol']
        for p in before['contents'])
    checks['backend'] = all((v.get('solution') or {}).get('provenance', {}).get('engine')
                            == freeze['expected_backend_engine'] for v in (before, after))
    dose = [e for e in rows[2]['events'] if e.get('event') == 'titrated']
    checks['one_full_dose'] = len(dose) == 1 and all(
        dose[0].get(k) == value
        and (finite(dose[0].get(k)) if type(value) in (int, float) else True)
        for k, value in freeze['dose'].items() if k != 'moles')
    if len(dose) == 1:
        checks['integer_dose_fields'] = all(type(dose[0].get(k)) is int for k in ('vessel', 'steps'))
        checks['delivered_moles'] = finite(dose[0].get('concentration')) \
            and finite(dose[0].get('total_volume')) \
            and dose[0]['concentration'] * dose[0]['total_volume'] == freeze['dose']['moles']
        curve = dose[0].get('curve')
        checks['full_dose_curve'] = isinstance(curve, list) and len(curve) == 2 \
            and all(isinstance(point, list) and len(point) == 2
                    and all(finite(value) for value in point) for point in curve) \
            and curve[0][0] == 0.0 and curve[1][0] == freeze['dose']['total_volume'] * 1000 \
            and curve[1][1] == dose[0].get('final_ph')
    checks['finite_endpoint'] = len(dose) == 1 and finite(dose[0].get('final_ph'))
    checks['endpoint_matches_solution'] = len(dose) == 1 and after.get('solution') is not None \
        and dose[0].get('final_ph') == after['solution'].get('ph')
    checks['final_strong_acidity'] = strong_acidity(after, freeze)
    checks['named_acid_projected'] = not any(p['species'] in ('HCl', 'H2SO4')
                                            and p['moles'] > freeze['cutoff_mol']
                                            for p in after['contents'])
    family = PARTNER_FAMILIES[case['expected_warning']['rule']]
    # A different carbonate/metal key or phase is a separate exposure domain,
    # not evidence that the original partner was simply consumed. Do not
    # normalize unreviewed partner projections to obtain a passing observation.
    checks['no_unreviewed_final_partner'] = not any(
        p['species'] in family and p['moles'] > freeze['cutoff_mol']
        and (p['species'] not in case['partner_keys']
             or case['partner_phase'] is not None and p['phase'] != case['partner_phase'])
        for p in after['contents'])
    if case['role'] == 'benign':
        checks['final_partner_absent'] = partner(after, case) == 0
        checks['setup_partner'] = partner(before, case) == 0
        checks['no_other_screened_partner'] = not any(
            p['species'] in family and p['moles'] > freeze['cutoff_mol']
            for v in (before, after) for p in v['contents'])
    result.update(setup_vessel=before, final_vessel=after, titrated=dose,
                  setup_partner_mol=setup_partner, final_partner_mol=final_partner)
    if not all(checks.values()):
        return result
    rule = case['expected_warning']['rule']
    setup_warnings = [e for row in rows[:2] for e in row['events']
                      if e.get('event') == 'hazard_warning' and e.get('rule') == rule]
    warnings = [e for e in rows[2]['events']
                if e.get('event') == 'hazard_warning' and e.get('rule') == rule]
    metadata = case['expected_warning']
    warning_checks = {'no_setup_primary_warning': not setup_warnings}
    expected_count = 1 if case['role'] == 'positive' else 0
    warning_checks['exact_warning'] = len(warnings) == expected_count and all(
        all(e.get(k) == value for k, value in metadata.items()) for e in warnings)
    warning_checks['text_pair'] = all(text.count(metadata[k]) == expected_count
                                     for k in ('hazard', 'real_world'))
    result.update(warning_checks=warning_checks, primary_warnings=warnings,
                  final_surviving_exposure=final_partner > freeze['cutoff_mol'])
    if not all(warning_checks.values()):
        result['classification'] = 'warning_contract_mismatch'
    elif case['role'] == 'benign':
        result['classification'] = 'qualifying_benign_counterpart'
    elif final_partner > freeze['cutoff_mol']:
        result['classification'] = 'qualifying_named_to_dissolved_projection'
    else:
        result['classification'] = 'qualifying_prospective_only_partner_consumed'
    return result


def collect(binary, script, out, json_mode):
    out.mkdir()
    command = [str(binary), 'run', str(script)] + (['--json'] if json_mode else [])
    with (out / 'stdout.txt').open('xb') as stdout, (out / 'stderr.txt').open('xb') as stderr:
        try:
            code = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=120).returncode
            timeout = False
        except subprocess.TimeoutExpired:
            code, timeout = None, True
    save(out / 'process.json', dict(command=command, exit_code=code, timeout=timeout,
                                   stdout_sha256=digest(out / 'stdout.txt'),
                                   stderr_sha256=digest(out / 'stderr.txt')))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--build-binding', type=Path, required=True)
    parser.add_argument('--source-manifest', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--review', action='store_true')
    args = parser.parse_args()
    freeze = json.loads(FREEZE.read_text())
    binding = json.loads(args.build_binding.read_text())
    manifest = json.loads(args.source_manifest.read_text())
    source_binding(binding, manifest, freeze)
    for case in freeze['cases']:
        assert digest(ROOT / case['path']) == case['sha256']
    collection = dict(build_binding_sha256=digest(args.build_binding),
                      source_manifest_sha256=digest(args.source_manifest), freeze_sha256=digest(FREEZE))
    if not args.review:
        assert args.out == Path(args.out.name), 'use a portable output directory name'
        assert args.binary and digest(args.binary) == binding['binary_sha256']
        args.out.mkdir(parents=True, exist_ok=False)
        save(args.out / 'collection-binding.json', collection)
        for case in freeze['cases']:
            directory = args.out / Path(case['path']).stem
            directory.mkdir()
            (directory / 'input.lab').write_bytes((ROOT / case['path']).read_bytes())
            for mode in ('json', 'text'):
                collect(args.binary.resolve(), directory / 'input.lab', directory / mode, mode == 'json')
    else:
        assert json.loads((args.out / 'collection-binding.json').read_text()) == collection
    results = {}
    for case in freeze['cases']:
        directory = args.out / Path(case['path']).stem
        assert digest(directory / 'input.lab') == case['sha256']
        processes = {}
        for mode in ('json', 'text'):
            p = directory / mode
            process = json.loads((p / 'process.json').read_text())
            assert process['command'][1:] == ['run', str(Path(args.out.name) / directory.name / 'input.lab')] + (
                ['--json'] if mode == 'json' else [])
            for stream in ('stdout', 'stderr'):
                assert digest(p / (stream + '.txt')) == process[stream + '_sha256']
            processes[mode] = process
        try:
            rows = [json.loads(line) for line in (directory / 'json/stdout.txt').read_text().splitlines()]
            result = analyse(rows, processes, (directory / 'text/stdout.txt').read_text(), case, freeze)
        except (AssertionError, ValueError, KeyError, TypeError, AttributeError, OverflowError) as error:
            result = dict(classification='invalid_or_incomplete_evidence', error=str(error))
        results[Path(case['path']).stem] = result
    save(args.report, dict(source_binding=binding, collection_binding=collection,
                           observations=results, accepted_repair_proof=False,
                           scope='Uncertified observations; requires independent workflow/build/raw-process '
                                 'binding review. Consumed partners do not prove final exposure detection.'))


if __name__ == '__main__':
    main()
