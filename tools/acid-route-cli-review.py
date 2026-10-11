#!/usr/bin/env python3
"""Bind acid-route observations to their actual run and accepted parent executable."""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
FOLDER = 'audits/production-acid-projection-20261010/cli-acid-routes'
MANIFEST = 'audits/production-acid-projection-20261010/cli-source-dbe86521.json'
COLLECTOR = 'tools/acid-route-cli-probes.py'
WORKFLOW = '.github/workflows/chemistry-audit.yml'
PARENT_JOB = 'Replay unchanged hundred cases on directional acid reporting repair'
SPEC = importlib.util.spec_from_file_location('cli_binding', ROOT / 'tools/cli-binding-review.py')
cli = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(cli)


def read(path):
    return json.loads(path.read_text())


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, data):
    with path.open('x') as stream:
        json.dump(data, stream, indent=2, allow_nan=False)
        stream.write('\n')


def recheck(script, arguments):
    # -B alone still reads old bytecode. Use a fresh cache namespace and
    # isolated Python so re-evaluation executes the verified source files.
    with tempfile.TemporaryDirectory(prefix='kero-acid-review-') as cache:
        subprocess.run([sys.executable, '-I', '-B', '-X', 'pycache_prefix=' + cache,
                        str(script), *arguments], check=True, capture_output=True, timeout=120)


def completed_job(run, head, name):
    assert run['headSha'] == head and run['event'] == 'workflow_dispatch'
    assert run['status'] == 'completed' and run['conclusion'] == 'success'
    active = [j for j in run['jobs'] if j['conclusion'] != 'skipped']
    assert len(active) == 1 and active[0]['name'] == name
    assert active[0]['status'] == 'completed' and active[0]['conclusion'] == 'success'
    return active[0]


def parent_metadata(directory, prepared):
    run = read(directory / 'run.json')
    assert run['id'] == prepared['parent_run'] and run['head_sha'] == prepared['parent_harness']
    assert run['event'] == 'workflow_dispatch'
    assert run['status'] == 'completed' and run['conclusion'] == 'success'
    jobs = read(directory / 'jobs.json')
    assert jobs['total_count'] == len(jobs['jobs'])
    completed_job(dict(headSha=run['head_sha'], event=run['event'], status=run['status'],
                       conclusion=run['conclusion'], jobs=jobs['jobs']), run['head_sha'], PARENT_JOB)
    artifacts = read(directory / 'artifacts.json')
    assert artifacts['total_count'] == len(artifacts['artifacts'])
    selected = [a for a in artifacts['artifacts'] if a['name'] == prepared['parent_artifact']]
    assert len(selected) == 1
    artifact = selected[0]
    assert artifact['expired'] is False and artifact == read(directory / 'artifact.json')
    assert artifact['workflow_run']['id'] == run['id']
    assert artifact['workflow_run']['head_sha'] == run['head_sha']
    assert (directory / 'artifact-id.txt').read_text().strip() == str(artifact['id'])
    assert artifact['digest'] == 'sha256:' + digest(directory / 'archive.zip')
    return run, artifact


def inherited_build(build, parent):
    expected = parent['build_evidence_sha256']
    assert {p.name for p in build.iterdir() if p.is_file()} == set(expected)
    assert all(digest(build / name) == sha for name, sha in expected.items())
    assert read(build / 'binding.json') == parent['source_binding']


def archive_matches_parent(archive, parent):
    # Verify the API-bound ZIP against the independently reviewed original
    # parent archive without creating another large extraction.
    def cache(name):
        return '__pycache__' in name.parts and name.suffix == '.pyc'

    with zipfile.ZipFile(archive) as zipped:
        seen = set()
        for member in zipped.infolist():
            name = PurePosixPath(member.filename)
            assert not name.is_absolute() and '..' not in name.parts
            assert member.filename not in seen
            seen.add(member.filename)
            if member.is_dir() or cache(name):
                continue
            with zipped.open(member) as stream:
                sha = hashlib.file_digest(stream, 'sha256').hexdigest()
            assert digest(parent / name) == sha, member.filename
        # Offline Python review can regenerate bytecode caches. The API digest
        # still binds the complete original ZIP; compare every non-cache file,
        # including all source, lock, executable and raw process evidence.
        assert {str(p.relative_to(parent)) for p in parent.rglob('*')
                if p.is_file() and not cache(p.relative_to(parent))} == {
            m.filename for m in zipped.infolist()
            if not m.is_dir() and not cache(PurePosixPath(m.filename))}


