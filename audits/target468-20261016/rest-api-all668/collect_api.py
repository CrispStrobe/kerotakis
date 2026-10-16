#!/usr/bin/env python3
"""Authenticate all668 issued API outcomes and compare saved native JSON; no execution."""
import argparse
from collections import Counter
import datetime
import hashlib
import json
from pathlib import Path
import statistics
import zipfile


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(path):
    return json.loads(path.read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--first', type=Path, required=True)
    parser.add_argument('--remaining', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    assert not args.out.exists()
    root = Path(__file__).resolve().parents[3]
    design_path = Path(__file__).with_name('design.json')
    design = load(design_path)
    native_path = root / 'audits/target468-20261016/common-engine-all668.json'
    native = {c['id']: c for c in load(native_path)['cases']}
    assert load(args.first / 'stopped.json')['http_attempts'] == 227
    completion = load(args.remaining / 'completion.json')
    assert completion['records'] == 441
    assert load(args.first / 'binding.json')['design_sha256'] == sha(design_path.read_bytes())
    recovery = Path(__file__).with_name('remaining441-design.json')
    assert load(args.remaining / 'binding.json')['design_sha256'] == sha(recovery.read_bytes())
    health = load(args.remaining / 'health-after.json')
    assert health['configured'] and health['revision'] == design['api_revision'] and health['engine'] == design['engine']
    archives = {
        38909751180: root / 'audits/target468-20261016/common-engine-completion/results/raw-artifact.zip',
        38871207777: root / 'audits/new-experiments-201-20261016/native-screen-results/raw-artifact.zip',
        38909739140: root / 'audits/target468-20261016/curriculum200-common-engine/results/raw-artifact.zip',
    }
    zips = {run: zipfile.ZipFile(path) for run, path in archives.items()}
    rows = []
    latencies = []
    for c in design['cases']:
        key = c['id'].replace(':', '_')
        matches = [p / key for p in (args.first, args.remaining) if (p / key).is_dir()]
        assert len(matches) == 1
        directory = matches[0]
        issued = sorted(directory.glob('issued-*.json'))
        assert len(issued) == 1, 'Unexpected retry; review separately rather than silently aggregate'
        body = (directory / 'request.json').read_bytes()
        assert load(issued[0])['request_sha256'] == sha(body)
        assert json.loads(body) == {'script': c['script']}
        assert sha(c['script'].encode()) == c['script_sha256'] == native[c['id']]['script_sha256']
        raw = (directory / 'response-1.body').read_bytes()
        http = load(directory / 'http-1.json')
        assert sha(raw) == http['response_sha256']
        latency = http['elapsed_seconds']
        latencies.append(latency)
        row = {'id': c['id'], 'script_sha256': c['script_sha256'],
               'request_sha256': sha(body), 'response_sha256': sha(raw),
               'http_status': http['http_status'], 'elapsed_seconds': latency,
               'native_exit_status': native[c['id']]['exit_code'], 'attempts': 1,
               'physical_verdict': 'not_assigned'}
        output = json.loads(raw)
        if http['http_status'] == 200:
            assert output['engine'] == design['engine'] and output['script_sha256'] == c['script_sha256']
            assert output['physical_verdict'] == 'not_assigned'
            n = native[c['id']]
            zipped = zips[n['run_id']]
            prefix = key + '/'
            stdout = zipped.read(prefix + 'stdout.jsonl')
            assert sha(stdout) == n['stdout_sha256']
            assert sha(zipped.read(prefix + 'experiment.lab')) == c['script_sha256']
            observations = [json.loads(line) for line in stdout.splitlines() if line.strip()]
            row.update(execution_status=output['execution_status'], exit_status=output['exit_status'],
                observations=len(output['observations']),
                native_exit_matches=output['exit_status'] == n['exit_code'],
                native_json_observations_equal=output['observations'] == observations,
                native_observations=len(observations), remote_engine_script_attestation=True)
        else:
            assert http['http_status'] in (502, 503, 504)
            row.update(execution_status='api_infrastructure_failure', remote_engine_script_attestation=False,
                       error=output, native_json_observations_equal=None, native_exit_matches=None)
        rows.append(row)
    assert len(rows) == len({r['id'] for r in rows}) == 668
    output = {'schema': 'kerotakis.api668-authenticated-results.v1',
        'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'design_sha256': sha(design_path.read_bytes()), 'native_baseline_sha256': sha(native_path.read_bytes()),
        'api_revision': design['api_revision'], 'engine': design['engine'], 'records': len(rows),
        'http_attempts': sum(r['attempts'] for r in rows), 'script_adaptations': 0, 'ambiguous_retries': 0,
        'execution_counts': dict(Counter(r['execution_status'] for r in rows)),
        'native_exit_matches': sum(r.get('native_exit_matches') is True for r in rows),
        'native_json_observations_equal': sum(r.get('native_json_observations_equal') is True for r in rows),
        'json_differences': [r['id'] for r in rows if r.get('native_json_observations_equal') is False],
        'elapsed_request_seconds': {'median': statistics.median(latencies), 'mean': statistics.mean(latencies),
            'sum': sum(latencies), 'maximum': max(latencies)},
        'qualification': 'HTTP success and exact saved CLI agreement are execution checks, not scientific passes. Four201 protocol corrections retain their separate original associations. Infrastructure errors supply no per-job engine/script attestation.',
        'physical_verdict': 'not_assigned', 'cases': rows}
    args.out.write_text(json.dumps(output, indent=2, allow_nan=False) + '\n')
    print(json.dumps({k: v for k, v in output.items() if k != 'cases'}, indent=2))


if __name__ == '__main__':
    main()
