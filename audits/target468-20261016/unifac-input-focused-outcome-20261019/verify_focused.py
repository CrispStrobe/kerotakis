"""Review saved focused controls; no engine invocation or product acceptance."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile

parser = argparse.ArgumentParser()
parser.add_argument('--evidence', type=Path, required=True)
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--binary-archive', type=Path)
args = parser.parse_args()
base = Path(__file__).resolve().parent
repo = base.parents[2]
p = json.loads((base / 'preparation.json').read_text())
e = args.evidence
sha = lambda raw: hashlib.sha256(raw).hexdigest()
assert sha((base / 'worker.py').read_bytes()) == p['uploaded_worker_sha256']
assert not (e / 'STOPPED.json').exists(), (e / 'STOPPED.json').read_text() if (e / 'STOPPED.json').exists() else ''
for path, digest in json.loads((e / 'SHA256.json').read_text()).items():
    assert sha((args.binary_archive if path == 'actual-test-binaries.tar.gz' and args.binary_archive else e / path).read_bytes()) == digest, path
start = json.loads((e / 'STARTED.json').read_text())
assert start['source'] == p['source'] and start['worker_sha256'] == p['uploaded_worker_sha256']
embedded = ast.literal_eval(ast.parse((base / 'worker.py').read_text()).body[0].value)
assert embedded == json.loads((e / 'frozen-plan.json').read_text())
for key, value in embedded.items():
    if key != 'lock_base64': assert p[key] == value, key
for path, digest in p['source_sha256'].items():
    assert sha(subprocess.check_output(['git', 'show', p['source'] + ':' + path], cwd=repo)) == digest, path
for ref in [p['source'], p['baseline_source']]:
    assert sha(subprocess.check_output(['git', 'show', ref + ':' + p['baseline_fixture']], cwd=repo)) == p['baseline_fixture_sha256']
record = json.loads((e / 'baseline.json').read_text())
assert record['argv'] == p['baseline_command'] and record['exit_status'] == 101
raw = (e / 'baseline.stdout').read_text()
assert 'error[E' not in (e / 'baseline.stderr').read_text() and '1 passed; 6 failed; 0 ignored' in raw
for name in p['expected_failing_names']: assert name + ' ... FAILED' in raw, name
assert p['positive_control'] + ' ... ok' in raw
for i, command in enumerate(p['fixed_commands']):
    record = json.loads((e / ('fixed-' + str(i) + '.json')).read_text())
    assert record['argv'] == command and record['exit_status'] == 0
    for stream in ['stdout', 'stderr']:
        assert sha((e / ('fixed-' + str(i) + '.' + stream)).read_bytes()) == record[stream + '_sha256']
raw = (e / 'fixed-2.stdout').read_text()
counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', raw)
assert counts and all(int(failed) == 0 and int(ignored) == 0 for _, failed, ignored in counts)
for name in p['expected_failing_names'] + [p['positive_control'], 'checked_refusals_keep_the_invalid_coordinate_typed', 'checked_infinite_dilution_propagates_a_missing_group_without_mutation', 'missing_forward_interaction_is_a_named_refusal', 'reverse_interaction_does_not_stand_in_for_missing_direction']:
    assert name + ' ... ok' in raw, name
assert sha((e / 'producer-Cargo.lock').read_bytes()) == p['lock_sha256']
binding = json.loads((e / 'TEST-BINARY-BINDING.json').read_text())
assert binding['source'] == p['source'] and binding['tree'] == p['tree'] and binding['actual_lock_sha256'] == p['lock_sha256']
binary_archive = args.binary_archive or e / 'actual-test-binaries.tar.gz'
assert sha(binary_archive.read_bytes()) == binding['archive_sha256']
found = {}
with tarfile.open(binary_archive) as archive:
    for member in archive.getmembers():
        if member.isfile(): found[member.name] = sha(archive.extractfile(member).read())
assert found == binding['compiled_test_binary_sha256'] and found
executables = [name for name in found if not name.endswith('.so')]
shared_libraries = [name for name in found if name.endswith('.so')]
assert len(executables) == len(counts)
for prefix in ['kerotakis_thermo-', 'unifac_admission-', 'unifac_checked_refusals-', 'thermo_oracle-']:
    assert sum(name.startswith(prefix) for name in executables) == 1, prefix
# The approved source table and constants themselves must not change.
old = subprocess.check_output(['git', 'show', p['baseline_source'] + ':crates/kerotakis-thermo/src/unifac.rs'], cwd=repo)
new = subprocess.check_output(['git', 'show', p['source'] + ':crates/kerotakis-thermo/src/unifac.rs'], cwd=repo)
section = lambda text: text.split(b'const SOURCE_1975:', 1)[1].split(b'pub type GroupDecomposition', 1)[0]
assert section(old) == section(new)
result = {'source': p['source'], 'tree': p['tree'], 'six_genuine_baseline_assertion_failures': True, 'baseline_positive_control_passed': True,
          'formatter_clippy_all_thermo_tests_and_core_compile_pass': True, 'thermo_tests_passed': sum(int(n) for n, _, _ in counts),
          'typed_and_checked_refusal_controls_pass': True, 'approved_parameter_table_unchanged': True, 'actual_lock_sha256': p['lock_sha256'],
          'compiled_payload_files_independently_verified': len(found),
          'test_executables_independently_verified': len(executables), 'compiled_shared_libraries_independently_verified': len(shared_libraries), 'full_product_or_deployment_acceptance': False,
          'scientific_catalog_additions': 0, 'original_experiment_requests': 0}
assert not args.out.exists()
args.out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