def inputs(artifact, prepared, freeze):
    expected = {WORKFLOW, COLLECTOR, MANIFEST, FOLDER + '/freeze.json',
                *(case['path'] for case in freeze['cases'])}
    actual = {}
    for line in (artifact / 'route-binding/harness-sha256.txt').read_text().splitlines():
        sha, path = line.split(maxsplit=1)
        assert path.startswith('harness/')
        path = path[len('harness/'):]
        assert path in expected and path not in actual
        data = cli.git('show', prepared['harness'] + ':' + path)
        assert hashlib.sha256(data).hexdigest() == sha
        if path != WORKFLOW:
            assert (artifact / 'harness' / path).read_bytes() == data
        actual[path] = sha
    assert set(actual) == expected
    assert actual[COLLECTOR] == prepared['collector_sha256']
    assert actual[FOLDER + '/freeze.json'] == prepared['freeze_sha256']
    workflow = cli.git('show', prepared['harness'] + ':' + WORKFLOW).decode()
    assert workflow.count('    name: ' + prepared['job_name'] + '\n') == 1
    return actual


def narration(text, case):
    metadata = case['expected_warning']
    count = 1 if case['role'] == 'positive' else 0
    # The frozen collector counted warning wording, but did not verify the
    # rendered severity label or dose narration. Check those independently.
    label = 'HAZARD (' + metadata['severity'].capitalize() + '): ' + metadata['hazard']
    return dict(exact_warning_wording=all(text.count(metadata[k]) == count
                                        for k in ('hazard', 'real_world')),
                exact_warning_severity=text.count(label) == count,
                exact_dose_narration=text.count('titrated with 1 mol/L hydrochloric acid') == 1
                                     and text.count('1 step(s), 10.0 mL total') == 1)


