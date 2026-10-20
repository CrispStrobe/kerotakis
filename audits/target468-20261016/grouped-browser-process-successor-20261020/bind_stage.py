"""Offline, once-only public evidence binding. Default performs NO action.

Future --execute creates/pushes exactly two public evidence files, then packs
private inputs for parent review. It never dispatches or accepts browser results.
"""
import argparse
import ast
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
from release_binding import SOURCE, TREE, RECEIPTS, checked_binding, digest, canonical, require
from pack_admission import reviewed_files

PINS = {'release_binding.py': 'ce1056bedda46b926d0edf86e5567028b1790ca796735f314e6edd557f405d25',
        'pack_admission.py': '15a80959f1a2216cf656016c6bc755c10a657c24384b8b63f050a193aefc39f7'}
WORKER = '7101e9c5024347eff903f0f5cc108d753058245f26dde7ac29bff548a9ff31c1'
PUBLIC = ('audits/live-rest-browser-20261018/live-client.mjs',
          'audits/live-rest-browser-20261018/release-binding.json')


def git(repo, *args):
    return subprocess.check_output(['git', '--no-replace-objects', *args], cwd=repo, timeout=120)


def regular(path):
    p = Path(path)
    require(p.is_file() and not p.is_symlink(), 'Missing or symlink input: ' + str(p))
    return p.read_bytes()


def resource_gate(path):
    memory = dict(line.split(':', 1) for line in Path('/proc/meminfo').read_text(encoding='utf-8').splitlines())
    require(int(memory['MemAvailable'].split()[0]) >= 3200 * 1024, 'Memory gate closed')
    require(os.getloadavg()[0] <= 3.5, 'Load gate closed')
    p = Path(path).resolve()
    while not p.exists():
        p = p.parent
    fs = os.statvfs(p)
    require(fs.f_bavail * fs.f_frsize >= 3 * 1024**3, 'Fast disk reserve gate closed')


def absent_branch(repo, branch):
    require(branch.startswith('audit/') and not branch.endswith('/') and '\n' not in branch, 'Use narrow audit/ branch')
    subprocess.run(['git', 'check-ref-format', 'refs/heads/' + branch], check=True, capture_output=True)
    local = subprocess.run(['git', 'show-ref', '--verify', '--quiet', 'refs/heads/' + branch], cwd=repo)
    require(local.returncode == 1, 'Local branch exists or ref lookup failed')
    remote = subprocess.run(['git', 'ls-remote', '--exit-code', '--heads', 'origin', 'refs/heads/' + branch], cwd=repo, capture_output=True, timeout=120)
    require(remote.returncode == 2 and not remote.stdout, 'Remote branch exists or lookup failed')


def sparse_create(repo, worktree, branch, source):
    require(not Path(worktree).exists(), 'Worktree already exists')
    git(repo, 'worktree', 'add', '--no-checkout', '-b', branch, str(worktree), source)
    git(worktree, 'sparse-checkout', 'init', '--no-cone')
    # --no-checkout leaves an empty index; seed it without materializing source.
    git(worktree, 'read-tree', 'HEAD')
    subprocess.run(['git', 'sparse-checkout', 'set', '--no-cone', '--stdin'], cwd=worktree,
                   input=('\n'.join('/' + p for p in PUBLIC) + '\n').encode(), check=True, capture_output=True)


def audit_commit(repo, source, expected):
    head = git(repo, 'rev-parse', 'HEAD').decode('ascii').strip()
    header = git(repo, 'cat-file', '-p', head).split(b'\n\n', 1)[0].splitlines()
    require([x.split()[1].decode('ascii') for x in header if x.startswith(b'parent ')] == [source], 'Wrong actual commit parent')
    changes = git(repo, 'diff', '--name-status', source, head).decode('utf-8').splitlines()
    require(set(changes) == {'A\t' + p for p in PUBLIC}, 'Commit must add only two public files')
    for path, raw in expected.items():
        require(git(repo, 'show', head + ':' + path) == raw, 'Committed public bytes differ')
    require(git(repo, 'status', '--porcelain') == b'', 'Dirty evidence worktree')
    return head


