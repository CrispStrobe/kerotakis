"""Real temporary external-command provenance tests, never product acceptance."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import signal
import sys
import tempfile
import time
import unittest
from review_twice import run_saved, check_original_invocation, stop_owned_group


class Commands(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.calls = 0

    def closure(self):
        self.calls += 1

    def assert_stopped(self, pid):
        # kill(2) delivery is asynchronous. Wait for observable termination;
        # the rejected implementation never kills the TERM-ignoring child.
        proc = Path('/proc') / str(pid) / 'stat'
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            try:
                if proc.read_text().split()[2] == 'Z': return
            except FileNotFoundError:
                return
            time.sleep(.01)
        self.fail('Timed-out owned descendant is still executing')

    def call(self, code, timeout=5):
        run_saved(self.root, 'synthetic', [sys.executable, '-c', code],
                  self.root, dict(os.environ), self.closure, timeout)
        return json.loads((self.root / 'synthetic.execution.json').read_bytes())

    def test_actual_command_streams_hash_and_exact_argv(self):
        code = "print('SYNTHETIC_EXTERNAL_COMMAND_ONLY')"
        result = self.call(code)
        raw = (self.root / 'synthetic.stdout').read_bytes()
        self.assertEqual(raw, b'SYNTHETIC_EXTERNAL_COMMAND_ONLY\n')
        self.assertEqual(result['argv'], [sys.executable, '-c', code])
        self.assertEqual(result['stdout_sha256'], hashlib.sha256(raw).hexdigest())
        self.assertEqual(result['exit_status'], 0)
        self.assertEqual(self.calls, 2)

    def test_nonzero_child_is_not_accepted_and_streams_remain(self):
        with self.assertRaises(ValueError):
            self.call("print('first real partial');raise SystemExit(7)")
        meta = json.loads((self.root / 'synthetic.execution.json').read_bytes())
        self.assertEqual(meta['exit_status'], 7)
        self.assertIn(b'first real partial', (self.root / 'synthetic.stdout').read_bytes())

    def test_timeout_kills_owned_grandchild_group(self):
        # Skip environment site startup and fork a real child directly. The
        # fixture measures process-tree termination, not site import latency.
        code = ('import os,time;\n'
                'pid=os.fork()\n'
                'if pid==0:\n time.sleep(30)\n'
                'else:\n open("grandchild.pid","w").write(str(pid))\n time.sleep(30)\n')
        with self.assertRaises(subprocess.TimeoutExpired):
            run_saved(self.root, 'synthetic', [sys.executable, '-S', '-c', code],
                      self.root, dict(os.environ), self.closure, timeout=1)
        meta = json.loads((self.root / 'synthetic.execution.json').read_bytes())
        self.assertEqual(meta['error_type'], 'TimeoutExpired')
        pid = int((self.root / 'grandchild.pid').read_text())
        self.assert_stopped(pid)

    def test_first_open_error_is_preserved(self):
        (self.root / 'synthetic.stderr').mkdir()
        with self.assertRaises(FileExistsError): self.call('pass')
        meta = json.loads((self.root / 'synthetic.execution.json').read_bytes())
        self.assertEqual(meta['error_type'], 'FileExistsError')
        self.assertIsNone(meta['stderr_sha256'])
        self.assertIsNone(meta['exit_status'])

    def test_timeout_kills_grandchild_ignoring_TERM_after_leader_exits(self):
        code = ('import os,time,signal;\n'
                'pid=os.fork()\n'
                'if pid==0:\n signal.signal(signal.SIGTERM,signal.SIG_IGN)\n'
                ' open("ignore-ready.pid","w").write(str(os.getpid()))\n time.sleep(30)\n'
                'else:\n time.sleep(30)\n')
        with self.assertRaises(subprocess.TimeoutExpired):
            run_saved(self.root, 'synthetic', [sys.executable, '-S', '-c', code],
                      self.root, dict(os.environ), self.closure, timeout=1)
        pid = int((self.root / 'ignore-ready.pid').read_text())
        def cleanup():
            try: os.kill(pid, signal.SIGKILL)
            except ProcessLookupError: pass
        self.addCleanup(cleanup)
        self.assert_stopped(pid)

    def test_cleanup_works_when_leader_already_exited_before_called(self):
        code = ('import os,time,signal;\n'
                'pid=os.fork()\n'
                'if pid==0:\n signal.signal(signal.SIGTERM,signal.SIG_IGN)\n'
                ' open("exited-ready.pid","w").write(str(os.getpid()))\n time.sleep(30)\n'
                'else:\n'
                ' while not os.path.exists("exited-ready.pid"): time.sleep(.001)\n'
                ' os._exit(0)\n')
        leader = subprocess.Popen([sys.executable, '-S', '-c', code], cwd=self.root,
                                  start_new_session=True)
        self.assertEqual(leader.wait(timeout=5), 0)
        self.assertIsNotNone(leader.poll())
        pid = int((self.root / 'exited-ready.pid').read_text())
        def cleanup():
            try: os.kill(pid, signal.SIGKILL)
            except ProcessLookupError: pass
        self.addCleanup(cleanup)
        stop_owned_group(leader)
        self.assert_stopped(pid)

    def test_existing_stream_is_not_overwritten(self):
        (self.root / 'synthetic.stdout').write_bytes(b'original partial')
        with self.assertRaises(FileExistsError): self.call('pass')
        self.assertEqual((self.root / 'synthetic.stdout').read_bytes(), b'original partial')

    def test_source_change_after_child_rejects(self):
        def changed():
            self.calls += 1
            if self.calls == 2: raise ValueError('actual source closure changed')
        with self.assertRaises(ValueError):
            run_saved(self.root, 'synthetic', [sys.executable, '-c', 'pass'],
                      self.root, dict(os.environ), changed)
        self.assertEqual(json.loads((self.root / 'synthetic.execution.json').read_bytes())['exit_status'], 0)

    def test_fabricated_nested_invocation_cannot_skip_actual_sources(self):
        (self.root / 'ORIGINAL-READER-EXECUTION.json').write_text(json.dumps({
            'exit_status': 0, 'reader_sha256': 'fabricated',
            'predicates_sha256': 'fabricated', 'browser_engine_provider_calls': 0}))
        with self.assertRaises(ValueError): check_original_invocation(self.root, 'locale18')


if __name__ == '__main__':
    unittest.main()
