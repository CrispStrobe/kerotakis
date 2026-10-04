"""Matched hosted CLI wall/CPU/RSS measurements; debug startup is included."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import signal
import threading
import subprocess
import time

ROOT = Path(__file__).resolve().parent


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--baseline', type=Path, required=True)
    p.add_argument('--candidate', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        p.error('Hosted only; local execution requires a separate resource gate')
    args.out.mkdir(parents=True, exist_ok=True)
    bins = {'baseline': args.baseline / 'kero', 'candidate': args.candidate / 'kero'}
    workloads = {i: ROOT / 'scripts' / (i + '.lab') for i in ['A01', 'A02', 'A11', 'A23', 'B02', 'B11', 'B16']}
    workloads['B20C'] = ROOT / 'followups/scripts/B20C.lab'
    for stages in [1, 16]:
        script = args.out / f'extract-{stages}.lab'
        script.write_text(f'new\nnew\nadd v1 water 100mL\nadd v1 I2 0.00001mol\nextract v1 v2 hexane 0.04mol stages {stages}\nmeasure v1 balance\nmeasure v2 balance\n')
        workloads[f'extract-{stages}'] = script
    receipt = dict(policy='Hosted serial ABBA/BAAB, one warmup per variant and four timed repetitions. High-resolution parent wall measurements include time-wrapper/CLI startup; GNUtime wall/user/system/RSS include CLI startup and child processes. Completion uses blocking wait with a process-group watchdog, avoiding timeout-polling lag. GNU wall/CPU clocks have centisecond granularity; RSS is a GNUtime child-process maximum, not aggregate simultaneous resident memory. Source-informed extraction workloads are separate from frozen50.', binaries={key: dict(sha256=sha(binary), validation=json.loads((binary.parent/'validation.json').read_text())['commit']) for key,binary in bins.items()}, started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),workloads=[])
    for name, script in workloads.items():
        runs=[]
        order=['baseline','candidate','baseline','candidate','candidate','baseline','candidate','baseline','baseline','candidate']
        for index, variant in enumerate(order):
            stem=args.out / f'{name}-{index}-{variant}'
            stdout=stem.with_suffix('.jsonl'); stderr=stem.with_suffix('.stderr'); timing=stem.with_suffix('.time')
            command=['/usr/bin/time','-f','%e %U %S %M','-o',str(timing),str(bins[variant]),'run',str(script),'--json']
            started = time.perf_counter()
            expired = threading.Event()
            with stdout.open('wb') as out,stderr.open('wb') as err:
                process = subprocess.Popen(command,stdout=out,stderr=err,start_new_session=True)
                def terminate_workload():
                    expired.set()
                    try:
                        os.killpg(process.pid,signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                watchdog = threading.Timer(60,terminate_workload)
                watchdog.start()
                try:
                    # A blocking wait avoids timeout polling's up-to-50ms
                    # completion lag contaminating short wall measurements.
                    returncode = process.wait()
                    elapsed = time.perf_counter() - started
                finally:
                    watchdog.cancel()
                    watchdog.join()
            if expired.is_set():
                raise subprocess.TimeoutExpired(command,60)
            result = subprocess.CompletedProcess(command,returncode)
            if result.returncode:
                raise RuntimeError(f'{name} {variant} failed: {stderr.read_text()}')
            frames=[json.loads(line) for line in stdout.read_text().splitlines()]
            if name.startswith('extract-'):
                assert any(e.get('event')=='extracted' for f in frames for e in f.get('events',[])), 'Refused extraction is not a completed workload'
            wall,user,system,rss=map(float,timing.read_text().split())
            runs.append(dict(variant=variant,warmup=index<2,wall_seconds=elapsed,gnu_wall_seconds=wall,user_seconds=user,system_seconds=system,max_rss_kib=rss,stdout_sha256=sha(stdout),stderr_sha256=sha(stderr),exit_code=result.returncode))
        medians={variant:{field:statistics.median(r[field] for r in runs if r['variant']==variant and not r['warmup']) for field in ['wall_seconds','user_seconds','system_seconds','max_rss_kib']} for variant in bins}
        receipt['workloads'].append(dict(name=name,script_sha256=sha(script),runs=runs,medians=medians,wall_ratio_baseline_over_candidate=medians['baseline']['wall_seconds']/medians['candidate']['wall_seconds']))
        (args.out/'profile.json').write_text(json.dumps(receipt,indent=2)+'\n')
        print(name,medians,flush=True)
    receipt['completed_utc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())
    (args.out/'profile.json').write_text(json.dumps(receipt,indent=2)+'\n')

if __name__=='__main__': main()
