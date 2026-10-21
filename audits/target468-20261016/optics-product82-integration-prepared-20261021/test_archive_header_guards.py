"""SYNTHETIC ARCHIVE GUARDS ONLY; no product/browser/chemistry outcomes."""
import io,tarfile,unittest
from project_app_assets import preflight_tar_headers,MAX_FILES,MAX_METADATA
class SyntheticArchiveGuard(unittest.TestCase):
 def test_all_directory_headers_count_toward_member_bound(self):
  data=io.BytesIO()
  with tarfile.open(fileobj=data,mode='w',format=tarfile.USTAR_FORMAT)as archive:
   for i in range(MAX_FILES+1):
    member=tarfile.TarInfo('dir'+str(i));member.type=tarfile.DIRTYPE;archive.addfile(member)
  with self.assertRaisesRegex(ValueError,'physical TAR member/header bound'):preflight_tar_headers(io.BytesIO(data.getvalue()))
 def test_oversized_pax_payload_rejected_before_reading_payload(self):
  member=tarfile.TarInfo('PaxHeader');member.type=tarfile.XHDTYPE;member.size=MAX_METADATA+1
  # Intentionally no payload: guard must refuse the header before attempting it.
  with self.assertRaisesRegex(ValueError,'GNU/PAX metadata payload bound'):preflight_tar_headers(io.BytesIO(member.tobuf(format=tarfile.USTAR_FORMAT)))
 def test_sparse_header_rejected_without_following_extension_blocks(self):
  member=tarfile.TarInfo('sparse');member.type=tarfile.GNUTYPE_SPARSE;member.size=0
  with self.assertRaisesRegex(ValueError,'physical TAR link/sparse/special member'):preflight_tar_headers(io.BytesIO(member.tobuf(format=tarfile.USTAR_FORMAT)))
 def test_pax_sparse_metadata_rejected_before_following_regular_header(self):
  for key,value in [('GNU.sparse.map','0,1'),('GNU.sparse.major','1'),('SCHILY.realsize','1'),('SCHILY.filetype','sparse'),('SCHILY.sparse.map','0,1')]:
   with self.subTest(key=key):
    content=(key+'='+value+'\n').encode();length=len(content)+2
    while length!=len(str(length))+1+len(content):length=len(str(length))+1+len(content)
    payload=str(length).encode()+b' '+content
    member=tarfile.TarInfo('PaxHeader');member.type=tarfile.XHDTYPE;member.size=len(payload)
    # Deliberately no subsequent REGTYPE file/payload: reject activation first.
    raw=member.tobuf(format=tarfile.USTAR_FORMAT)+payload
    with self.assertRaisesRegex(ValueError,'PAX sparse metadata refused before stdlib'):preflight_tar_headers(io.BytesIO(raw))
 def test_directory_payload_refused_before_hidden_header_interpretation(self):
  member=tarfile.TarInfo('directory');member.type=tarfile.DIRTYPE;member.size=512
  # No payload needed: zero directory size is required at the fixed header.
  with self.assertRaisesRegex(ValueError,'TAR directory must not declare a payload'):preflight_tar_headers(io.BytesIO(member.tobuf(format=tarfile.USTAR_FORMAT)))
if __name__=='__main__':unittest.main()
