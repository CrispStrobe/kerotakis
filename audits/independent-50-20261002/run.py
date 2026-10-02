"""Replay all frozen designs through the real CLI; archive stdout/stderr verbatim."""
import concurrent.futures, hashlib, json, os, subprocess, time
from pathlib import Path
root = Path(__file__).resolve().parent
repo = root.parents[1]
binary = Path(os.environ.get('KERO_BIN', str(repo/'target/debug/kero'))).resolve()
cases = json.loads((root/'predictions.json').read_text())
results = Path(os.environ.get('KERO_RESULTS_DIR', str(root/'results'))).resolve()
results.mkdir(parents=True, exist_ok=True)
def run(c):
    result = dict(id=c['id'], title=c['title'], runs={})
    for mode in ('json','text'):
        args = [str(binary), 'run', str(root/'scripts'/f"{c['id']}.lab")]
        if mode=='json': args.append('--json')
        start=time.perf_counter()
        try:
            p=subprocess.run(args, cwd=repo, capture_output=True, text=True, timeout=90)
            stdout, stderr, code = p.stdout, p.stderr, p.returncode
        except subprocess.TimeoutExpired as e:
            stdout=(e.stdout or b'').decode() if isinstance(e.stdout,bytes) else e.stdout or ''
            stderr=(e.stderr or b'').decode() if isinstance(e.stderr,bytes) else e.stderr or ''
            code='timeout'
        (results/f"{c['id']}.{mode}.stdout").write_text(stdout)
        (results/f"{c['id']}.{mode}.stderr").write_text(stderr)
        result['runs'][mode]=dict(exit_code=code,seconds=round(time.perf_counter()-start,4))
    print(c['id'], result['runs'], flush=True)
    return result
metadata=dict(commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
    binary=str(binary),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
    predictions_sha256=hashlib.sha256((root/'predictions.json').read_bytes()).hexdigest(),
    started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),workers=int(os.environ.get("KERO_WORKERS", "1")))
with concurrent.futures.ThreadPoolExecutor(max_workers=int(os.environ.get("KERO_WORKERS", "1"))) as pool:
    metadata['cases']=sorted(pool.map(run,cases),key=lambda c:c['id'])
(results/'execution.json').write_text(json.dumps(metadata,indent=2)+'\n')