def check_bound_source(template_raw, bound_raw, config, config_sha):
    def normalized(raw):
        tree = ast.parse(raw)
        seen = set()
        actual_values = {}
        for node in tree.body:
            if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name) and node.targets[0].id in ('CONFIG', 'CONFIG_SHA256'):
                name = node.targets[0].id
                require(name not in seen, 'Duplicate worker assignment')
                seen.add(name)
                actual_values[name] = ast.literal_eval(node.value)
                node.value = ast.Constant(None)
        require(seen == {'CONFIG', 'CONFIG_SHA256'}, 'Worker assignment boundary changed')
        return ast.dump(tree, include_attributes=False), actual_values
    template_ast, template_values = normalized(template_raw)
    bound_ast, bound_values = normalized(bound_raw)
    require(template_values == {'CONFIG': None, 'CONFIG_SHA256': None}, 'Template is already bound')
    require(bound_values == {'CONFIG': config, 'CONFIG_SHA256': config_sha}, 'Bound assignments differ from actual admission')
    require(template_ast == bound_ast, 'Bound worker differs beyond configuration')


def validate_bound(template, bound, config, config_sha):
    template_raw, bound_raw = regular(template), regular(bound)
    check_bound_source(template_raw, bound_raw, config, config_sha)
    require(digest(template_raw) == WORKER, 'Only the reviewed worker may be imported')
    # Only the independently pinned unbound template is imported; never bound main.
    require(digest(regular(template)) == WORKER, 'Template changed before import')
    spec = importlib.util.spec_from_file_location('reviewed_grouped_template', template)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.validate_config(config, config_sha)


