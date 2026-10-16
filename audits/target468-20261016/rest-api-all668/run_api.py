#!/usr/bin/env python3
"""Execute the frozen corpus serially through one immutable authenticated REST deployment."""
import argparse
from collections import Counter
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--design', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--owner-key-file', type=Path, required=True)
    parser.add_argument('--protection-cookie-file', type=Path, required=True)
    args = parser.parse_args()
    raw_design = args.design.read_bytes()
    design = json.loads(raw_design)
    cases = design['cases']
    assert len(cases) == design['records'] == 668
    assert len({c['id'].replace(':', '_') for c in cases}) == 668
    for c in cases:
        assert sha(c['script'].encode()) == c['script_sha256']
    origin = urllib.parse.urlsplit(design['origin'])
    assert origin.scheme == 'https' and origin.hostname and origin.path == ''
    assert not origin.username and not origin.password and not origin.query and not origin.fragment
    key = json.loads(args.owner_key_file.read_text())['key']
    cookie = args.protection_cookie_file.read_text().strip()
    assert len(key) >= 32 and cookie and not any(c in key + cookie for c in '\r\n')
    opener = urllib.request.build_opener(NoRedirect())
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out / 'design.json', design)
    started = time.monotonic()
    last_start = 0.0
    results = []
    attempts = 0

    def request(path, data=None):
        headers = {'Cookie': cookie}
        if data is not None:
            headers.update({'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json'})
        request = urllib.request.Request(design['origin'] + path, data=data, headers=headers)
        start = time.monotonic()
        try:
            response = opener.open(request, timeout=75)
        except urllib.error.HTTPError as exc:
            response = exc
        with response:
            raw = response.read(2_200_001)
            status = response.code
        assert len(raw) <= 2_200_000
        # Neither API nor protection credentials may be reflected into evidence.
        assert key.encode() not in raw
        for part in cookie.split('; '):
            assert part.split('=', 1)[-1].encode() not in raw
        return status, raw, time.monotonic() - start

    def verify_health(name):
        status, raw, _ = request('/api/health')
        (args.out / name).write_bytes(raw)
        assert status == 200
        health = json.loads(raw)
        assert health['configured'] and health['mode'] == 'vercel'
        assert health['execution_backend'] == 'microvm'
        assert health['revision'] == design['api_revision'] and health['engine'] == design['engine']
        return health

    try:
        health = verify_health('health-before.json')
        save(args.out / 'binding.json', {
            'started_utc': utc(), 'design_sha256': sha(raw_design),
            'runner_sha256': sha(Path(__file__).read_bytes()), 'health': health,
            'input_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
            'credentials_in_evidence': False, 'redirects_allowed': False,
            'local_engine_execution': False, 'script_adaptations': 0})
        for case in cases:
            target = args.out / case['id'].replace(':', '_')
            target.mkdir(exist_ok=False)
            body = json.dumps({'script': case['script']}, ensure_ascii=False).encode()
            (target / 'request.json').write_bytes(body)
            for attempt in range(6):
                time.sleep(max(0, last_start + design['minimum_seconds_between_starts'] - time.monotonic()))
                last_start = time.monotonic()
                save(target / f'issued-{attempt + 1}.json', {'issued_utc': utc(), 'request_sha256': sha(body), 'attempt': attempt + 1})
                status, raw, elapsed = request('/api/run', body)
                attempts += 1
                (target / f'response-{attempt + 1}.body').write_bytes(raw)
                save(target / f'http-{attempt + 1}.json', {'http_status': status, 'elapsed_seconds': elapsed, 'response_sha256': sha(raw)})
                if status != 429:
                    break
                if attempt == 5:
                    raise RuntimeError('Explicit admission retries exhausted')
                time.sleep(60)
            assert status == 200, 'Unexpected HTTP result; preserve and stop, no ambiguous retry'
            output = json.loads(raw, parse_constant=lambda _: (_ for _ in ()).throw(ValueError('Nonfinite JSON')))
            assert output['engine'] == design['engine']
            assert output['script_sha256'] == case['script_sha256']
            assert output['execution_status'] in ('completed', 'refused', 'timeout', 'output_limit')
            row = {'id': case['id'], 'http_status': status, 'attempts': attempt + 1,
                   'elapsed_seconds': elapsed, 'execution_status': output['execution_status'],
                   'exit_status': output['exit_status'], 'script_sha256': case['script_sha256'],
                   'response_sha256': sha(raw), 'observations': len(output['observations']),
                   'physical_verdict': 'not_assigned'}
            save(target / 'execution.json', row)
            results.append(row)
            save(args.out / 'progress.json', {'updated_utc': utc(), 'records': len(results),
                  'http_attempts': attempts, 'execution_counts': dict(Counter(r['execution_status'] for r in results)),
                  'results': results, 'physical_verdict': 'not_assigned'})
            print(case['id'], status, output['execution_status'], len(results), flush=True)
        verify_health('health-after.json')
        save(args.out / 'completion.json', {'completed_utc': utc(), 'records': len(results),
             'http_attempts': attempts, 'paced_seconds': time.monotonic() - started,
             'execution_counts': dict(Counter(r['execution_status'] for r in results)),
             'physical_verdict': 'not_assigned'})
    except Exception as exc:
        save(args.out / 'stopped.json', {'recorded_utc': utc(), 'reason_type': type(exc).__name__,
             'records_with_verified_response': len(results), 'http_attempts': attempts,
             'policy': 'Keep all first evidence; do not resubmit issued ambiguous requests or overwrite this directory.'})
        raise


if __name__ == '__main__':
    main()
