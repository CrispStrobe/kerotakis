"""Prepare immutable grouped-browser inputs from reviewed code and actual receipts.

No provider calls, requests, browser/engine execution, Git mutation or dispatch.
Missing new-release acceptance is an error before an output directory is created.
"""
import argparse
import base64
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
from release_binding import (SOURCE, TREE, CANDIDATE, RECEIPTS,
                             checked_binding, canonical, digest, require)

MODULES = {
    'live-client.mjs': '1f0ddea0b8a9e16d78cb15cfaaa319abbe2e6cd817999d7075c4e9c235a903a0',
    'targeted.mjs': 'f17c5c7fe143d9930efeeafa8cf9c14c1a873ba10ea4927ca5fc30fa91cba4f9',
    'locale-controls.mjs': '7290327a0d59d981bcc8bc0e35b5d0941c3559d14067ecdf488ae4f24c0f8819',
    'run-locale18.mjs': '8456b089e8389c2995fd1c58bcd316c64fffcfcad4db002927f336fa58a8bd21',
    'heat-controls.mjs': 'cf19825bd6a273115eb686feab0b3242dfebb5533e571fed4765b8f49c371d6f',
    'run-heat.mjs': '316980a04ba85c94a54665f1ad5418a310f8485892afe97b85c9ba97daa6d592',
}
GROUPS = [('browser37', 'live-client.mjs', 37, 240),
          ('targeted7', 'targeted.mjs', 7, 240),
          ('locale18', 'run-locale18.mjs', 18, 300),
          ('heat2', 'run-heat.mjs', 2, 300)]


def encoded(raw):
    return {'base64': base64.b64encode(raw).decode('ascii'), 'sha256': digest(raw)}


def safe_relative(name):
    require(isinstance(name, str) and name and not PurePosixPath(name).is_absolute()
            and all(x not in ('', '.', '..') for x in name.split('/'))
            and '\\' not in name and '\x00' not in name, 'Unsafe input alias')
    return name


