import sys,json,hashlib,subprocess
from pathlib import Path
p=Path(sys.argv[1]); out=Path(sys.argv[2]); r=json.loads((p/'validation.json').read_text()); checks=[]
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  while b:=f.read(65536): h.update(b)
 return h.hexdigest()
def check(name,value):
 checks.append({'name':name,'passed':bool(value)})
check('full validation',r['passed'] and len(r['stages'])==14 and all(x['exit_code']==0 for x in r['stages']))
for name,s in r['source_hashes'].items():
 data=subprocess.check_output(['git','show',r['commit']+':'+name],cwd='/mnt/volume1/kerotakis'); check('source '+name,hashlib.sha256(data).hexdigest()==s)
for s in r['stages']: check('log '+s['name'],sha(p/s['log'])==s['log_sha256'])
check('dependency lock',sha(p/'Cargo.lock')==r['cargo_lock_sha256']); check('executable',sha(p/'kero')==r['binary_sha256'])
for name in ('distillation','trace-phase','paired-binary','still-failure'):
 e=json.loads((p/name/'execution.json').read_text()); v=json.loads((p/name/'verification.json').read_text())
 check(name+' controls passed',v['passed']); check(name+' binary identity',e['binary_sha256']==r['binary_sha256'])
 for run in e['runs']:
  for stream in ('stdout','stderr'): check(name+' '+run['id']+' '+run['mode']+' '+stream,sha(p/name/run[stream])==run[stream+'_sha256'])
result={'passed':all(x['passed'] for x in checks),'checks':len(checks),'commit':r['commit'],'binary_sha256':r['binary_sha256'],'details':checks}
out.write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps({k:v for k,v in result.items() if k!='details'})); assert result['passed']