def preflight(args):
    for name, want in PINS.items():
        require(digest(regular(Path(__file__).with_name(name))) == want, 'Admission helper changed')
    require(digest(regular(args.worker)) == WORKER, 'Worker changed')
    review_raw = regular(args.review)
    require(digest(review_raw) == args.review_sha256, 'Parent review changed')
    review = json.loads(review_raw)
    closure_raw = regular(args.closure)
    require(digest(closure_raw) == args.closure_sha256, 'Static closure changed')
    closure = json.loads(closure_raw)
    require(review['inputs'] == closure['files'], 'Review does not exactly cover static closure')
    require(review['worker_template_sha256'] == WORKER and review['admission_programs_sha256'] == PINS, 'Source review code pins differ')
    require(review['template_sha256'] == args.template_sha256, 'Template not parent reviewed')
    receipts = {name: regular(args.production / name) for name in RECEIPTS}
    frozen = reviewed_files(review)
    require(len(frozen) <= 244, 'Too many static inputs for bounded worker')
    import base64
    sizes = [len(base64.b64decode(row['base64'], validate=True)) for row in frozen.values()]
    require(all(n <= 2 * 1024**2 for n in sizes) and sum(sizes) <= 8 * 1024**2, 'Static input byte budget exceeded')
    require(not {'forecasts/original-api-protocols.json', 'forecasts/parent-source-review-projection.json'} & set(frozen), 'Static closure collides with dynamic packer inputs')
    for row in closure['files'].values():
        require(digest(regular(row['path'])) == row['sha256'], 'Static input changed')
    template = regular(args.template)
    require(digest(template) == args.template_sha256, 'Template changed')
    # These actual receipts are deliberately read before creating any state.
    binding, prior = checked_binding(json.loads(template)['binding'], receipts, SOURCE, TREE, regular(args.original_protocols))
    require(git(args.repo, 'rev-parse', SOURCE + '^{tree}').decode('ascii').strip() == TREE, 'Integration tree mismatch')
    require(digest(git(args.repo, 'show', SOURCE + ':web/rest.html')) == prior['client_sha256'], 'Delivered REST source mismatch')
    for path in PUBLIC:
        exists = subprocess.run(['git', 'cat-file', '-e', SOURCE + ':' + path], cwd=args.repo, capture_output=True)
        require(exists.returncode != 0, 'Expected new public evidence path already exists')
    helper = regular(closure['files']['live-client.mjs']['path'])
    public = {PUBLIC[0]: helper, PUBLIC[1]: canonical(binding) + b'\n'}
    for raw in public.values():
        require(not any(s in raw for s in (b'/mnt/', b'/home/', b'PRIVATE KEY', b'KAGGLE_KEY')), 'Private content in public evidence')
    require(not args.out.exists() and not args.worktree.exists(), 'Output/worktree must be unused')
    absent_branch(args.repo, args.branch)
    resource_gate(args.worktree.parent)
    return public


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('repo', 'production', 'template', 'original-protocols', 'closure', 'worker', 'review', 'out', 'worktree'):
        p.add_argument('--' + name, type=Path, required=True)
    for name in ('template-sha256', 'closure-sha256', 'review-sha256', 'branch'):
        p.add_argument('--' + name, required=True)
    p.add_argument('--signing-key')
    p.add_argument('--execute', action='store_true')
    a = p.parse_args()
    if not a.execute:
        print('NOT EXECUTED: actual receipts and parent review required; no mutation performed')
        return
    require(a.out.is_absolute() and a.worktree.is_absolute() and a.repo.is_absolute(), 'Mutation paths must be absolute')
    public = preflight(a)
    a.out.mkdir(mode=0o700, exist_ok=False)
    def save(name, data):
        with (a.out / name).open('xb') as stream:
            stream.write(canonical(data) + b'\n')
    save('STARTED.json', {'source': SOURCE, 'tree': TREE, 'branch': a.branch, 'review_sha256': a.review_sha256, 'closure_sha256': a.closure_sha256, 'state': 'OFFLINE_BINDING_ONLY_NO_ISSUE'})
    try:
        sparse_create(a.repo, a.worktree, a.branch, SOURCE)
        for path, raw in public.items():
            target = a.worktree / path
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open('xb') as stream:
                stream.write(raw)
        git(a.worktree, 'add', '--', *PUBLIC)
        cmd = ['git']
        if a.signing_key:
            cmd += ['-c', 'gpg.format=ssh', '-c', 'user.signingkey=' + a.signing_key]
        cmd += ['commit', '-S', '-s', '-m', 'audit: bind reviewed PR850 release browser controls (never merge)']
        subprocess.run(cmd, cwd=a.worktree, check=True, capture_output=True, timeout=120)
        head = audit_commit(a.worktree, SOURCE, public)
        # Branch is local now: only the remote must still be absent before push.
        remote = subprocess.run(['git', 'ls-remote', '--exit-code', '--heads', 'origin', 'refs/heads/' + a.branch], cwd=a.repo, capture_output=True, timeout=120)
        require(remote.returncode == 2 and not remote.stdout, 'Remote ref no longer unused')
        save('PUSH-ATTEMPT.json', {'head': head, 'branch': a.branch, 'automatic_retry': False})
        git(a.worktree, 'push', 'origin', 'HEAD:refs/heads/' + a.branch)
        command = [sys.executable, str(Path(__file__).with_name('pack_admission.py')),
                   '--production', str(a.production), '--template', str(a.template), '--template-sha256', a.template_sha256,
                   '--original-protocols', str(a.original_protocols), '--review', str(a.review), '--review-sha256', a.review_sha256,
                   '--harness-repo', str(a.worktree), '--worker-template', str(a.worker), '--out', str(a.out / 'bound')]
        packed = subprocess.run(command, capture_output=True, timeout=180)
        for name, raw in [('PACKER.stdout', packed.stdout), ('PACKER.stderr', packed.stderr)]:
            with (a.out / name).open('xb') as stream:
                stream.write(raw)
        packed.check_returncode()
        config_raw = regular(a.out / 'bound/CONFIG.json')
        config = json.loads(config_raw)
        require(config_raw == canonical(config) + b'\n', 'Admission is not canonical immutable bytes')
        validate_bound(a.worker, a.out / 'bound/grouped_browser.py', config, digest(canonical(config)))
        save('BOUND-NOT-ISSUED.json', {'state': 'BOUND_READY_FOR_PARENT_REVIEW_NOT_ISSUED', 'runtime_source': SOURCE, 'runtime_tree': TREE, 'harness_head': head,
                                     'config_sha256': digest(canonical(config)), 'worker_sha256': digest(regular(a.out / 'bound/grouped_browser.py')),
                                     'provider_calls': 0, 'browser_acceptance_claim': False})
    except BaseException as exc:
        for name, raw in [('FIRST-FAILURE.stdout', getattr(exc, 'stdout', None)), ('FIRST-FAILURE.stderr', getattr(exc, 'stderr', None))]:
            if raw is not None:
                with (a.out / name).open('xb') as stream:
                    stream.write(raw if isinstance(raw, bytes) else raw.encode('utf-8'))
        save('STOPPED.json', {'type': type(exc).__name__, 'error': str(exc), 'automatic_retry': False, 'partial_state_preserved': True})
        raise


if __name__ == '__main__':
    main()
