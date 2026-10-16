#!/usr/bin/env python3
"""Execute a frozen small API follow-up once; keep credentials out of evidence."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import time
import urllib.error
import urllib.request


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('design', 'out', 'key-file', 'cookie-file'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--origin', required=True)
    args = parser.parse_args()
    assert args.origin.startswith('https://') and '?' not in args.origin
    design_raw = args.design.read_bytes()
    design = json.loads(design_raw)
    key = json.loads(args.key_file.read_text())['key']
    cookie = args.cookie_file.read_text().strip()
    args.out.mkdir(mode=0o700, exist_ok=False)
    (args.out / 'frozen-design.json').write_bytes(design_raw)
    opener = urllib.request.build_opener(NoRedirect())
    headers = {'Cookie': cookie, 'Authorization': 'Bearer ' + key}
    with opener.open(urllib.request.Request(args.origin + '/api/health', headers=headers), timeout=30) as response:
        health_raw = response.read(100_001)
        assert response.status == 200
    health = json.loads(health_raw)
    (args.out / 'health-before.json').write_bytes(health_raw)
    assert health['configured'] and health['revision'] == design['api_revision']
    assert health['engine'] == design['engine']
    rows = []
    for case in design['cases']:
        assert sha(args.design.read_bytes()) == sha(design_raw)
        assert sha(case['script'].encode()) == case['script_sha256']
        folder = args.out / case['id']
        folder.mkdir()
        request_raw = json.dumps({'script': case['script']}, ensure_ascii=False).encode()
        (folder / 'request.json').write_bytes(request_raw)
        issued = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  'design_sha256': sha(design_raw), 'request_sha256': sha(request_raw),
                  'script_sha256': case['script_sha256'], 'attempt': 1}
        with (folder / 'issued.json').open('x') as handle:
            json.dump(issued, handle, indent=2)
        request = urllib.request.Request(args.origin + '/api/run', data=request_raw,
                                        headers={**headers, 'Content-Type': 'application/json'})
        started = time.monotonic()
        try:
            response = opener.open(request, timeout=70)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            body = response.read(2_000_001)
            status = response.status
        elapsed = time.monotonic() - started
        assert key.encode() not in body and cookie.encode() not in body
        (folder / 'response.body').write_bytes(body)
        receipt = {'id': case['id'], 'http_status': status, 'elapsed_seconds': elapsed,
                   'response_sha256': sha(body), 'script_sha256': case['script_sha256'],
                   'attempts': 1, 'physical_verdict': 'not_assigned'}
        (folder / 'http.json').write_text(json.dumps(receipt, indent=2) + '\n')
        assert len(body) <= 2_000_000
        result = json.loads(body)
        if status == 200:
            assert result['engine'] == design['engine']
            assert result['script_sha256'] == case['script_sha256']
            receipt.update(execution_status=result['execution_status'],
                           exit_status=result['exit_status'], observations=len(result['observations']))
        else:
            receipt['execution_status'] = 'http_failure_preserved'
        rows.append(receipt)
        (args.out / 'results.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(case['id'], status, receipt['execution_status'], flush=True)
        time.sleep(3.1)
    (args.out / 'completion.json').write_text(json.dumps({'records': len(rows),
        'design_sha256': sha(design_raw), 'experiment_retries': 0}, indent=2) + '\n')


if __name__ == '__main__':
    main()
