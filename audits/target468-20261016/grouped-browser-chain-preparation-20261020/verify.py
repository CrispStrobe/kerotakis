from pathlib import Path
import ast,hashlib,json
P=Path(__file__).resolve().parent
for name,want in json.loads((P/'SHA256.json').read_bytes()).items():
 assert hashlib.sha256((P/name).read_bytes()).hexdigest()==want,name
for path in P.rglob('*.py'):ast.parse(path.read_bytes(),filename=str(path))
collector=(P/'host/collect_once.py').read_text()
assert 'ApiGetKernelSessionStatusRequest'in collector and 'version_label'in collector
print('PASS: frozen source-chain hashes/syntax only; actual bound issue/collection/raw reviews remain pending.')
