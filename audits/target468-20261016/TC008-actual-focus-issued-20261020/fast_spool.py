"""Serial bounded ext4 download followed by separately verified storage copy."""
import hashlib,os,pathlib,shutil,time,urllib.parse
CHUNK=262144
MAX_FILE=2*1024**3
MAX_TOTAL=20*1024**3

def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  while chunk:=f.read(CHUNK):h.update(chunk)
 return h.hexdigest()

def safe_name(name):
 rel=pathlib.PurePosixPath(name)
 assert name and not rel.is_absolute() and all(p not in ['', '.', '..'] for p in name.split('/')) and '\\' not in name and '\x00' not in name
 return rel

def download(session,url,partial,verified,expected,gate,deadline,totals,maximum=MAX_FILE):
 """No storage open occurs here. On any failure preserve the exact partial."""
 assert not partial.exists() and not verified.exists() and not partial.is_symlink() and not verified.is_symlink()
 parsed=urllib.parse.urlparse(url);assert parsed.scheme=='https' and parsed.netloc and not parsed.username and not parsed.password
 gate(maximum);h=hashlib.sha256();length=0
 with session.get(url,stream=True,timeout=(30,90),allow_redirects=False,headers={'Accept-Encoding':'identity'}) as response:
  assert response.status_code==200 and response.headers.get('Content-Encoding','identity') in ['', 'identity']
  size=int(response.headers.get('Content-Length','0'));assert 0<=size<=maximum
  with partial.open('xb') as out:
   for chunk in response.iter_content(chunk_size=CHUNK):
    if not chunk:continue
    length+=len(chunk);totals[0]+=len(chunk)
    assert length<=maximum and totals[0]<=MAX_TOTAL and time.monotonic()<deadline
    out.write(chunk);h.update(chunk)
   out.flush();os.fsync(out.fileno())
  if size:assert length==size,'Content-Length mismatch'
 actual=h.hexdigest();assert expected is None or actual==expected,'Producer checksum mismatch on ext4'
 assert digest(partial)==actual,'Ext4 readback mismatch'
 partial.rename(verified)
 return {'sha256':actual,'bytes':length}

def copy_verified(verified,target,expected,deadline):
 """A failed storage operation retains the intact verified ext4 source."""
 assert verified.is_file() and digest(verified)==expected
 assert not target.exists() and not target.is_symlink()
 target.parent.mkdir(parents=True,exist_ok=True)
 for parent in target.parents:
  assert not parent.is_symlink(),'Storage target ancestor is symlink'
 stage=target.with_name(target.name+'.storage-partial');assert not stage.exists() and not stage.is_symlink()
 h=hashlib.sha256();copied=0
 with verified.open('rb') as src,stage.open('xb') as out:
  while chunk:=src.read(CHUNK):
   assert time.monotonic()<deadline
   out.write(chunk);h.update(chunk);copied+=len(chunk)
  out.flush();os.fsync(out.fileno())
 assert copied==verified.stat().st_size and h.hexdigest()==expected and digest(stage)==expected
 stage.rename(target);assert digest(target)==expected
 return {'bytes':copied,'sha256':expected,'download_verified_on_ext4_before_storage_open':True,'storage_readback_verified':True}
