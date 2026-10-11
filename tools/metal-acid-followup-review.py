#!/usr/bin/env python3
"""Review a separately frozen Mg/MgSO4 pair; never revise the first route freeze."""
import argparse
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import types

ROOT = Path(__file__).resolve().parents[1]
FOLDER = 'audits/production-acid-projection-20261010/cli-acid-routes'
FREEZE = FOLDER + '/metal-followup/freeze.json'
MANIFEST = 'audits/production-acid-projection-20261010/cli-source-dbe86521.json'
COLLECTOR = 'tools/acid-route-cli-probes.py'
COLLECTOR_SHA = '0c887888675552f9ca8c8f6fd7df26fdcada2ba61aa2a97ca47940ee72b27cf5'
PARENT = 'audits/production-acid-projection-20261010/'
NOTICES = [dict(event='not_yet_modeled', cause='rate-not-modelled',
                reason_key='not-modeled.stays-as-the-metal'),
           dict(event='not_yet_modeled', cause='no-solver',
                reason_key='not-modeled.no-dissolution-solver')]
REFUSALS = {'solver_failed', 'not_yet_modeled', 'safety_veto'}
NOTICE_TEMPLATES = {
    'not-modeled.stays-as-the-metal': "{name} stays as the metal: nothing dissolved here sits below it in the activity series. Its slow reaction with water itself — hydrogen over hours, a passivating hydroxide skin — is a rate this lab does not model",
    'not-modeled.no-dissolution-solver': '{name} in contact with liquid: no wired solver models this dissolution/reaction',
}


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def decode(text):
    def pairs(items):
        result = {}
        for key, value in items:
            assert key not in result, 'duplicate JSON key'
            result[key] = value
        return result
    def constant(value):
        raise ValueError('nonfinite JSON constant: ' + value)
    return json.loads(text, object_pairs_hook=pairs, parse_constant=constant)


def read(path):
    return decode(path.read_text())


def load_source(name, path):
    module = types.ModuleType(name)
    module.__file__ = str(path)
    exec(compile(path.read_bytes(), str(path), 'exec'), module.__dict__)
    return module


def frozen_contract(freeze):
    original = read(ROOT / FOLDER / 'freeze.json')
    for key in ('source_observed', 'source_tree', 'cutoff_mol', 'acidity_gate_mol',
                'strong_acid_ph_max', 'dose', 'expected_backend_engine'):
        assert freeze[key] == original[key], 'changed source/dose/threshold'
    assert freeze['expected_processes'] == 4 and len(freeze['cases']) == 2
    assert freeze['expected_cases'] == 2 and freeze['collector_sha256'] == COLLECTOR_SHA
    names = {Path(c['path']).stem for c in freeze['cases']}
    assert names == {'metal-positive', 'metal-benign-sulfate'}
    for case in freeze['cases']:
        name = Path(case['path']).stem
        role = 'positive' if name == 'metal-positive' else 'benign'
        assert case['path'] == FOLDER + '/metal-followup/' + name + '.lab'
        assert case['role'] == role
        old = next(c for c in original['cases'] if c['path'].endswith('/metal-' + role + '.lab'))
        for key in ('added_moles', 'partner_keys', 'partner_phase', 'expected_warning'):
            assert case[key] == old[key], 'changed partner/warning contract'
        assert case['added_species'] == ('Mg' if role == 'positive' else 'MgSO4')
        assert case['allowed_setup_notices'] == (NOTICES if role == 'positive' else [])
        assert case['setup_backend'] == ('analytic-solvent-only' if role == 'positive' else 'PHREEQC')
        assert digest(ROOT / case['path']) == case['sha256']
        expected = (ROOT / old['path']).read_bytes()
        if role == 'benign':
            expected = (b'# Separately frozen registered-salt benign counterpart; original MgCl2 outcome retained.\n'
                        + b'\n'.join(expected.replace(b'MgCl2', b'MgSO4').splitlines()[1:]) + b'\n')
        assert (ROOT / case['path']).read_bytes() == expected, 'unexpected script mutation'


def scope_notices(rows, case):
    """Whitelist only the two named Mg/water setup limitations, retaining originals."""
    assert len(rows) == 3
    found = []
    kept = deepcopy(rows)
    for index, row in enumerate(rows):
        for event in row['events']:
            if event.get('event') not in REFUSALS:
                continue
            assert index == 1 and row['step'] == 1, 'refusal outside setup operator'
            assert type(event.get('vessel')) is int and event['vessel'] == 0
            reason = event.get('reason', {})
            signature = dict(event=event['event'], cause=event.get('cause'),
                             reason_key=reason.get('key'))
            assert signature in case['allowed_setup_notices'], 'unapproved setup notice'
            template = NOTICE_TEMPLATES[signature['reason_key']]
            assert reason.get('en') == template
            assert reason.get('slots') == [['name', {'term': {'en': 'magnesium',
                                                            'section': 'species'}}]]
            assert event.get('what') == template.replace('{name}', 'magnesium')
            found.append(signature)
    assert Counter(tuple(sorted(v.items())) for v in found) == Counter(
        tuple(sorted(v.items())) for v in case['allowed_setup_notices']), 'missing/duplicate setup notice'
    kept[1]['events'] = [e for e in kept[1]['events'] if e.get('event') not in REFUSALS]
    return kept, found


