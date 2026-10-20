"""Run frozen saved readers twice and authenticate actual invocation provenance.

No browser, simulator, deployment or provider calls. Real collected output and
the actual external source-bound issue are mandatory, before any output exists.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
from saved_envelope import authenticate, digest, load

SOURCE = '33e043ef4a4a4225e5ea8d49ba93e8df99716c5f'
PINS = {
    'read_group.py': '327ee4aba48ab59806a971489096365ff9c6e5841a1ecd12015a78289b40a16e',
    'saved_envelope.py': 'e3228a3b68ba1d3745f645d036eb2631921961894acd680a6f588a58f2459c2f',
    'group37_predicates.py': '2f7d23773e98068068a5d730983821a65095c90a62d6b472e2ef55e98105d944',
    'group7_predicates.py': '81206e7ac27de50244effe2912469a345c1fba5bb50b1f3d0a2d52e6769d1216',
    'AST-EQUALITY-PROOF.json': '71d02d65417b60737a5a6af71112e7b30c260c93811eac3488d78dfccc53c596',
    'final_common.py': '700fec14967e1519a0c004558cd2cd8cf895cfb67d0a46649a0343432b3fe868',
    'build_locale_context.py': '18913cef5f83475b13efc0e7df3eb2ae9058d45529b7494ea738f062930e2813',
    'review_heat_group.py': 'b17fd2a6fc05df1d83fc9f3b8b5f03d33b54adf9042ddeacdcccd825a442574b',
    'aggregate_saved.py': '6ebc3427d6979fe99663c2204c8413033fb4cb9f92e8d1acb73a0fc0f39cffe8',
}
ORIGINAL = {
    'locale18': ('review-locale18-848.py', 'locale18_saved_predicates.py',
                 '84fe50ca53fb67a27bdda1d40869df70a56948b31c01daa1ad79b8c1597cec6a',
                 '4b66938c88db0da8863d44a10a7e4a34e72aa578891133e72ac1818d7dd4e063'),
    'heat2': ('review_saved.py', 'heat_saved_predicates.py',
              '817d9fdf9106d5581919a5b7696658074033e6737154c1336b5cbc0f3d516252',
              '705915e2fdb218c754c69c7408d135f1432bd521343336383f9aeadee0ae9264'),
}


def require(value, message):
    if not value:
        raise ValueError(message)


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def save(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')


def check_original_invocation(directory, group):
    reader, predicates, reader_sha, predicates_sha = ORIGINAL[group]
    meta = load(directory / 'ORIGINAL-READER-EXECUTION.json')
    require(meta['exit_status'] == 0 and meta['reader_sha256'] == reader_sha and
            meta['predicates_sha256'] == predicates_sha and
            meta['browser_engine_provider_calls'] == 0,
            'Original saved-reader invocation did not succeed')
    require(digest(directory / reader) == reader_sha and
            digest(directory / predicates) == predicates_sha and
            digest(directory / 'original-reader.stdout') == meta['stdout_sha256'] and
            digest(directory / 'original-reader.stderr') == meta['stderr_sha256'],
            'Original saved-reader source/stream bytes changed')
    return meta


def stop_owned_group(process):
    # The leader can exit between TimeoutExpired and cleanup; it never gates
    # termination of the group that may still contain a nested reader.
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait(timeout=10)


def run_saved(root, label, argv, cwd, env, closure, timeout=300):
    """Actual owned process and streams, no shell, no fabricated invocation."""
    closure()
    start = utc()
    tick = time.monotonic()
    process = None
    error = None
    status = None
    try:
        with (root / (label + '.stdout')).open('xb') as out, \
                (root / (label + '.stderr')).open('xb') as err:
            process = subprocess.Popen(argv, cwd=cwd, env=env, stdout=out,
                                       stderr=err, start_new_session=True)
            try:
                status = process.wait(timeout=timeout)
            except BaseException:
                stop_owned_group(process)
                status = process.returncode
                raise
    except BaseException as exc:
        error = type(exc).__name__
        raise
    finally:
        save(root / (label + '.execution.json'), {
            'started_utc': start, 'finished_utc': utc(), 'argv': argv,
            'exit_status': status, 'error_type': error,
            'elapsed_seconds': time.monotonic() - tick,
            'stdout_sha256': digest(root / (label + '.stdout')) if (root / (label + '.stdout')).is_file() else None,
            'stderr_sha256': digest(root / (label + '.stderr')) if (root / (label + '.stderr')).is_file() else None,
            'pid': process.pid if process else None, 'fresh_owned_process_group': process is not None,
            'saved_only_no_product_calls': True})
    closure()
    require(status == 0, 'Saved reader failed: ' + label)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('evidence', 'design', 'worker', 'issue', 'api', 'verified',
                 'native-reference', 'collected', 'out-dir'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--collected-sha256', required=True)
    args = parser.parse_args()
    require(not args.out_dir.exists() and not args.out_dir.is_symlink(), 'Never repeat or overwrite a review stage')
    memory = int(next(s.split()[1] for s in Path('/proc/meminfo').read_text().splitlines()
                      if s.startswith('MemAvailable:'))) // 1024
    require(memory >= 3200 and os.getloadavg()[0] <= 3.5, 'Source review resource gate closed')
    require(digest(args.collected) == args.collected_sha256 and
            not args.collected.resolve().is_relative_to(args.evidence.resolve()),
            'Collected receipt must be an externally pinned host record')
    collected = load(args.collected)
    require(collected['version'] == 1 and collected['runtime_source'] == SOURCE and
            collected['serial_ext4_before_storage_verified'] is True and
            collected['parent_acceptance_assigned'] is False and
            collected['all_frozen_inputs_materialized'] is True,
            'Complete actual source-bound collection is required')
    envs = {group: authenticate(
        evidence=args.evidence, design_path=args.design, worker_path=args.worker,
        issue_path=args.issue, api_path=args.api, verified_path=args.verified,
        native_reference_path=args.native_reference, group=group)
        for group in ('browser37', 'targeted7', 'locale18', 'heat2')}
    base = envs['browser37']
    require(base['S'] == SOURCE and collected['worker_sha256'] == base['worker_sha256'] and
            collected['config_sha256'] == base['config_canonical_sha256'] and
            collected['issue_sha256'] == base['external_issue_sha256'] and
            collected['producer_manifest_sha256'] == base['top_manifest_sha256'],
            'Collection/raw/source identity mismatch')
    self_sha = digest(Path(__file__))
    paths_by_hash = {}
    for path in base['input_paths']:
        paths_by_hash.setdefault(digest(path), path)
    require(self_sha in paths_by_hash and digest(Path(__file__).with_name('saved_envelope.py')) ==
            PINS['saved_envelope.py'], 'Runner and envelope must be frozen before issue')
    source_bytes = {name: paths_by_hash[want].read_bytes() for name, want in PINS.items()}
    args.out_dir.mkdir(mode=0o700)
    save(args.out_dir / 'STARTED.json', {
        'utc': utc(), 'source': SOURCE, 'runner_sha256': self_sha,
        'collected_receipt_sha256': args.collected_sha256,
        'execution_design_sha256': base['execution_design_sha256'],
        'top_manifest_sha256': base['top_manifest_sha256'],
        'actual_saved_commands_pending': True, 'product_calls': 0})
    try:
        code = args.out_dir / 'code'
        code.mkdir(mode=0o700)
        for name, raw in source_bytes.items():
            with (code / name).open('xb') as stream:
                stream.write(raw)
        def closure():
            require(digest(Path(__file__)) == self_sha, 'Runner bytes changed')
            for name, want in PINS.items():
                require(digest(code / name) == want, 'Saved reader source changed: ' + name)
            require(digest(base['E'] / 'SHA256.json') == base['top_manifest_sha256'],
                    'Immutable producer manifest changed')
        child_env = dict(os.environ)
        child_env.pop('PYTHONPATH', None)
        child_env.pop('PYTHONHOME', None)
        child_env['PYTHONDONTWRITEBYTECODE'] = '1'
        child_env['PYTHONNOUSERSITE'] = '1'
        common = []
        for name in ('evidence', 'design', 'worker', 'issue', 'api', 'verified', 'native_reference'):
            common += ['--' + name.replace('_', '-'), str(getattr(args, name).resolve())]
        first, second = args.out_dir / 'first-review', args.out_dir / 'second-review'
        for label, directory in [('first', first), ('second', second)]:
            directory.mkdir(mode=0o700)
            p37 = directory / 'BROWSER37-ACCEPTED.json'
            p44 = directory / 'PRIOR44-ACCEPTED.json'
            run_saved(args.out_dir, label + '-37', [sys.executable, str((code / 'read_group.py').resolve()),
                '--group', 'browser37', *common, '--out', str(p37.resolve())], code, child_env, closure)
            run_saved(args.out_dir, label + '-7', [sys.executable, str((code / 'read_group.py').resolve()),
                '--group', 'targeted7', *common, '--prior37', str(p37.resolve()), '--out', str(p44.resolve())],
                code, child_env, closure)
            parents = ['--prior37', str(p37.resolve()), '--prior37-sha256', digest(p37),
                       '--prior44', str(p44.resolve()), '--prior44-sha256', digest(p44)]
            locale_dir = directory / 'context18'
            run_saved(args.out_dir, label + '-18', [sys.executable, str((code / 'build_locale_context.py').resolve()),
                *common, *parents, '--out-dir', str(locale_dir.resolve()), '--run-reader'], code, child_env, closure)
            check_original_invocation(locale_dir, 'locale18')
            prior62 = locale_dir / 'original-locale18-result.json'
            heat_dir = directory / 'heat-review'
            run_saved(args.out_dir, label + '-heat2', [sys.executable, str((code / 'review_heat_group.py').resolve()),
                *common, *parents, '--prior62', str(prior62.resolve()), '--prior62-sha256', digest(prior62),
                '--out-dir', str(heat_dir.resolve()), '--run-reader'], code, child_env, closure)
            check_original_invocation(heat_dir, 'heat2')
        agreement = args.out_dir / 'all64-agreement.json'
        run_saved(args.out_dir, 'aggregate', [sys.executable, str((code / 'aggregate_saved.py').resolve()),
            *common, '--first-review', str(first.resolve()), '--second-review', str(second.resolve()),
            '--out', str(agreement.resolve())], code, child_env, closure)
        result = load(agreement)
        require(result['all64_raw_saved_reviews_agree'] is True and
                result['source'] == SOURCE and result['retained_browser_controls'] == 62 and
                result['supplemental_heat_controls'] == 2 and
                result['top_manifest_sha256'] == base['top_manifest_sha256'] and
                result['worker_sha256'] == base['worker_sha256'], 'Actual saved reviews do not agree')
        # Final raw authentication after every subprocess, without a product call.
        for group in envs:
            authenticate(evidence=args.evidence, design_path=args.design, worker_path=args.worker,
                         issue_path=args.issue, api_path=args.api, verified_path=args.verified,
                         native_reference_path=args.native_reference, group=group)
        records = {p.name: digest(p) for p in args.out_dir.glob('*.execution.json')}
        require(len(records) == 9, 'All eight saved group invocations and aggregator are mandatory')
        save(args.out_dir / 'ALL64-SAVED-INVOCATIONS-VERIFIED.json', {
            'utc': utc(), 'source': SOURCE, 'engine': base['binding']['engine'],
            'runner_sha256': self_sha, 'collected_receipt_sha256': args.collected_sha256,
            'worker_sha256': base['worker_sha256'], 'external_issue_sha256': base['external_issue_sha256'],
            'execution_design_sha256': base['execution_design_sha256'],
            'config_canonical_sha256': base['config_canonical_sha256'],
            'top_manifest_sha256': base['top_manifest_sha256'],
            'actual_invocations_sha256': records, 'agreement_sha256': digest(agreement),
            'retained_browser_controls': 62, 'supplemental_heat_controls': 2,
            'saved_predicates_and_actual_invocations_verified': True,
            'new_browser_engine_API_provider_calls': 0, 'scientific_selections': 0,
            'empirical_calibration_claim': False})
    except BaseException as error:
        save(args.out_dir / 'STOPPED.json', {'utc': utc(), 'error_type': type(error).__name__,
             'error': str(error), 'all_first_outputs_preserved': True,
             'no_automatic_retry_or_product_reexecution': True})
        raise


if __name__ == '__main__':
    main()
