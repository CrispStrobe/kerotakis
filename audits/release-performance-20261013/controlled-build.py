#!/usr/bin/env python3
"""Hosted-only, build-only evidence collector; no CLI experiments or timings."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time
import zipfile


SOURCES = {
    'baseline': ('2bf9c771975f24b9e5b933728ef440cbca9a8ec6',
                 '8613e46420a8f8d76f39732c5a50d3dcd8417811'),
    'candidate': ('d06ffe3a1740728656878c4aea0e9507ca83c808',
                  '25d1dd490acc080a59f2a02e5b9399a22bd0f4bc'),
}
LOCK_SHA = '5135e2cca3fa785a2d109409c0540eb1265cf7f6670637ab87e3d8af1b836a93'
ZIP_SHA = 'e44101a69e91b0a4d857467f842a84f2f8d6dcf00a6080b8a11a1c5a87924d10'
TARGET = 'x86_64-unknown-linux-gnu'


def require(value, message):
    if not value:
        raise ValueError(message)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1048576), b''):
            h.update(chunk)
    return h.hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def gate(evidence):
    memory = int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines()
                      if line.startswith('MemAvailable:'))) // 1024
    record = dict(time=time.time(), load1=os.getloadavg()[0], memory_mib=memory,
                  free_mib=shutil.disk_usage(evidence).free // 1048576)
    with (evidence / 'gates.jsonl').open('a') as stream:
        stream.write(json.dumps(record) + '\n')
    require(record['load1'] <= 3.5 and memory >= 3200 and record['free_mib'] >= 6144,
            'hosted build resource gate closed')


def source_guard(source, head, tree):
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=source).decode().strip()
    require(git('rev-parse', 'HEAD') == head and git('rev-parse', 'HEAD^{tree}') == tree,
            'source identity differs')
    require(not git('status', '--porcelain', '--untracked-files=no'), 'tracked source modified')
    require(not git('diff', '--ignore-submodules=none', 'HEAD'), 'native source modified')
    modules = git('submodule', 'status', '--recursive')
    # git output is stripped above, so inspect prefixes on the raw output instead.
    raw = subprocess.check_output(['git', 'submodule', 'status', '--recursive'], cwd=source).decode()
    require(all(line.startswith(' ') for line in raw.splitlines()), 'native module differs')
    return dict(head=head, tree=tree, recursive_submodules=modules,
                lock_sha256=digest(source / 'Cargo.lock'))


def main():
    require(__debug__ and os.environ.get('GITHUB_ACTIONS') == 'true', 'hosted runner required')
    require(os.environ.get('GITHUB_REPOSITORY') == 'CrispStrobe/kerotakis', 'repository differs')
    require(os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch', 'manual dispatch required')
    workspace = Path(os.environ['GITHUB_WORKSPACE'])
    evidence = workspace / 'paired-build-evidence'
    evidence.mkdir(exist_ok=False)
    try:
        audit = workspace / 'audit'
        audit_head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=audit).decode().strip()
        require(audit_head == os.environ['GITHUB_SHA'], 'audit checkout differs')
        require(not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'],
                                           cwd=audit).strip(), 'audit source modified')
        write(evidence / 'identity.json', dict(run=os.environ['GITHUB_RUN_ID'],
              attempt=os.environ['GITHUB_RUN_ATTEMPT'], audit_head=os.environ['GITHUB_SHA'],
              workflow_sha256=digest(audit / '.github/workflows/release-performance-paired-build.yml'),
              collector_sha256=digest(Path(__file__)), sources=SOURCES,
              candidate_execution_started=False, purpose='Controlled builds only; outputs unaccepted.'))
        gate(evidence)
        # Use the already-reviewed actual dependency lock for both clean builds.
        help_text = subprocess.check_output(['gh', 'api', '--help']).decode()
        args = ['gh', 'api']
        if '--allow-escape-sequences' in help_text:
            args.append('--allow-escape-sequences')
        args.append('repos/CrispStrobe/kerotakis/actions/artifacts/11777801862/zip')
        with (evidence / 'original-lock-artifact.zip').open('wb') as stream:
            subprocess.run(args, stdout=stream, check=True, timeout=180)
        require(digest(evidence / 'original-lock-artifact.zip') == ZIP_SHA, 'lock supplier ZIP differs')
        with zipfile.ZipFile(evidence / 'original-lock-artifact.zip') as archive:
            lock = archive.read('Cargo.lock')
        require(hashlib.sha256(lock).hexdigest() == LOCK_SHA, 'dependency lock differs')
        (evidence / 'Cargo.lock').write_bytes(lock)
        # Do not inherit unrecorded compiler overrides or credentials into builds.
        env = {key: os.environ[key] for key in
               ('PATH', 'HOME', 'USER', 'LOGNAME', 'SHELL', 'TMPDIR', 'TMP', 'TEMP',
                'CARGO_HOME', 'RUSTUP_HOME') if key in os.environ}
        env.update(CC='/usr/bin/gcc', CXX='/usr/bin/g++', RUSTFLAGS='',
                   CARGO_ENCODED_RUSTFLAGS='', CFLAGS='', CXXFLAGS='', LDFLAGS='',
                   CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER='/usr/bin/gcc',
                   CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2', CMAKE_BUILD_PARALLEL_LEVEL='2',
                   CARGO_TERM_COLOR='never')
        flag_keys = ['CC', 'CXX', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CFLAGS',
                     'CXXFLAGS', 'LDFLAGS', 'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER',
                     'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'CMAKE_BUILD_PARALLEL_LEVEL']
        def environment(source):
            tools = {}
            for tool in ('rustc', 'cargo', '/usr/bin/gcc', '/usr/bin/g++', 'cmake', 'make'):
                launcher = Path(shutil.which(tool)).resolve()
                actual = Path(subprocess.check_output(['rustup', 'which', tool], env=env,
                              cwd=source).decode().strip()) if tool in ('rustc', 'cargo') else launcher
                tools[tool] = dict(path=str(actual), launcher_path=str(launcher),
                    launcher_sha256=digest(launcher),
                    version=subprocess.check_output([tool, '--version'], env=env, cwd=source).decode(),
                    binary_sha256=digest(actual))
            return dict(tools=tools, explicit_flags={k: env[k] for k in flag_keys},
                        complete_build_environment=env,
                        target=TARGET,
                        rustc_verbose=subprocess.check_output(['rustc', '-vV'], env=env, cwd=source).decode(),
                        cargo_toml_sha256=digest(source / 'Cargo.toml'),
                        cargo_config_sha256=digest(source / '.cargo/config.toml'))
        receipts = {}
        for variant, (head, tree) in SOURCES.items():
            gate(evidence)
            source = workspace / variant
            (source / 'Cargo.lock').write_bytes(lock)
            folder = evidence / variant
            folder.mkdir()
            before = source_guard(source, head, tree)
            require(before['lock_sha256'] == LOCK_SHA, 'prepared lock differs')
            observed = environment(source)
            write(folder / 'before.json', dict(source=before, environment=observed))
            target_dir = workspace / ('controlled-target-' + variant)
            require(not target_dir.exists(), 'build target must be fresh')
            build_env = dict(env, CARGO_TARGET_DIR=str(target_dir))
            argv = ['cargo', 'build', '--locked', '--release', '--target', TARGET,
                    '-p', 'kerotakis-cli', '-vv']
            write(folder / 'command.json', dict(argv=argv, cwd=variant,
                  target_directory=target_dir.name, explicit_flags=observed['explicit_flags']))
            with (folder / 'build.log').open('wb') as stream:
                child = subprocess.Popen(argv, cwd=source, env=build_env, stdout=stream,
                                         stderr=subprocess.STDOUT, start_new_session=True)
                timed_out = False
                try:
                    code = child.wait(timeout=1500)
                except subprocess.TimeoutExpired:
                    timed_out = True
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    code = child.wait()
                write(folder / 'build-status.json', dict(exit_code=code, timed_out=timed_out,
                                                       process_group_cleanup_on_timeout=True))
            after = source_guard(source, head, tree)
            write(folder / 'after.json', dict(source=after, environment=environment(source)))
            require(before == after and observed == environment(source), 'source/toolchain changed')
            require(code == 0 and not timed_out, 'build failed or exceeded group timeout')
            shutil.copyfile(target_dir / TARGET / 'release/kero', folder / 'kero')
            (folder / 'kero').chmod(0o400)
            shutil.copyfile(source / 'Cargo.lock', folder / 'Cargo.lock')
            native_caches = {}
            for cache in target_dir.rglob('CMakeCache.txt'):
                raw = cache.read_bytes()
                (folder / ('native-cache-' + str(len(native_caches)) + '.txt')).write_bytes(raw)
                native_caches[str(cache.relative_to(target_dir))] = hashlib.sha256(raw).hexdigest()
            receipts[variant] = dict(source_before=before, source_after=after,
                  environment=observed, binary_sha256=digest(folder / 'kero'),
                  build_log_sha256=digest(folder / 'build.log'), native_cache_sha256=native_caches)
            write(folder / 'receipt.json', receipts[variant])
        require(receipts['baseline']['environment'] == receipts['candidate']['environment'],
                'observed build environments differ')
        write(evidence / 'build-pair.json', dict(builds_complete=True,
              observed_environments_equal=True, variants=receipts,
              candidate_executed=False, compiler_effective_invocations_review_pending=True,
              performance_accepted=False))
    except Exception as error:
        write(evidence / 'failure.json', dict(type=type(error).__name__, message=str(error),
              candidate_executed=False, builds_complete=False))
        raise


if __name__ == '__main__':
    main()