def analyse(rows, processes, text, case, freeze, collector):
    kept, found = scope_notices(rows, case)
    original_result = collector.analyse(rows, processes, text, case, freeze)
    # This copy changes only the explicitly checked setup notices. The original
    # backend check remains false for Mg in water; no provenance is rewritten.
    result = collector.analyse(kept, processes, text, case, freeze)
    checks = result['checks']
    assert all(value is True for key, value in checks.items() if key != 'backend')
    required = {'processes_completed', 'parsed_inputs', 'water_amount', 'integer_step_bound',
                'ph_endpoint', 'no_refusal', 'setup_partner', 'setup_not_strong_acid',
                'backend', 'one_full_dose', 'integer_dose_fields', 'delivered_moles',
                'full_dose_curve', 'finite_endpoint', 'endpoint_matches_solution',
                'final_strong_acidity', 'named_acid_projected', 'no_unreviewed_final_partner'}
    if case['role'] == 'benign':
        required |= {'final_partner_absent', 'no_other_screened_partner'}
    assert set(checks) == required, 'incomplete original collector validation'
    before, after = collector.vessel(rows[1]), collector.vessel(rows[2])
    setup = before['solution']
    if case['role'] == 'positive':
        setup_matches = (setup['scope'] == 'solvent_only'
                         and setup['provenance']['engine'] == 'Kerotakis analytic equilibrium evaluator'
                         and setup['provenance']['routing_phrase']['key'] == 'routing.solvent-relation-only')
        assert checks['backend'] is False
    else:
        setup_matches = setup['provenance']['engine'] == freeze['expected_backend_engine']
        assert checks['backend'] is True
    scoped = dict(setup_backend=setup_matches,
                  titration_backend=after['solution']['provenance']['engine'] == freeze['expected_backend_engine'])
    assert all(scoped.values()), 'unexpected setup/titration backend'
    metadata = case['expected_warning']
    rule = metadata['rule']
    setup_warnings = [e for row in rows[:2] for e in row['events']
                      if e.get('event') == 'hazard_warning' and e.get('rule') == rule]
    warnings = [e for e in rows[2]['events']
                if e.get('event') == 'hazard_warning' and e.get('rule') == rule]
    count = 1 if case['role'] == 'positive' else 0
    warning_checks = dict(no_setup_primary_warning=not setup_warnings,
                         exact_warning=len(warnings) == count and all(
                             all(e.get(k) == v for k, v in metadata.items()) for e in warnings),
                         exact_text_wording=all(text.count(metadata[k]) == count
                                               for k in ('hazard', 'real_world')),
                         exact_text_severity=text.count('HAZARD (' + metadata['severity'].capitalize()
                                                        + '): ' + metadata['hazard']) == count,
                         exact_dose_narration=text.count('titrated with 1 mol/L hydrochloric acid') == 1
                                              and text.count('1 step(s), 10.0 mL total') == 1)
    notices = [line for line in text.splitlines() if 'not yet modelled' in line.lower()]
    expected_notices = ['  v1: not yet modelled — ' + e['what'] for e in rows[1]['events']
                        if e.get('event') == 'not_yet_modeled']
    warning_checks['exact_text_setup_notices'] = notices == expected_notices
    warning_checks['no_text_solver_or_safety_refusal'] = not any(
        re.search(pattern, text, re.IGNORECASE) for pattern in
        (r"solver\s+'[^']*'\s+failed", r'solver failed', r'SAFETY VETO',
         r"The lab won't do that", r"The lab couldn't work out"))
    assert all(warning_checks.values()), 'warning/narration mismatch'
    final = result['final_partner_mol']
    classification = ('qualifying_benign_counterpart' if case['role'] == 'benign'
                      else 'qualifying_named_to_dissolved_projection' if final > freeze['cutoff_mol']
                      else 'qualifying_prospective_only_partner_consumed')
    return dict(classification=classification, scoped_collector_checks=checks,
                scoped_collector_classification=result['classification'],
                original_collector_checks=original_result['checks'],
                original_collector_classification=original_result['classification'],
                allowed_setup_notices_observed=found, scoped_backend_checks=scoped,
                warning_checks=warning_checks, setup_partner_mol=result['setup_partner_mol'],
                final_partner_mol=final, final_surviving_exposure=final > freeze['cutoff_mol'])


