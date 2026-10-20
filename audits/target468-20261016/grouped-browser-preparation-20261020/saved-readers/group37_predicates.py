"""Frozen original assertions; extracted unchanged, no dispatch/engine code."""
import hashlib, json, pathlib
sha = lambda b: hashlib.sha256(b).hexdigest()

def review(e, design, binding, S, B, O):
    for n, d in json.loads((e / 'SHA256.json').read_text()).items():
        p = pathlib.Path(n)
        assert not p.is_absolute() and '..' not in p.parts
        assert sha((e / p).read_bytes()) == d, n
    assert json.loads((e / 'STARTED.json').read_text())['script_sha256'] == design['script_sha256']
    done = json.loads((e / 'COMPLETED.json').read_text())
    assert done['browser_exit_status'] == 0 and done['harness_head'] == binding['harness_head'] and (done['runtime_source'] == S)
    source = json.loads((e / 'source.json').read_text())
    for key in ['harness_head', 'harness_tree', 'binding_sha256']:
        assert source[key] == binding[key]
    assert source['harness_script_sha256'] == binding['harness_script_sha256']
    assert json.loads((e / 'source-unchanged.json').read_text())['exit_status'] == 0
    report = json.loads((e / 'actual-browser/report.json').read_text())
    assert report['source_binding'] == binding['binding'] and report['all_checks_passed'] and (len(report['checks']) == 37) and all((c['passed'] for c in report['checks']))
    assert report['owner_credentials_supplied'] is False and report['new_corpus_executions'] == 0
    return report
