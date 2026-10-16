#!/usr/bin/env python3
"""Two frozen command-exposure controls; credentials private, no ambiguous retries."""
import argparse
import hashlib
import json
from pathlib import Path
import time
import urllib.error
import urllib.parse
import urllib.request


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--origin', required=True)
    parser.add_argument('--design', type=Path, default=Path(__file__).with_name('preview-controls-design.json'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--owner-key-file', type=Path, required=True)
    parser.add_argument('--cookie-file', type=Path, required=True)
    parser.add_argument('--corpus-completion', type=Path, required=True)
    args = parser.parse_args()
    assert json.loads(args.corpus_completion.read_text())['records'] == 441
    root = Path(__file__).resolve().parent
    design_raw = args.design.read_bytes()
    design = json.loads(design_raw)
    origin = urllib.parse.urlsplit(args.origin)
    assert origin.scheme == 'https' and origin.hostname and not origin.path
    assert not origin.username and not origin.password and not origin.query and not origin.fragment
    key = json.loads(args.owner_key_file.read_text())['key']
    cookie = args.cookie_file.read_text().strip()
    args.out.mkdir(parents=True, exist_ok=False)
    opener = urllib.request.build_opener(NoRedirect())

    def request(endpoint, script=None):
        data = None if script is None else json.dumps({'script': script}, ensure_ascii=False).encode()
        headers = {'Cookie': cookie}
        if data is not None:
            headers.update(Authorization='Bearer ' + key, **{'Content-Type': 'application/json'})
        try:
            result = opener.open(urllib.request.Request(args.origin + '/api/' + endpoint, data=data, headers=headers), timeout=60)
        except urllib.error.HTTPError as exc:
            result = exc
        with result:
            raw = result.read(2_100_001)
        assert len(raw) <= 2_100_000 and key.encode() not in raw
        assert cookie.split('=', 1)[1].encode() not in raw
        return result.code, raw

    status, raw = request('health')
    (args.out / 'health-before.json').write_bytes(raw)
    health = json.loads(raw)
    assert status == 200 and health['configured'] and health['revision'] == design['api_revision']
    assert health['engine'] == design['engine'] and health['execution_backend'] == 'microvm'
    rows = []
    for case in design['cases']:
        assert sha(case['script'].encode()) == case['script_sha256']
        target = args.out / case['id']
        target.mkdir()
        (target / 'issued.json').write_text(json.dumps({'script_sha256': case['script_sha256']}) + '\n')
        status, raw = request('run', case['script'])
        (target / 'response.json').write_bytes(raw)
        data = json.loads(raw)
        assert status == 200 and data['script_sha256'] == case['script_sha256'] and data['engine'] == design['engine']
        assert data['execution_status'] == 'completed' and data['exit_status'] == 0
        assert data['observations'] and data['physical_verdict'] == 'not_assigned'
        rows.append({'id': case['id'], 'http_status': status, 'execution_status': data['execution_status'],
            'exit_status': data['exit_status'], 'observations': len(data['observations']),
            'script_sha256': case['script_sha256'], 'response_sha256': sha(raw)})
        time.sleep(3.1)
    refusals = []
    for script in ('extract v1 v2 carbon dioxide 1mol stages 17',
                   'titrate v1 sodium chloride 1M 1mL until ph 7 max 2001', 'help', 'lang de'):
        status, raw = request('run', script)
        assert status == 422
        refusals.append({'script_sha256': sha(script.encode()), 'http_status': status, 'response_sha256': sha(raw)})
    status, raw = request('health')
    (args.out / 'health-after.json').write_bytes(raw)
    assert status == 200 and json.loads(raw) == health
    receipt = {'api_revision': design['api_revision'], 'engine': design['engine'],
        'design_sha256': sha(design_raw), 'runner_sha256': sha(Path(__file__).read_bytes()),
        'cases': rows, 'admission_refusals': refusals, 'ambiguous_retries': 0,
        'physical_verdict': 'not_assigned', 'qualification': 'Command exposure and refusal controls only; no empirical calibration.'}
    (args.out / 'completion.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
