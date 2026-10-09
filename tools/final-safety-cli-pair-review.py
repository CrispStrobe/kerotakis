#!/usr/bin/env python3
"""Verify paired actual-CLI warning observations, not integration or chemistry accuracy."""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
FREEZE = 'audits/production-final-safety-20261009/cli-warning-probes/freeze.json'
COLLECTOR = 'tools/final-safety-cli-probes-v2.py'
WORKFLOW = '.github/workflows/chemistry-audit.yml'
JOB = 'Replay unchanged hundred cases on lint-compatible production safety repair'


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


baseline = module('baseline', ROOT / 'tools/final-safety-cli-baseline-review.py')
cli = module('cli_binding', ROOT / 'tools/cli-binding-review.py')
probes = baseline.probes


def parsed_inputs(rows, positive):
    assert len(rows) == 4
    species = ['water', 'NaOCl' if positive else 'NaCl', 'NH4Cl']
    for i, key in enumerate(species):
        op = rows[i]['operator']
        assert op['op'] == 'add' and op['species'] == key and type(op['vessel']) is int and op['vessel'] == 0
        assert probes.finite(op['moles']) and op['moles'] > 0
        if i:
            assert op['moles'] == 1e-10
    op = rows[3]['operator']
    for key, value in {'op': 'titrate', 'vessel': 0, 'titrant': 'NaOH',
                       'concentration': 0.001, 'step': 0.001,
                       'target_ph': 12, 'max_steps': 1}.items():
        assert op.get(key) == value, key
    assert type(op['vessel']) is int and type(op['max_steps']) is int


def qualifying_observation(rows, process, text, positive, before):
    parsed_inputs(rows, positive)
    computed = probes.analyse(rows, process, positive, True)
    assert computed['classification'] == 'qualifying_inventory_and_warning_observation'
    assert all(v is True for v in computed['checks'].values())
    assert all(v is True for v in computed['warning_checks'].values())
    assert before['classification'] == 'qualifying_inventory_and_warning_observation'
    assert all(v is True for v in before['checks'].values())
    assert all(v is True for v in before['warning_checks'].values())
    # Require the declared same backend/model route, rather than borrowing a baseline's answer.
    for key, baseline_key in [('setup_solution', 'setup_backend'), ('final_solution', 'final_backend')]:
        actual = computed[key]['provenance']
        previous = before[baseline_key]
        for field in ('engine', 'dataset', 'model', 'routing'):
            assert actual[field] == previous[field] and actual[field], field
    assert 'titrated with 0.001 mol/L' in text and '1 step(s), 1.0 mL total' in text
    if positive:
        assert text.count(probes.HAZARD) == 1 and text.count(probes.REAL_WORLD) == 1
        lines = [line for line in text.splitlines() if probes.HAZARD in line]
        assert len(lines) == 1 and re.search(r'\bdanger\b', lines[0], re.IGNORECASE)
    else:
        assert 'chloramine' not in text.lower()
    return computed


