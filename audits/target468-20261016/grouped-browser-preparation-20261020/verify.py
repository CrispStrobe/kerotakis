from pathlib import Path
import ast,hashlib,json
P=Path(__file__).resolve().parent
for n,h in json.loads((P/'SHA256.json').read_bytes()).items():assert hashlib.sha256((P/n).read_bytes()).hexdigest()==h,n
for p in P.rglob('*.py'):ast.parse(p.read_bytes(),filename=str(p))
proof=json.loads((P/'saved-readers/AST-EQUALITY-PROOF.json').read_bytes())
for block in proof['blocks']:
 raw=(P/'saved-readers'/block['new_module']).read_bytes();assert hashlib.sha256(raw).hexdigest()==block['new_module_sha256']
 fn=next(n for n in ast.parse(raw).body if isinstance(n,ast.FunctionDef));tree=ast.Module(body=fn.body[:-1],type_ignores=[])
 assert hashlib.sha256(ast.dump(tree,include_attributes=False).encode()).hexdigest()==block['assertion_block_AST_sha256']
 assert sum(isinstance(n,ast.Assert)for n in ast.walk(tree))==block['all_asserts_recursive']
worker=(P/'producer/grouped_browser.py').read_text();assert '\nCONFIG=None\n'in worker and '\nCONFIG_SHA256=None\n'in worker
print('PASS: source preparation hashes/syntax/extracted AST; actual new-release/grouped execution and acceptance remain pending.')
