"""Replay frozen, original forecasts through the real CLI, serially and with resource gates."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def resources():
    mem = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        key, value = line.split(':', 1)
        mem[key] = int(value.split()[0]) / 1024
    return dict(load1=os.getloadavg()[0], available_mib=mem['MemAvailable'],
                swap_free_mib=mem['SwapFree'], disk_free_mib=shutil.disk_usage(REPO).free / 2**20,
                system_free_mib=shutil.disk_usage('/').free / 2**20)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--suite', type=Path, default=ROOT)
    p.add_argument('--binary-source-commit')
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    binary = args.binary.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    suite = args.suite.resolve()
    temp = out / 'tmp'
    temp.mkdir(exist_ok=True)
    cli_env = dict(os.environ, TMPDIR=str(temp))
    manifest = json.loads((suite / 'freeze.json').read_text())
    assert sha(suite / 'predictions.json') == manifest['predictions_sha256']
    cases = json.loads((suite / 'predictions.json').read_text())
    assert len(cases) == manifest['count'] and len({c['id'] for c in cases}) == len(cases)
    known_binary = sha(binary) == '8c511a646bd5fa25f6842ce921e0ba96bc477816eb4d740ac920b1820b4fee66'
    receipt = dict(predictions_sha256=manifest['predictions_sha256'], binary_source_commit=args.binary_source_commit or ('b131444248625737a1edd5c02559fa50f1e83203' if known_binary else None), binary_validation_run='https://github.com/CrispStrobe/kerotakis/actions/runs/37048306794' if known_binary else None, binary=str(binary),
                   binary_sha256=sha(binary), checkout=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
                   policy=dict(max_load1=3.5, min_available_mib=3200, min_swap_free_mib=600, min_disk_free_mib=2048, min_system_free_mib=512),
                   started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), runs=[])
    for case in cases:
        script = suite / 'scripts' / (case['id'] + '.lab')
        assert script.read_text() == case['script']
        for mode in ('text', 'json'):
            if os.environ.get('GITHUB_ACTIONS') != 'true':
                while True:
                    r = resources()
                    if r['load1'] <= 3.5 and r['available_mib'] >= 3200 and r['swap_free_mib'] >= 600 and r['disk_free_mib'] >= 2048 and r['system_free_mib'] >= 512:
                        break
                    print('Waiting for resource headroom: ' + json.dumps(r), flush=True)
                    time.sleep(15)
            else:
                r = dict(hosted_runner=True)
            command = [str(binary), 'run', str(script)] + (['--json'] if mode == 'json' else [])
            started = time.monotonic()
            stdout = out / (case['id'] + '.' + mode + '.stdout')
            stderr = out / (case['id'] + '.' + mode + '.stderr')
            with stdout.open('wb') as a, stderr.open('wb') as b:
                try:
                    result = subprocess.run(['nice', '-n', '10'] + command, cwd=REPO, stdout=a, stderr=b, timeout=60, env=cli_env)
                    code = result.returncode
                except subprocess.TimeoutExpired:
                    code = 124
            receipt['runs'].append(dict(id=case['id'], mode=mode, command=command, exit_code=code,
                                        seconds=round(time.monotonic()-started, 3), resources=r,
                                        stdout=stdout.name, stderr=stderr.name, stdout_sha256=sha(stdout), stderr_sha256=sha(stderr)))
            (out / 'execution.json').write_text(json.dumps(receipt, indent=2) + '\n')
            print(f"{case['id']} {mode}: {code}", flush=True)
    receipt['completed_utc'] = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
    (out / 'execution.json').write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