def reviewed_files(review):
    """Only explicitly reviewed aliases are admitted. No recursive host uploads."""
    require(review['execution_admission_order_approved'] is True and
            review['original_modules_byte_identical'] is True and
            review['all64_final_acceptance_mandatory'] is True,
            'Exact source/reader review remains incomplete')
    files = {}
    for name, row in review['inputs'].items():
        safe_relative(name)
        require(name in MODULES or name.startswith(('saved-readers/', 'forecasts/')),
                'Unreviewed upload class')
        path = Path(row['path'])
        require(path.is_absolute() and path.is_file() and not path.is_symlink(),
                'Reviewed input must be an actual regular file')
        raw = path.read_bytes()
        require(digest(raw) == row['sha256'], 'Reviewed source bytes changed: ' + name)
        files[name] = encoded(raw)
    require(set(MODULES) <= set(files), 'Missing unchanged original browser module')
    for name, want in MODULES.items():
        require(files[name]['sha256'] == want, 'Original assertion module changed: ' + name)
    require(any(n.startswith('saved-readers/') for n in files) and
            any(n.startswith('forecasts/') for n in files),
            'Saved-reader and forecast closure missing')
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--production', type=Path, required=True)
    parser.add_argument('--template', type=Path, required=True)
    parser.add_argument('--template-sha256', required=True)
    parser.add_argument('--original-protocols', type=Path, required=True)
    parser.add_argument('--review', type=Path, required=True)
    parser.add_argument('--review-sha256', required=True)
    parser.add_argument('--harness-repo', type=Path, required=True)
    parser.add_argument('--worker-template', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    review_raw = args.review.read_bytes()
    require(digest(review_raw) == args.review_sha256, 'Parent source review changed')
    review = json.loads(review_raw)
    inputs = reviewed_files(review)
    worker = args.worker_template.read_bytes()
    require(digest(worker) == review['worker_template_sha256'], 'Unreviewed worker template')
    for name in ('release_binding.py', 'pack_admission.py'):
        require(digest(Path(__file__).with_name(name).read_bytes()) ==
                review['admission_programs_sha256'][name], 'Unreviewed admission helper')
    template_raw = args.template.read_bytes()
    require(digest(template_raw) == args.template_sha256, 'Binding template changed')
    raw = {n: (args.production / n).read_bytes() for n in RECEIPTS}
    original = args.original_protocols.read_bytes()
    binding, prior = checked_binding(json.loads(template_raw)['binding'], raw, SOURCE, TREE, original)

    def git(*argv):
        return subprocess.check_output(['git', '--no-replace-objects', *argv],
                                       cwd=args.harness_repo, timeout=60)
    require(git('status', '--porcelain') == b'', 'Harness worktree is dirty')
    head = git('rev-parse', 'HEAD').decode('ascii').strip()
    tree = git('rev-parse', 'HEAD^{tree}').decode('ascii').strip()
    # Inspect the actual commit object; shallow graph traversal hides parents.
    header = git('cat-file', '-p', head).split(b'\n\n', 1)[0].splitlines()
    parents = [s.split()[1].decode('ascii') for s in header if s.startswith(b'parent ')]
    require(parents == [SOURCE], 'Evidence commit must have the exact runtime parent')
    require(git('rev-parse', SOURCE + '^{tree}').decode().strip() == TREE,
            'Actual integration tree differs')
    changed = set(git('diff', '--name-only', SOURCE, head).decode('utf-8').splitlines())
    require(changed == {'audits/live-rest-browser-20261018/live-client.mjs',
                        'audits/live-rest-browser-20261018/release-binding.json'},
            'Evidence commit changed unrelated source')
    require(digest(git('show', head + ':web/rest.html')) == prior['client_sha256'],
            'Harness REST source differs from actual delivered bytes')
    bind_raw = git('show', head + ':audits/live-rest-browser-20261018/release-binding.json')
    helper_raw = git('show', head + ':audits/live-rest-browser-20261018/live-client.mjs')
    require(json.loads(bind_raw) == binding and digest(helper_raw) == MODULES['live-client.mjs'],
            'Harness modules or actual release binding differ')
    inputs['release-binding.json'] = encoded(bind_raw)
    inputs['heat-release-binding.json'] = encoded(canonical(dict(binding, reviewed_candidate_source=CANDIDATE)) + b'\n')
    for name, data in raw.items():
        inputs['prior-acceptance/' + name] = encoded(data)
    inputs['forecasts/original-api-protocols.json'] = encoded(original)
    # Host filenames stay in the private parent review. Archive the explicit
    # alias/hash projection and bind it to the full review digest before issue.
    inputs['forecasts/parent-source-review-projection.json'] = encoded(canonical({
        'qualification': 'Preissue alias/hash projection; private host paths omitted',
        'private_parent_source_review_sha256': args.review_sha256,
        'input_aliases_sha256': {n: row['sha256'] for n, row in review['inputs'].items()},
        'worker_template_sha256': review['worker_template_sha256'],
        'admission_programs_sha256': review['admission_programs_sha256'],
        'execution_admission_order_approved': True,
        'original_modules_byte_identical': True,
        'all64_final_acceptance_mandatory': True}) + b'\n')
    envelope = dict(runtime_source=SOURCE,
                    engine=json.loads(raw['API-ACCEPTED.json'])['engine'],
                    harness_head=head, harness_tree=tree,
                    never_merge_validation_ref=True,
                    never_merge_evidence_branch=True,
                    new_owner_keys_in_browser_or_kaggle=False,
                    harness_script_sha256=MODULES['live-client.mjs'],
                    binding_sha256=digest(bind_raw), binding=binding)
    config = dict(reviewed_candidate_source=CANDIDATE, runtime_source=SOURCE,
                  runtime_tree=TREE, harness_head=head, harness_tree=tree,
                  binding_sha256=digest(bind_raw),
                  heat_binding_sha256=inputs['heat-release-binding.json']['sha256'],
                  inputs=inputs, groups=[dict(zip(('id', 'entry', 'expected_checks', 'timeout_seconds'), g)) for g in GROUPS],
                  prior_release=prior, binding=envelope)
    config_sha = digest(canonical(config))
    text = worker.decode('utf-8')
    require(text.count('\nCONFIG=None\n') == text.count('\nCONFIG_SHA256=None\n') == 1,
            'Reviewed worker placeholders changed')
    bound = text.replace('\nCONFIG=None\n', '\nCONFIG=' + repr(config) + '\n').replace(
        '\nCONFIG_SHA256=None\n', '\nCONFIG_SHA256=' + repr(config_sha) + '\n').encode('utf-8')
    compile(bound, 'bound-grouped-browser.py', 'exec')  # Syntax only; no import or execution.
    args.out.mkdir(mode=0o700, exist_ok=False)
    for name, content in [('CONFIG.json', canonical(config) + b'\n'),
                          ('grouped_browser.py', bound),
                          ('BOUND-NOT-ISSUED.json', canonical({
                              'state': 'BOUND_ONLY_PARENT_ISSUE_REVIEW_STILL_REQUIRED',
                              'runtime_source': SOURCE, 'runtime_tree': TREE,
                              'worker_sha256': digest(bound), 'config_sha256': config_sha,
                              'worker_template_sha256': digest(worker),
                              'parent_source_review_sha256': args.review_sha256,
                              'provider_calls': 0, 'browser_acceptance_claim': False}) + b'\n')]:
        with (args.out / name).open('xb') as stream:
            stream.write(content)


if __name__ == '__main__':
    main()
