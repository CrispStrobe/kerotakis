#!/usr/bin/env python3
"""Read saved seven-control streams; independently recompute their contracts."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import zipfile

ZIP_SHA = '45e9d561944e983c059f4cad70fd43adb7acbf5a63036e06ced7dce452ccd9a2'
HEAD = 'd06ffe3a1740728656878c4aea0e9507ca83c808'
TREE = '25d1dd490acc080a59f2a02e5b9399a22bd0f4bc'


def check(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    check(__debug__ and os.environ.get('GITHUB_ACTIONS') == 'true', 'real hosted runner required')
    check(os.environ.get('GITHUB_REPOSITORY') == 'CrispStrobe/kerotakis', 'repository differs')
    workspace = Path(os.environ['GITHUB_WORKSPACE'])
    definitions = workspace / 'audit/audits/release-performance-20261013'
    protocol = definitions / 'protocol-v2'
    out = workspace / 'control-raw-review-evidence'
    out.mkdir(exist_ok=False)
    def write(name, value):
        (out / name).write_text(json.dumps(value, indent=2) + '\n')
    try:
        spec = importlib.util.spec_from_file_location('readonly_resource_gate', definitions / 'paired.py')
        resource = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(resource)
        resource.gate(out, workspace)
        load = lambda path: json.loads(path.read_bytes())
        freeze = load(protocol / 'paired-freeze.json')
        for name, digest in freeze['files_sha256'].items():
            check(sha(protocol / name) == digest, 'frozen definition differs')
        plan = load(protocol / 'candidate-plan.json')
        reference = load(protocol / 'candidate-controls-reference.json')
        check(reference['candidate_plan_sha256'] == sha(protocol / 'candidate-plan.json'), 'reference plan differs')
        help_text = subprocess.check_output(['gh', 'api', '--help']).decode()
        flags = ['--allow-escape-sequences'] if '--allow-escape-sequences' in help_text else []
        def api(name, endpoint, binary=False):
            resource.gate(out, workspace)
            with (out / name).open('wb') as stream:
                subprocess.run(['gh', 'api', *flags, 'repos/CrispStrobe/kerotakis/' + endpoint],
                               stdout=stream, check=True, timeout=180)
            return None if binary else load(out / name)
        metadata = api('artifact-api.json', 'actions/artifacts/11821110986')
        run = api('control-run.json', 'actions/runs/38407663054')
        job = api('control-job.json', 'actions/jobs/115331419135')
        check(run['head_sha'] == 'f8ca430eeed82f4c9e78fe8db972f4786c130576'
              and run['run_attempt'] == 1 and run['conclusion'] == 'success', 'control run differs')
        check(job['run_id'] == run['id'] and job['head_sha'] == run['head_sha']
              and job['conclusion'] == 'success', 'control job differs')
        check(metadata['workflow_run']['id'] == run['id']
              and metadata['workflow_run']['head_sha'] == run['head_sha']
              and metadata['name'] == 'candidate-explanation-controls-38407663054-1'
              and metadata['digest'] == 'sha256:' + ZIP_SHA
              and metadata['size_in_bytes'] == 6830488 and not metadata['expired'], 'artifact differs')
        api('artifact.zip', 'actions/artifacts/11821110986/zip', True)
        check(sha(out / 'artifact.zip') == ZIP_SHA and (out / 'artifact.zip').stat().st_size == 6830488,
              'actual archive bytes differ')
        resource.gate(out, workspace)
        extracted = out / 'extracted'
        extracted.mkdir()
        with zipfile.ZipFile(out / 'artifact.zip') as archive:
            infos = archive.infolist()
            check(len(infos) <= 128 and sum(i.file_size for i in infos) <= 33554432,
                  'archive size budget exceeded')
            check(len({i.filename for i in infos}) == len(infos), 'duplicate members')
            for info in infos:
                path = PurePosixPath(info.filename)
                check(not path.is_absolute() and '..' not in path.parts
                      and not stat.S_ISLNK(info.external_attr >> 16), 'unsafe archive path')
                if info.is_dir():
                    continue
                check(stat.S_IFMT(info.external_attr >> 16) in (0, stat.S_IFREG), 'nonregular member')
                dest = extracted / path
                dest.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(info) as src, dest.open('wb') as target:
                    shutil.copyfileobj(src, target, 1048576)
                dest.chmod(0o400)
        identity = load(extracted / 'identity.json')
        check(identity['audit_head'] == run['head_sha'] and identity['run'] == str(run['id'])
              and identity['attempt'] == '1', 'saved run identity differs')
        result = load(extracted / 'controls/controls.json')
        artifact_review = load(definitions / 'candidate-artifact-independent-review-v2.json')
        check(sha(definitions / 'candidate-artifact-independent-review-v2.json')
              == '8ec124715750fbb0b316d7354f5b924ff9517ec6ff92ad7cdf59c879e0995c9c', 'review bytes differ')
        candidate = artifact_review['candidate']
        check(candidate['head'] == HEAD and candidate['tree'] == TREE, 'candidate identity differs')
        check(result['source_before'] == result['source_after'], 'recorded source guards differ')
        check(all(result['source_before'][k] == candidate[k]
                  for k in ('head', 'tree', 'binary_sha256', 'lock_sha256')), 'source binding differs')
        check(result['candidate_head'] == HEAD and result['candidate_binary_sha256'] == candidate['binary_sha256']
              and result['runner_sha256'] == freeze['files_sha256']['controls.py']
              and result['paired_freeze_sha256'] == sha(protocol / 'paired-freeze.json')
              and result['candidate_plan_sha256'] == sha(protocol / 'candidate-plan.json')
              and result['candidate_controls_reference_sha256'] == sha(protocol / 'candidate-controls-reference.json')
              and result['bindings_sha256'] == sha(extracted / 'bindings.json'), 'consumer identity differs')
        check(result['all_checks_pass'] is True and len(result['results']) == 7
              and [r['id'] for r in result['results']] == [c['id'] for c in plan['cases']], 'result inventory differs')
        command = load(extracted / 'control-command.json')['argv']
        check([item['id'] for item in reference['results']] == [case['id'] for case in plan['cases']],
              'baseline reference inventory differs')
        recorded_binary = command[command.index('--binary') + 1]
        recorded_source = command[command.index('--source') + 1]
        observations = []
        for case, reported, baseline in zip(plan['cases'], result['results'], reference['results']):
            resource.gate(out, workspace)
            folder = extracted / 'controls' / case['id']
            stdout, stderr = (folder / 'stdout').read_bytes(), (folder / 'stderr').read_bytes()
            argv = load(folder / 'command.json')['argv']
            check(argv[0] == recorded_binary and argv[1] == 'run' and argv[3:] == case['args']
                  and load(folder / 'command.json')['cwd'] == recorded_source
                  and argv[2].endswith('/candidate-controls/' + Path(case['script']).name), 'invocation differs')
            check(reported['exit'] == case['expected_exit'] and reported['timed_out'] is False
                  and reported['passed'] is True and reported['unmet'] == [], 'process result differs')
            check(sha(folder / 'stdout') == reported['stdout_sha256']
                  and sha(folder / 'stderr') == reported['stderr_sha256'], 'raw stream hash differs')
            check(bool(stderr) == (not case['expected_stderr_empty']), 'stderr expectation differs')
            if case.get('expected_stdout_empty'):
                check(not stdout, 'stdout should be empty')
            for field, present in [('stderr_required', True), ('stderr_forbidden', False)]:
                if case.get(field):
                    check((case[field].encode() in stderr) == present, 'diagnostic contract differs')
            canonical = None
            if '--json' in case['args'] and reported['exit'] == 0:
                def reject(value):
                    raise ValueError('nonfinite JSON token ' + value)
                rows = [json.loads(line, parse_constant=reject) for line in stdout.splitlines()]
                canonical = hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'),
                                            ensure_ascii=False, allow_nan=False).encode()).hexdigest()
                check(len(rows) == case['expected_rows']
                      and sum(row.get('operator', {}).get('op') == 'explain' for row in rows)
                      == case['expected_explain_rows'], 'row/explanation inventory differs')
                a, b = case['unchanged_bench_json_rows']
                check(rows[a]['bench'] == rows[b]['bench'], 'explanation changed bench')
            check(canonical == reported['canonical_stdout_sha256'], 'reported canonical hash differs')
            check(canonical == baseline['canonical_stdout_sha256'] if canonical is not None
                  else sha(folder / 'stdout') == baseline['stdout_sha256'], 'baseline output differs')
            normalized = stderr
            if case['expected_exit'] == 1:
                prefix = ('kero: ' + argv[2] + ':').encode()
                check(stderr.startswith(prefix), 'diagnostic prefix differs')
                normalized = b'kero: {script}:' + stderr[len(prefix):]
            check(hashlib.sha256(normalized).hexdigest() == baseline['normalized_stderr_sha256'],
                  'baseline diagnostic differs')
            observations.append(dict(id=case['id'], passed=True, stdout_sha256=sha(folder / 'stdout'),
                                     stderr_sha256=sha(folder / 'stderr'), canonical_stdout_sha256=canonical))
        proof = dict(saved_raw_control_review_pass=True, candidate=candidate, controls=observations,
                     underlying_controls_sha256=sha(extracted / 'controls/controls.json'),
                     input_artifact_zip_sha256=ZIP_SHA, input_run=run['id'],
                     paired_freeze_sha256=sha(protocol / 'paired-freeze.json'),
                     reviewer_sha256=sha(Path(__file__)), cli_executed=False, performance_measured=False,
                     source_blobs_independently_reread=False, recursive_native_modules_rechecked=False,
                     scope='Independent saved-output calculation, not an independent human/source-code audit. '
                           'Exit/watchdog/source guards checked as recorded; no runtime remeasurement. '
                           'Text equality preserves old quest transcript, not new privacy/science proof.')
        write('review.json', proof)
        print('SAVED_CONTROL_REVIEW_JSON=' + json.dumps(proof, sort_keys=True))
    except Exception as error:
        write('failure.json', dict(type=type(error).__name__, message=str(error), saved_raw_control_review_pass=False))
        raise


if __name__ == '__main__':
    main()