def raw_case(artifact, case):
    directory = artifact / Path(case['path']).stem
    assert digest(directory / 'input.lab') == case['sha256']
    processes = {}
    for mode in ('json', 'text'):
        folder = directory / mode
        process = read(folder / 'process.json')
        command = process['command']
        assert isinstance(command, list) and all(isinstance(v, str) for v in command)
        assert command[1:] == ['run', directory.name + '/input.lab'] + (['--json'] if mode == 'json' else [])
        assert Path(command[0]).is_absolute() and Path(command[0]).name == 'kero'
        assert process['timeout'] is False and type(process['exit_code']) is int and process['exit_code'] == 0
        assert (folder / 'stderr.txt').read_bytes() == b''
        for stream in ('stdout', 'stderr'):
            assert digest(folder / (stream + '.txt')) == process[stream + '_sha256']
        processes[mode] = process
    assert processes['json']['command'][0] == processes['text']['command'][0]
    rows = [decode(line) for line in (directory / 'json/stdout.txt').read_text().splitlines()]
    return rows, processes, (directory / 'text/stdout.txt').read_text()


def review(args):
    freeze = read(ROOT / FREEZE)
    frozen_contract(freeze)
    assert digest(ROOT / COLLECTOR) == COLLECTOR_SHA
    collector = load_source('frozen_acid_collector', ROOT / COLLECTOR)
    cli = load_source('parent_cli_binding', ROOT / 'tools/cli-binding-review.py')
    published = read(ROOT / PARENT / 'cli-bindings-review-dbe86521.json')
    parent = cli.review(args.parent_artifact, args.parent_run_metadata, published['harness'], MANIFEST,
                        ROOT / PARENT / 'cli-results-review-dbe86521.json')
    assert parent == published and parent['accepted_executable_bindings'] is True
    assert parent['run'] == 38147457930
    binding = read(args.artifact / 'binding.json')
    assert binding['source_binding'] == parent['source_binding']
    assert binding['binary_sha256'] == parent['source_binding']['binary_sha256']
    assert binding['freeze_sha256'] == digest(ROOT / FREEZE)
    assert binding['collector_sha256'] == COLLECTOR_SHA
    collector.source_binding(binding['source_binding'], read(ROOT / MANIFEST), freeze)
    expected = {'binding.json'}
    summaries, hashes, commands = {}, {}, set()
    for case in freeze['cases']:
        name = Path(case['path']).stem
        expected.add(name + '/input.lab')
        for mode in ('json', 'text'):
            expected.update(name + '/' + mode + '/' + f for f in ('process.json', 'stdout.txt', 'stderr.txt'))
        rows, processes, text = raw_case(args.artifact, case)
        commands.update(process['command'][0] for process in processes.values())
        summaries[name] = analyse(rows, processes, text, case, freeze, collector)
    actual = {str(p.relative_to(args.artifact)) for p in args.artifact.rglob('*') if p.is_file()}
    assert actual == expected, 'unexpected or missing collection file'
    assert len(commands) == 1
    # The verified executable may be copied to fast storage for these four
    # processes; verify the actual recorded path as well as the parent copy.
    assert digest(Path(next(iter(commands)))) == binding['binary_sha256']
    assert digest(args.parent_artifact / 'combined-build/kero') == binding['binary_sha256']
    for name in sorted(expected):
        hashes[name] = digest(args.artifact / name)
    return dict(source_binding=parent['source_binding'], parent_run=parent['run'],
                parent_bindings_review_sha256=digest(ROOT / PARENT / 'cli-bindings-review-dbe86521.json'),
                parent_run_metadata_sha256=digest(args.parent_run_metadata), freeze_sha256=digest(ROOT / FREEZE),
                collector_sha256=COLLECTOR_SHA, reviewer_sha256=digest(Path(__file__)),
                raw_evidence_sha256=hashes, processes=4, observations=summaries,
                accepted_collection_bindings=True, accepted_supplemental_warning_slice=True,
                accepted_original_six_case_slice=False, accepted_repair_proof=False,
                scope='Separate Mg/MgSO4 virtual pair with exact setup limitations and operator-scoped '
                      'backends. Original six-case results remain unchanged. Consumed Mg proves '
                      'prospective narration only; no empirical chemistry or integration acceptance.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('artifact', 'parent-artifact', 'parent-run-metadata', 'report'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.report.exists()
    result = review(args)
    with args.report.open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')


if __name__ == '__main__':
    main()
