"""Frozen original assertions; extracted unchanged, no dispatch/engine code."""
import hashlib, json, pathlib
sha = lambda b: hashlib.sha256(b).hexdigest()

def review(e, design, binding, S, B, O):
    for n, d in json.loads((e / 'SHA256.json').read_text()).items():
        p = pathlib.Path(n)
        assert not p.is_absolute() and '..' not in p.parts
        assert sha((e / p).read_bytes()) == d, n
    assert json.loads((e / 'STARTED.json').read_text())['script_sha256'] == design['worker_sha256']
    assert json.loads((e / 'COMPLETED.json').read_text())['browser_exit_status'] == 0
    assert json.loads((e / 'source-unchanged.json').read_text())['exit_status'] == 0
    report = json.loads((e / 'actual-browser/report.json').read_text())
    binding = design['binding']
    assert report['source_binding'] == binding['binding'] and binding['runtime_source'] == S
    assert report['all_checks_passed'] and len(report['checks']) == 7 and all((c['passed'] for c in report['checks'])) and (report['owner_credentials_supplied'] is False) and (report['new_corpus_executions'] == 0)
    assert set(report['assets']) == set(binding['binding']['frontend_assets'])
    for n, d in binding['binding']['frontend_assets'].items():
        assert report['assets'][n]['status'] == 200 and report['assets'][n]['sha256'] == d
    raw = report['raw']
    assert set(raw) == {'dry', 'aqueous', 'transition', 'de', 'fr', 'methanol', 'drain'}

    def vessel(result):
        return result['bench']['vessels'][0]

    def boundary(result):
        return any((ev.get('reason', {}).get('key') == 'not-modeled.peroxide-rich-medium' for step in result['steps'] for ev in step['events'] if ev.get('reason')))

    def withdrawn(result):
        v = vessel(result)
        return v.get('solution') is None and v['free_proton'] == 0 and (v['free_hydroxide'] == 0)

    def amount(result, species):
        return sum((p['moles'] for p in vessel(result)['contents'] if p['species'] == species))
    for case in raw.values():
        rev = case['meta']['git_rev']
        assert len(rev) >= 7 and S.startswith(rev)
    for name in ['dry', 'de', 'fr']:
        case = raw[name]
        assert case['before_calls'] == 0 and boundary(case['before']) and withdrawn(case['before'])
        assert len(case['before']['steps']) == 4 and 0 < amount(case['before'], 'H2O2') < 0.326375 and (amount(case['before'], 'water') > 0)
    case = raw['aqueous']
    assert case['before_calls'] > 0 and (not boundary(case['before']))
    ph = vessel(case['before'])['solution']['ph']
    assert isinstance(ph, (float, int)) and abs(ph) < 100
    case = raw['transition']
    assert case['before_calls'] > 0 and vessel(case['before'])['solution'] is not None and (case['after_calls'] == case['before_calls']) and boundary(case['after']) and withdrawn(case['after'])
    for locale, label, notice in [('de', 'Peroxidzersetzung', 'Diese peroxidhaltige Flüssigkeit'), ('fr', 'Décomposition du peroxyde', 'Ce liquide')]:
        lines = [s['rendered'] for s in raw[locale]['before']['steps']]
        text = '\n'.join((x if isinstance(x, str) else '\n'.join(x) for x in lines))
        assert label in text and notice in text and ('peroxide-decomposition' not in text) and ('this peroxide-containing liquid' not in text)
    expected = json.loads((B / 'partition-factor-targeted-before-binding.json').read_text())
    case = raw['methanol']
    assert abs(vessel(case['before'])['temperature'] - 298.15) < 1e-09
    peaks = [p for step in case['before']['steps'] for event in step['events'] if event['event'] == 'chromatographed' for p in event['peaks'] if p['species'] == 'methanol']
    assert len(peaks) == 1
    k = peaks[0]['partition_k']
    assert abs((k - expected['reference_methanol_gamma_ratio']) / expected['reference_methanol_gamma_ratio']) < 0.0001
    case = raw['drain']
    assert len(case['before']['bench']['vessels']) == 1 and len(case['after']['bench']['vessels']) == 2
    shares = [sum((p['moles'] for p in v['contents'] if p['species'] == 'ethanol')) for v in case['after']['bench']['vessels']]
    assert all((0 < n < 0.01 for n in shares)) and abs(sum(shares) - 0.01) < 1e-12
    for species, expected_moles in [('water', 2.0), ('hexane', 0.5)]:
        assert abs(sum((p['moles'] for v in case['after']['bench']['vessels'] for p in v['contents'] if p['species'] == species)) - expected_moles) < 1e-10
    base = json.loads((O / 'BROWSER37-ACCEPTED.json').read_text())
    api = json.loads((B / 'browser-contract-848-production-delivery-20261020/API-ACCEPTED.json').read_text())
    assert base['source'] == api['source'] == S and base['engine'] == api['engine'] == binding['engine'] and api['eleven_original_protocols_pass'] and (api['concurrency'] == [200, 429])
    return report
