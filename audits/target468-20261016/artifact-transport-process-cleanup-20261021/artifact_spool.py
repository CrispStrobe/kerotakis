"""Bounded process stdout to verified ext4, then separately verified storage publication."""
from pathlib import Path
import hashlib,os,selectors,signal,subprocess,time
CHUNK=262144
MAX_ARCHIVE=2*1024**3

def digest(path):
 with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def _cleanup_owned_group(process):
 """Only a transport created with start_new_session=True may be supplied."""
 failures=[]
 try:
  try:os.killpg(process.pid,signal.SIGTERM)
  except ProcessLookupError:pass
  except BaseException as error:failures.append(error)
  try:process.wait(timeout=.25)
  except subprocess.TimeoutExpired:pass
  except BaseException as error:failures.append(error)
 finally:
  # A dead leader does not establish that its inherited-pipe descendants exited.
  try:os.killpg(process.pid,signal.SIGKILL)
  except ProcessLookupError:pass
  except BaseException as error:failures.append(error)
  try:process.wait(timeout=5)
  except BaseException as error:failures.append(error)
 if failures:
  for other in failures[1:]:failures[0].add_note('Additional owned-group cleanup error: '+repr(other))
  raise failures[0]

def receive(argv,partial,verified,expected_sha,stderr,deadline,maximum=MAX_ARCHIVE):
 assert not partial.exists()and not verified.exists()and not partial.is_symlink()and not verified.is_symlink()
 h=hashlib.sha256();length=0;process=None;selector=None;primary=None;group_cleaned=False
 try:
  with partial.open('xb')as out,stderr.open('xb')as err:
   process=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=err,start_new_session=True)
   selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ)
   while True:
    assert time.monotonic()<deadline,'Artifact transfer deadline'
    if not selector.select(timeout=min(1,max(.001,deadline-time.monotonic()))):continue
    chunk=os.read(process.stdout.fileno(),CHUNK)
    if not chunk:break
    length+=len(chunk);assert length<=maximum,'Artifact transfer size bound';out.write(chunk);h.update(chunk)
   assert process.wait(timeout=max(.001,deadline-time.monotonic()))==0,'Artifact transport command failed'
   out.flush();os.fsync(out.fileno())
  _cleanup_owned_group(process);group_cleaned=True
  assert length>0 and h.hexdigest()==expected_sha and digest(partial)==expected_sha,'Artifact metadata checksum/readback mismatch'
  partial.rename(verified);return {'sha256':expected_sha,'bytes':length,'ext4_readback_verified':True}
 except BaseException as error:
  primary=error;raise
 finally:
  cleanup_errors=[]
  if process is not None and not group_cleaned:
   try:_cleanup_owned_group(process)
   except BaseException as error:cleanup_errors.append(error)
  if selector is not None:
   try:selector.close()
   except BaseException as error:cleanup_errors.append(error)
  if process is not None and process.stdout is not None:
   try:process.stdout.close()
   except BaseException as error:cleanup_errors.append(error)
  if cleanup_errors:
   if primary is not None:
    for error in cleanup_errors:primary.add_note('Owned transport cleanup error (primary failure preserved): '+repr(error))
   else:
    for error in cleanup_errors[1:]:cleanup_errors[0].add_note('Additional transport cleanup error: '+repr(error))
    raise cleanup_errors[0]

def publish(verified,target,expected_sha,deadline):
 assert verified.is_file()and digest(verified)==expected_sha
 assert not target.exists()and not target.is_symlink();target.parent.mkdir(parents=True,exist_ok=True)
 for parent in target.parents:assert not parent.is_symlink(),'Resolve trusted root once; refuse owned descendant aliases'
 stage=target.with_name(target.name+'.storage-partial');assert not stage.exists()and not stage.is_symlink();h=hashlib.sha256();length=0
 with verified.open('rb')as src,stage.open('xb')as out:
  while chunk:=src.read(CHUNK):assert time.monotonic()<deadline;out.write(chunk);h.update(chunk);length+=len(chunk)
  out.flush();os.fsync(out.fileno())
 assert length==verified.stat().st_size and h.hexdigest()==expected_sha and digest(stage)==expected_sha
 stage.rename(target);assert digest(target)==expected_sha
 return {'sha256':expected_sha,'bytes':length,'storage_readback_verified':True,'verified_ext4_spool_retained':str(verified)}