def review(args):
    # Re-evaluate the actual baseline archive; a caller-authored accepted flag is insufficient.
    before = baseline.review(args.baseline_artifact, args.baseline_run_metadata,
                             args.baseline_harness, args.baseline_observations)
    executable = cli.review(args.artifact, args.run_metadata, args.harness,
                            args.source_manifest, args.results_review)
    # The CLI binding check establishes the archived evaluator's Git bytes.
    # Recompute all raw replay outcomes too; copying an accepted-looking report cannot suffice.
    replay_proof = args.report.with_name(args.report.stem + '-independent-replay.json')
    assert not replay_proof.exists()
    subprocess.run([
        sys.executable, str(args.artifact / 'harness/tools/combined-cli-replay.py'),
        '--review', '--build-binding', str(args.artifact / 'combined-build/binding.json'),
        '--source-manifest', str(args.artifact / 'harness' / args.source_manifest),
        '--out', str(args.artifact / 'combined-cli-replay'), '--report', str(replay_proof),
    ], check=True, timeout=120, capture_output=True)
    assert baseline.read(replay_proof) == baseline.read(args.results_review)
    run = baseline.read(args.run_metadata)
    active = [j for j in run['jobs'] if j['conclusion'] != 'skipped']
    assert len(active) == 1 and active[0]['name'] == JOB
    build = args.artifact / 'combined-build'
    binding = baseline.read(build / 'binding.json')
    manifest_bytes = baseline.git_bytes(args.harness, args.source_manifest)
    freeze_bytes = baseline.git_bytes(args.harness, FREEZE)
    assert freeze_bytes == baseline.git_bytes(args.baseline_harness, FREEZE)
    freeze = json.loads(freeze_bytes)
    assert before['source_binding']['compiled_source'] == freeze['prerepair_source']
    assert binding['compiled_source'] != freeze['prerepair_source']
    assert binding == executable['source_binding']
    evidence = args.artifact / 'probe-binding'
    assert (evidence / 'harness-commit.txt').read_text().strip() == args.harness
    expected = {COLLECTOR, FREEZE, WORKFLOW, args.source_manifest, *freeze['scripts']}
    hashes = {}
    for line in (evidence / 'harness-sha256.txt').read_text().splitlines():
        recorded, path = line.split(maxsplit=1)
        assert path.startswith('harness/')
        path = path[len('harness/'):]
        assert path in expected and path not in hashes
        data = baseline.git_bytes(args.harness, path)
        assert baseline.sha(data) == recorded
        if path != WORKFLOW:
            assert (args.artifact / 'harness' / path).read_bytes() == data
        hashes[path] = recorded
    assert set(hashes) == expected
    collection = baseline.read(args.artifact / 'cli-warning-probes/collection-binding.json')
    assert collection == {'build_binding_sha256': probes.digest(build / 'binding.json'),
                          'freeze_sha256': baseline.sha(freeze_bytes), 'scripts': freeze['scripts'],
                          'repair_source_manifest_sha256': baseline.sha(manifest_bytes)}
    observations = baseline.read(evidence / 'observations.json')
    for key in ('compiled_source', 'source_tree', 'binary_sha256'):
        assert observations[key] == binding[key]
    assert observations['build_binding_sha256'] == collection['build_binding_sha256']
    assert observations['freeze_sha256'] == collection['freeze_sha256']
    assert observations['repair_source_manifest_sha256'] == collection['repair_source_manifest_sha256']
    assert observations['accepted_repair_proof'] is False
    summary = {}
    for path, value in freeze['scripts'].items():
        name = Path(path).stem
        directory = args.artifact / 'cli-warning-probes' / name
        assert hashes[path] == value and probes.digest(directory / 'input.lab') == value
        processes = {}
        for mode in ('json', 'text'):
            target = directory / mode
            process = baseline.read(target / 'process.json')
            assert process['exit_code'] == 0 and process['timeout'] is False
            assert process['command'][0].endswith('/combined-build/kero')
            assert process['command'][1:] == ['run', 'cli-warning-probes/' + name + '/input.lab'] + (
                ['--json'] if mode == 'json' else [])
            for stream in ('stdout', 'stderr'):
                assert probes.digest(target / (stream + '.txt')) == process[stream + '_sha256']
            processes[mode] = process
        rows = [json.loads(line) for line in (directory / 'json/stdout.txt').read_text().splitlines()]
        # Recheck the baseline parser's input interpretation too, outside its original collector verdict.
        original_rows = [json.loads(line) for line in (
            args.baseline_artifact / 'cli-warning-probes' / name / 'json/stdout.txt').read_text().splitlines()]
        positive = name == 'introduced-chloramine'
        parsed_inputs(original_rows, positive)
        text = (directory / 'text/stdout.txt').read_text()
        computed = qualifying_observation(rows, processes['json'], text, positive, before['observations'][name])
        recorded = observations['observations'][name]
        assert {k: recorded[k] for k in computed} == computed
        assert recorded['text_process'] == processes['text']
        summary[name] = {'checks': computed['checks'], 'warning_checks': computed['warning_checks'],
                         'setup_inventory': computed['setup_inventory'], 'final_inventory': computed['final_inventory'],
                         'setup_backend': computed['setup_solution']['provenance'],
                         'final_backend': computed['final_solution']['provenance'],
                         'ordinary_text_full_dose_observed': True,
                         'ordinary_text_chloramine_warning_observed': positive,
                         'stream_sha256': {mode: {k: v for k, v in process.items() if k.endswith('_sha256')}
                                           for mode, process in processes.items()}}
    assert set(summary) == set(observations['observations'])
    return {'run': run['databaseId'], 'run_url': run['url'], 'harness': args.harness,
            'source_binding': binding, 'baseline_run': before['run'],
            'baseline_source_binding': before['source_binding'],
            'probe_harness_sha256': hashes, 'observations': summary,
            'cli_processes': executable['processes'], 'cli_counts': executable['counts'],
            'strict_stage_checks': executable['strict_stage_checks'],
            'run_metadata_sha256': probes.digest(args.run_metadata),
            'baseline_run_metadata_sha256': probes.digest(args.baseline_run_metadata),
            'baseline_corrected_observations_sha256': probes.digest(args.baseline_observations),
            'collector_observations_sha256': probes.digest(evidence / 'observations.json'),
            'independent_replay_sha256': probes.digest(replay_proof),
            'reviewer_sha256': probes.digest(Path(__file__)),
            'accepted_paired_cli_warning_slice': True, 'integration_acceptance': 'pending_separate_gates',
            'scope': 'Two frozen virtual engineering probes: independently bound pre-repair missing warning and repaired positive/benign JSON/text behavior, plus unchanged hundred-case replay. Independent source/build/lock identities, not a same-lock or empirical chemical accuracy claim. No broader rule/phase/capability-matrix/intermediate-product or historical full-audit acceptance.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--harness', required=True)
    parser.add_argument('--source-manifest', required=True)
    parser.add_argument('--results-review', type=Path, required=True)
    parser.add_argument('--baseline-artifact', type=Path, required=True)
    parser.add_argument('--baseline-run-metadata', type=Path, required=True)
    parser.add_argument('--baseline-harness', required=True)
    parser.add_argument('--baseline-observations', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    result = review(args)
    with args.report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False)
        output.write('\n')


if __name__ == '__main__':
    main()
