#!/usr/bin/env python3
"""Recompute baseline admission and statistics from saved evidence; never run kero."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent
EMPTY_SHA = hashlib.sha256(b'').hexdigest()
METRICS = ('parent_wall_seconds', 'wrapper_and_waited_child_user_seconds',
           'wrapper_and_waited_child_system_seconds', 'max_rss_kib')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_bytes(), parse_constant=reject_constant)


def reject_constant(value):
    raise ValueError('nonfinite JSON: ' + value)


def finite(value):
    if isinstance(value, float):
        require(math.isfinite(value), 'nonfinite number')
    elif isinstance(value, dict):
        for item in value.values():
            finite(item)
    elif isinstance(value, list):
        for item in value:
            finite(item)


def rows_and_digest(path):
    rows = [json.loads(line, parse_constant=reject_constant)
            for line in path.read_bytes().splitlines()]
    finite(rows)
    canonical = json.dumps(rows, sort_keys=True, separators=(',', ':'),
                           ensure_ascii=False, allow_nan=False).encode()
    return rows, hashlib.sha256(canonical).hexdigest()


def admitted(case, rows, stderr, measurement):
    reasons = []
    if measurement['exit'] != case['expected_exit'] or measurement['timed_out']:
        reasons.append('unsuccessful or timed-out process')
    if stderr:
        reasons.append('nonempty stderr')
    if len(rows) != case['expected_rows']:
        reasons.append('row inventory')
    events = []
    for row in rows:
        require(isinstance(row, dict) and isinstance(row.get('events'), list), 'invalid event row')
        require(isinstance(row.get('bench'), dict)
                and isinstance(row['bench'].get('vessels'), list), 'invalid bench row')
        for event in row['events']:
            require(isinstance(event, dict) and isinstance(event.get('event'), str), 'invalid event')
            events.append(event)
    kinds = {event['event'] for event in events}
    if kinds & {'solver_failed', 'safety_veto', 'not_yet_modeled', 'stock_exhausted'}:
        reasons.append('refused route')
    if not set(case['required_events']) <= kinds:
        reasons.append('missing completed event')
    for species, count in case['expected_added_counts'].items():
        if sum(e['event'] == 'added' and e.get('species') == species for e in events) != count:
            reasons.append('completed add inventory: ' + species)
    if case['require_endpoint_reached']:
        endpoints = [e for e in events if e['event'] == 'titrated']
        if len(endpoints) != 1 or endpoints[0].get('endpoint_reached') is not True:
            reasons.append('endpoint not reached')
    warnings = {e.get('rule') for e in events if e['event'] == 'hazard_warning'}
    if not set(case['target_warning_rules']) <= warnings:
        reasons.append('missing warning')
    if not measurement['gnu_time_valid']:
        reasons.append('invalid GNU time')
    return reasons


def measurement(folder, expected, case):
    actual = load(folder / 'measurement.json')
    require(all(actual[k] == v for k, v in expected.items()
                if k not in {'warmup', 'canonical_output_sha256', 'correctness_equal_to_pilot'}),
            'measurement differs from profile: ' + folder.name)
    require(sha(folder / 'stdout.jsonl') == actual['stdout_sha256'], 'stdout hash')
    require(sha(folder / 'stderr.log') == actual['stderr_sha256'], 'stderr hash')
    for field in METRICS:
        value = actual[field]
        require(type(value) in (float, int) and math.isfinite(value) and value >= 0, 'invalid ' + field)
    if actual['gnu_time_valid']:
        fields = list(map(float, (folder / 'gnu-time.txt').read_text().split()))
        require(len(fields) == 4 and all(math.isfinite(v) and v >= 0 for v in fields)
                and fields[3] > 0, 'invalid raw GNU time')
        require(dict(zip(('wall_seconds', 'user_seconds', 'system_seconds', 'max_rss_kib'), fields))
                == actual['gnu_time'], 'GNU time differs')
    command = load(folder / 'command.json')['argv']
    require(len(command) == 9 and command[:4] == ['/usr/bin/time', '-f', '%e %U %S %M', '-o']
            and command[6] == 'run' and command[8] == '--json'
            and Path(command[7]).name == Path(case['script']).name, 'unexpected invocation')
    return actual


def review(evidence, dispatch_path=HERE / 'dispatch.json'):
    freeze = load(HERE / 'freeze.json')
    dispatch = load(dispatch_path)
    require(sha(HERE / 'freeze.json') == dispatch['freeze_sha256'], 'freeze changed')
    for name, expected in freeze['files_sha256'].items():
        require(sha(HERE / name) == expected, 'frozen input changed: ' + name)
    manifest = load(HERE / 'manifest.json')
    profile = load(evidence / 'profile/profile.json')
    binding = load(evidence / 'artifact-source-binding.json')
    require(binding['hosted_run_id'] == str(dispatch['run'])
            and binding['hosted_run_attempt'] == '1'
            and binding['audit_revision'] == dispatch['audit_head'], 'hosted identity')
    for field, expected in [('accepted_main', manifest['accepted_main']),
                            ('accepted_tree', manifest['source_tree']),
                            ('artifact_source_head', manifest['actual_binary_source_head']),
                            ('binary_sha256', manifest['baseline_binary_sha256']),
                            ('actual_artifact_lock_sha256', manifest['actual_fleet_lock_sha256'])]:
        require(binding[field] == expected, 'artifact binding: ' + field)
    require(binding['source_tree_equivalent'] is True and binding['new_binary_built'] is False,
            'baseline build scope')
    require(sha(evidence / 'captured-artifact-Cargo.lock') == manifest['actual_fleet_lock_sha256'], 'lock bytes')
    require(sha(evidence / 'profile.py') == freeze['files_sha256']['profile.py'], 'executed profiler')
    require(profile['manifest_sha256'] == freeze['files_sha256']['manifest.json']
            and profile['profiler_sha256'] == freeze['files_sha256']['profile.py']
            and profile['binary_sha256'] == manifest['baseline_binary_sha256']
            and profile['actual_lock_sha256'] == manifest['actual_fleet_lock_sha256'], 'profile binding')
    require(profile['runner']['GITHUB_RUN_ID'] == str(dispatch['run'])
            and profile['runner']['GITHUB_RUN_ATTEMPT'] == '1'
            and profile['runner']['GITHUB_SHA'] == dispatch['audit_head'], 'profile runner')
    before, after = load(evidence / 'source-before.json'), load(evidence / 'source-after.json')
    require(before == after and before['head'] == manifest['accepted_main'], 'outer source guard')
    require(profile['source_before'] == profile['source_after']
            and profile['source_before']['head'] == manifest['accepted_main']
            and profile['source_before']['tree'] == manifest['source_tree'], 'profiler source guard')
    command = load(evidence / 'profiler-command.json')
    require(command['exit'] == 0
            and sha(evidence / 'profiler.stdout') == command['stdout_sha256']
            and sha(evidence / 'profiler.stderr') == command['stderr_sha256'] == EMPTY_SHA,
            'profiler completion')
    require([w['id'] for w in profile['workloads']] == [c['id'] for c in manifest['cases']], 'workload inventory')
    reports = []
    for case, workload in zip(manifest['cases'], profile['workloads']):
        folder = evidence / 'profile' / case['id']
        require(workload['script_sha256'] == case['script_sha256'], 'workload script')
        pilot = measurement(folder / 'pilot', workload['pilot'], case)
        try:
            rows, canonical = rows_and_digest(folder / 'pilot/stdout.jsonl')
            reasons = admitted(case, rows, (folder / 'pilot/stderr.log').read_bytes(), pilot)
        except (ValueError, KeyError, TypeError):
            canonical = None
            reasons = ['invalid structured pilot output']
        require(workload['pilot_admission']['admitted'] == (not reasons), 'pilot admission differs')
        if not reasons:
            require(workload['pilot_canonical_output_sha256'] == canonical, 'pilot canonical output')
        else:
            require(not workload['runs'], 'refused workload was timed')
        samples = []
        for index, sample in enumerate(workload['runs']):
            actual = measurement(folder / f'run-{index:02}', sample, case)
            try:
                _, sample_digest = rows_and_digest(folder / f'run-{index:02}/stdout.jsonl')
            except (ValueError, TypeError):
                sample_digest = None
            correct = (actual['exit'] == 0 and not actual['timed_out'] and actual['gnu_time_valid']
                       and actual['stderr_sha256'] == EMPTY_SHA and sample_digest == canonical)
            require(sample['canonical_output_sha256'] == sample_digest
                    and sample['correctness_equal_to_pilot'] == correct
                    and sample['warmup'] == (index < 2), 'sample correctness')
            if index >= 2 and correct:
                samples.append(actual)
        complete = not reasons and len(workload['runs']) == 10 and len(samples) == 8
        require(workload.get('complete_eight_timed_samples', False) == complete, 'completion differs')
        stats = {}
        for field in METRICS if samples else ():
            values = [s[field] for s in samples]
            median = statistics.median(values)
            stats[field] = dict(median=median, minimum=min(values), maximum=max(values),
                                mad=statistics.median(abs(v - median) for v in values))
        require(workload.get('statistics', {}) == stats, 'statistics differ')
        reports.append(dict(id=case['id'], pilot_admitted=not reasons, unmet=reasons,
                            complete_eight_timed_samples=complete, statistics=stats))
    complete = all(w['complete_eight_timed_samples'] for w in reports)
    require(profile['baseline_collection_complete'] == complete, 'overall completion differs')
    return dict(schema_version=1, run=dispatch['run'], audit_head=dispatch['audit_head'],
                freeze_sha256=dispatch['freeze_sha256'], profile_sha256=sha(evidence / 'profile/profile.json'),
                reviewer_sha256=sha(Path(__file__)), baseline_collection_complete=complete, workloads=reports,
                scope='Saved output admission and numeric summary recomputation. Source guards are checked as recorded evidence, not re-executed. Runner API/job/upload association and artifact ZIP/binary bytes need separate independent review. No optimization, empirical accuracy or speedup claim.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--dispatch', type=Path, default=HERE / 'dispatch.json',
                        help='Public binding for the specific baseline run being reviewed')
    args = parser.parse_args()
    result = review(args.evidence, args.dispatch)
    with args.out.open('x') as stream:
        stream.write(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(baseline_collection_complete=result['baseline_collection_complete'],
                          workloads=len(result['workloads']))))


if __name__ == '__main__':
    main()
