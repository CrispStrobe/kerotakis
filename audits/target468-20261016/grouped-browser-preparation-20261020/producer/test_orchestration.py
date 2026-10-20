"""SYNTHETIC external-command fixtures only. No browser/product/provider/build execution."""
from pathlib import Path
import hashlib,json,os,sys,tempfile,unittest,subprocess
import grouped_browser as worker
class Orchestration(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name);self.package=self.root/'package';self.package.mkdir();self.out=self.root/'evidence';self.out.mkdir();self.calls=[]
  self.groups=[{'id':name,'entry':entry,'expected_checks':count,'timeout_seconds':5}for name,entry,count,_ in worker.GROUPS]
  for g in self.groups:(self.package/g['entry']).write_text('// explicitly synthetic orchestration fixture\n',encoding='utf-8')
  self.hashes={g['entry']:worker.digest(self.package/g['entry'])for g in self.groups};self.fields={'runtime_source':'a'*40,'harness_head':'b'*40,'harness_tree':'c'*40,'harness_script_sha256':self.hashes[self.groups[0]['entry']],'binding_sha256':'d'*64,'binding':{'synthetic_fixture':True},'modules_sha256':self.hashes}
 def closure(self):
  self.calls.append('checked')
  for name,want in self.hashes.items():self.assertEqual(worker.digest(self.package/name),want)
  return {'input_files_sha256':self.hashes}
 def argv(self,g,exit_status=0,report=True,extra=''):
  code="import os,json,pathlib; p=pathlib.Path(os.environ['BROWSER_REPORT_DIR']);p.mkdir(); "+("(p/'report.json').write_text(json.dumps({'all_checks_passed':"+str(exit_status==0)+",'synthetic_fixture':True,'pid':os.getpid(),'sid':os.getsid(0),'group':"+repr(g['id'])+",'checks':[{'passed':True}]*"+str(g['expected_checks'])+"}));"if report else '')+extra+'raise SystemExit('+str(exit_status)+')'
  return [sys.executable,'-c',code]
 def run_groups(self,factory=None,groups=None):return worker.execute_groups(groups or self.groups,self.package,self.out,dict(os.environ),self.closure,self.fields,'e'*64,argv_for=factory or self.argv)
 def test_four_separate_process_boundaries_and_exact_argv(self):
  expected={g['id']:self.argv(g)for g in self.groups};rows=self.run_groups(lambda g:expected[g['id']]);self.assertEqual(len(rows),4);pids=[]
  for g in self.groups:
   root=self.out/'groups'/g['id'];record=json.loads((root/'actual-live-browser.json').read_text());raw=json.loads((root/'actual-browser/report.json').read_text());self.assertEqual(record['argv'],expected[g['id']]);self.assertTrue(record['fresh_owned_session']);self.assertEqual(raw['pid'],raw['sid']);pids.append(raw['pid']);self.assertTrue((root/'source-before.json').exists()and(root/'source-after.json').exists());self.assertIn('actual-browser/report.json',json.loads((root/'SHA256.json').read_text()))
  self.assertEqual(len(set(pids)),4);self.assertEqual(len(self.calls),8)
 def test_group2_failure_stops_later_groups_preserves_partial_report(self):
  with self.assertRaises(AssertionError):self.run_groups(lambda g:self.argv(g,7 if g['id']=='targeted7' else 0))
  second=self.out/'groups/targeted7';self.assertTrue((second/'actual-browser/report.json').exists());self.assertEqual(json.loads((second/'actual-live-browser.json').read_text())['exit_status'],7);self.assertTrue((second/'SHA256.json').exists());self.assertFalse((self.out/'groups/locale18').exists());self.assertFalse((self.out/'groups/heat2').exists());self.assertEqual(len(self.calls),4)
 def test_module_byte_tampering_after_child_rejected(self):
  extra="pathlib.Path("+repr(str(self.package/self.groups[0]['entry']))+").write_text('tampered');"
  with self.assertRaises(AssertionError):self.run_groups(lambda g:self.argv(g,extra=extra))
  root=self.out/'groups/browser37';self.assertTrue((root/'source-after-error.json').exists());self.assertTrue((root/'actual-browser/report.json').exists());self.assertFalse((self.out/'groups/targeted7').exists())
 def test_missing_raw_report_rejected(self):
  with self.assertRaises(AssertionError):self.run_groups(lambda g:self.argv(g,report=False))
  root=self.out/'groups/browser37';self.assertFalse((root/'actual-browser/report.json').exists());self.assertEqual(json.loads((root/'COMPLETED.json').read_text())['browser_exit_status'],0);self.assertTrue((root/'STOPPED.json').exists())
 def test_nonzero_first_child_preserves_command_streams(self):
  with self.assertRaises(AssertionError):self.run_groups(lambda g:self.argv(g,3))
  root=self.out/'groups/browser37';self.assertTrue((root/'actual-live-browser.stdout').exists()and(root/'actual-live-browser.stderr').exists());self.assertEqual(json.loads((root/'actual-live-browser.json').read_text())['exit_status'],3)
 def test_timeout_still_checks_after_closure_and_preserves_partial(self):
  groups=[dict(self.groups[0],timeout_seconds=.05)]
  with self.assertRaises(Exception):self.run_groups(lambda g:[sys.executable,'-c','import time;time.sleep(30)'],groups)
  root=self.out/'groups/browser37';self.assertTrue((root/'source-after.json').exists());self.assertEqual(len(self.calls),2);self.assertEqual(json.loads((root/'actual-live-browser.json').read_text())['error_type'],'TimeoutExpired');self.assertTrue((root/'SHA256.json').exists())
 def test_report_short_check_list_rejected(self):
  extra="r=json.loads((p/'report.json').read_text());r['checks']=[];(p/'report.json').write_text(json.dumps(r));"
  with self.assertRaises(AssertionError):self.run_groups(lambda g:self.argv(g,extra=extra))
  self.assertFalse((self.out/'groups/targeted7').exists())
 def test_report_failed_check_rejected(self):
  extra="r=json.loads((p/'report.json').read_text());r['checks'][0]['passed']=False;(p/'report.json').write_text(json.dumps(r));"
  with self.assertRaises(AssertionError):self.run_groups(lambda g:self.argv(g,extra=extra))
  self.assertTrue((self.out/'groups/browser37/actual-browser/report.json').exists())
 def test_actual_shallow_git_parent_object_is_available(self):
  origin=self.root/'origin';clone=self.root/'clone'
  def git(*args,cwd=None):return subprocess.check_output(['git',*args],cwd=cwd,stderr=subprocess.PIPE).decode('utf-8').strip()
  git('init','-q',str(origin));git('config','user.email','synthetic@example.invalid',cwd=origin);git('config','user.name','Synthetic source fixture',cwd=origin)
  (origin/'source.txt').write_text('actual local git fixture',encoding='utf-8');git('add','source.txt',cwd=origin);git('-c','commit.gpgsign=false','commit','-qm','runtime',cwd=origin);runtime=git('rev-parse','HEAD',cwd=origin)
  (origin/'binding.json').write_text('{}',encoding='utf-8');git('add','binding.json',cwd=origin);git('-c','commit.gpgsign=false','commit','-qm','harness',cwd=origin);head=git('rev-parse','HEAD',cwd=origin)
  git('clone','--depth','1',origin.as_uri(),str(clone));self.assertEqual(git('show','-s','--format=%P','HEAD',cwd=clone),'');self.assertEqual(worker.raw_commit_parents(clone,'HEAD'),[runtime]);git('fetch','--depth','1','origin',runtime,cwd=clone);self.assertEqual(worker.raw_commit_parents(clone,head),[runtime])
 def test_emitted_admission_bytes_exact_canonical_preissue_design(self):
  config={'synthetic_fixture':True,'a':[1,2],'unicode':'température'};path=self.root/'execution-admission.json';worker.write_admission(path,config);self.assertEqual(path.read_bytes(),worker.canonical(config)+b'\n');self.assertEqual(hashlib.sha256(path.read_bytes()[:-1]).hexdigest(),hashlib.sha256(worker.canonical(config)).hexdigest())
 def test_unbound_admission_rejects_before_producer_work(self):
  with self.assertRaises(AssertionError):worker.validate_config(None,None)
 def test_shell_metacharacters_are_literal_argv(self):
  root=self.root/'command';root.mkdir();argument='; touch SHOULD_NOT_EXIST $(echo invalid)';argv=[sys.executable,'-c','import sys,json;print(json.dumps(sys.argv[1:]))',argument];status=worker.command(root,'literal',argv,self.package,dict(os.environ),5);self.assertEqual(status,0);self.assertEqual(json.loads((root/'literal.stdout').read_text()),[argument]);self.assertFalse((self.package/'SHOULD_NOT_EXIST').exists())
if __name__=='__main__':unittest.main()
