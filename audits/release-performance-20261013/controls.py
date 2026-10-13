#!/usr/bin/env python3
"""Check the candidate's seven frozen controls against saved baseline streams."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import threading

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('paired_review', HERE / 'paired.py')
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)


def normalized_stderr(raw, script, case):
    if case['expected_exit'] == 1:
        prefix = ('kero: ' + str(script) + ':').encode()
        paired.require(raw.startswith(prefix), 'invalid-vessel diagnostic prefix differs')
        return b'kero: {script}:' + raw[len(prefix):]
    return raw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ('binary', 'source', 'lock', 'bindings', 'artifact-review',
                  'repository', 'fast-volume', 'out'):
        parser.add_argument('--' + field, type=Path, required=True)
    args = parser.parse_args()
    paired.require(not args.out.exists(), 'exclusive evidence destination required')
    args.out.mkdir(parents=True)
    result = dict(schema_version=1, results=[], all_checks_pass=False,
                  scope='Candidate control output comparison only; no performance claim. '
                        'Invalid-vessel stderr normalizes only the exact script argument prefix. '
                        'Text references preserve baseline behavior; independent quest tests still required.')

    def checkpoint():
        (args.out / 'controls.json').write_text(json.dumps(result, indent=2) + '\n')

    before, binding = None, None
    try:
        paired.require(__debug__, 'optimized Python disables frozen integrity checks')
        paired.gate(args.out, args.fast_volume)
        freeze = paired.load(HERE / 'paired-freeze.json')
        for name, expected in freeze['files_sha256'].items():
            paired.require(paired.baseline.digest(HERE / name) == expected, 'paired definition changed: ' + name)
        plan = paired.load(HERE / 'candidate-plan.json')
        reference = paired.load(HERE / 'candidate-controls-reference.json')
        paired.require(reference['candidate_plan_sha256'] == paired.baseline.digest(HERE / 'candidate-plan.json'),
                       'control reference plan differs')
        for name, expected in plan['files_sha256'].items():
            paired.require(paired.baseline.digest(HERE / name) == expected, 'control script differs')
        binding = paired.load(args.bindings)
        paired.require(binding['independent_artifact_review_passed'] is True, 'artifact review not accepted')
        paired.require(paired.baseline.digest(args.artifact_review) == binding['artifact_review_receipt_sha256'],
                       'artifact review receipt bytes differ')
        artifact = paired.load(args.artifact_review)
        paired.require(artifact['all_checks_pass'] is True, 'artifact review failed')
        for field in ('head', 'tree', 'binary_sha256', 'lock_sha256'):
            paired.require(artifact['candidate'][field] == binding['candidate'][field],
                           'artifact review candidate differs: ' + field)
        paired.require(binding['candidate']['head'] == plan['candidate_head']
                       and binding['candidate']['tree'] == plan['candidate_tree'], 'candidate source identity differs')
        before = paired.source_guard(args.repository, args.source, binding['candidate'], args.binary, args.lock)
        result.update(candidate_head=plan['candidate_head'], candidate_binary_sha256=before['binary_sha256'],
                      candidate_plan_sha256=paired.baseline.digest(HERE / 'candidate-plan.json'),
                      candidate_controls_reference_sha256=paired.baseline.digest(HERE / 'candidate-controls-reference.json'),
                      source_before=before, bindings_sha256=paired.baseline.digest(args.bindings),
                      runner_sha256=paired.baseline.digest(Path(__file__)),
                      paired_freeze_sha256=paired.baseline.digest(HERE / 'paired-freeze.json'))
        paired.require([r['id'] for r in reference['results']] == [c['id'] for c in plan['cases']],
                       'baseline control inventory differs')
        for case, expected in zip(plan['cases'], reference['results']):
            paired.gate(args.out, args.fast_volume)
            folder = args.out / case['id']
            folder.mkdir()
            script = (HERE / case['script']).resolve()
            argv = [str(args.binary.resolve()), 'run', str(script), *case['args']]
            (folder / 'command.json').write_text(json.dumps(dict(argv=argv, cwd=str(args.source.resolve())),
                                                          indent=2) + '\n')
            expired = threading.Event()
            with (folder / 'stdout').open('wb') as stdout, (folder / 'stderr').open('wb') as stderr:
                child = subprocess.Popen(argv, cwd=args.source, stdout=stdout, stderr=stderr, start_new_session=True)

                def stop():
                    expired.set()
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass

                timer = threading.Timer(60, stop)
                timer.start()
                try:
                    code = child.wait()
                finally:
                    timer.cancel()
                    timer.join()
            stdout = (folder / 'stdout').read_bytes()
            stderr = (folder / 'stderr').read_bytes()
            unmet, canonical = [], None
            if code != case['expected_exit'] or expired.is_set():
                unmet.append('exit or timeout differs')
            if (not stderr) != case['expected_stderr_empty']:
                unmet.append('stderr expectation differs')
            if case.get('expected_stdout_empty') and stdout:
                unmet.append('stdout must be empty')
            for field, present in [('stderr_required', True), ('stderr_forbidden', False)]:
                if case.get(field) and (case[field].encode() in stderr) != present:
                    unmet.append(field + ' differs')
            try:
                if '--json' in case['args'] and code == 0:
                    rows = [json.loads(line) for line in stdout.splitlines()]
                    paired.baseline.finite(rows)
                    canonical = paired.baseline.canonical_digest(folder / 'stdout')
                    if len(rows) != case['expected_rows']:
                        unmet.append('row inventory differs')
                    if sum(r.get('operator', {}).get('op') == 'explain' for r in rows) != case['expected_explain_rows']:
                        unmet.append('explanation inventory differs')
                    a, b = case['unchanged_bench_json_rows']
                    if rows[a]['bench'] != rows[b]['bench']:
                        unmet.append('explanation changed bench')
                if expected['canonical_stdout_sha256'] is not None:
                    if canonical != expected['canonical_stdout_sha256']:
                        unmet.append('canonical baseline output differs')
                elif paired.baseline.digest(folder / 'stdout') != expected['stdout_sha256']:
                    unmet.append('exact baseline text differs')
                normalized = normalized_stderr(stderr, script, case)
                if hashlib.sha256(normalized).hexdigest() != expected['normalized_stderr_sha256']:
                    unmet.append('baseline stderr differs')
            except (ValueError, TypeError, KeyError, IndexError, AssertionError) as error:
                unmet.append('invalid output: ' + str(error))
            result['results'].append(dict(id=case['id'], passed=not unmet, unmet=unmet, exit=code,
                                          timed_out=expired.is_set(), canonical_stdout_sha256=canonical,
                                          stdout_sha256=paired.baseline.digest(folder / 'stdout'),
                                          stderr_sha256=paired.baseline.digest(folder / 'stderr')))
            checkpoint()
        result['all_checks_pass'] = len(result['results']) == 7 and all(r['passed'] for r in result['results'])
    except BaseException as error:
        result['failure'] = dict(type=type(error).__name__, message=str(error))
        raise
    finally:
        try:
            if before is not None:
                result['source_after'] = paired.source_guard(args.repository, args.source, binding['candidate'],
                                                             args.binary, args.lock)
                paired.require(result['source_after'] == before, 'candidate source or artifact changed')
        except BaseException as error:
            result['all_checks_pass'] = False
            result['source_guard_failure'] = dict(type=type(error).__name__, message=str(error))
            raise
        finally:
            checkpoint()
    print(json.dumps(dict(all_checks_pass=result['all_checks_pass'], controls=len(result['results']))))
    if not result['all_checks_pass']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
