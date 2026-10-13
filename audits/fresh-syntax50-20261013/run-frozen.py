#!/usr/bin/env python3
"""Execute only a complete frozen fifty, preserving first outcomes and gate refusals."""
import argparse, datetime, hashlib, json, os, pathlib, shutil, subprocess, time

def sha(path):
    h=hashlib.sha256()
    with open(path,'rb') as f:
        for chunk in iter(lambda:f.read(1024*1024),b''): h.update(chunk)
    return h.hexdigest()

def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()

def gate(fast, storage):
    available=int(next(x.split()[1] for x in pathlib.Path('/proc/meminfo').read_text().splitlines() if x.startswith('MemAvailable:')))//1024
    r={'utc':now(),'load1':os.getloadavg()[0],'memory_available_mib':available,'fast_free_mib':shutil.disk_usage(fast).free//1048576,'storage_free_mib':shutil.disk_usage(storage).free//1048576}
    r['permitted']=r['load1']<=3.5 and available>=3200 and r['fast_free_mib']>=4224 and r['storage_free_mib']>=8192
    return r

ap=argparse.ArgumentParser()
ap.add_argument('--design',type=pathlib.Path,required=True)
ap.add_argument('--binary',type=pathlib.Path,required=True)
ap.add_argument('--binary-sha256',required=True)
ap.add_argument('--out',type=pathlib.Path,required=True)
ap.add_argument('--fast',type=pathlib.Path,required=True)
ap.add_argument('--storage',type=pathlib.Path,required=True)
a=ap.parse_args()
a.out.mkdir(exist_ok=True)
assert not (a.out/'completed.json').exists(), 'completed run must not repeat'
freeze=json.loads((a.design/'freeze.json').read_text())
# Operator creates execution-binding.json independently after verifying all original design hashes.
binding=json.loads((a.design.parent/'execution-binding.json').read_text())
files=binding['design_files']
assert files==freeze['files'] and freeze['experiment_count']==50
assert len(json.loads((a.design/'expectations.json').read_text())['experiments'])==50
assert len([p for p in files if p.endswith('.lab')])==50
for name,digest in files.items():
    path=a.design/name
    assert path.resolve().is_relative_to(a.design.resolve()) and sha(path)==digest,(name,'changed frozen member')
assert sha(a.design/'freeze.json')==binding['design_freeze_sha256']
assert a.binary_sha256==binding['binary_sha256']
pre=gate(a.fast,a.storage)
with (a.out/'resource-journal.jsonl').open('a') as f: f.write(json.dumps(pre)+'\n')
if not pre['permitted']: raise SystemExit('resource gate closed before binary verification')
assert sha(a.binary)==a.binary_sha256
assert sha(pathlib.Path(__file__))==binding['runner_sha256'], 'runner changed'
if (a.out/'binding.json').exists():
    assert json.loads((a.out/'binding.json').read_text())==binding, 'binding changed'
else: (a.out/'binding.json').write_text(json.dumps(binding,indent=2)+'\n')
scripts=sorted(p for p in files if p.endswith('.lab'))
for i,name in enumerate(scripts,1):
    stem=f'{i:02d}'
    result=a.out/(stem+'.result.json')
    if result.exists():
        old=json.loads(result.read_text())
        assert old['case']==name and old['script_sha256']==files[name]
        assert not old['timeout'], 'timeout requires separate review; no automatic resume'
        assert sha(a.out/(stem+'.stdout.jsonl'))==old['stdout_sha256']
        assert sha(a.out/(stem+'.stderr.txt'))==old['stderr_sha256']
        continue
    assert not (a.out/(stem+'.stdout.jsonl')).exists() and not (a.out/(stem+'.stderr.txt')).exists(), 'incomplete execution must be reviewed, never overwritten'
    g=gate(a.fast,a.storage)
    with (a.out/'resource-journal.jsonl').open('a') as f: f.write(json.dumps(dict(g,case=name))+'\n')
    if not g['permitted']: raise SystemExit('resource gate closed; preserved partial results; do not overwrite')
    assert sha(a.design/name)==files[name]
    stem=f'{i:02d}'
    started=now(); start=time.monotonic(); timed_out=False
    env=dict(os.environ,KERO_LANG='en',LANG='C.UTF-8')
    cmd=[str(a.binary.resolve()),'run',name,'--json']
    with (a.out/(stem+'.stdout.jsonl')).open('xb') as out, (a.out/(stem+'.stderr.txt')).open('xb') as err:
        proc=subprocess.Popen(cmd,stdout=out,stderr=err,cwd=a.design,env=env,start_new_session=True)
        try: code=proc.wait(timeout=60)
        except subprocess.TimeoutExpired:
            import signal
            os.killpg(proc.pid,signal.SIGKILL); code=proc.wait(); timed_out=True
    record={'case':name,'ordinal':i,'started_utc':started,'elapsed_seconds':time.monotonic()-start,'exit_code':code,'timeout':timed_out,'stdout_sha256':sha(a.out/(stem+'.stdout.jsonl')),'stderr_sha256':sha(a.out/(stem+'.stderr.txt')),'stdout_bytes':(a.out/(stem+'.stdout.jsonl')).stat().st_size,'stderr_bytes':(a.out/(stem+'.stderr.txt')).stat().st_size,'script_sha256':files[name]}
    (a.out/(stem+'.result.json')).write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps(record),flush=True)
    if timed_out: raise SystemExit('timeout preserved; review before continuing')
(a.out/'completed.json').write_text(json.dumps({'completed_utc':now(),'cases':50,'binding_sha256':sha(a.out/'binding.json'),'original_outcomes_preserved':True},indent=2)+'\n')
