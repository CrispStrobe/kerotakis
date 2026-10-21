"""Bounded saved-artifact projection. No extraction, network or engine operation.

Caller supplies externally trusted artifact/accepted-verifier SHA and source.
This projection is not browser or future-release acceptance.
"""
import argparse,gzip,hashlib,json,re,stat,tarfile,zipfile
from pathlib import Path,PurePosixPath
MAX_ARCHIVE=2*1024**3;MAX_ENTRY=64*1024**2;MAX_TOTAL=1024**3;MAX_FILES=10000
MAX_NAME_BYTES=512;MAX_METADATA=8192;MAX_TOTAL_METADATA=65536

def require(value,message):
 if not value:raise ValueError(message)
def digest(path):
 h=hashlib.sha256()
 with Path(path).open('rb')as f:
  while chunk:=f.read(262144):h.update(chunk)
 return h.hexdigest()
def stream_hash(stream,size):
 require(0<=size<=MAX_ENTRY,'entry exceeds size bound');h=hashlib.sha256();count=0
 while chunk:=stream.read(262144):
  count+=len(chunk);require(count<=size,'entry exceeds declared size');h.update(chunk)
 require(count==size,'entry truncated');return h.hexdigest()
def safe(name):
 require(type(name)is str and len(name.encode('utf-8'))<=MAX_NAME_BYTES and '\\'not in name and '\x00'not in name and not name.startswith('/'),'unsafe archive name')
 while name.startswith('./'):name=name[2:]
 if name in ['', '.']:return ''
 parts=name.rstrip('/').split('/');require(all(p not in ['','..','.']for p in parts),'unsafe path component');return '/'.join(parts)
def reject_sparse_pax(payload):
 """Parse only bounded POSIX length records; reject sparse activation metadata."""
 offset=0
 while offset<len(payload):
  space=payload.find(b' ',offset,min(len(payload),offset+12))
  require(space>offset and payload[offset:space].isdigit(),'malformed bounded PAX record length')
  length=int(payload[offset:space]);require(length>space-offset+2 and offset+length<=len(payload),'PAX record exceeds bounded payload')
  record=payload[space+1:offset+length];require(record.endswith(b'\n')and b'='in record,'malformed bounded PAX key/value')
  key,value=record[:-1].split(b'=',1);require(0<len(key)<=256 and b'\0'not in key,'invalid bounded PAX key')
  key=key.decode('utf-8','strict').lower()
  sparse=(key=='gnu.sparse'or key.startswith('gnu.sparse.')or key=='schily.realsize'or key=='schily.sparse'or key.startswith('schily.sparse.')or(key=='schily.filetype'and value.strip().lower()==b'sparse'))
  require(not sparse,'PAX sparse metadata refused before stdlib extension processing')
  offset+=length

