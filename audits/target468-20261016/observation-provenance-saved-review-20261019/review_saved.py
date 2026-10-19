"""Review the saved partial run; never execute an engine or assign full acceptance."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib

parser = argparse.ArgumentParser()
parser.add_argument('--binary-archive', type=Path)
parser.add_argument('--out', type=Path, required=True)
args = parser.parse_args()
base = Path(__file__).resolve().parent.parent
proof = base / 'observation-provenance-first-outcome-20261019'
root = base.parents[1]
sha = lambda raw: hashlib.sha256(raw).hexdigest()
for name, digest in json.loads((proof / 'evidence-sha256.json').read_text()).items():
    assert sha((proof / name).read_bytes()) == digest, name
source = 'fa99b687b7ed14cdc40b22a6e7105095a11bdafd'
baseline = 'ab10933c4a20e03cd156ccc711f35d6e9667ca66'
def git(ref, path):
    return subprocess.check_output(['git', 'show', ref + ':' + path], cwd=root)
module = lambda ref: git(ref, 'crates/kerotakis-cli/src/coverage.rs').split(b'#[cfg(test)]\nmod tests {', 1)[1]
assert module(source) == module(baseline)
failed = ['unsupported_dissolution_and_visual_inspection_are_not_computed',
          'unsupported_digestion_and_visual_inspection_are_not_computed',
          'unrelated_vessel_bookkeeping_cannot_upgrade_an_unsupported_observation',
          'a_model_limit_and_handling_without_observation_are_not_a_computation']
positive = 'independent_positive_kinetic_results_keep_their_declared_computed_grade'
for stem, code in [('observation-baseline-test', 101), ('additional-gate-0', 0)]:
    record = json.loads((proof / (stem + '.json')).read_text())
    assert record['exit_status'] == code
    assert record['argv'] == ['cargo', 'test', '-p', 'kerotakis-cli', '--bin', 'kero', 'coverage::tests::', '--', '--nocapture']
    for stream in ['stdout', 'stderr']:
        assert sha((proof / (stem + '.' + stream)).read_bytes()) == record[stream + '_sha256']
    raw = (proof / (stem + '.stdout')).read_text()
    assert 'error[E' not in (proof / (stem + '.stderr')).read_text()
    assert positive + ' ... ok' in raw
    for name in failed:
        assert name + (' ... FAILED' if code else ' ... ok') in raw, name
    assert ('7 passed; 4 failed; 0 ignored' if code else '11 passed; 0 failed; 0 ignored') in raw
for stem in ['combined-native-0', 'combined-native-1']:
    assert json.loads((proof / (stem + '.json')).read_text())['exit_status'] == 0
report = json.loads((proof / 'additional-gate-1.stdout').read_text())
assert len(report['prompts']) == 500 and len({row['id'] for row in report['prompts']}) == 500
assert report['unmet_requirements'] == 0 and not report['failures']
assert [row['id'] for row in report['baseline_drift']] == ['bio-108', 'th-049', 'th-063']
for row in report['baseline_drift']:
    assert row['baseline']['outcome'] == 'computed' and row['baseline']['reason_code'] == 'typed-engine-event'
    assert row['observed']['outcome'] == 'qualitative' and row['observed']['reason_code'] == 'typed-observation'
# Verify that the implementation changed only the passive-event fallback.
old_guard = git(baseline, 'crates/kerotakis-cli/src/coverage.rs').decode()
new_guard = git(source, 'crates/kerotakis-cli/src/coverage.rs').decode()
start = new_guard.index('/// Handling and a visual description beside an explicit model limitation')
end = new_guard.index('fn execute_prompt(', start)
without_guard = new_guard[:start] + new_guard[end:]
without_guard = without_guard.replace('    } else if let Some(observation) = unsupported_observation(&all_events) {\n        observation\n', '')
assert without_guard == old_guard
accepted_lock = (base / 'diagnostics-atomicity-outcome-20261019/producer-Cargo.lock').read_bytes()
actual_lock = (proof / 'debug-producer-Cargo.lock').read_bytes()
binding = json.loads((proof / 'debug-engine-binding.json').read_text())
assert binding['source'] == source and sha(actual_lock) == binding['actual_producer_lock_sha256']
a = tomllib.loads(accepted_lock.decode()); b = tomllib.loads(actual_lock.decode())
# Multiple versions can share a name; compare ordered package records, not an alias map.
assert len(a['package']) == len(b['package'])
changes = [(x, y) for x, y in zip(a['package'], b['package']) if x != y]
assert [(x['name'], x['version'], y['version']) for x, y in changes] == [('cc', '1.8.0', '1.9.0'), ('find-msvc-tools', '0.1.14', '0.1.15')]
for x, y in changes:
    assert {k: v for k, v in x.items() if k not in ['version', 'checksum']} == {k: v for k, v in y.items() if k not in ['version', 'checksum']}
archive_reviewed = False
if args.binary_archive:
    assert sha(args.binary_archive.read_bytes()) == binding['gzip_sha256']
    h = hashlib.sha256()
    with gzip.open(args.binary_archive, 'rb') as f:
        while chunk := f.read(1024 * 1024): h.update(chunk)
    assert h.hexdigest() == binding['binary_sha256']
    archive_reviewed = True
result = {'source': source, 'four_actual_baseline_assertion_failures_and_positive_control_pass': True,
          'identical_five_regressions_pass_on_fixed_source': True, 'all11_fixed_cli_tests_pass': True,
          'all500_unmet': 0, 'all500_solver_failures': 0, 'three_further_descriptive_drifts_preserved': True,
          'only_generic_passive_event_guard_changed': True, 'actual_lock_diff_limited_to_two_build_tools': True,
          'debug_binary_archive_independently_verified': archive_reviewed, 'binary_sha256': binding['binary_sha256'],
          'actual_producer_lock_sha256': binding['actual_producer_lock_sha256'],
          'full_native_core_product_acceptance': False, 'scientific_catalog_additions': 0}
assert not args.out.exists()
args.out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
