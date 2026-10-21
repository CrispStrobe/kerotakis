"""Tiny real local subprocesses only; no transport network, engine or provider."""
from pathlib import Path
import hashlib,json,os,signal,subprocess,sys,tempfile,time,unittest
from artifact_spool import receive
PYTHON=[sys.executable,'-S','-c']
FAMILY=r'''
import json,os,pathlib,signal,sys,time
root=pathlib.Path(sys.argv[1]);mode=sys.argv[2];child=os.fork()
if child==0:
 signal.signal(signal.SIGTERM,signal.SIG_IGN)
 if mode=='success':os.close(1)
 (root/'child.json').write_text(json.dumps({'pid':os.getpid(),'pgid':os.getpgrp()}))
 while True:time.sleep(1)
else:
 deadline=time.monotonic()+2
 while not(root/'child.json').exists():
  assert time.monotonic()<deadline
  time.sleep(.005)
 (root/'family.json').write_text(json.dumps({'leader':os.getpid(),'pgid':os.getpgrp(),'child':child}))
 os.write(1,b'abc');os.write(2,b'family-ready\n')
 if mode in ['leader-exit','success']:os._exit(0)
 while True:time.sleep(1)
'''
def live(pid):
 try:return Path('/proc/'+str(pid)+'/stat').read_text().split(') ',1)[1].split()[0]!='Z'
 except FileNotFoundError:return False
class OwnedSpoolProcesses(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory(prefix='owned-spool-control-');self.root=Path(self.temp.name)
  self.partial=self.root/'partial';self.verified=self.root/'verified';self.stderr=self.root/'stderr'
 def tearDown(self):
  f=self.root/'family.json'
  if f.exists():
   row=json.loads(f.read_text());self.assertNotEqual(row['pgid'],os.getpgrp())
   try:os.killpg(row['pgid'],signal.SIGKILL)
   except ProcessLookupError:pass
  self.temp.cleanup()
 def run_transfer(self,code,data=b'abc',seconds=3,maximum=1024):
  return receive(PYTHON+[code],self.partial,self.verified,hashlib.sha256(data).hexdigest(),self.stderr,time.monotonic()+seconds,maximum)
 def assert_family_stopped(self):
  row=json.loads((self.root/'family.json').read_text());child=json.loads((self.root/'child.json').read_text())
  self.assertEqual(row['pgid'],row['leader']);self.assertEqual(child['pgid'],row['pgid']);self.assertEqual(child['pid'],row['child']);self.assertNotEqual(row['pgid'],os.getpgrp())
  until=time.monotonic()+2
  while live(row['child'])and time.monotonic()<until:time.sleep(.01)
  self.assertFalse(live(row['child']),'owned TERM-ignoring descendant survived')
  self.assertFalse(live(row['leader']),'owned leader survived')
 def family_transfer(self,mode):
  return receive(PYTHON+[FAMILY,str(self.root),mode],self.partial,self.verified,hashlib.sha256(b'abc').hexdigest(),self.stderr,time.monotonic()+.7,1024)
 def test_normal_authenticated_transfer(self):
  value=self.run_transfer("import os;os.write(1,b'abc');os.write(2,b'actual-stderr')")
  self.assertEqual(self.verified.read_bytes(),b'abc');self.assertFalse(self.partial.exists());self.assertEqual(self.stderr.read_bytes(),b'actual-stderr');self.assertEqual(value['bytes'],3);self.assertTrue(value['ext4_readback_verified'])
 def test_size_bound_preserves_partial_and_primary_message(self):
  with self.assertRaisesRegex(AssertionError,'^Artifact transfer size bound$'):self.run_transfer("import os;os.write(1,b'abcdef')",maximum=2)
  self.assertTrue(self.partial.exists());self.assertTrue(self.stderr.exists());self.assertFalse(self.verified.exists())
 def test_checksum_mismatch_retains_actual_partial(self):
  with self.assertRaisesRegex(AssertionError,'^Artifact metadata checksum/readback mismatch$'):self.run_transfer("import os;os.write(1,b'abc')",data=b'wrong')
  self.assertEqual(self.partial.read_bytes(),b'abc');self.assertFalse(self.verified.exists())
 def test_nonzero_transport_preserves_error_and_stderr(self):
  with self.assertRaisesRegex(AssertionError,'^Artifact transport command failed$'):self.run_transfer("import os;os.write(1,b'abc');os.write(2,b'real-failure');os._exit(7)")
  self.assertEqual(self.partial.read_bytes(),b'abc');self.assertEqual(self.stderr.read_bytes(),b'real-failure');self.assertFalse(self.verified.exists())
 def test_deadline_cleans_live_leader_and_inherited_stdout_descendant(self):
  with self.assertRaisesRegex(AssertionError,'^Artifact transfer deadline$'):self.family_transfer('live-leader')
  self.assert_family_stopped();self.assertFalse(self.verified.exists());self.assertEqual(self.partial.read_bytes(),b'abc')
 def test_deadline_cleans_descendant_after_leader_already_exited(self):
  with self.assertRaisesRegex(AssertionError,'^Artifact transfer deadline$'):self.family_transfer('leader-exit')
  self.assert_family_stopped();self.assertFalse(self.verified.exists())
 def test_success_cleans_remaining_group_after_leader_exit(self):
  result=self.family_transfer('success');self.assertEqual(result['bytes'],3);self.assert_family_stopped();self.assertEqual(self.verified.read_bytes(),b'abc')
 def test_setup_error_remains_file_not_found_and_keeps_created_evidence(self):
  with self.assertRaises(FileNotFoundError):receive(['/definitely-missing-owned-spool-control'],self.partial,self.verified,'0'*64,self.stderr,time.monotonic()+1)
  self.assertTrue(self.partial.exists());self.assertTrue(self.stderr.exists());self.assertFalse(self.verified.exists())
 def test_real_cleanup_signal_error_does_not_mask_primary_size_error(self):
  class RealCleanupSignalError(RuntimeError):pass
  def interrupted(signum,frame):raise RealCleanupSignalError('actual SIGUSR1 during owned cleanup')
  previous=signal.signal(signal.SIGUSR1,interrupted)
  child="import os,signal,time;signal.signal(signal.SIGTERM,lambda *a:(os.kill(os.getppid(),signal.SIGUSR1),os._exit(0)));os.write(1,b'abcdef');time.sleep(30)"
  try:
   with self.assertRaisesRegex(AssertionError,'^Artifact transfer size bound$')as caught:self.run_transfer(child,maximum=2)
   self.assertTrue(any('RealCleanupSignalError' in note for note in getattr(caught.exception,'__notes__',[])),'real cleanup error note was not retained')
   self.assertFalse(self.verified.exists());self.assertTrue(self.partial.exists())
  finally:signal.signal(signal.SIGUSR1,previous)
 def test_unrelated_owned_sentinel_group_is_not_signaled(self):
  sentinel=subprocess.Popen(PYTHON+["import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);time.sleep(30)"],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
  try:
   with self.assertRaisesRegex(AssertionError,'^Artifact transfer deadline$'):self.family_transfer('leader-exit')
   self.assert_family_stopped();self.assertIsNone(sentinel.poll());self.assertTrue(live(sentinel.pid))
  finally:
   try:os.killpg(sentinel.pid,signal.SIGKILL)
   except ProcessLookupError:pass
   sentinel.wait(timeout=3)
if __name__=='__main__':unittest.main(verbosity=2)
