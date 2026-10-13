#!/usr/bin/env python3
"""Review saved paired measurements independently of the collection program."""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import shutil
import statistics


HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('saved_baseline_review', HERE / 'review.py')
saved = importlib.util.module_from_spec(spec)
spec.loader.exec_module(saved)
ORDER = ['baseline', 'candidate', 'candidate', 'baseline',
         'candidate', 'baseline', 'baseline', 'candidate'] * 2


def check(ok, message):
    if not ok:
        raise ValueError(message)


def gate(evidence):
    memory = int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines()
                      if line.startswith('MemAvailable:'))) // 1024
    record = dict(load1=os.getloadavg()[0], available_memory_mib=memory,
                  evidence_free_mib=shutil.disk_usage(evidence).free // 1048576)
    check(record['load1'] <= 3.5 and memory >= 3200 and record['evidence_free_mib'] >= 8192,
          'saved-output review resource gate closed')
    return record


def statistics_from(samples):
    result = {}
    for field in saved.METRICS:
        values = [row[field] for row in samples]
        check(values and all(type(v) in (int, float) and math.isfinite(v) and v >= 0
                             for v in values), 'invalid metric: ' + field)
        middle = statistics.median(values)
        result[field] = dict(median=middle, minimum=min(values), maximum=max(values),
                             mad=statistics.median(abs(value - middle) for value in values))
    return result


def review(args):
    check(__debug__, 'optimized Python disables review checks')
    gates = [gate(args.evidence)]
    protocol = args.protocol.resolve()
    freeze = saved.load(protocol / 'paired-freeze.json')
    for name, expected in freeze['files_sha256'].items():
        check(saved.sha(protocol / name) == expected, 'frozen definition differs: ' + name)
    plan = saved.load(protocol / 'candidate-plan.json')
    manifest = saved.load(protocol / 'manifest.json')
    reference = saved.load(protocol / 'paired-reference.json')
    binding = saved.load(args.bindings)
    report = saved.load(args.evidence / 'comparison.json')
    check(report['comparison_complete'] is True and not report.get('failure')
          and not report.get('source_guard_failure'), 'collection is incomplete or failed')
    check(report['bindings_sha256'] == saved.sha(args.bindings)
          and report['runner_sha256'] == freeze['files_sha256']['paired.py']
          and report['paired_freeze_sha256'] == saved.sha(protocol / 'paired-freeze.json')
          and report['paired_reference_sha256'] == saved.sha(protocol / 'paired-reference.json'),
          'collection definition identity differs')
    check(report['reference_run'] == reference['baseline_run'] == freeze['reference_run']
          and report['reference_profile_sha256'] == reference['baseline_profile_sha256']
          == freeze['reference_profile_sha256'], 'original baseline reference differs')
    check(binding['baseline']['head'] == manifest['actual_binary_source_head']
          and binding['baseline']['tree'] == manifest['source_tree']
          and binding['baseline']['binary_sha256'] == manifest['baseline_binary_sha256']
          and binding['candidate']['head'] == plan['candidate_head']
          and binding['candidate']['tree'] == plan['candidate_tree'], 'artifact/source binding differs')
    check(binding['baseline']['lock_sha256'] == binding['candidate']['lock_sha256']
          == manifest['actual_fleet_lock_sha256'], 'actual dependency lock differs')
    check(report['source_before'] == report['source_after'], 'recorded source guards differ')
    for variant in ('baseline', 'candidate'):
        for field in ('head', 'tree', 'binary_sha256', 'lock_sha256'):
            check(report['source_before'][variant][field] == binding[variant][field],
                  'recorded source identity differs: ' + variant + ' ' + field)
    reviews = {}
    for name, path in [('artifact', args.artifact_review), ('build_environment', args.build_review),
                       ('candidate_controls', args.controls_review)]:
        check(saved.sha(path) == binding[name + '_review_receipt_sha256'],
              'accepted prerequisite receipt bytes differ: ' + name)
        reviews[name] = saved.load(path)
    artifact = reviews['artifact']
    check(artifact['all_checks_pass'] is True, 'artifact review failed')
    for variant in ('baseline', 'candidate'):
        check(all(artifact[variant][key] == binding[variant][key]
                  for key in ('head', 'tree', 'binary_sha256', 'lock_sha256')),
              'artifact review identity differs: ' + variant)
    build = reviews['build_environment']
    check(build['same_build_environment_verified'] is True
          and build['baseline_environment'] == build['candidate_environment'], 'build review failed')
    check(all(build[variant + '_binary_sha256'] == binding[variant]['binary_sha256']
              for variant in ('baseline', 'candidate')), 'build review binary differs')
    environment = build['baseline_environment']
    check(all(environment.get(field) for field in
              ('rustc', 'cargo', 'target', 'release_profile_sha256', 'native_toolchain'))
          and all(field in environment and isinstance(environment[field], (str, list))
                  for field in ('rustflags', 'native_build_flags')), 'build witnesses missing')
    controls = reviews['candidate_controls']
    check(controls['all_checks_pass'] is True
          and [row['id'] for row in controls['results']] == [case['id'] for case in plan['cases']]
          and all(row['passed'] is True and not row['unmet'] for row in controls['results'])
          and controls['source_before'] == controls['source_after'], 'candidate controls not accepted')
    check(controls['runner_sha256'] == freeze['files_sha256']['controls.py']
          and controls['paired_freeze_sha256'] == report['paired_freeze_sha256']
          and controls['candidate_plan_sha256'] == saved.sha(protocol / 'candidate-plan.json')
          and controls['candidate_controls_reference_sha256']
          == saved.sha(protocol / 'candidate-controls-reference.json')
          and report['candidate_controls_review_receipt_sha256'] == saved.sha(args.controls_review),
          'candidate controls consumer differs')
    check(controls['candidate_head'] == binding['candidate']['head']
          and controls['candidate_binary_sha256'] == binding['candidate']['binary_sha256']
          and all(controls['source_before'][key] == binding['candidate'][key]
                  for key in ('head', 'tree', 'binary_sha256', 'lock_sha256')),
          'candidate controls source identity differs')
    workloads = {case['id']: case for case in manifest['cases']}
    check([work['id'] for work in report['workloads']] == plan['performance_workloads'],
          'workload inventory or order differs')
    hashes = {}
    results = []
    for work in report['workloads']:
        gates.append(gate(args.evidence))
        case = workloads[work['id']]
        folder = args.evidence / work['id']
        canonical = reference['canonical_output_sha256'][work['id']]
        check(work['canonical_output_sha256'] == canonical, 'original output reference differs')
        check(all(item['canonical_output_sha256'] == canonical for item in work['paired_samples']),
              'recorded timed canonical output differs')
        check([row['variant'] for row in work['paired_samples']] == ORDER
              and [row['index'] for row in work['paired_samples']] == list(range(16)),
              'paired order/inventory differs')
        check([row['variant'] for row in work['warmups']] == ['baseline', 'candidate'] * 2,
              'warmup order/inventory differs')
        samples = {'baseline': [], 'candidate': []}
        rows = [(variant + '-pilot', variant, work['pilots'][variant])
                for variant in ('baseline', 'candidate')]
        rows += [('warmup-' + str(i), row['variant'], row['measurement'])
                 for i, row in enumerate(work['warmups'])]
        rows += [('paired-' + str(i), row['variant'], row['measurement'])
                 for i, row in enumerate(work['paired_samples'])]
        for name, variant, expected in rows:
            sample_folder = folder / name
            actual = saved.measurement(sample_folder, expected, case)
            output, observed = saved.rows_and_digest(sample_folder / 'stdout.jsonl')
            unmet = saved.admitted(case, output, (sample_folder / 'stderr.log').read_bytes(), actual)
            check(not unmet,
                  'sample not admitted: ' + work['id'] + '/' + name)
            if name.endswith('-pilot'):
                check(work['pilot_admission'][variant]['admitted'] is True
                      and not work['pilot_admission'][variant]['unmet'], 'recorded pilot admission differs')
            check(observed == canonical, 'sample output changed: ' + work['id'] + '/' + name)
            command = saved.load(sample_folder / 'command.json')
            check(command['argv'][5] == args.runtime_map[variant]['binary']
                  and command['cwd'] == args.runtime_map[variant]['source']
                  and command['argv'][7] == args.runtime_map['scripts'][work['id']]
                  and command['argv'][4] == str(Path(args.runtime_map['comparison_root']) /
                                               work['id'] / name / 'gnu-time.txt'),
                  'exact recorded runtime invocation differs')
            for member in ('measurement.json', 'stdout.jsonl', 'stderr.log', 'gnu-time.txt', 'command.json'):
                hashes[work['id'] + '/' + name + '/' + member] = saved.sha(sample_folder / member)
            if name.startswith('paired-'):
                samples[variant].append(actual)
        computed = {variant: statistics_from(values) for variant, values in samples.items()}
        check(computed == work['statistics'], 'reported statistics differ: ' + work['id'])
        results.append(dict(id=work['id'], timed_samples_each=8, statistics=computed,
            median_ratios={field: (computed['candidate'][field]['median'] /
                          computed['baseline'][field]['median'])
                          if computed['baseline'][field]['median'] else None for field in saved.METRICS}))
    return dict(schema_version=1, saved_outputs_and_statistics_pass=True,
        reviewer_sha256=saved.sha(Path(__file__)), shared_saved_review_sha256=saved.sha(HERE / 'review.py'),
        comparison_sha256=saved.sha(args.evidence / 'comparison.json'),
        bindings_sha256=saved.sha(args.bindings), runtime_map_sha256=saved.sha(args.runtime_map_path),
        paired_freeze_sha256=saved.sha(protocol / 'paired-freeze.json'), runtime_map=args.runtime_map,
        prerequisite_receipts_sha256={name: binding[name + '_review_receipt_sha256'] for name in reviews},
        workloads=results, timed_samples=64, raw_evidence_sha256=hashes, gates=gates,
        speedup_accepted=False, merge_accepted=False,
        limitations=['Recorded source guards rechecked for equality/identity; source blobs not reread.',
                     'Accepted artifact/build/control receipts trusted externally; their bytes/identity checked.',
                     'Parent wall and waited-resource clocks cannot be independently remeasured from saved data.',
                     'Invocation map supplied by operating review; physical filesystem placement remains external.',
                     'Review recomputes output admission/equality and all statistics; no CLI is executed.',
                     'Ratios describe these fresh-process workloads; variance/regressions need separate acceptance.'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('evidence', 'protocol', 'bindings', 'artifact-review', 'build-review',
                 'controls-review', 'runtime-map', 'out'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    check(not args.out.exists(), 'exclusive review destination required')
    try:
        args.runtime_map_path = args.runtime_map
        args.runtime_map = saved.load(args.runtime_map)
        result = review(args)
    except Exception as error:
        args.out.write_text(json.dumps(dict(saved_outputs_and_statistics_pass=False,
                            failure=dict(type=type(error).__name__, message=str(error))), indent=2) + '\n')
        raise
    args.out.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')


if __name__ == '__main__':
    main()
