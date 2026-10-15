"""Run the frozen 200 scripts through production, preserving every HTTP attempt.

Usage: run_api.py PRIVATE_KEY_JSON NEW_OUTPUT_DIRECTORY VERIFIED_CLI_EXECUTIONS
Only remote execution occurs. No redirects, parallel jobs or blind retries.
HTTP429 may be retried because admission refused execution; ambiguous transport
failures and HTTP5xx are retained without automatic re-execution.
"""
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import sys
import time
from urllib.error import HTTPError, URLError
from urllib.request import HTTPRedirectHandler, HTTPSHandler, ProxyHandler, Request, build_opener

ORIGIN = 'https://kerotakis-web-payload.vercel.app'
INTERVAL = 3.1  # Below both API30/min and initial Sandbox20vCPU/min.
MAX_RESPONSE = 2_000_000


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + '\n')
    temporary.replace(path)


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        return None


def main():
    key_path, output, cli = map(Path, sys.argv[1:])
    key = json.loads(key_path.read_text())['key']
    assert isinstance(key, str) and 32 <= len(key) <= 512 and '\n' not in key and '\r' not in key
    root = Path(__file__).resolve().parent
    design = root / 'design'
    freeze = json.loads((design / 'freeze.json').read_text())
    for filename, digest in freeze['files'].items():
        assert sha(design / filename) == digest, filename
    records = sorted(json.loads((design / 'manifest.json').read_text())['records'], key=lambda r: r['rank'])
    assert len(records) == len({r['id'] for r in records}) == 200
    cli_results = {r['id']: r for r in json.loads((cli / 'executions.json').read_text())}
    assert len(cli_results) == 200
    cli_binding = json.loads((cli / 'binding.json').read_text())
    opener = build_opener(ProxyHandler({}), NoRedirect(), HTTPSHandler())

    def http(path, script=None):
        headers = {'Accept': 'application/json'}
        body = None
        if script is not None:
            body = json.dumps({'script': script}, ensure_ascii=False).encode()
            headers.update({'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json'})
        request = Request(ORIGIN + path, data=body, headers=headers)
        try:
            response = opener.open(request, timeout=45)
        except HTTPError as exc:
            response = exc
        except (URLError, TimeoutError, OSError) as exc:
            return 0, {}, b'', type(exc).__name__
        with response:
            try:
                raw = response.read(MAX_RESPONSE + 1)
            except (URLError, TimeoutError, OSError) as exc:
                return response.status, {}, b'', type(exc).__name__
            assert len(raw) <= MAX_RESPONSE, 'Oversized API response'
            assert key.encode() not in raw, 'Unexpected credential reflection; do not publish response'
            selected = {name: response.headers[name] for name in
                ('Content-Type', 'Retry-After', 'Cache-Control', 'X-Vercel-Id') if name in response.headers}
            return response.status, selected, raw, None

    code, _headers, raw, error = http('/api/health')
    assert code == 200 and not error
    health = json.loads(raw)
    assert health['configured'] and health['execution_backend'] == 'microvm'
    assert health['engine']['binary_sha256'] == cli_binding['binary_sha256'], 'Different engine: freeze a different comparison'
    output.mkdir()
    binding = {'created_utc': datetime.now(timezone.utc).isoformat(), 'origin': ORIGIN,
        'health': health, 'runner_sha256': sha(Path(__file__)),
        'design_freeze_sha256': sha(design / 'freeze.json'), 'manifest_sha256': sha(design / 'manifest.json'),
        'cli_executions_sha256': sha(cli / 'executions.json'), 'cli_binding_sha256': sha(cli / 'binding.json'),
        'record_count': 200, 'minimum_request_interval_seconds': INTERVAL,
        'retry_policy': 'Only explicit HTTP429 admission refusal; at most5 retries, at least60 seconds. No ambiguous retries.',
        'physical_verdict': 'not_assigned'}
    save(output / 'binding.json', binding)
    shutil.copyfile(__file__, output / 'frozen-runner.py')
    (output / 'initial-health.json').write_bytes(raw)
    results = []
    last_request = 0.0
    for record in records:
        assert sha(Path(__file__)) == binding['runner_sha256'], 'Runner changed during execution'
        assert sha(design / 'freeze.json') == binding['design_freeze_sha256']
        assert shutil.disk_usage(output).free >= 1024 ** 3, 'Evidence storage gate closed'
        if results and len(results) % 25 == 0:
            status, _, raw_health, health_error = http('/api/health')
            assert status == 200 and not health_error and json.loads(raw_health) == health, 'API identity/configuration changed'
            (output / f'health-after-{len(results):03d}.json').write_bytes(raw_health)
        script_path = design / record['script']
        assert sha(script_path) == freeze['files'][record['script']]
        script = script_path.read_text()
        stem = f"{record['rank']:03d}-{record['id']}"
        for attempt in range(1, 7):
            time.sleep(max(0, last_request + INTERVAL - time.monotonic()))
            last_request = time.monotonic()
            started = time.monotonic()
            status, headers, body, transport_error = http('/api/run', script)
            elapsed = time.monotonic() - started
            body_path = output / f'{stem}.attempt-{attempt:02d}.body'
            body_path.write_bytes(body)
            save(output / f'{stem}.attempt-{attempt:02d}.json', {
                'rank': record['rank'], 'id': record['id'], 'attempt': attempt,
                'http_status': status, 'headers': headers, 'elapsed_seconds': elapsed,
                'transport_error': transport_error, 'response_sha256': sha(body_path),
                'script_sha256': sha(script_path)})
            if status != 429:
                break
            assert attempt < 6, 'Persistent quota refusal; preserve attempts and resume deliberately'
            retry = headers.get('Retry-After', '60')
            delay = max(60, min(300, int(retry) if retry.isdigit() else 60))
            print(record['rank'], record['id'], 'admission429; waiting', delay, flush=True)
            time.sleep(delay)
        assert status not in (301, 302, 303, 307, 308, 401, 403), 'Authentication/redirect failure; stop batch'
        response = None
        try:
            response = json.loads(body)
        except (ValueError, UnicodeError):
            pass
        observations = []
        execution, exit_status = 'transport_error' if transport_error else f'http_{status}', None
        if status == 200 and not transport_error:
            assert isinstance(response, dict) and response['engine'] == health['engine'], 'Engine identity changed'
            assert response['script_sha256'] == sha(script_path) and response['physical_verdict'] == 'not_assigned'
            execution, exit_status, observations = response['execution_status'], response['exit_status'], response['observations']
            assert isinstance(observations, list) and all(isinstance(row, dict) for row in observations)
        baseline = cli_results[record['id']]
        assert baseline['script_sha256'] == sha(script_path)
        baseline_stdout = cli / (stem + '.stdout')
        assert sha(baseline_stdout) == baseline['stdout_sha256']
        baseline_rows = [json.loads(line) for line in baseline_stdout.read_text().splitlines()]
        status_matches = execution == 'completed' and baseline['execution_status'] == 'completed' or execution == 'refused' and baseline['execution_status'] == 'nonzero_exit'
        result = {'rank': record['rank'], 'id': record['id'], 'http_status': status,
            'attempts': attempt, 'execution_status': execution, 'exit_status': exit_status,
            'elapsed_seconds': elapsed, 'observations': len(observations),
            'script_sha256': sha(script_path), 'response_file': body_path.name,
            'response_sha256': sha(body_path), 'transport_error': transport_error,
            'cli_execution_status': baseline['execution_status'], 'cli_exit_status': baseline['exit_status'],
            'execution_matches_cli': bool(status_matches and exit_status == baseline['exit_status']),
            'observations_equal_cli': status == 200 and observations == baseline_rows,
            'all_add_commands_reached': sum(row.get('operator', {}).get('op') in ('add', 'add_material') for row in observations)
                == sum(line.startswith('add ') for line in script.splitlines()),
            'first_diagnostic': response.get('diagnostics', response.get('detail', ''))[:600] if isinstance(response, dict) else '',
            'physical_verdict': 'not_assigned'}
        save(output / (stem + '.result.json'), result)
        results.append(result)
        save(output / 'executions.json', results)
        print(record['rank'], record['id'], execution, 'HTTP', status,
              'CLI-execution-match', result['execution_matches_cli'], 'observation-match', result['observations_equal_cli'], flush=True)
    save(output / 'completed.json', {'records': len(results),
        'execution_counts': dict(Counter(r['execution_status'] for r in results)),
        'http_counts': dict(Counter(str(r['http_status']) for r in results)),
        'execution_matches_cli': sum(r['execution_matches_cli'] for r in results),
        'observations_equal_cli': sum(r['observations_equal_cli'] for r in results),
        'executions_sha256': sha(output / 'executions.json'), 'physical_verdict': 'not_assigned'})
    print('COMPLETE', (output / 'completed.json').read_text(), flush=True)


if __name__ == '__main__':
    main()