def preflight_tar_headers(stream):
 """Bound physical headers/metadata BEFORE tarfile can allocate PAX/GNU payloads.

 Reads one512-byte physical header, streams/skips bounded payload and padding;
 TarInfo.frombuf parses ONLY this fixed header, never extension payloads.
 """
 count=0;total=0;metadata=0
 while True:
  header=stream.read(512);require(len(header)==512,'truncated physical TAR header')
  if header==b'\0'*512:
   require(stream.read(512)==b'\0'*512,'missing TAR end marker')
   # Standard record padding only: no concatenated hidden archive/data.
   trailing=0
   while chunk:=stream.read(262144):
    trailing+=len(chunk);require(trailing<=10240 and not chunk.strip(b'\0'),'nonzero/excess trailing TAR data')
   return count
  count+=1;require(count<=MAX_FILES,'physical TAR member/header bound')
  info=tarfile.TarInfo.frombuf(header,'utf-8','strict');safe(info.name)
  size=info.size;require(type(size)is int and 0<=size<=MAX_ENTRY,'physical TAR entry size bound')
  if info.type==tarfile.DIRTYPE:require(size==0,'TAR directory must not declare a payload')
  total+=size;require(total<=MAX_TOTAL,'physical TAR total payload bound')
  if info.type in [tarfile.XHDTYPE,tarfile.XGLTYPE,tarfile.GNUTYPE_LONGNAME,tarfile.GNUTYPE_LONGLINK]:
   require(size<=MAX_METADATA,'GNU/PAX metadata payload bound');metadata+=size;require(metadata<=MAX_TOTAL_METADATA,'GNU/PAX aggregate metadata bound')
  elif info.type not in [tarfile.REGTYPE,tarfile.AREGTYPE,tarfile.DIRTYPE]:raise ValueError('physical TAR link/sparse/special member')
  remaining=((size+511)//512)*512
  if info.type in [tarfile.XHDTYPE,tarfile.XGLTYPE]:
   # Already bounded to8KiB above; inspect BEFORE stdlib _proc_pax can act.
   payload=stream.read(size);require(len(payload)==size,'truncated bounded PAX payload');reject_sparse_pax(payload);remaining-=size
  while remaining:
   chunk=stream.read(min(262144,remaining));require(chunk,'truncated physical TAR payload');remaining-=len(chunk)

def project(artifact,artifact_sha256,verified,verified_sha256,source):
 for value,length in [(artifact_sha256,64),(verified_sha256,64),(source,40)]:require(re.fullmatch('[a-f0-9]{'+str(length)+'}',value)is not None,'malformed external pin')
 require(Path(artifact).stat().st_size<=MAX_ARCHIVE,'artifact bound')
 require(digest(artifact)==artifact_sha256,'artifact external SHA mismatch')
 require(Path(verified).stat().st_size<=4*1024**2,'verifier receipt bound')
 require(digest(verified)==verified_sha256,'accepted verifier external SHA mismatch');receipt=json.loads(Path(verified).read_bytes())
 require(receipt['artifact_sha256']==artifact_sha256 and receipt['engine']['source_head']==source,'accepted verifier/source/artifact mismatch')
 require(receipt['actual_binary_bytes_rehashed']is True and receipt['actual_producer_lock_bytes_rehashed']is True and receipt['source_and_tree_equal_accepted_main']is True,'verifier did not authenticate actual release inputs')
 with zipfile.ZipFile(artifact)as archive:
  entries={};total=0
  for row in archive.infolist():
   name=safe(row.filename);require(name and name not in entries,'duplicate/empty ZIP member');entries[name]=row
   mode=row.external_attr>>16;kind=stat.S_IFMT(mode);require(kind in [0,stat.S_IFREG,stat.S_IFDIR],'ZIP link/special member')
   total+=row.file_size;require(row.file_size<=MAX_ENTRY and total<=MAX_TOTAL and len(entries)<=MAX_FILES,'ZIP size/member bound')
  require('rest-frontend.tar.gz'in entries,'frontend archive missing');frontend=entries['rest-frontend.tar.gz']
  with archive.open(frontend)as stream:require(stream_hash(stream,frontend.file_size)==receipt['frontend_archive_sha256'],'frontend archive differs from accepted verifier')
  # Inspect physical extension headers before stdlib PAX/GNU processing.
  with archive.open(frontend)as compressed,gzip.GzipFile(fileobj=compressed)as expanded:preflight_tar_headers(expanded)
  app={};engine={};names=set();total=0;files=0;members=0
  with archive.open(frontend)as compressed,tarfile.open(fileobj=compressed,mode='r|gz')as tar:
   for member in tar:
    members+=1;require(members<=MAX_FILES,'all TAR members including directories bound')
    name=safe(member.name)
    require(name not in names,'duplicate normalized TAR member');names.add(name)
    if member.isdir():continue
    require(name and member.isfile()and not member.issym()and not member.islnk(),'TAR link/special/empty member')
    require(not member.pax_headers.get('linkpath'),'TAR link metadata')
    files+=1;total+=member.size;require(files<=MAX_FILES and total<=MAX_TOTAL,'TAR size/member bound')
    stream=tar.extractfile(member);require(stream is not None,'TAR stream unavailable')
    sha=stream_hash(stream,member.size)
    if name.startswith('app/'):app[name]={'sha256':sha,'bytes':member.size}
    if name in receipt['frontend_assets']:
     require({'sha256':sha,'bytes':member.size}==receipt['frontend_assets'][name],'engine/REST member differs from verifier');engine[name]={'sha256':sha,'bytes':member.size}
 require('app/index.html'in app and any(n.startswith('app/assets/')and n.endswith('.js')for n in app),'actual app document/JS absent')
 require(engine==receipt['frontend_assets'],'accepted engine/REST member omitted')
 return {'state':'SAVED_ARCHIVE_PROJECTION_ONLY_NOT_BROWSER_ACCEPTANCE','source':source,'tree':receipt['engine']['source_tree'],'artifact_sha256':artifact_sha256,'verified_release_sha256':verified_sha256,'frontend_archive_sha256':receipt['frontend_archive_sha256'],'native_binary_sha256':receipt['engine']['binary_sha256'],'actual_producer_lock_sha256':receipt['engine']['cargo_lock_sha256'],'app_assets':app,'engine_and_rest_assets':engine,'actual_tar_files_read':files,'actual_tar_uncompressed_bytes':total,'catalog_additions':0}
def main():
 p=argparse.ArgumentParser();p.add_argument('--artifact',type=Path,required=True);p.add_argument('--artifact-sha256',required=True);p.add_argument('--verified',type=Path,required=True);p.add_argument('--verified-sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();r=project(a.artifact,a.artifact_sha256,a.verified,a.verified_sha256,a.source)
 with a.out.open('x',encoding='utf-8')as f:json.dump(r,f,indent=2);f.write('\n')
if __name__=='__main__':main()
