"""Once-only unchanged six demonstrated partition/ammonia/filter/transfer beneficiaries on independently accepted production. No retries."""
import argparse,datetime,hashlib,json,time,urllib.request,urllib.error
from pathlib import Path
sha=lambda b:hashlib.sha256(b).hexdigest()
def save(p,d):p.write_text(json.dumps(d,indent=2,allow_nan=False)+'\n')
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*a,**k):return None
p=argparse.ArgumentParser();p.add_argument('--design',type=Path,required=True);p.add_argument('--owner-key-file',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();raw=a.design.read_bytes();d=json.loads(raw);assert d['bound_to_authenticated_accepted_artifact'] is True and len(d['cases'])==6 and {c['id'] for c in d['cases']}=={'batch2:gs-014','batch2:gs-018','batch2:ki-002','batch2:se-009','batch2:tc-002','batch2:tc-009'};assert d['origin']=='https://kerotakis-web-payload.vercel.app';assert not a.out.exists();key=json.loads(a.owner_key_file.read_bytes())['key'];assert len(key)>=32 and not any(c in key for c in '\r\n');a.out.mkdir(mode=0o700);(a.out/'design.json').write_bytes(raw);save(a.out/'STARTED.json',{'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'design_sha256':sha(raw),'runner_sha256':sha(Path(__file__).read_bytes()),'attempts':1,'no_retries':True});op=urllib.request.build_opener(NoRedirect());results=[]
for c in d['cases']:
 assert sha(c['script'].encode())==c['script_sha256'];f=a.out/c['id'].replace(':','-');f.mkdir();body=json.dumps({'script':c['script']},ensure_ascii=False).encode();(f/'request.json').write_bytes(body);save(f/'issued.json',{'id':c['id'],'script_sha256':c['script_sha256'],'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'attempt':1});req=urllib.request.Request(d['origin']+'/api/run',data=body,headers={'Content-Type':'application/json','Authorization':'Bearer '+key},method='POST');start=time.monotonic()
 try:
  try:r=op.open(req,timeout=75)
  except urllib.error.HTTPError as e:r=e
  with r:response=r.read(2200001);status=r.code
  assert len(response)<=2200000 and key.encode() not in response;(f/'response.body').write_bytes(response);receipt={'id':c['id'],'http_status':status,'elapsed_seconds':time.monotonic()-start,'response_sha256':sha(response),'attempts':1};save(f/'http.json',receipt);assert status==200,(c['id'],status);value=json.loads(response);assert value['engine']==d['accepted_engine'] and value['script_sha256']==c['script_sha256'];assert value['execution_status'] in ['completed','refused'];results.append(receipt)
 except Exception as e:save(f/'STOPPED.json',{'error_type':type(e).__name__,'already_issued':True,'retry_forbidden':True});raise
 time.sleep(3.1)
save(a.out/'completion.json',{'cases':results,'physical_assessment':'pending independent owned material and phase/energy review','catalog_additions':0,'no_retries':True});print(json.dumps({'executed':len(results),'catalog_additions':0}))
