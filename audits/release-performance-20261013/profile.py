#!/usr/bin/env python3
"""Preserve admission pilots and serial exact-source release baseline measurements."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import statistics
import subprocess
import threading
import time

HERE = Path(__file__).resolve().parent


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def finite(value):
    if isinstance(value, float):
        assert math.isfinite(value), 'nonfinite JSON'
    elif isinstance(value, dict):
        for item in value.values():
            finite(item)
    elif isinstance(value, list):
        for item in value:
            finite(item)


def canonical_digest(path):
    rows = [json.loads(line, parse_constant=lambda v: (_ for _ in ()).throw(ValueError(v)))
            for line in path.read_bytes().splitlines()]
    finite(rows)
    raw = json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False, allow_nan=False).encode()
    return hashlib.sha256(raw).hexdigest()


def source_guard(source, manifest):
    def git(*args):
        return subprocess.check_output(['git', '-c', 'gc.auto=0', '-C', str(source), *args])
    assert git('rev-parse', 'HEAD').decode().strip() == manifest['accepted_main']
    assert git('rev-parse', 'HEAD^{tree}').decode().strip() == manifest['source_tree']
    count = 0
    for entry in git('ls-tree', '-rz', '--full-tree', 'HEAD').split(b'\0'):
        if not entry:
            continue
        meta, name = entry.split(b'\t', 1)
        mode, kind, oid = meta.decode().split()
        if kind == 'commit':
            continue
        path = source / os.fsdecode(name)
        raw = os.readlink(path).encode() if mode == '120000' else path.read_bytes()
        assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == oid, os.fsdecode(name)
        count += 1
    return dict(head=manifest['accepted_main'], tree=manifest['source_tree'], verified_regular_blobs=count,
                submodule_status=git('submodule', 'status', '--recursive').decode().splitlines(),
                submodule_regular_blobs_rechecked=False)


def admission(case, stdout, stderr, code):
    unmet = []
    if code != case['expected_exit']:
        unmet.append('exit differs')
    if stderr:
        unmet.append('stderr is not empty')
    rows = []
    try:
        for line in stdout.splitlines():
            row = json.loads(line, parse_constant=lambda v: (_ for _ in ()).throw(ValueError(v)))
            finite(row)
            assert isinstance(row, dict) and isinstance(row['events'], list)
            assert isinstance(row['bench']['vessels'], list)
            assert all(isinstance(e, dict) and isinstance(e.get('event'), str) for e in row['events'])
            rows.append(row)
    except (ValueError, AssertionError, TypeError, KeyError) as error:
        unmet.append('invalid structured output: ' + str(error))
    if len(rows) != case['expected_rows']:
        unmet.append('row inventory differs')
    events = [e for row in rows for e in row['events']]
    kinds = {e.get('event') for e in events}
    for required in case['required_events']:
        if required not in kinds:
            unmet.append('missing completed event: ' + required)
    if kinds & {'solver_failed', 'safety_veto', 'not_yet_modeled', 'stock_exhausted'}:
        unmet.append('refused or failed route is not an admitted workload')
    for species, expected in case.get('expected_added_counts', {}).items():
        if sum(e.get('event') == 'added' and e.get('species') == species for e in events) != expected:
            unmet.append('completed add inventory differs: ' + species)
    if case.get('require_endpoint_reached'):
        finished = [e for e in events if e.get('event') == 'titrated']
        if len(finished) != 1 or finished[0].get('endpoint_reached') is not True:
            unmet.append('titration endpoint was not reached')
    rules = {e.get('rule') for e in events if e.get('event') == 'hazard_warning'}
    for rule in case['target_warning_rules']:
        if rule not in rules:
            unmet.append('missing expected warning: ' + rule)
    return dict(admitted=not unmet, unmet=unmet, rows=len(rows), event_counts={k: sum(e.get('event') == k for e in events) for k in sorted(kinds)})


def measure(binary, script, source, folder, seconds):
    folder.mkdir()
    command = ['/usr/bin/time', '-f', '%e %U %S %M', '-o', str(folder / 'gnu-time.txt'),
               str(binary), 'run', str(script), '--json']
    (folder / 'command.json').write_text(json.dumps(dict(argv=command, cwd=str(source)), indent=2) + '\n')
    expired = threading.Event()
    started = time.perf_counter()
    with (folder / 'stdout.jsonl').open('wb') as out, (folder / 'stderr.log').open('wb') as err:
        process = subprocess.Popen(command, cwd=source, stdout=out, stderr=err, start_new_session=True)
        def stop():
            expired.set()
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        watchdog = threading.Timer(seconds, stop)
        watchdog.start()
        try:
            _, status, usage = os.wait4(process.pid, 0)
            code = os.waitstatus_to_exitcode(status)
            process.returncode = code
            wall = time.perf_counter() - started
        finally:
            watchdog.cancel()
            watchdog.join()
    result = dict(exit=code, timed_out=expired.is_set(), parent_wall_seconds=wall,
                  wrapper_and_waited_child_user_seconds=usage.ru_utime,
                  wrapper_and_waited_child_system_seconds=usage.ru_stime,
                  max_rss_kib=usage.ru_maxrss,
                  gnu_time_valid=False,
                  stdout_sha256=digest(folder / 'stdout.jsonl'), stderr_sha256=digest(folder / 'stderr.log'))
    timing = folder / 'gnu-time.txt'
    if code == 0 and timing.exists():
        fields = timing.read_text().split()
        if len(fields) == 4:
            try:
                values = list(map(float, fields))
                if all(math.isfinite(x) and x >= 0 for x in values) and values[3] > 0:
                    result['gnu_time'] = dict(zip(['wall_seconds', 'user_seconds', 'system_seconds', 'max_rss_kib'], values))
                    result['gnu_time_valid'] = True
            except ValueError:
                pass
    (folder / 'measurement.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--hosted', action='store_true')
    args = parser.parse_args()
    if not __debug__:
        raise RuntimeError('Optimized Python would disable integrity guards')
    assert __debug__ and args.hosted and os.environ.get('GITHUB_ACTIONS') == 'true', 'Hosted only; local execution needs a separately reviewed resource gate'
    assert not args.out.exists(), 'Evidence destination must be exclusive'
    manifest = json.loads((HERE / 'manifest.json').read_bytes())
    freeze = json.loads((HERE / 'freeze.json').read_bytes())
    for name, expected in freeze['files_sha256'].items():
        assert digest(HERE / name) == expected, name
    assert digest(args.binary) == manifest['baseline_binary_sha256']
    assert digest(args.source / 'Cargo.lock') == manifest['actual_fleet_lock_sha256']
    args.out.mkdir(parents=True)
    before = source_guard(args.source, manifest)
    result = dict(schema_version=1, source_before=before, manifest_sha256=digest(HERE / 'manifest.json'),
                  profiler_sha256=digest(Path(__file__)), binary_sha256=digest(args.binary),
                  actual_lock_sha256=digest(args.source / 'Cargo.lock'),
                  runner={k: os.environ.get(k) for k in ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_SHA', 'RUNNER_OS', 'RUNNER_ARCH']},
                  scope='Own accepted release baseline only. Fresh-process startup and serialization included. No candidate speedup, warm-kernel, allocation attribution or browser claim.', workloads=[])
    for case in manifest['cases']:
        script = (HERE / case['script']).resolve()
        assert digest(script) == case['script_sha256']
        folder = args.out / case['id']
        folder.mkdir()
        pilot_folder = folder / 'pilot'
        pilot = measure(args.binary.resolve(), script, args.source.resolve(), pilot_folder, manifest['timeout_seconds'])
        checks = admission(case, (pilot_folder / 'stdout.jsonl').read_bytes(), (pilot_folder / 'stderr.log').read_bytes(), pilot['exit'])
        if not pilot['gnu_time_valid']:
            checks['admitted'] = False
            checks['unmet'].append('pilot measurement lacks valid GNU time metrics')
        workload = dict(id=case['id'], script_sha256=case['script_sha256'], pilot=pilot, pilot_admission=checks, runs=[])
        result['workloads'].append(workload)
        if checks['admitted'] and not pilot['timed_out']:
            workload['pilot_canonical_output_sha256'] = canonical_digest(pilot_folder / 'stdout.jsonl')
            for index in range(10):
                sample = measure(args.binary.resolve(), script, args.source.resolve(), folder / f'run-{index:02}', manifest['timeout_seconds'])
                sample['warmup'] = index < 2
                try:
                    sample['canonical_output_sha256'] = canonical_digest(folder / f'run-{index:02}' / 'stdout.jsonl')
                except (ValueError, TypeError, AssertionError):
                    sample['canonical_output_sha256'] = None
                sample['correctness_equal_to_pilot'] = sample['exit'] == 0 and not sample['timed_out'] and sample['gnu_time_valid'] and sample['canonical_output_sha256'] == workload['pilot_canonical_output_sha256'] and sample['stderr_sha256'] == pilot['stderr_sha256']
                workload['runs'].append(sample)
                if not sample['correctness_equal_to_pilot']:
                    break
            valid = [x for x in workload['runs'] if not x['warmup'] and x['correctness_equal_to_pilot']]
            workload['complete_eight_timed_samples'] = len(valid) == 8
            if valid:
                workload['statistics'] = {}
                for field in ['parent_wall_seconds', 'wrapper_and_waited_child_user_seconds', 'wrapper_and_waited_child_system_seconds', 'max_rss_kib']:
                    values = [x[field] for x in valid]
                    median = statistics.median(values)
                    workload['statistics'][field] = dict(median=median, minimum=min(values), maximum=max(values), mad=statistics.median(abs(x - median) for x in values))
        (args.out / 'profile.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps({k: v for k, v in workload.items() if k not in ['runs', 'pilot']}), flush=True)
    after = source_guard(args.source, manifest)
    assert before == after and digest(args.binary) == result['binary_sha256'] and digest(args.source / 'Cargo.lock') == result['actual_lock_sha256']
    result['source_after'] = after
    result['baseline_collection_complete'] = all(w.get('complete_eight_timed_samples', False) for w in result['workloads'])
    (args.out / 'profile.json').write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    main()
