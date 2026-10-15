"""Run the eight frozen identity controls once using a separately bound binary.

Usage: python3 run.py BINARY BINDING OUTPUT_DIRECTORY
The output directory must not exist. No scientific verdict is assigned here.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resources(output):
    memory = dict(line.split(':', 1) for line in Path('/proc/meminfo').read_text().splitlines())
    return {'load1': os.getloadavg()[0],
            'available_memory_mib': int(memory['MemAvailable'].split()[0]) // 1024,
            'free_disk_mib': shutil.disk_usage(output).free // (1024 * 1024)}


def main():
    binary, binding_path, output = map(Path, sys.argv[1:])
    binary = binary.resolve()
    design = Path(__file__).resolve().parent / 'design'
    binding = json.loads(binding_path.read_text())
    assert sha(binary) == binding['binary_sha256']
    assert sha(Path(__file__)) == binding['runner_sha256']
    frozen = json.loads((design / 'freeze.json').read_text())
    for name, digest in frozen['files'].items():
        assert sha(design / name) == digest, name
    output.mkdir()  # Refuse to overwrite an earlier execution, even a partial one.
    (output / 'binding.json').write_bytes(binding_path.read_bytes())
    results = []
    for script in sorted((design / 'scripts').glob('*.lab')):
        gate = resources(output)
        if gate['load1'] > 3.5 or gate['available_memory_mib'] < 4224 or gate['free_disk_mib'] < 4224:
            (output / 'gate-stop.json').write_text(json.dumps(gate, indent=2) + '\n')
            raise RuntimeError('Resource gate closed; preserve completed cases, do not rerun them')
        assert sha(binary) == binding['binary_sha256']
        assert sha(script) == frozen['files']['scripts/' + script.name]
        start = time.monotonic()
        with (output / (script.stem + '.stdout')).open('wb') as stdout, (output / (script.stem + '.stderr')).open('wb') as stderr:
            try:
                result = subprocess.run([str(binary), 'run', str(script), '--json'],
                                        stdout=stdout, stderr=stderr, timeout=60,
                                        env={'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'},
                                        cwd=output)
                status, timed_out = result.returncode, False
            except subprocess.TimeoutExpired:
                status, timed_out = None, True
        results.append({'script': script.name, 'script_sha256': sha(script),
                        'exit_status': status, 'timed_out': timed_out,
                        'elapsed_seconds': time.monotonic() - start, 'resources_before': gate,
                        'stdout_sha256': sha(output / (script.stem + '.stdout')),
                        'stderr_sha256': sha(output / (script.stem + '.stderr'))})
        (output / 'executions.json').write_text(json.dumps(results, indent=2) + '\n')
        print(script.name, status, 'timeout' if timed_out else 'recorded', flush=True)
        if status != 0 or timed_out:
            raise RuntimeError('First process failure preserved; inspect before further execution')


if __name__ == '__main__':
    main()
