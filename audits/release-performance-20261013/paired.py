#!/usr/bin/env python3
"""Resource-gated comparison of reviewed release artifacts; no builds or network."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import time

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('frozen_baseline', HERE / 'profile.py')
baseline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(baseline)


def require(ok, message):
    if not ok:
        raise ValueError(message)


def load(path):
    return json.loads(path.read_bytes())


def gate(out, fast):
    available = int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines()
                         if line.startswith('MemAvailable:'))) // 1024
    value = dict(load1=os.getloadavg()[0], available_memory_mib=available,
                 fast_free_mib=shutil.disk_usage(fast).free // 1048576,
                 evidence_free_mib=shutil.disk_usage(out).free // 1048576,
                 unix_time=time.time())
    with (out / 'gates.jsonl').open('a') as stream:
        stream.write(json.dumps(value) + '\n')
    require(value['load1'] <= 3.5 and available >= 3200
            and value['fast_free_mib'] >= 4224 and value['evidence_free_mib'] >= 8192,
            'local execution resource gate closed')


def source_guard(repository, source, binding, binary, lock):
    tree = subprocess.check_output(['git', 'rev-parse', binding['head'] + '^{tree}'],
                                   cwd=repository).decode().strip()
    require(tree == binding['tree'], 'source tree binding differs')
    rows = []
    entries = subprocess.check_output(['git', 'ls-tree', '-rz', '--full-tree', binding['head']],
                                      cwd=repository).split(b'\0')
    for entry in entries:
        if not entry:
            continue
        meta, name = entry.split(b'\t', 1)
        mode, kind, oid = meta.decode().split()
        if kind == 'commit':
            continue
        path = source / os.fsdecode(name)
        raw = os.readlink(path).encode() if mode == '120000' else path.read_bytes()
        require(hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == oid,
                'source blob differs: ' + os.fsdecode(name))
        rows.append(dict(path=os.fsdecode(name), git_blob=oid,
                         sha256=hashlib.sha256(raw).hexdigest()))
    require(baseline.digest(binary) == binding['binary_sha256'], 'binary differs')
    require(baseline.digest(lock) == binding['lock_sha256'], 'actual artifact lock differs')
    return dict(head=binding['head'], tree=tree, tracked_parent_blobs=rows,
                binary_sha256=baseline.digest(binary), lock_sha256=baseline.digest(lock),
                recursive_submodule_files_rechecked=False)


def summary(samples):
    result = {}
    for field in ('parent_wall_seconds', 'wrapper_and_waited_child_user_seconds',
                  'wrapper_and_waited_child_system_seconds', 'max_rss_kib'):
        values = [sample[field] for sample in samples]
        median = statistics.median(values)
        result[field] = dict(median=median, minimum=min(values), maximum=max(values),
                            mad=statistics.median(abs(value - median) for value in values))
    return result


def compare(args):
    require(__debug__, 'optimized Python disables frozen profiler guards')
    gate(args.out, args.fast_volume)
    binding = load(args.bindings)
    plan = load(HERE / 'candidate-plan.json')
    freeze = load(HERE / 'freeze.json')
    manifest = load(HERE / 'manifest.json')
    reference = load(HERE / 'candidate-controls-baseline.json')
    performance_reference = load(HERE / 'paired-reference.json')
    paired_freeze = load(HERE / 'paired-freeze.json')
    for name, expected in paired_freeze['files_sha256'].items():
        require(baseline.digest(HERE / name) == expected, 'paired definition changed: ' + name)
    require(performance_reference['baseline_run'] == paired_freeze['reference_run']
            and performance_reference['baseline_profile_sha256'] == paired_freeze['reference_profile_sha256'],
            'paired reference identity changed')
    require(binding['independent_artifact_review_passed'] is True, 'artifact review not accepted')
    require(binding['same_build_environment_verified'] is True, 'build environment not matched')
    require(binding['baseline']['head'] == manifest['actual_binary_source_head']
            and binding['baseline']['tree'] == manifest['source_tree']
            and binding['baseline']['binary_sha256'] == manifest['baseline_binary_sha256'],
            'wrong baseline identity')
    require(binding['candidate']['head'] == plan['candidate_head']
            and binding['candidate']['tree'] == plan['candidate_tree'], 'wrong candidate identity')
    require(binding['candidate']['binary_sha256'] != binding['baseline']['binary_sha256'],
            'candidate must not reuse the baseline binary')
    require(binding['baseline']['lock_sha256'] == binding['candidate']['lock_sha256']
            == manifest['actual_fleet_lock_sha256'], 'dependency lock differs')
    require(binding['build_environment_review_receipt_sha256']
            and binding['artifact_review_receipt_sha256'], 'review receipt hashes required')
    reviews = {}
    for name, path in [('artifact', args.artifact_review), ('build_environment', args.build_review),
                       ('candidate_controls', args.candidate_controls_review)]:
        require(baseline.digest(path) == binding[name + '_review_receipt_sha256'],
                'review receipt bytes differ: ' + name)
        reviews[name] = load(path)
    require(reviews['artifact']['all_checks_pass'] is True, 'artifact review failed')
    for variant in ('baseline', 'candidate'):
        for field in ('head', 'tree', 'binary_sha256', 'lock_sha256'):
            require(reviews['artifact'][variant][field] == binding[variant][field],
                    'artifact review identity differs: ' + variant + ' ' + field)
    build = reviews['build_environment']
    require(build['same_build_environment_verified'] is True, 'build review failed')
    require(build['baseline_binary_sha256'] == binding['baseline']['binary_sha256']
            and build['candidate_binary_sha256'] == binding['candidate']['binary_sha256'],
            'build review binary identity differs')
    require(build['baseline_environment'] == build['candidate_environment'], 'build environments differ')
    environment = build['baseline_environment']
    for field in ('rustc', 'cargo', 'target', 'release_profile_sha256', 'native_toolchain'):
        require(environment.get(field), 'missing build environment witness: ' + field)
    require('rustflags' in environment and 'native_build_flags' in environment,
            'explicit build flag witnesses required, including empty flags')
    require(isinstance(environment['rustflags'], (str, list))
            and isinstance(environment['native_build_flags'], (str, list)), 'invalid build flag witnesses')
    controls = reviews['candidate_controls']
    require(controls['candidate_head'] == plan['candidate_head']
            and controls['candidate_binary_sha256'] == binding['candidate']['binary_sha256'],
            'candidate control review identity differs')
    require(controls['all_checks_pass'] is True
            and [row['id'] for row in controls['results']] == [row['id'] for row in plan['cases']]
            and all(row['passed'] is True for row in controls['results']),
            'candidate control review is incomplete or failed')
    require(controls['source_before'] == controls['source_after'], 'candidate control source guard differs')
    for field in ('head', 'tree', 'binary_sha256', 'lock_sha256'):
        require(controls['source_before'][field] == binding['candidate'][field],
                'candidate control identity differs: ' + field)
    require(controls['runner_sha256'] == paired_freeze['files_sha256']['controls.py']
            and controls['paired_freeze_sha256'] == baseline.digest(HERE / 'paired-freeze.json')
            and controls['candidate_plan_sha256'] == baseline.digest(HERE / 'candidate-plan.json')
            and controls['candidate_controls_reference_sha256'] == baseline.digest(HERE / 'candidate-controls-reference.json'),
            'candidate controls were not produced by frozen consumer and references')
    for name, expected in freeze['files_sha256'].items():
        require(baseline.digest(HERE / name) == expected, 'frozen baseline definition changed')
    require(baseline.digest(HERE / 'candidate-plan.json') == reference['candidate_plan_sha256'],
            'candidate plan changed after baseline controls')
    for name, expected in plan['files_sha256'].items():
        require(baseline.digest(HERE / name) == expected, 'frozen candidate control changed')
    before = {}
    for variant in ('baseline', 'candidate'):
        before[variant] = source_guard(args.repository, getattr(args, variant + '_source'),
                                       binding[variant], getattr(args, variant + '_binary'),
                                       getattr(args, variant + '_lock'))
    result = dict(schema_version=1, bindings_sha256=baseline.digest(args.bindings),
                  runner_sha256=baseline.digest(Path(__file__)), source_before=before,
                  reference_run=performance_reference['baseline_run'],
                  reference_profile_sha256=performance_reference['baseline_profile_sha256'],
                  paired_reference_sha256=baseline.digest(HERE / 'paired-reference.json'),
                  paired_freeze_sha256=baseline.digest(HERE / 'paired-freeze.json'),
                  workloads=[], comparison_complete=False,
                  scope='Local paired fresh-process release measurements with startup/serialization included. '
                        'Parent source blobs checked; module contents need independent artifact/source review. '
                        'Recorded build-environment receipt is required; this runner does not independently '
                        'reconstruct the compiler invocation. No warm-kernel or scientific accuracy claim.')

    def checkpoint():
        (args.out / 'comparison.json').write_text(json.dumps(result, indent=2) + '\n')

    def execute(variant, case, folder):
        gate(args.out, args.fast_volume)
        # The frozen measure helper uses blocking wait4 and a group watchdog.
        return baseline.measure(getattr(args, variant + '_binary').resolve(),
                                (HERE / case['script']).resolve(),
                                getattr(args, variant + '_source').resolve(), folder, 60)

    try:
        # A separately reviewed candidate-control receipt is required so neither
        # old baseline controls nor benchmark outputs substitute for candidate checks.
        require(binding['candidate_controls_review_passed'] is True,
                'candidate seven-control comparison not accepted')
        require(binding['candidate_controls_review_receipt_sha256'], 'candidate control receipt required')
        result['candidate_controls_review_receipt_sha256'] = binding['candidate_controls_review_receipt_sha256']
        for case in manifest['cases']:
            if case['id'] not in plan['performance_workloads']:
                continue
            folder = args.out / case['id']
            folder.mkdir()
            pilots, canonical = {}, {}
            workload = dict(id=case['id'], pilots=pilots, pilot_admission={}, warmups=[], paired_samples=[])
            result['workloads'].append(workload)
            for variant in ('baseline', 'candidate'):
                pilot_folder = folder / (variant + '-pilot')
                pilots[variant] = execute(variant, case, pilot_folder)
                checks = baseline.admission(case, (pilot_folder / 'stdout.jsonl').read_bytes(),
                                            (pilot_folder / 'stderr.log').read_bytes(), pilots[variant]['exit'])
                workload['pilot_admission'][variant] = checks
                checkpoint()
                require(checks['admitted'] and pilots[variant]['gnu_time_valid']
                        and not pilots[variant]['timed_out'], 'workload pilot not admitted: ' + case['id'])
                canonical[variant] = baseline.canonical_digest(pilot_folder / 'stdout.jsonl')
            require(canonical['baseline'] == canonical['candidate'], 'candidate pilot output changed')
            require(canonical['baseline'] == performance_reference['canonical_output_sha256'][case['id']],
                    'paired baseline differs from admitted hosted output')
            workload['canonical_output_sha256'] = canonical['baseline']
            checkpoint()
            for index, variant in enumerate(['baseline', 'candidate'] * 2):
                sample_folder = folder / ('warmup-' + str(index))
                sample = execute(variant, case, sample_folder)
                require(sample['exit'] == 0 and not sample['timed_out'] and sample['gnu_time_valid']
                        and sample['stderr_sha256'] == pilots[variant]['stderr_sha256']
                        and baseline.canonical_digest(sample_folder / 'stdout.jsonl') == canonical[variant],
                        'warmup output changed')
                workload['warmups'].append(dict(variant=variant, measurement=sample))
            order = ['baseline', 'candidate', 'candidate', 'baseline',
                     'candidate', 'baseline', 'baseline', 'candidate'] * 2
            for index, variant in enumerate(order):
                sample_folder = folder / ('paired-' + str(index))
                sample = execute(variant, case, sample_folder)
                item = dict(index=index, variant=variant, measurement=sample,
                            canonical_output_sha256=baseline.canonical_digest(sample_folder / 'stdout.jsonl'))
                workload['paired_samples'].append(item)
                checkpoint()
                require(sample['exit'] == 0 and not sample['timed_out'] and sample['gnu_time_valid']
                        and sample['stderr_sha256'] == pilots[variant]['stderr_sha256']
                        and item['canonical_output_sha256'] == canonical[variant], 'timed output changed')
            workload['statistics'] = {variant: summary([item['measurement'] for item in workload['paired_samples']
                                                        if item['variant'] == variant])
                                      for variant in ('baseline', 'candidate')}
            checkpoint()
        require(len(result['workloads']) == 4, 'paired workload inventory incomplete')
        result['comparison_complete'] = True
    except BaseException as error:
        result['failure'] = dict(type=type(error).__name__, message=str(error))
        raise
    finally:
        try:
            result['source_after'] = {variant: source_guard(args.repository, getattr(args, variant + '_source'),
                                                            binding[variant], getattr(args, variant + '_binary'),
                                                            getattr(args, variant + '_lock'))
                                      for variant in ('baseline', 'candidate')}
            require(result['source_after'] == before, 'post-run source or artifact changed')
        except BaseException as error:
            result['comparison_complete'] = False
            result['source_guard_failure'] = dict(type=type(error).__name__, message=str(error))
            raise
        finally:
            checkpoint()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for variant in ('baseline', 'candidate'):
        for field in ('binary', 'source', 'lock'):
            parser.add_argument('--' + variant + '-' + field, type=Path, required=True)
    for name in ('bindings', 'repository', 'fast-volume', 'out', 'artifact-review',
                 'build-review', 'candidate-controls-review'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    require(not args.out.exists(), 'exclusive evidence destination required')
    args.out.mkdir(parents=True)
    try:
        compare(args)
    except BaseException as error:
        (args.out / 'failure.json').write_text(json.dumps(dict(type=type(error).__name__,
                                                              message=str(error)), indent=2) + '\n')
        raise


if __name__ == '__main__':
    main()