def review(args):
    prepared = read(ROOT / FOLDER / 'prepared-reuse.json')
    run = read(args.run_metadata)
    job = completed_job(run, prepared['harness'], prepared['job_name'])
    assert (args.artifact / 'route-binding/harness-commit.txt').read_text().strip() == prepared['harness']
    frozen_bytes = cli.git('show', prepared['harness'] + ':' + FOLDER + '/freeze.json')
    freeze = json.loads(frozen_bytes)
    assert hashlib.sha256(frozen_bytes).hexdigest() == prepared['freeze_sha256']
    bound_inputs = inputs(args.artifact, prepared, freeze)
    parent_run, parent_artifact = parent_metadata(args.artifact / 'parent-binding', prepared)
    parent = cli.review(args.parent_artifact, args.parent_run_metadata, prepared['parent_harness'],
                        MANIFEST, args.parent_results_review)
    assert parent == read(args.parent_bindings_review)
    assert parent['run'] == prepared['parent_run'] and parent['source_binding']['compiled_source'] == prepared['source']
    inherited_build(args.artifact / 'prior-cli/combined-build', parent)
    archive_matches_parent(args.artifact / 'parent-binding/archive.zip', args.parent_artifact)
    parent_pair = read(args.parent_pair_review)
    assert parent_pair['accepted_paired_cli_warning_slice'] is True
    assert parent_pair['run'] == prepared['parent_run'] and parent_pair['harness'] == prepared['parent_harness']
    assert parent_pair['source_binding'] == parent['source_binding']
    assert parent_pair['reviewer_sha256'] == digest(ROOT / 'tools/final-safety-cli-pair-review.py')
    assert parent_pair['run_metadata_sha256'] == digest(args.parent_run_metadata)
    assert parent_pair['collector_observations_sha256'] == digest(args.parent_artifact / 'probe-binding/observations.json')
    assert parent_pair['cli_counts'] == parent['counts'] and parent_pair['cli_processes'] == 164
    assert parent_pair['strict_stage_checks'] == parent['strict_stage_checks']
    assert set(parent_pair['observations']) == {'introduced-chloramine', 'benign-counterpart'}
    assert all(v is True for o in parent_pair['observations'].values()
               for group in ('checks', 'warning_checks') for v in o[group].values())
    parent_proof = args.report.with_name(args.report.stem + '-parent-replay.json')
    recheck(args.parent_artifact / 'harness/tools/combined-cli-replay.py', [
                    '--review', '--build-binding', str(args.parent_artifact / 'combined-build/binding.json'),
                    '--source-manifest', str(args.parent_artifact / 'harness' / MANIFEST),
                    '--out', str(args.parent_artifact / 'combined-cli-replay'), '--report', str(parent_proof)])
    assert read(parent_proof) == read(args.parent_results_review)
    assert digest(parent_proof) == parent_pair['independent_replay_sha256']
    proof = args.report.with_name(args.report.stem + '-observations.json')
    recheck(args.artifact / 'harness' / COLLECTOR, ['--review',
                    '--build-binding', str(args.artifact / 'prior-cli/combined-build/binding.json'),
                    '--source-manifest', str(args.artifact / 'harness' / MANIFEST),
                    '--out', str(args.artifact / 'acid-route-observations'), '--report', str(proof)])
    observed = read(proof)
    assert observed == read(args.artifact / 'route-binding/observations.json')
    assert observed['accepted_repair_proof'] is False and observed['source_binding'] == parent['source_binding']
    assert set(observed['observations']) == {Path(c['path']).stem for c in freeze['cases']}
    processes = 0
    summaries = {}
    expected_process_paths = set()
    for case in freeze['cases']:
        name = Path(case['path']).stem
        directory = args.artifact / 'acid-route-observations' / name
        for mode in ('json', 'text'):
            expected_process_paths.add(directory / mode / 'process.json')
            process = read(directory / mode / 'process.json')
            assert process['command'][0].endswith('/prior-cli/combined-build/kero')
            assert process['command'][1:] == ['run', 'acid-route-observations/' + name + '/input.lab'] + (
                ['--json'] if mode == 'json' else [])
            processes += 1
        result = observed['observations'][name]
        # Public receipt omits full raw vessel/runner records and private paths.
        summaries[name] = {k: result[k] for k in ('classification', 'checks', 'warning_checks',
                          'setup_partner_mol', 'final_partner_mol', 'final_surviving_exposure') if k in result}
        text_checks = narration((directory / 'text/stdout.txt').read_text(), case)
        summaries[name]['independent_text_checks'] = text_checks
        if result['classification'].startswith('qualifying_') and not all(text_checks.values()):
            summaries[name]['collector_classification'] = result['classification']
            summaries[name]['classification'] = 'text_warning_contract_mismatch'
    assert processes == freeze['expected_processes'] == prepared['expected_processes']
    assert set((args.artifact / 'acid-route-observations').rglob('process.json')) == expected_process_paths
    allowed = {'positive': {'qualifying_named_to_dissolved_projection',
                            'qualifying_prospective_only_partner_consumed'},
               'benign': {'qualifying_benign_counterpart'}}
    warning_slice = all(summaries[Path(c['path']).stem]['classification'] in allowed[c['role']]
                        for c in freeze['cases'])
    return dict(run=run['databaseId'], run_url=run['url'], job=job['databaseId'],
                harness=prepared['harness'], source_binding=parent['source_binding'],
                parent_run=parent_run['id'], parent_artifact_id=parent_artifact['id'],
                parent_archive_digest=parent_artifact['digest'], harness_sha256=bound_inputs,
                parent_bindings_review_sha256=digest(args.parent_bindings_review),
                parent_pair_review_sha256=digest(args.parent_pair_review),
                run_metadata_sha256=digest(args.run_metadata), observations_sha256=digest(proof),
                reviewer_sha256=digest(Path(__file__)),
                processes=processes, observations=summaries,
                classification_counts=dict(Counter(o['classification'] for o in summaries.values())),
                accepted_collection_bindings=True, accepted_declared_warning_slice=warning_slice,
                scope='Six source-informed virtual prospective/projection probes. Non-exercising cases and '
                      'consumed partners retain their categories. No empirical chemistry, newly introduced '
                      'final-only exposure, intermediate products or integration acceptance.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('artifact', 'run-metadata', 'parent-artifact', 'parent-run-metadata',
                 'parent-results-review', 'parent-bindings-review', 'parent-pair-review', 'report'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.report.exists()
    save(args.report, review(args))


if __name__ == '__main__':
    main()
