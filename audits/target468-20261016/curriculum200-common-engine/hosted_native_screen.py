#!/usr/bin/env python3
"""Execute the frozen curriculum200 protocol screen once using an authenticated prebuilt CLI."""
import argparse
from collections import Counter
import datetime
import hashlib
import json
import os
from pathlib import Path
import resource
import re
import signal
import subprocess
import time

HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


def limits():
    resource.setrlimit(resource.RLIMIT_AS, (1024**3, 1024**3))
    resource.setrlimit(resource.RLIMIT_CPU, (15, 16))
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024**2, 1024**2))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--verify-only', action='store_true')
    args = parser.parse_args()
    design = json.loads((HERE / 'native-screen-design.json').read_text())
    bundle = args.artifact.resolve() / 'rest-native'
    manifest = json.loads((bundle / 'manifest.json').read_text())
    for key, value in design['engine'].items():
        if manifest[key] != value:
            raise ValueError(f'Producer identity mismatch: {key}')
    if sha(bundle / 'kero') != manifest['binary_sha256']:
        raise ValueError('Binary checksum mismatch')
    for name, expected in manifest['libraries'].items():
        if Path(name).name != name or sha(bundle / 'lib' / name) != expected:
            raise ValueError('Runtime library checksum mismatch')
    paths = []
    artifact_names = set()
    for case in design['cases']:
        if not re.fullmatch('[a-zA-Z0-9_:-]{1,100}', case['id']):
            raise ValueError('Unexpected case identifier')
        name = case['id'].replace(':', '_')
        if name in artifact_names:
            raise ValueError('Colliding artifact directory name')
        artifact_names.add(name)
        script = (HERE / case['selected_script']).resolve()
        if not script.is_relative_to(HERE) or sha(script) != case['selected_script_sha256']:
            raise ValueError('Selected protocol path/hash mismatch')
        paths.append(script)
    if len(paths) != 200 or len({c['id'] for c in design['cases']}) != 200:
        raise ValueError('Require exactly 200 unique frozen cases')
    if args.verify_only:
        print('Authenticated producer binary/libraries and all200 frozen protocols; no process execution.')
        return
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out / 'design.json', design)
    save(args.out / 'producer-manifest.json', manifest)
    save(args.out / 'binding.json', {
        'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'design_sha256': sha(HERE / 'native-screen-design.json'),
        'runner_sha256': sha(Path(__file__)),
        'workflow_commit': os.environ.get('GITHUB_SHA'),
        'run_id': os.environ.get('GITHUB_RUN_ID'),
        'input_checkout_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
        'engine': design['engine'], 'local_vps_engine_execution': False,
        'lock_qualification': 'Producer-reported manifest hash; artifact omits raw Cargo.lock bytes.'})
    loader = bundle / 'lib' / manifest['loader']
    loader.chmod(0o700)
    (bundle / 'kero').chmod(0o700)
    results = []
    for case, script in zip(design['cases'], paths):
        target = args.out / case['id'].replace(':', '_')
        target.mkdir(exist_ok=False)
        (target / 'experiment.lab').write_bytes(script.read_bytes())
        start = time.monotonic()
        timed_out = False
        with (target / 'stdout.jsonl').open('xb') as stdout, (target / 'stderr.txt').open('xb') as stderr:
            process = subprocess.Popen(
                [str(loader), '--library-path', str(bundle / 'lib'), str(bundle / 'kero'),
                 'run', str(script), '--json'], stdout=stdout, stderr=stderr,
                start_new_session=True, preexec_fn=limits,
                env={'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', 'RUST_BACKTRACE': '0'})
            try:
                exit_code = process.wait(timeout=30)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(process.pid, signal.SIGKILL)
                exit_code = process.wait()
        parsed_count = 0
        parse_error = None
        try:
            for line in (target / 'stdout.jsonl').read_text().splitlines():
                if not line.strip():
                    continue
                item = json.loads(line, parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))
                if not isinstance(item, dict):
                    raise ValueError('Observation is not an object')
                parsed_count += 1
        except (ValueError, UnicodeError) as exc:
            parse_error = type(exc).__name__
        status = ('wall_timeout' if timed_out else 'output_limit' if exit_code == -signal.SIGXFSZ
                  else 'cpu_limit' if exit_code == -signal.SIGXCPU
                  else 'killed' if exit_code == -signal.SIGKILL
                  else 'zero_exit' if exit_code == 0 else 'nonzero_exit')
        row = {'id': case['id'], 'status': status, 'exit_code': exit_code,
               'elapsed_seconds': time.monotonic() - start,
               'script_sha256': sha(script), 'stdout_sha256': sha(target / 'stdout.jsonl'),
               'stderr_sha256': sha(target / 'stderr.txt'), 'parsed_observations': parsed_count,
               'stdout_parse_error': parse_error, 'physical_verdict': 'not_assigned'}
        save(target / 'execution.json', row)
        results.append(row)
        save(args.out / 'execution.json', {'results': results, 'engine': design['engine']})
        print(case['id'], status, parsed_count, flush=True)
    save(args.out / 'completion.json', {
        'completed_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'attempts': len(results), 'counts': dict(Counter(r['status'] for r in results)),
        'stdout_parse_errors': sum(r['stdout_parse_error'] is not None for r in results),
        'physical_verdict': 'not_assigned', 'retries': 0})


if __name__ == '__main__':
    main()
