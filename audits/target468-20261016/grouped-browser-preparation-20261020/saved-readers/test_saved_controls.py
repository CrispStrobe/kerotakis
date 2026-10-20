"""Synthetic filesystem/AST orchestration controls ONLY, never browser acceptance."""
import ast,hashlib,json,pathlib,tempfile,unittest
from saved_envelope import manifest,ExplicitAliases
P=pathlib.Path(__file__).resolve().parent
class SavedControls(unittest.TestCase):
 def test_exact_original_block_fingerprints(self):
  for proof in json.loads((P/'AST-EQUALITY-PROOF.json').read_text())['blocks']:
   t=ast.parse((P/proof['new_module']).read_bytes());fn=next(n for n in t.body if isinstance(n,ast.FunctionDef));b=ast.Module(body=fn.body[:-1],type_ignores=[]);self.assertEqual(hashlib.sha256(ast.dump(b,include_attributes=False).encode()).hexdigest(),proof['assertion_block_AST_sha256'])
 def test_changed_assertion_cannot_keep_AST_identity(self):
  proof=json.loads((P/'AST-EQUALITY-PROOF.json').read_text())['blocks'][1];t=ast.parse((P/proof['new_module']).read_bytes());fn=next(n for n in t.body if isinstance(n,ast.FunctionDef));assertion=next(n for x in fn.body for n in ast.walk(x) if isinstance(n,ast.Assert));assertion.test=ast.Constant(value=True);b=ast.Module(body=fn.body[:-1],type_ignores=[]);self.assertNotEqual(hashlib.sha256(ast.dump(b,include_attributes=False).encode()).hexdigest(),proof['assertion_block_AST_sha256'])
 def tree(self):
  t=tempfile.TemporaryDirectory();self.addCleanup(t.cleanup);p=pathlib.Path(t.name);(p/'raw.json').write_text('{"SYNTHETIC_ORCHESTRATION_ONLY":true}\n');h=hashlib.sha256((p/'raw.json').read_bytes()).hexdigest();(p/'SHA256.json').write_text(json.dumps({'raw.json':h}));return p
 def test_exact_manifest(self):self.assertEqual(set(manifest(self.tree())),{'raw.json'})
 def test_unmanifested_file_refused(self):
  p=self.tree();(p/'unlisted').write_text('not a real report')
  with self.assertRaises(AssertionError):manifest(p)
 def test_mutated_raw_refused(self):
  p=self.tree();(p/'raw.json').write_text('changed')
  with self.assertRaises(AssertionError):manifest(p)
 def test_traversal_refused(self):
  p=self.tree();(p/'SHA256.json').write_text(json.dumps({'../escape':'0'*64}))
  with self.assertRaises(AssertionError):manifest(p)
 def test_symlink_refused(self):
  p=self.tree();(p/'linked').symlink_to(p/'raw.json')
  with self.assertRaises(AssertionError):manifest(p)
 def test_aliases_are_read_only_actual_input_paths(self):
  p=self.tree();aliases=ExplicitAliases({'literal-old-alias':p/'raw.json'});self.assertEqual((aliases/'literal-old-alias').read_bytes(),(p/'raw.json').read_bytes())
  with self.assertRaises(AssertionError):aliases/'unapproved-old-global-path'
if __name__=='__main__':unittest.main()
