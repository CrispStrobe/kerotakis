from pathlib import Path
import hashlib,json,subprocess
P=Path(__file__).resolve().parent;R=P.parents[2]
load=lambda name:json.loads((P/name).read_bytes())
for n,h in load('SHA256.json').items():assert hashlib.sha256((P/n).read_bytes()).hexdigest()==h,n
integrated=load('INTEGRATED.json');ci=load('CI-FOUR-JOB-PASS.json')
assert integrated['head']=='1fe4c9f07b36d18194393c1a5a2c77b3e98998fd' and integrated['merge']=='b5a261cfbd59c9f3036985f7398d810c6baf7e1b'
assert integrated['parents']==['33e043ef4a4a4225e5ea8d49ba93e8df99716c5f',integrated['head']]
assert ci['head']==integrated['head'] and ci['run']==39485496097
names={'Classify complete change range','Full preflight gate','Test (native) (ubuntu-latest)','Test (native) (macos-latest)'}
executed=[j for j in ci['jobs']if j['conclusion']!='skipped'];assert len(executed)==4 and {j['name']for j in executed}==names and all(j['conclusion']=='success'for j in executed)
for platform in ['ubuntu','macos']:
 independent=load('parent-docs851-'+platform+'-small-job.json')['actual_job'];job=next(j for j in executed if j['databaseId']==independent['id'])
 assert independent['head_sha']==integrated['head'] and independent['conclusion']==job['conclusion']=='success'
 for steps in [job['steps'],independent['steps']]:
  assert [s['name']for s in steps if s['conclusion']=='success']==['Set up job','Reviewed documentation needs no native build','Complete job']
  assert all(s['conclusion']in ['success','skipped']for s in steps)
def git(*a):return subprocess.check_output(['git','--no-replace-objects',*a],cwd=R,timeout=60).decode('utf-8').strip()
assert git('show','-s','--format=%P',integrated['merge']).split()==integrated['parents']
assert git('rev-parse',integrated['merge']+'^{tree}')==git('merge-tree','--write-tree',*integrated['parents']).splitlines()[0]==integrated['actual_tree']==integrated['reconstructed_tree']
assert git('diff','--name-only',integrated['parents'][0],integrated['head']).splitlines()==integrated['changed_paths'] and len(integrated['changed_paths'])==8
print('PASS: genuine four-small-job documentation CI and actual normal integration; no product/scientific acceptance implied.')
