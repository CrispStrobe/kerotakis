"""Run every frozen record, preserving refusals and continuing to the next one.

Usage: run_all.py BINARY EXECUTION_BINDING NEW_OUTPUT_DIRECTORY
No process outcome is a scientific pass. No compilation takes place here.
"""
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import resource
import shutil
import subprocess
import sys
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def child_limits():
    resource.setrlimit(resource.RLIMIT_AS, (1024 ** 3, 1024 ** 3))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def main():
    binary, binding_path, output = map(Path, sys.argv[1:])
    binary = binary.resolve()
    root = Path(__file__).resolve().parent
    binding = json.loads(binding_path.read_text())
    assert sha(binary) == binding['binary_sha256']
    assert sha(Path(__file__)) == binding['runner_sha256']
    design = root / 'design'
    assert sha(design / 'freeze.json') == binding['design_freeze_sha256']
    freeze = json.loads((design / 'freeze.json').read_text())
    for name, digest in freeze['files'].items():
        assert sha(design / name) == digest, name
    manifest = json.loads((design / 'manifest.json').read_text())
    records = manifest['records']
    assert len(records) == 200 and len({r['id'] for r in records}) == 200
    output.mkdir()
    (output / 'binding.json').write_bytes(binding_path.read_bytes())
    results = []
    for record in sorted(records, key=lambda record: record['rank']):
        memory = dict(line.split(':', 1) for line in Path('/proc/meminfo').read_text().splitlines())
        gate = {'load1': os.getloadavg()[0],
                'available_memory_mib': int(memory['MemAvailable'].split()[0]) // 1024,
                'disk_free_mib': shutil.disk_usage(output).free // 1024 ** 2}
        if gate['load1'] > 3.5 or gate['available_memory_mib'] < 4224 or gate['disk_free_mib'] < 4224:
            (output / 'gate-stop.json').write_text(json.dumps(gate, indent=2) + '\n')
            raise RuntimeError('Resource gate closed; preserve partial results and do not repeat completed cases')
        script = design / record['script']
        assert sha(script) == freeze['files'][record['script']]
        assert sha(binary) == binding['binary_sha256']
        stem = f"{record['rank']:03d}-{record['id']}"
        started = time.monotonic()
        with (output / (stem + '.stdout')).open('wb') as stdout, (output / (stem + '.stderr')).open('wb') as stderr:
            try:
                child = subprocess.run([str(binary), 'run', str(script), '--json'],
                    stdout=stdout, stderr=stderr, timeout=30, preexec_fn=child_limits,
                    cwd=output, env={'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', 'KERO_LANG': 'en'})
                status, timed_out = child.returncode, False
            except subprocess.TimeoutExpired:
                status, timed_out = None, True
        parsed, invalid = [], 0
        for line in (output / (stem + '.stdout')).read_text(errors='replace').splitlines():
            try:
                row = json.loads(line)
                if not isinstance(row, dict):
                    raise ValueError('Observation is not an object')
                parsed.append(row)
            except (ValueError, json.JSONDecodeError):
                invalid += 1
        execution = 'timeout' if timed_out else 'nonzero_exit' if status != 0 else 'invalid_json' if invalid else 'completed'
        result = {'rank': record['rank'], 'id': record['id'], 'execution_status': execution,
            'exit_status': status, 'timed_out': timed_out, 'elapsed_seconds': time.monotonic() - started,
            'resources_before': gate, 'child_address_space_limit_mib': 1024,
            'script_sha256': sha(script), 'stdout_sha256': sha(output / (stem + '.stdout')),
            'stderr_sha256': sha(output / (stem + '.stderr')), 'parsed_observations': len(parsed),
            'invalid_json_lines': invalid, 'physical_verdict': 'not_assigned'}
        (output / (stem + '.result.json')).write_text(json.dumps(result, indent=2) + '\n')
        results.append(result)
        (output / 'executions.json').write_text(json.dumps(results, indent=2) + '\n')
        print(record['rank'], record['id'], execution, flush=True)
    summary = {'executed_count': len(results), 'execution_counts': dict(Counter(r['execution_status'] for r in results)),
               'physical_verdict': 'not_assigned', 'binary_sha256': sha(binary),
               'executions_sha256': sha(output / 'executions.json')}
    (output / 'completed.json').write_text(json.dumps(summary, indent=2) + '\n')


if __name__ == '__main__':
    main()
