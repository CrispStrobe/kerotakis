#!/usr/bin/env python3
"""Print bounded compiler witnesses from structurally verified controlled builds."""
import hashlib, json
from pathlib import Path
root = Path('controlled-artifact-evidence')
extracted = root / 'extracted'
load = lambda name: json.loads((extracted / name).read_bytes())
collection = json.loads((root / 'collection.json').read_bytes())
assert collection['structural_collection_pass'] is True
witnesses = dict(identity=load('identity.json'), routing=load('routing-witness.json'),
                 variants=collection['variants'], environments={}, native_caches={}, native_log_lines={})
for variant in ('baseline', 'candidate'):
    receipt = load(variant + '/receipt.json')
    witnesses['environments'][variant] = receipt['environment']
    witnesses['native_caches'][variant] = {}
    for path in (extracted / variant).glob('native-cache-*.txt'):
        witnesses['native_caches'][variant][path.name] = [line for line in path.read_text().splitlines()
            if any(key in line for key in ('CMAKE_C_COMPILER:', 'CMAKE_CXX_COMPILER:',
              'CMAKE_C_FLAGS', 'CMAKE_CXX_FLAGS', 'CMAKE_BUILD_TYPE:', 'CMAKE_LINKER:'))]
    lines = (extracted / variant / 'build.log').read_text().splitlines()
    matches = [line for line in lines if any(key in line for key in
               ('/usr/bin/gcc', '/usr/bin/g++', 'CFLAGS', 'CXXFLAGS', 'CMAKE_C_COMPILER', 'CMAKE_CXX_COMPILER'))]
    witnesses['native_log_lines'][variant] = dict(match_count=len(matches), first_lines=matches[:30])
assert witnesses['environments']['baseline'] == witnesses['environments']['candidate']
witnesses.update(independent_effective_compiler_review_complete=False, candidate_executed=False,
                 artifact_zip_sha256='e121b7648165c121f8e9826ffac50f497cff46e884d505aa0a3ef08efabcaa16')
(root / 'compiler-witnesses.json').write_text(json.dumps(witnesses, indent=2) + '\n')
print('CONTROLLED_BUILD_WITNESSES_JSON=' + json.dumps(witnesses, sort_keys=True))
