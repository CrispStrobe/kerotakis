#!/usr/bin/env python3
"""Bind the first archived actual-CLI safety baseline without executing Kero."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = 'audits/titration-trial-publication-20261009/cli-bindings-review-b9f998e2.json'
FREEZE = 'audits/production-final-safety-20261009/cli-warning-probes/freeze.json'
COLLECTOR = 'tools/final-safety-cli-probes.py'
WORKFLOW = '.github/workflows/chemistry-audit.yml'
JOB = 'Collect frozen final-safety CLI route baseline using verified executable'
SPEC = importlib.util.spec_from_file_location('probes', ROOT / COLLECTOR)
probes = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probes)


def git_bytes(harness, path):
    return subprocess.check_output(['git', 'show', harness + ':' + path], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text())


def review(artifact, run_path, harness, observation_path):
    run = read(run_path)
    assert run['headSha'] == harness and run['event'] == 'workflow_dispatch'
    assert run['status'] == 'completed' and run['conclusion'] == 'success'
    jobs = [j for j in run['jobs'] if j['conclusion'] != 'skipped']
    assert len(jobs) == 1 and jobs[0]['name'] == JOB and jobs[0]['conclusion'] == 'success'
    evidence = artifact / 'probe-binding'
    assert (evidence / 'harness-commit.txt').read_text().strip() == harness
    freeze = json.loads(git_bytes(harness, FREEZE))
    expected = {COLLECTOR, FREEZE, RECEIPT, WORKFLOW, *freeze['scripts']}
    hashes = {}
    for line in (evidence / 'harness-sha256.txt').read_text().splitlines():
        recorded, path = line.split(maxsplit=1)
        assert path.startswith('harness/')
        path = path[len('harness/'):]
        assert path in expected and path not in hashes
        data = git_bytes(harness, path)
        assert sha(data) == recorded
        if path != WORKFLOW:
            assert (artifact / 'harness' / path).read_bytes() == data
        hashes[path] = recorded
    assert set(hashes) == expected
    receipt = read(artifact / 'harness' / RECEIPT)
    assert receipt['accepted_executable_bindings'] is True and receipt['run'] == 37953313299
    parent = read(evidence / 'parent-run.json')
    assert parent['id'] == receipt['run'] and parent['head_sha'] == receipt['harness']
    assert parent['event'] == 'workflow_dispatch'
    assert parent['status'] == 'completed' and parent['conclusion'] == 'success'
    prior_artifact = read(evidence / 'parent-artifact.json')
    assert prior_artifact['id'] == 11627946038 and prior_artifact['expired'] is False
    assert prior_artifact['workflow_run']['id'] == receipt['run']
    assert prior_artifact['name'] == 'titration-publication-cli-replay-37953313299'
    build = artifact / 'prior-cli/combined-build'
    binding = read(build / 'binding.json')
    assert binding == receipt['source_binding']
    assert binding['compiled_source'] == freeze['prerepair_source']
    tree = subprocess.check_output(['git', 'rev-parse', binding['compiled_source'] + '^{tree}'], cwd=ROOT)
    assert tree.decode().strip() == binding['source_tree']
    for name, value in receipt['build_evidence_sha256'].items():
        assert Path(name).name == name and probes.digest(build / name) == value
    assert probes.digest(build / 'kero') == binding['binary_sha256']
    assert probes.digest(build / 'Cargo.lock') == binding['generated_lock_sha256']
    assert not (build / 'status-before.txt').read_text().strip()
    collection = read(artifact / 'cli-warning-probes/collection-binding.json')
    assert collection == {'build_binding_sha256': probes.digest(build / 'binding.json'),
                          'freeze_sha256': hashes[FREEZE], 'scripts': freeze['scripts']}
    observations = read(observation_path)
    for key in ('compiled_source', 'source_tree', 'binary_sha256'):
        assert observations[key] == binding[key]
    assert observations['build_binding_sha256'] == probes.digest(build / 'binding.json')
    assert observations['freeze_sha256'] == hashes[FREEZE]
    assert observations['accepted_repair_proof'] is False
    summary = {}
    for path, value in freeze['scripts'].items():
        assert hashes[path] == value
        name = Path(path).stem
        directory = artifact / 'cli-warning-probes' / name
        assert probes.digest(directory / 'input.lab') == value
        processes = {}
        for mode in ('json', 'text'):
            target = directory / mode
            process = read(target / 'process.json')
            assert process['exit_code'] == 0 and process['timeout'] is False
            command = process['command']
            assert command[0].endswith('/prior-cli/combined-build/kero')
            assert command[1:] == ['run', 'cli-warning-probes/' + name + '/input.lab'] + (
                ['--json'] if mode == 'json' else [])
            for stream in ('stdout', 'stderr'):
                assert probes.digest(target / (stream + '.txt')) == process[stream + '_sha256']
            processes[mode] = process
        rows = [json.loads(line) for line in (directory / 'json/stdout.txt').read_text().splitlines()]
        computed = probes.analyse(rows, processes['json'], name == 'introduced-chloramine', False)
        recorded = observations['observations'][name]
        assert {k: recorded[k] for k in computed} == computed
        assert recorded['text_process'] == processes['text']
        # These assertions concern this baseline's declared backend, not chemistry accuracy.
        provenance = [computed[key]['provenance'] for key in ('setup_solution', 'final_solution')]
        assert all(p['engine'] == 'PHREEQC (IPhreeqc, USGS)' for p in provenance)
        assert all(p.get('dataset') and p.get('model') and p.get('routing') for p in provenance)
        text = (directory / 'text/stdout.txt').read_text()
        assert 'titrated with 0.001 mol/L' in text and '1 step(s), 1.0 mL total' in text
        assert 'chloramine' not in text.lower()
        summary[name] = {'classification': computed['classification'], 'checks': computed['checks'],
                         'warning_checks': computed.get('warning_checks'),
                         'setup_inventory': computed['setup_inventory'],
                         'final_inventory': computed['final_inventory'],
                         'setup_backend': provenance[0], 'final_backend': provenance[1],
                         'stream_sha256': {mode: {k: v for k, v in process.items() if k.endswith('_sha256')}
                                           for mode, process in processes.items()},
                         'ordinary_text_full_dose_observed': True,
                         'ordinary_text_chloramine_warning_observed': False}
    assert set(summary) == set(observations['observations'])
    return {'run': run['databaseId'], 'run_url': run['url'], 'harness': harness,
            'source_binding': binding, 'harness_sha256': hashes,
            'run_metadata_sha256': probes.digest(run_path),
            'corrected_observations_sha256': probes.digest(observation_path),
            'original_collector_observations_sha256': probes.digest(evidence / 'observations.json'),
            'current_observation_reviewer_sha256': probes.digest(ROOT / COLLECTOR),
            'accepted_evidence_bindings': True, 'accepted_repair_proof': False,
            'observations': summary,
            'scope': 'Bound four-process pre-repair observations, re-evaluated with corrected zero-based CLI indexing. Original collector report is preserved. Positive final exposure is present without warning; negative has no warning. No repaired runtime or empirical chemical accuracy claim. Explicit capability-matrix coverage and intermediate hazardous products remain separate.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--run-metadata', type=Path, required=True)
    parser.add_argument('--harness', required=True)
    parser.add_argument('--observations', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    result = review(args.artifact, args.run_metadata, args.harness, args.observations)
    with args.report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False)
        output.write('\n')


if __name__ == '__main__':
    main()
